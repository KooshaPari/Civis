"""Remove false container-bound requirement tags, replacing each with the reason.

Every removal is driven by an explicit allowlist of (file, tag) pairs that were
hand-reviewed against the authoritative spec text. A module-level comment records
why the block was dropped, so the next reader does not simply re-paste the tags.

Run with --apply to write, without it to preview.
"""
import argparse
import pathlib
import re
import sys

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")

# --------------------------------------------------------------------------
# (1) The WorldState / Simulation tag blocks in crates/engine/src/engine.rs
# --------------------------------------------------------------------------

# Tags removed from the `pub struct WorldState` block, each with the reason it
# cannot be discharged by a flat scalar aggregate. Mirrors the triaged verdicts
# in docs/audits/triage-container-engine-core.md and
# docs/audits/triage-container-sim-domain.md.
WORLDSTATE_REMOVED = {
    # CIV-0106 social dynamics: the required functions do not exist anywhere in
    # the workspace, and civ-social has zero dependents.
    "FR-SOC-INS-001": "needs compute_insurgency_risk_from_params; 0 occurrences repo-wide",
    "FR-SOC-INS-002": "needs measure_net_compliance_effect; 0 occurrences repo-wide",
    "FR-SOC-INS-003": "cell formation needs advance_tick_capture_events; 0 occurrences",
    "FR-SOC-INS-004": "AmnestyCampaign; 0 occurrences repo-wide",
    "FR-SOC-INS-005": "no mobilization scalar or non-linear risk curve exists",
    "FR-SOC-INS-006": "no InsurgencyCell lifecycle type exists",
    "FR-SOC-INS-007": "no counterinsurgency detection probability model exists",
    "FR-SOC-CIV-001": "no civic R0 computation exists",
    "FR-SOC-CIV-002": "no E+A+R civic compartment representation exists",
    "FR-SOC-COH-001": "phase_cohesion's fabric_score has no coercion term",
    "FR-SOC-COH-002": "phase_cohesion's fabric_score has no welfare-floor term",
    "FR-SOC-COH-003": "no polarization variable exists in any crate",
    "FR-SOC-COH-004": "phase_cohesion iterates settlements independently; no adjacency term",
    "FR-SOC-INT-001": "no intervention registry or apply function exists",
    "FR-SOC-INT-002": "no ideology diffusion rate exists; 0 'diffusion' hits in crates/social",
    "FR-SOC-INT-003": "no intervention event types exist to emit",
    "FR-SOC-INT-004": "no intervention lifetime or expiry exists",
    "FR-SOC-INTG-001": "cross-module coupling cannot run: civ-social has zero dependents",
    "FR-SOC-INTG-002": "no diplomacy -> insurgency edge exists",
    "FR-SOC-INTG-003": "no dissenting-stage -> susceptibility edge exists",
    "FR-SOC-INTG-004": "no coalition-stability metric exists",
    "FR-SOC-INTG-005": "no health -> joule coupling; civ-social/health.rs is unread",
    "FR-SOC-INTG-006": "no radicalization attractor dynamics; IdeologyScore is static",
    "FR-SOC-INTG-007": "no civic recovery state machine exists",
    # Simulation-loop requirements implemented elsewhere, in a different crate.
    "FR-CIV-CORE-002": "determinism is proven by hash comparison in integrity.rs, not by this struct",
    "FR-CIV-CORE-004": "a 16 ms wall-clock budget needs a timing harness; tick_compute_time has 0 hits",
    "FR-CIV-CORE-019": "the ECS world is the separate hecs World field, not WorldState",
    "FR-CIV-ARCH-006": "no spec defines this ID",
    "FR-CIV-ARCH-NOSVG-001": "a CI bundle script assertion about asset-pipeline, not a state field",
    "FR-CIV-PERF-RT-003": "sprite-pool pre-warm is a client render behavior; no pool exists",
    "NFR-CIV-PERF-002": "a 60 FPS Metal NFR is a measured property; the named bench does not exist",
    "FR-PROT-001": "JSON-RPC dispatch is implemented in crates/server, a different crate",
    "FR-PROT-002": "notification broadcast is implemented in crates/server/ws_bridge.rs",
    "FR-PROT-003": "envelope fields are implemented in crates/server/jsonrpc.rs",
    "FR-PROT-005": "bearer parsing is implemented in crates/server/authn.rs",
    "FR-SOC-CIV-001 ": "duplicate entry in the same block",
}

