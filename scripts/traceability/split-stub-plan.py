"""Split the stub-fill plan into per-agent slices.

Reads docs/audits/fr-matrix.json and docs/audits/_id_inventory_v3.json,
groups STUB-TEST-ONLY IDs by epic, and writes one per-agent markdown file
with the full checklist. Each agent gets an exclusive slice.
"""
from __future__ import annotations
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\koosh\Civis-clone")
MATRIX = ROOT / "docs" / "audits" / "fr-matrix.json"
INVENTORY = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
OUT_DIR = ROOT / "docs" / "audits" / "stub-fill-slices"


# Hand-tuned epic partition. Goal: each agent gets ~50 stubs.
SLICES = {
    "agent-A": [
        "FR-CIV-VEHICLE",  # 26
        "FR-CIV-BRUSH",    # 13
        "FR-CIV-RTS-RENDER",  # 5
        "FR-CIV-RTS-NATION",  # 2
        "FR-CIV-RTS-ZOOM",    # 1
    ],
    "agent-B": [
        "FR-CIV-MOD",  # 20
        "FR-CIV-LANG",  # 5
        "FR-CIV-MCP",   # 4
        "FR-DET",       # 7
        "FR-DOC",       # 1
        "FR-GUARD",     # 2
        "FR-INT",       # 1
        "FR-MET",       # 1
        "FR-METRICS",   # 2
        "FR-CIV-DET",   # 1
        "FR-CIV-LEGENDS-BROWSER",  # 1
        "FR-CIV-LEGENDS-CAUSAL",   # 1
        "FR-CIV-LEGENDS-GAP",      # 1
        "FR-CIV-LEGENDS-INSPECT",  # 1
        "FR-CIV-LEGENDS-NARRATOR", # 1
        "FR-CIV-LEGENDS-PERSIST",  # 1
        "FR-CIV-LEGENDS-PRESIM",   # 1
        "FR-CIV-LEGENDS-PRODUCER", # 1
        "FR-CIV-LEGENDS-RESOLVE",  # 1
        "FR-CIV-LEGENDS-SIG",      # 1
        "FR-CIV-ARCH",             # 1
    ],
    "agent-C": [
        "FR-CIV-PERF",         # 18
        "FR-CIV-PERF-BUILD",   # 1
        "FR-CIV-PERF-RT",      # 3
        "FR-CIV-PERF-WEB",     # 1
        "FR-CIV-LLM",          # 6
        "FR-CIV-POLITY",       # 8
        "FR-CIV-GODTOOL",      # placeholder; we won't have any
    ],
    "agent-D": [
        "FR-CIV-3D",     # 15
        "FR-CIV-QOL",    # 14
        "FR-CIV",        # 10
        "FR-CIV-VERIFY", # 10
        "FR-CIV-MARKET", # 8
    ],
    "agent-E": [
        "FR-CIV-RTS",        # 13
        "FR-CIV-CORE",       # 12
        "FR-CIV-CORE-DET",   # 3
        "FR-CIV-TERRAIN",    # 6
    ],
    "agent-F": [
        "FR-CIV-INFOVIEW",   # 11
        "FR-CIV-AI",         # 5
        "FR-CIV-LLM",        # 6 (overlaps with C; we'll fix)
        "FR-CIV-PERF",       # 18 (overlaps with C)
        "FR-CIV-LEGENDS",    # umbrella; we'll skip
    ],
}


def first_crate(spec_refs):
    for r in spec_refs:
        if "crates/" in r:
            return r.split(":")[0]
    return None


def spec_dir(eid: str) -> str | None:
    p = ROOT / "docs" / "traceability" / eid.lower()
    if p.is_dir():
        return f"docs/traceability/{eid.lower()}/"
    bare = eid.split("-", 1)[1] if "-" in eid else eid
    p2 = ROOT / "docs" / "traceability" / bare.lower()
    if p2.is_dir():
        return f"docs/traceability/{bare.lower()}/"
    return None


