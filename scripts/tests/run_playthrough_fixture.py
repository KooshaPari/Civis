#!/usr/bin/env python3
"""Run a bounded, owned-local-server playthrough fixture on Linux.

Operator strings are explicit test inputs, not authenticated actor entitlements.
The stock playthrough and its release E2E wrapper are neither changed nor certified.
"""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone


PINNED_WEBSOCAT_SHA256 = "8f84c57103d33ab73888707041765e0e7e6a43a91fbb6e1828cd5eabc19ae32c"
FUNCTIONS = (
    "json_get", "json_ok", "ws_url", "rpc_buffer_size", "rpc",
    "rpc_read_response", "rpc_select_response", "json_quote",
)
BAD_DIAGNOSTICS = (
    "Failed to decode user-supplised base64", "ignored null byte", "ignored NUL",
    "jq: error", "jq: parse error", "parse error:", "Incoming message too long",
    "unrecognized transport framing",
)


class FixtureFailure(RuntimeError):
    """A required fixture assertion or prerequisite failed."""


def utc() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds")


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def process_session(session_id: int) -> list[int]:
    found = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            fields = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            if int(fields[3]) == session_id:
                found.append(int(entry.name))
        except (OSError, ValueError, IndexError):
            continue
    return found


def transport_definitions(source: str) -> str:
    definitions = []
    for name in FUNCTIONS:
        match = re.search(rf"(?ms)^{name}\(\) \{{\n.*?^\}}\n", source)
        if match is None:
            raise FixtureFailure(f"Cannot extract exact transport function {name}")
        definitions.append(match.group())
    failure = re.search(r"(?m)^fail\(\).*?$", source)
    if failure is None:
        raise FixtureFailure("Cannot extract exact failure diagnostic function")
    return "\n".join([*definitions, failure.group()])


def available_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


