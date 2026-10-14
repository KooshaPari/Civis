#!/usr/bin/env python3
"""Bounded lifecycle regressions for the isolated live fixture runner."""

import argparse
import ctypes
import importlib.util
import json
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SOURCE = Path(__file__).with_name("run_playthrough_fixture.py")
SPEC = importlib.util.spec_from_file_location("playthrough_fixture", SOURCE)
fixture_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixture_module)


@unittest.skipUnless(sys.platform.startswith("linux"), "Linux ownership probes")
class FixtureLifecycleTests(unittest.TestCase):
    def fixture(self, directory):
        args = argparse.Namespace(
            server_binary=Path("/tmp/unused/civ-server"),
            websocat=Path("/tmp/unused/websocat"),
            report=Path(directory) / "report.json",
        )
        return fixture_module.Fixture(args)

    def test_unexpected_schema_error_always_calls_cleanup(self):
        error = KeyError("unexpected wire schema")
        fixture = mock.Mock()
        fixture.run.side_effect = error
        fixture.finish.return_value = 1
        args = [
            str(SOURCE), "--server-binary", "/tmp/unused/civ-server",
            "--websocat", "/tmp/unused/websocat", "--report", "/tmp/unused/report",
        ]
        with mock.patch.object(sys, "argv", args):
            with mock.patch.object(fixture_module, "Fixture", return_value=fixture):
                self.assertEqual(fixture_module.main(), 1)
        fixture.finish.assert_called_once_with(error)

    def test_binding_log_ignores_ansi_but_requires_exact_loopback_address(self):
        plain = "INFO ws bridge listening addr=127.0.0.1:3800 max_clients=16"
        colored = "\x1b[32mINFO\x1b[0m ws bridge listening \x1b[3maddr\x1b[0m\x1b[2m=\x1b[0m127.0.0.1:3800 max_clients=16"
        for line in (plain, colored):
            with self.subTest(line=line):
                self.assertEqual(fixture_module.binding_log_confirmation([line], 3800), {"log_line": line, "normalized_log_line": plain})
        for line in (colored.replace(":3800 ", ":38001 "), colored.replace("127.0.0.1", "0.0.0.0"), colored.replace("ws bridge listening", "unrelated log")):
            with self.subTest(line=line):
                self.assertIsNone(fixture_module.binding_log_confirmation([line], 3800))

    def test_failure_stdout_preserves_binding_and_rpc_error_without_trace_noise(self):
        with tempfile.TemporaryDirectory(prefix="civis-fixture-test-") as directory:
            fixture = self.fixture(directory)
            line = "INFO ws bridge listening addr=127.0.0.1:3800 max_clients=16"
            (fixture.artifacts / "server.log").write_text(line + "\n\x1b[31m ERROR \x1b[0m save.slot failed: json: key must be a string\nTRACE unrelated transport poll\n")
            fixture.report["owned_listener_evidence"] = {"log_line": line, "normalized_log_line": line, "pid": 123, "socket_inode": "456"}
            response = {"jsonrpc": "2.0", "id": 2040, "error": {"code": -32603, "message": "json: key must be a string"}}
            fixture.report["requests"].append({"id": 2040, "method": "save.slot", "exit": 2, "response": response})
            with mock.patch("builtins.print") as output:
                self.assertEqual(fixture.finish(fixture_module.FixtureFailure("save failed")), 1)
            evidence = json.loads(output.call_args.args[0])["failure_evidence"]
            self.assertEqual(evidence["failing_rpc"]["response"], response)
            self.assertEqual(evidence["owned_listener"]["normalized_log_line"], line)
            self.assertIn(" ERROR  save.slot failed: json: key must be a string", evidence["owned_server_log_excerpt"])
            self.assertFalse(any("TRACE" in line or "\x1b" in line for line in evidence["owned_server_log_excerpt"]))

    def test_other_process_listener_is_never_owned(self):
        with tempfile.TemporaryDirectory(prefix="civis-fixture-test-") as directory:
            fixture = self.fixture(directory)
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                listener.listen()
                fixture.port = listener.getsockname()[1]
                process = subprocess.Popen(
                    [sys.executable, "-c", "import time; time.sleep(10)"],
                    start_new_session=True,
                )
                fixture.server = process
                fixture.sessions.append(process)
                try:
                    self.assertIsNone(fixture.owned_listener())
                finally:
                    fixture.stop(process)
                self.assertEqual(fixture_module.process_session(process.pid), [])

    def test_term_ignoring_child_is_killed_after_parent_exits_and_reported(self):
        self.assertEqual(ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0), 0)
        with tempfile.TemporaryDirectory(prefix="civis-fixture-test-") as directory:
            fixture = self.fixture(directory)
            marker = Path(directory) / "child.pid"
            child_code = (
                "import os, signal, time\n"
                "from pathlib import Path\n"
                "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
                f"Path({str(marker)!r}).write_text(str(os.getpid()))\n"
                "time.sleep(10)\n"
            )
            parent_code = (
                "import subprocess, sys, time\n"
                "from pathlib import Path\n"
                f"subprocess.Popen([sys.executable, '-c', {child_code!r}])\n"
                f"marker = Path({str(marker)!r})\n"
                "while not marker.exists(): time.sleep(0.01)\n"
            )
            process = subprocess.Popen(
                [sys.executable, "-c", parent_code], start_new_session=True,
            )
            fixture.sessions.append(process)
            try:
                process.wait(timeout=2)
                child_pid = int(marker.read_text())
                self.assertIn(child_pid, fixture_module.process_session(process.pid))
                self.assertEqual(fixture.stop(process), [])
                self.assertIn(child_pid, fixture.reaped)
                with mock.patch("builtins.print"):
                    self.assertEqual(fixture.finish(None), 1)
                report = json.loads(fixture.report_path.read_text())
                self.assertFalse(report["passed"])
                self.assertFalse(report["cleanup"]["zero_new_orphans"])
            finally:
                fixture.stop(process)


if __name__ == "__main__":
    unittest.main()