SIMULATION_REMOVED = {
    "FR-CIV-CORE-003": "the seeded ChaCha8Rng is Simulation.rng; the tag belongs on the SimRng alias",
    "FR-CIV-CORE-009": "JSON-RPC method dispatch lives in crates/server, not on Simulation",
    "FR-CIV-CORE-017": "get_snapshot_for_session ignores subscribed_frame_kinds; no filtering exists",
    "FR-CIV-GODTOOL-921": "no undo exists; `fn undo` has 0 hits in the engine",
}

# --------------------------------------------------------------------------
# (2) The SESSION_HISTORY_CAP block in crates/server/src/session.rs
# --------------------------------------------------------------------------
# Each entry states what the requirement needs and why session.rs cannot
# supply it, taken from the requirement text in
# docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md.
SESSION_REMOVED = {
    "FR-SESSION-001": "needs a pve session type with one human plus AI nations; no session-type field exists",
    "FR-SESSION-002": "needs a per-AI-nation ChaCha20Rng sub-stream from the session seed; no RNG field exists",
    "FR-SESSION-003": "needs human permanent input authority; SharedSession.role is an operator role",
    "FR-SESSION-004": "needs a NationAction queue; that type does not exist anywhere in crates/",
    "FR-SESSION-005": "needs rejection of non-NationAction AI submissions; depends on that absent type",
    "FR-SESSION-006": "needs a hot_seat multi-human shared WebSocket; no session-type field exists",
    "FR-SESSION-007": "needs turn-token enforcement with error -32001; no turn token, and -32001 has 0 hits in the crate",
    "FR-SESSION-008": "needs a session.turn.end RPC advancing and validating rotation; the method enum has no turn methods",
    "FR-SESSION-009": "needs turn-timeout auto-advance at expires_at_tick; no such field exists",
    "FR-SESSION-010": "needs simultaneous-turn action collection and deterministic resolution; not implemented",
    "FR-SESSION-011": "needs observers that receive broadcasts without injection ability; no observer flag exists",
    "FR-SESSION-012": "needs observer RPCs rejected with a specific error code; neither exists",
    "FR-SESSION-013": "needs omniscient observer mode with tick_stride; no mode field exists",
    "FR-SESSION-014": "needs server-side visibility filtering; get_snapshot_for_session returns the full snapshot",
    "FR-SESSION-015": "needs an ENDED-session replay observer with seek; no session status or seek handler exists",
    "FR-SESSION-016": "needs POST /api/v1/challenges with challenge_id and queue position; no HTTP route exists",
    "FR-SESSION-017": "needs fully headless challenge sessions at max tick rate; no such mode exists",
    "FR-SESSION-018": "needs a baseline score from an AI-only session; no scoring code exists",
    "FR-SESSION-019": "needs weighted normalized metric deltas in fixed point; no score computation exists",
    "FR-SESSION-020": "needs GET /api/v1/challenges/{id}/replay storing .civreplay; no route or storage exists",
    "FR-SESSION-021": "needs a session.pause RPC halting the tick loop; no such method or field exists",
    "FR-SESSION-022": "needs a session.resume RPC restoring the loop and BLAKE3 chain; not implemented",
    "FR-SESSION-023": "needs session.set_speed accepting 1..=100 applied at a boundary; no handler exists",
    "FR-SESSION-024": "needs session.fast_forward suppressing broadcasts then sending a final snapshot; not implemented",
    "FR-SESSION-025": "needs session.paused.v1 / resumed.v1 / speed_changed.v1 events; no such event types exist",
    "FR-SESSION-026": "needs a session.save RPC writing a named slot with a BLAKE3 hash; no save RPC exists here",
    "FR-SESSION-027": "needs a session.load RPC verifying the BLAKE3 hash before restore; no load RPC exists here",
    "FR-SESSION-028": "needs autosave to the autosave slot every autosave_interval_ticks; no timer exists",
    "FR-SESSION-029": "needs loading from an ENDED session to branch a new session_id; no status or branch logic",
    "FR-SESSION-030": "needs a UUIDv7 at session.create; SharedSession::new mints a UUID v4 and no create RPC exists",
    "FR-SESSION-031": "needs full SessionConfig validation at create; no SessionConfig type exists",
    "FR-SESSION-032": "needs persisting session state to a sessions table; the crate has no database layer",
    "FR-SESSION-033": "needs reloading incomplete sessions on restart; there is no persistence to reload from",
}