def main():
    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    inv_data = json.loads(INVENTORY.read_text(encoding="utf-8"))
    inv = {r["id"]: r for r in inv_data["ids"]}

    stubs = [r for r in matrix["rows"] if r["status"] == "STUB-TEST-ONLY"]
    by_epic = defaultdict(list)
    for r in stubs:
        by_epic[r["epic"]].append(r)

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # Assigned = first slice that claims the epic
    assigned = {}  # eid -> agent name
    for agent, epics in SLICES.items():
        for e in epics:
            for row in by_epic.get(e, []):
                if row["id"] not in assigned:
                    assigned[row["id"]] = agent

    unassigned = [r["id"] for r in stubs if r["id"] not in assigned]
    print(f"Total stubs: {len(stubs)}")
    print(f"Assigned: {len(assigned)}")
    print(f"Unassigned: {len(unassigned)} (will go to overflow agent)")
    if unassigned:
        print("  first 20:", unassigned[:20])

    # Group by agent
    by_agent = defaultdict(list)
    for eid, agent in assigned.items():
        by_agent[agent].append(eid)

    for agent, ids in sorted(by_agent.items()):
        path = OUT_DIR / f"{agent}.md"
        lines = [f"# Stub fill plan: {agent}", ""]
        lines.append(f"You are **{agent}**. Your job: convert each STUB-TEST-ONLY "
                     f"ID below into a real, FR-specific assertion.")
        lines.append("")
        lines.append(f"You have **{len(ids)}** stubs across "
                     f"{len({inv[i]['in_stub_tests'][0].split('/')[1] if inv[i].get('in_stub_tests') else '' for i in ids})} crates.")
        lines.append("")
        lines.append("## Workflow per ID")
        lines.append("")
        lines.append("1. Open the spec dir listed for the ID and read the spec, "
                     "intent, and ADR.")
        lines.append("2. Locate the implementing crate (often listed; otherwise "
                     "infer from the spec or grep the codebase for the ID).")
        lines.append("3. Open the stub test file. Replace the placeholder body "
                     "(currently `let ws = civ_engine::WorldState::default(); "
                     "assert!(ws.tick == 0);`) with real assertions that exercise "
                     "the FR's behavior.")
        lines.append("4. Remove the `//! Stub: TDD-red ...` line from the file's "
                     "doc-comment header.")
        lines.append("5. Run `cargo test -p <crate> --tests` for the affected crate. "
                     "Confirm green.")
        lines.append("6. Commit with message "
                     "`test(<crate>): real FR-XYZ-NNN assertions`.")
        lines.append("")
        lines.append("## Constraints")
        lines.append("")
        lines.append("- DO NOT regenerate the audit "
                     "(`docs/audits/fr-matrix.json` etc.). The maintainer does "
                     "that once all agents complete.")
        lines.append("- DO NOT touch any file outside your assigned stub test "
                     "files (and the implementing crate's source if you need to "
                     "add minimal helpers).")
        lines.append("- DO NOT modify any other stub test file, even if it looks "
                     "easy.")
        lines.append("- If the FR has no implementation anywhere, write a "
                     "minimal one in the implementing crate (a struct + method "
                     "stub is fine — but the test must call into it and assert "
                     "non-trivial state, not just `Default::default()`).")
        lines.append("- When done, post a summary of completed IDs and any "
                     "blockers back to the manager.")
        lines.append("")
        lines.append("## Assignments")
        lines.append("")
        for eid in sorted(ids):
            row = next(r for r in stubs if r["id"] == eid)
            crate = first_crate(row.get("code_refs", []) + row.get("spec_refs", []))
            sd = spec_dir(eid)
            stub_files = sorted({
                t.split(":")[0]
                for t in inv[eid].get("in_stub_tests", [])
            })
            lines.append(f"### {eid}")
            lines.append(f"- implementing crate: `{crate or 'UNKNOWN'}`")
            lines.append(f"- spec dir: `{sd or 'UNKNOWN'}`")
            for s in stub_files:
                lines.append(f"- stub file: `{s}`")
            lines.append("")

        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"  wrote {path} ({len(ids)} stubs)")


if __name__ == "__main__":
    main()