class Fixture:
    def __init__(self, args: argparse.Namespace) -> None:
        self.server_binary = args.server_binary.resolve()
        self.websocat = args.websocat.resolve()
        self.report_path = args.report.resolve()
        if self.report_path.exists():
            raise FixtureFailure("Report already exists; choose a fresh report path")
        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.artifacts = Path(tempfile.mkdtemp(prefix="civis-fixture-", dir=self.report_path.parent))
        self.repository = Path(__file__).resolve().parents[2]
        self.source_path = self.repository / "scripts/playthrough.sh"
        self.server: subprocess.Popen | None = None
        self.sessions: list[subprocess.Popen] = []
        self.reaped: list[int] = []
        self.started = time.monotonic()
        self.deadline = self.started + 120
        self.phase = "preflight"
        self.report = {
            "scope": "owned-local-server positive and negative test fixture only",
            "authenticated_identity_verified": False, "stock_e2e_certified": False,
            "started_utc": utc(), "artifact_directory": str(self.artifacts),
            "requests": [], "steps": [], "negative_cases": [], "assertions": [],
            "role_inputs": "operator only on explicit positive spawn/tick requests",
            "runtime_cap_seconds": 30, "per_rpc_timeout_seconds": 2,
        }

    def require(self, condition: bool, description: str) -> None:
        self.report["assertions"].append({"phase": self.phase, "check": description, "passed": bool(condition)})
        if not condition:
            raise FixtureFailure(description)

    def budget(self) -> float:
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise FixtureFailure("Fixture execution budget expired")
        return remaining

    def prepare(self) -> None:
        self.require(sys.platform.startswith("linux"), "Linux process ownership support is available")
        prerequisites = {}
        for name in ("bash", "jq", "timeout", "python3"):
            executable = shutil.which(name)
            self.require(executable is not None, f"Required tool {name} is available")
            version = subprocess.check_output([executable, "--version"], text=True, timeout=2).splitlines()[0]
            prerequisites[name] = {"path": executable, "version": version}
        self.require(self.server_binary.name == "civ-server" and os.access(self.server_binary, os.X_OK), "Explicit civ-server binary is executable")
        self.require(digest(self.websocat) == PINNED_WEBSOCAT_SHA256, "Websocat matches pinned official 1.13.0 SHA256")
        version = subprocess.check_output([str(self.websocat), "--version"], text=True, timeout=2).strip()
        self.require(version == "websocat 1.13.0", "Websocat version is exactly 1.13.0")
        if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
            raise FixtureFailure("Cannot establish fixture child-subreaper ownership")
        with socket.socket() as probe:
            try:
                probe.bind(("0.0.0.0", 9090))
            except OSError as error:
                raise FixtureFailure("Existing fixed metrics port 9090 is unavailable; no existing process was touched") from error
        self.report["source_sha256"] = digest(self.source_path)
        self.report["runner_sha256"] = digest(Path(__file__))
        self.report["tools"] = {"websocat": version, "websocat_sha256": PINNED_WEBSOCAT_SHA256, "python": sys.version.split()[0], "server_binary": str(self.server_binary), "server_sha256": digest(self.server_binary), "prerequisites": prerequisites}
        self.prepare_environment()
        self.prepare_transport()

    def prepare_environment(self) -> None:
        self.env = os.environ.copy()
        for key in list(self.env):
            if key.startswith("OTEL_EXPORTER_"):
                self.env.pop(key)
        for key in ("CIV_AUTOLOAD", "CIVIS_TICK_BROADCAST", "RPC_MAX_MESSAGE_BYTES", "CIVIS_FIXTURE_FORGED_HEADER"):
            self.env.pop(key, None)
        self.port = available_port()
        self.env.update({"CIVIS_WS_ADDR": f"127.0.0.1:{self.port}", "CIV_SERVER_PORT": str(self.port), "CIVIS_REQUIRE_ROLE": "true", "CIVIS_MAP_SEED": "42", "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4317", "RUST_LOG": "info"})
        for key, folder in (("CIVIS_SAVES_DIR", "saves"), ("CIVIS_REPLAYS_DIR", "replays")):
            directory = self.artifacts / folder
            directory.mkdir()
            self.env[key] = str(directory)
        self.report["isolation"] = {"ws_address": self.env["CIVIS_WS_ADDR"], "owned_fresh_process": True, "role_enforcement": True, "tick_broadcast": "default both", "metrics_listener": "unchanged 0.0.0.0:9090", "telemetry": "inherited exporter endpoint/headers cleared; OTLP loopback only", "save_directory": self.env["CIVIS_SAVES_DIR"], "replay_directory": self.env["CIVIS_REPLAYS_DIR"]}

    def prepare_transport(self) -> None:
        directory = self.artifacts / "bin"
        directory.mkdir()
        wrapper = directory / "websocat"
        wrapper.write_text("#!/usr/bin/env python3\nimport os, sys\n" + f"args = [{str(self.websocat)!r}]\n" + "if os.environ.get('CIVIS_FIXTURE_FORGED_HEADER') == '1':\n    args += ['-H', 'x-civis-role: operator']\nos.execv(args[0], args + sys.argv[1:])\n")
        wrapper.chmod(0o700)
        self.env["PATH"] = str(directory) + os.pathsep + self.env["PATH"]
        self.transport = self.artifacts / "transport.sh"
        setup = "set -u\nset -o pipefail\nC_FAIL=\nC_RST=\nCIV_WS_HOST=127.0.0.1\nCIV_WS_PATH=/ws\nRPC_TIMEOUT_SECONDS=2\n"
        self.transport.write_text(setup + transport_definitions(self.source_path.read_text()) + '\nrpc "$@"\n')
        self.report["transport"] = {"source": str(self.source_path), "functions": [*FUNCTIONS, "fail"], "exact_source_definitions": True, "test_wrapper": "pinned real websocat; forged header applied only in explicit negative cases"}

    def start_server(self) -> None:
        self.deadline = min(self.deadline, time.monotonic() + 30)
        self.server_log = (self.artifacts / "server.log").open("wb")
        self.server = subprocess.Popen([str(self.server_binary)], cwd=self.artifacts, env=self.env, stdin=subprocess.DEVNULL, stdout=self.server_log, stderr=subprocess.STDOUT, start_new_session=True)
        self.sessions.append(self.server)
        self.report["server_pid"] = self.server.pid
        until = time.monotonic() + 4
        while self.server.poll() is None and time.monotonic() < until:
            lines = (self.artifacts / "server.log").read_text().splitlines()
            confirmation = next((line for line in lines if "ws bridge listening" in line and f"addr=127.0.0.1:{self.port} " in line), None)
            listener = self.owned_listener()
            if confirmation is not None and listener is not None:
                self.report["owned_listener_evidence"] = {"log_line": confirmation, **listener}
                return
            time.sleep(0.03)
        raise FixtureFailure("Owned server did not confirm its own loopback listener within four seconds")

    def owned_listener(self) -> dict | None:
        if self.server is None or self.server.poll() is not None:
            return None
        process = Path("/proc") / str(self.server.pid)
        try:
            sockets = {entry.readlink().name for entry in (process / "fd").iterdir()}
            for line in (process / "net/tcp").read_text().splitlines()[1:]:
                fields = line.split()
                if fields[1] == f"0100007F:{self.port:04X}" and fields[3] == "0A" and f"socket:[{fields[9]}]" in sockets:
                    return {"pid": self.server.pid, "socket_inode": fields[9], "address": f"127.0.0.1:{self.port}", "ownership": "listening socket inode belongs to spawned server PID"}
        except (OSError, IndexError):
            return None
        return None

    def reap_orphans(self, excluded: tuple[int, ...] = ()) -> None:
        for entry in Path("/proc").iterdir():
            if not entry.name.isdigit() or int(entry.name) in excluded:
                continue
            try:
                fields = (entry / "stat").read_text().rsplit(")", 1)[1].split()
                if fields[0] == "Z" and int(fields[1]) == os.getpid():
                    pid, _ = os.waitpid(int(entry.name), os.WNOHANG)
                    if pid:
                        self.reaped.append(pid)
            except (OSError, ValueError, IndexError):
                continue

    def stop(self, process: subprocess.Popen) -> list[int]:
        for pid in process_session(process.pid):
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                continue
        try:
            process.wait(timeout=0.3)
        except subprocess.TimeoutExpired:
            pass
        # A child can outlive its session leader; escalation never depends on
        # whether the parent has already exited.
        for pid in process_session(process.pid):
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                continue
        process.wait(timeout=1)
        excluded = (self.server.pid,) if self.server is not None and self.server.poll() is None else ()
        until = time.monotonic() + 0.5
        while process_session(process.pid) and time.monotonic() < until:
            self.reap_orphans(excluded)
            time.sleep(0.01)
        self.reap_orphans(excluded)
        return process_session(process.pid)

    def rpc(self, method: str, params: dict, forged_header: bool = False) -> tuple[int, dict]:
        self.require(self.owned_listener() is not None, "Every RPC targets a listener still owned by the live spawned server")
        request_id = 2001 + len(self.report["requests"])
        env = self.env.copy()
        if forged_header:
            env["CIVIS_FIXTURE_FORGED_HEADER"] = "1"
        command = ["bash", str(self.transport), str(request_id), method, json.dumps(params, separators=(",", ":"))]
        prefix = self.artifacts / f"rpc-{request_id}"
        started = time.monotonic()
        record = {"id": request_id, "method": method, "params": params, "forged_role_header": forged_header, "started_utc": utc()}
        with prefix.with_suffix(".stdout").open("wb") as out, prefix.with_suffix(".stderr").open("wb") as err:
            process = subprocess.Popen(command, cwd=self.artifacts, env=env, stdin=subprocess.DEVNULL, stdout=out, stderr=err, start_new_session=True)
            self.sessions.append(process)
            try:
                process.wait(timeout=min(4, self.budget()))
            finally:
                remaining = self.stop(process)
        record.update({"exit": process.returncode, "duration_seconds": round(time.monotonic() - started, 6), "ended_utc": utc(), "remaining_owned_pids": remaining})
        diagnostics = prefix.with_suffix(".stderr").read_text()
        record["stderr"] = diagnostics
        output = prefix.with_suffix(".stdout").read_text()
        record["response"] = json.loads(output) if output else None
        self.report["requests"].append(record)
        self.require(not remaining, f"RPC {request_id} leaves no owned helper process")
        self.require(not any(pattern.lower() in diagnostics.lower() for pattern in BAD_DIAGNOSTICS), f"RPC {request_id} has no framing/base64/NUL/jq/splitting diagnostic")
        body = record["response"]
        self.require(isinstance(body, dict) and body.get("jsonrpc") == "2.0" and type(body.get("id")) is int and body["id"] == request_id, f"RPC {request_id} returns exactly its matching JSON-RPC response")
        return process.returncode, body

    def success(self, method: str, params: dict | None = None) -> dict:
        status, body = self.rpc(method, {} if params is None else params)
        self.require(status == 0 and "result" in body and "error" not in body, f"{method} succeeds explicitly")
        return body["result"]

    def snapshot(self) -> dict:
        state = self.success("sim.snapshot")
        self.require(state.get("speed_multiplier") == 0, "Snapshot confirms paused simulation clock")
        self.require(type(state.get("tick")) is int and type(state.get("population")) is int and isinstance(state.get("civ_pins"), list), "Snapshot has tick, population, and civilian pin projection")
        pins = sorted(state["civ_pins"], key=lambda pin: json.dumps(pin, sort_keys=True))
        return {"tick": state["tick"], "population": state["population"], "civ_pins": pins}

    def step(self, number: int, name: str, action) -> None:
        self.phase = f"step {number}: {name}"
        start = len(self.report["assertions"])
        record = {"number": number, "name": name, "passed": False}
        self.report["steps"].append(record)
        action()
        record.update({"passed": True, "assertions": len(self.report["assertions"]) - start})

    def health(self) -> None:
        value = self.success("sim.status")
        self.require(type(value.get("tick")) is int, "Health exposes a simulation tick")

    def attach(self) -> None:
        value = self.success("sim.status")
        self.require(type(value.get("population")) is int, "Attach exposes simulation population")

    def reset(self) -> None:
        paused = self.success("sim.set_speed", {"multiplier": 0})
        self.require(paused.get("accepted") is True and paused.get("multiplier") == 0, "Existing public clock control acknowledges pause")
        previous = self.snapshot()
        stable = 0
        for _ in range(8):
            time.sleep(0.12)
            current = self.snapshot()
            stable = stable + 1 if current == previous else 0
            previous = current
            if stable >= 2:
                break
        self.require(stable >= 2, "Pause reaches measured quiescence before reset")
        reset = self.success("sim.reset", {"seed": 42})
        self.require(reset.get("seed") == 42 and reset.get("tick") == 0, "World reset returns fixed seed and tick zero")
        self.initial = self.snapshot()
        self.require(self.initial["tick"] == 0 and 0 < len(self.initial["civ_pins"]) <= 252, "Reset leaves stable tick-zero projection below pin truncation cap")

    def negatives(self) -> None:
        self.phase = "negative authorization cases"
        for method in ("sim.spawn_civilian", "sim.command"):
            for case in ("missing", "viewer", "forged_header"):
                params = {"x": 0.5, "y": 0.5, "faction": 0} if method == "sim.spawn_civilian" else {"action": "tick"}
                if case == "viewer":
                    params["role"] = "viewer"
                before = self.snapshot()
                status, body = self.rpc(method, params, case == "forged_header")
                error = body.get("error", {})
                self.require(status == 2 and "result" not in body and error.get("code") == -32003 and error.get("data", {}).get("required_role") == "operator", f"{method}/{case} has exact operator denial")
                after = self.snapshot()
                self.require(before == after, f"{method}/{case} leaves paused tick/population/civilian pins unchanged")
                self.report["negative_cases"].append({"method": method, "case": case, "passed": True, "projection_unchanged": True, "projection": before})

    def spawn(self) -> None:
        before = self.snapshot()
        points = ((0.50, 0.50, 0), (0.51, 0.49, 0), (0.49, 0.51, 1), (0.50, 0.52, 1))
        handles = []
        for x, y, faction in points:
            result = self.success("sim.spawn_civilian", {"x": x, "y": y, "faction": faction, "role": "operator"})
            self.require(result.get("ok") is True and type(result.get("entity_id")) is int, "Spawn returns a concrete successful entity handle")
            handles.append(result["entity_id"])
        after = self.snapshot()
        self.require(len(set(handles)) == 4, "Four explicit positive spawns return distinct entity handles")
        self.require(len(after["civ_pins"]) == len(before["civ_pins"]) + 4, "Four spawns add exactly four visible civilian pins")
        for x, y, _ in points:
            self.require(any(abs(pin["x"] - x) < 0.0001 and abs(pin["y"] - y) < 0.0001 for pin in after["civ_pins"]), f"Spawn coordinate {x},{y} appears in the paused pin projection")
        self.report["spawn_evidence"] = {"entity_handles": handles, "before_pin_count": len(before["civ_pins"]), "after_pin_count": len(after["civ_pins"]), "population_note": "sim.state.population need not immediately reflect direct ECS spawns"}

    def catalog(self) -> None:
        factions = self.success("sim.get_factions")
        tech = self.success("sim.tech_state")
        self.require(isinstance(factions.get("factions"), list), "Faction retrieval returns its documented array shape")
        snapshot = self.success("sim.snapshot")
        self.require({0, 1}.issubset({row["id"] for row in snapshot["factions"]}), "Full snapshot exposes fixture factions zero and one")
        self.report["faction_retrieval"] = {"endpoint_count": len(factions["factions"]), "snapshot_count": len(snapshot["factions"]), "known_endpoint_gap": "Live sim.get_factions receives no dispatch snapshot and currently returns an empty array; no server repair is included"}
        self.require(isinstance(tech.get("available"), list) and bool(tech["available"]), "Tech state supplies available technologies")

    def diplomacy(self, action: str, expected: str) -> None:
        result = self.success("sim.diplomacy_action", {"source_faction": 0, "target_faction": 1, "action": action})
        # The unchanged bridge Debug-formats the engine's string relation,
        # producing a JSON string containing quotes, rather than a kind enum.
        self.require(result.get("action") == action and result.get("source_faction") == 0 and result.get("target_faction") == 1 and result.get("relation") == json.dumps(expected), f"{action} produces the actual {expected} relation-effect receipt")
        self.report.setdefault("diplomacy_effects", []).append({"action": action, "relation": expected, "raw_relation": result["relation"]})

    def war(self) -> None:
        # Explicit actions update a bounded score; war after +1 trade first
        # reaches neutral, then hostile. Assert both effects without skipping.
        self.diplomacy("declare_war", "neutral")
        self.diplomacy("declare_war", "hostile")
        self.report["diplomacy_contract"] = "Existing non-idempotent score updates: trade adds 1; each war subtracts 1. From allied, two explicit declarations reach neutral then hostile. Bridge Debug-formats the string relation, preserving nested quotes; no production API repair is included."

    def save(self) -> None:
        self.saved_projection = self.snapshot()
        result = self.success("save.slot", {"slot_name": "slot-1"})
        self.require(result.get("saved") is True and result.get("tick") == self.saved_projection["tick"], "Save receipt confirms the exact saved tick")
        path = Path(result.get("path", "")).resolve()
        self.require(path.is_relative_to(self.artifacts / "saves") and path.is_file() and path.stat().st_size > 0, "Owned save archive exists inside the fixture save directory")
        self.report["saved_archive"] = {"path": str(path), "bytes": path.stat().st_size, "sha256": digest(path)}

    def load(self) -> None:
        tick = self.saved_projection["tick"]
        for _ in range(5):
            result = self.success("sim.command", {"action": "tick", "role": "operator"})
            self.require(result.get("tick") == tick + 1, "Each explicit positive tick advances exactly once while paused")
            tick += 1
        self.require(self.snapshot()["tick"] == tick, "Five successful ticks genuinely diverge from the saved state")
        result = self.success("save.load", {"slot_name": "slot-1"})
        self.require(result.get("loaded") is True and result.get("tick") == self.saved_projection["tick"], "Load receipt restores the saved tick")
        self.require(self.snapshot() == self.saved_projection, "Load restores the saved tick/population/civilian-pin projection")

    def smite(self) -> None:
        result = self.success("sim.god_action", {"action": "smite", "x": 0.50, "y": 0.50, "radius_voxels": 5})
        self.require(result.get("ok") is True and result.get("accepted") is True, "Known smite action is accepted explicitly")
        self.report["god_action_scope"] = "Existing smite dispatch acknowledgement; visual/rendered consequences are outside this fixture"

    def run(self) -> None:
        self.prepare()
        self.start_server()
        self.step(1, "health", self.health)
        self.step(2, "attach contract", self.attach)
        self.step(3, "pause and seeded reset", self.reset)
        self.negatives()
        self.step(4, "four explicit positive civilian spawns", self.spawn)
        self.step(5, "faction and technology state", self.catalog)
        self.step(6, "trade agreement effect", lambda: self.diplomacy("offer_trade", "allied"))
        self.step(7, "two explicit war declarations reach hostility", self.war)
        self.step(8, "owned slot save", self.save)
        self.step(9, "five explicit ticks and verified slot load", self.load)
        self.step(10, "accepted smite dispatch", self.smite)
        self.require(self.server is not None and self.server.poll() is None, "Owned server remains alive after all ten intended actions")
        self.report["server_alive_before_cleanup"] = True
        self.require(digest(self.source_path) == self.report["source_sha256"], "Reused production transport source remains unchanged during the fixture")

    def finish(self, failure: BaseException | None) -> int:
        self.report["server_alive_before_cleanup"] = self.server is not None and self.server.poll() is None
        remaining = []
        for process in reversed(self.sessions):
            remaining.extend(self.stop(process))
        if hasattr(self, "server_log"):
            self.server_log.close()
        self.report.update({"finished_utc": utc(), "duration_seconds": round(time.monotonic() - self.started, 6), "cleanup": {"remaining_owned_pids": sorted(set(remaining)), "reaped_orphan_pids": self.reaped, "zero_new_orphans": not remaining and not self.reaped}})
        passed = failure is None and not remaining and not self.reaped
        self.report["passed"] = passed
        self.report["failure"] = None if failure is None else {"phase": self.phase, "type": type(failure).__name__, "message": str(failure)}
        self.report["counts"] = {"positive_steps_passed": sum(step["passed"] for step in self.report["steps"]), "negative_cases_passed": len(self.report["negative_cases"]), "assertions_passed": sum(check["passed"] for check in self.report["assertions"]), "assertions_failed": sum(not check["passed"] for check in self.report["assertions"])}
        with self.report_path.open("x") as output:
            json.dump(self.report, output, indent=2)
            output.write("\n")
        print(json.dumps({"passed": passed, "report": str(self.report_path), "counts": self.report["counts"], "failure": self.report["failure"]}))
        return 0 if passed else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server-binary", type=Path, required=True)
    parser.add_argument("--websocat", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    fixture = Fixture(args)
    failure = None
    try:
        fixture.run()
    except BaseException as error:
        failure = error
    finally:
        result = fixture.finish(failure)
    return result


if __name__ == "__main__":
    sys.exit(main())