SITES = [
    {
        "path": "crates/engine/src/engine.rs",
        "anchor": "pub struct WorldState {",
        "removed": WORLDSTATE_REMOVED,
        "summary": "removed %d tags from the WorldState aggregate",
        "note": [
            "`WorldState` is a passive aggregate of scalar fields with no `impl` block",
            "performing any of the behaviors below, so none of these requirements can be",
            "discharged by this struct. Each was removed rather than left to imply coverage.",
        ],
    },
    {
        "path": "crates/engine/src/engine.rs",
        "anchor": "pub struct Simulation {",
        "removed": SIMULATION_REMOVED,
        "summary": "removed %d tags from the Simulation aggregate",
        "note": [
            "`Simulation` is the tick-loop owner. The requirements below are implemented",
            "elsewhere or not at all, so a tag on the struct claims coverage that does",
            "not exist at the tagged symbol.",
        ],
    },
    {
        "path": "crates/server/src/session.rs",
        "anchor": "pub const SESSION_HISTORY_CAP",
        "removed": SESSION_REMOVED,
        "summary": "removed %d tags from SESSION_HISTORY_CAP",
        "note": [
            "`SESSION_HISTORY_CAP` is a ring-buffer size for the audit log. The PvE session",
            "requirements below need turn tokens, hot-seat, observers, challenge HTTP routes,",
            "UUIDv7 ids, and autosave timers; none exist in this file, and the crate has no",
            "database layer. `SESSION_HISTORY_CAP` is 32, a tuning constant, not a session model.",
        ],
    },
]

ID_RE = re.compile(r"\b(?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*\b")


def collect_ids(text: str) -> set[str]:
    return set(ID_RE.findall(text))


def is_tag_line(line: str) -> bool:
    """A line that is purely a requirement-tag comment."""
    s = line.strip()
    return s.startswith("//") and bool(ID_RE.search(line))


def is_doc_line(line: str) -> bool:
    """A rustdoc comment line, which may sit between tags and the declaration."""
    s = line.strip()
    return s.startswith("///") and not is_tag_line(line)


def is_attr_line(line: str) -> bool:
    """A derive/attribute line, which may sit between the tags and the struct."""
    return line.strip().startswith("#[") or line.strip().startswith("#![")


def process(site: dict, apply: bool) -> tuple[str, int, int]:
    path = REPO / site["path"]
    lines = path.read_text(encoding="utf-8").split("\n")

    # Find the declaration line.
    try:
        decl_idx = next(i for i, l in enumerate(lines) if site["anchor"] in l)
    except StopIteration:
        print(f"  ANCHOR NOT FOUND: {site['anchor']} in {site['path']}")
        return "\n".join(lines), 0, 0

    # Walk upward from the declaration, allowing doc-comment and attribute lines,
    # and collect the contiguous run of tag lines. The run ends at the first
    # line that is none of those three.
    def skippable(line: str) -> bool:
        return is_doc_line(line) or is_tag_line(line) or is_attr_line(line)

    tags_start = None
    i = decl_idx - 1
    while i >= 0 and skippable(lines[i]):
        if is_tag_line(lines[i]):
            # Walking upward, so each new tag found is at a *lower* index. Keep
            # overwriting so tags_start ends at the topmost tag of the block.
            tags_start = i
        i -= 1
    if tags_start is None:
        print(f"  no tag block above {site['anchor']}")
        return "\n".join(lines), 0, 0

    # Idempotence: this tool writes a marker comment for every removed tag, and
    # those lines themselves contain requirement ids, so a re-run would treat
    # its own output as a fresh tag block. The marker can be separated from the
    # declaration by a section banner (the walk above stops at `// ====...`),
    # so search the whole file for the anchor-specific marker instead.
    marker = f"// requirement tags were removed from {site['anchor'].rstrip(' {')}"
    if any(marker in l for l in lines):
        print(f"  already processed: {site['anchor']}")
        return "\n".join(lines), 0, 0

    start = tags_start
    # The block spans from the first tag line through the line just above the
    # declaration. That range includes any `#[derive(..)]` and doc lines that
    # sit between the tags and the struct, so they are preserved in place.
    end = decl_idx
    block = lines[start:end]
    present = collect_ids("\n".join(block))
    removable = sorted(present & set(site["removed"]))
    unknown = sorted(present - set(site["removed"]))

    if unknown:
        print(f"  WARNING unclassified tags left in place: {unknown}")

    if not removable:
        print(f"  nothing removable above {site['anchor']}")
        return "\n".join(lines), 0, 0

    # Keep every line that is not purely a removed tag. A line counts as
    # "removed" only when every requirement id on it is in the removal set, so a
    # mixed line is preserved, and attribute/doc lines (which carry no ids at
    # all) are never treated as removable.
    def is_removable_line(line: str) -> bool:
        ids = collect_ids(line)
        return bool(ids) and ids <= set(site["removed"])

    kept = [l for l in block if not is_removable_line(l)]
    reason_lines = [f"// {tag}: {site['removed'][tag]}" for tag in removable]

    # Rustdoc and attributes must sit immediately above the declaration, so the
    # explanatory header goes first and every surviving doc/attribute line stays
    # last, in its original order. Partition by index rather than by value so
    # repeated identical lines are not collapsed.
    trailing_idx = {i for i, l in enumerate(kept) if is_doc_line(l) or is_attr_line(l)}
    split = min(trailing_idx) if trailing_idx else len(kept)
    leading = kept[:split]
    trailing = kept[split:]

    header = [
        f"// The following {len(removable)} requirement tags were removed from"
        f" {site['anchor'].rstrip(' {')}",
        "// declaration. They are not implemented at this symbol, and leaving them",
        "// here claimed coverage that no code in this repository provides.",
    ]
    header += [f"// {line}" for line in site["note"]]
    header += [
        "//",
        "// Removed, with the reason each cannot be discharged here:",
    ]
    new_block = header + reason_lines
    if leading:
        new_block.append("//")
        new_block.append("// Tags that remain and why they stay:")
        new_block += leading
    new_block += trailing

    new_lines = lines[:start] + new_block + lines[end:]
    if apply:
        path.write_text("\n".join(new_lines), encoding="utf-8", newline="\n")
    print(f"  {site['summary'] % len(removable)} (kept {len(kept)} tag line(s))")
    return "\n".join(new_lines), len(removable), len(kept)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()

    total_removed = 0
    for site in SITES:
        print(f"{site['path']} :: {site['anchor']}")
        _, removed, _ = process(site, args.apply)
        total_removed += removed
    print(f"\ntotal removed: {total_removed}")
    if not args.apply:
        print("(preview only; pass --apply to write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
