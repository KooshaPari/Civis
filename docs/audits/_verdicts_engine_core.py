"""Machine-readable transcription of the FALSE verdicts in
``docs/audits/triage-container-engine-core.md``.

Verdict values that are FALSE: ``CONTAINER-ONLY`` and ``NOT-IMPLEMENTED``.
Verdict values deliberately excluded as legitimate: ``DATA-SHAPE-ONLY`` and
``IMPLEMENTED-BY-BEHAVIOR``.

The report was written before an earlier commit unbound 70 false tags from
``WorldState`` and ``SESSION_HISTORY_CAP``. Only rows whose id is still tagged
on a data container today are transcribed below; the rest are listed in
``ALREADY_REMOVED`` for the self-check arithmetic.

Running this module as a script prints the self-check and exits 0.
"""

# ---------------------------------------------------------------------------
# Report accounting. The report's own verdict-summary table and its row tables
# disagree with each other. Both figures are carried here so nothing is
# silently dropped; see _self_check() for how the discrepancy is reported.
# ---------------------------------------------------------------------------
# Section headers claim: FR-SOC 23, FR-PROT 4, ARCH/NFR 6, CORE 13.
SUMMARY_TABLE_FALSE_TOTAL = 32  # 18 CONTAINER-ONLY + 14 NOT-IMPLEMENTED
# The row tables actually contain: 23 + 4 + 4 + 13 = 44 verdict rows, of which
# 33 carry a FALSE verdict (10 CONTAINER-ONLY + 23 NOT-IMPLEMENTED).
ROW_TABLE_FALSE_TOTAL = 33
ROW_TABLE_TOTAL = 47

# ---------------------------------------------------------------------------
# FALSE rows in the report that no longer exist as live tags on a container.
# Keyed by id; the value is the note explaining what happened to the tag.
# ---------------------------------------------------------------------------
ALREADY_REMOVED = {
    "FR-SOC-CIV-001": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-CIV-002": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-COH-001": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-COH-002": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-COH-003": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-COH-004": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-001": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-002": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-003": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-004": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-005": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-006": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INS-007": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INT-001": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INT-002": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INT-003": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INT-004": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-001": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-002": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-003": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-004": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-005": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-006": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-SOC-INTG-007": (
        "Already unbound from the WorldState tag block; the only remaining "
        "mentions are the per-id doc comments in the stub tests under "
        "crates/engine/tests/."
    ),
    "FR-CIV-ARCH-006": (
        "Already unbound from the WorldState tag block; the surviving mention at "
        "engine.rs:389 is a removal marker, not a live tag."
    ),
    "NFR-CIV-PERF-002": (
        "Already unbound from the WorldState tag block; the surviving mention at "
        "engine.rs:423 is a removal marker, not a live tag."
    ),
    "FR-CIV-CORE-004": (
        "Already unbound from the WorldState tag block; the surviving mention at "
        "engine.rs:392 is a removal marker, not a live tag."
    ),
    "FR-CIV-CORE-019": (
        "Already unbound from the WorldState tag block; the surviving mention at "
        "engine.rs:393 is a removal marker, not a live tag."
    ),
    "FR-CIV-PERF-RT-003": (
        "Already unbound from the WorldState tag block; the surviving mention at "
        "engine.rs:394 is a removal marker, not a live tag. The report cites no "
        "spec file:line for this row, marks it 'listed for completeness', and it "
        "is the only FALSE row whose id does not start with one of the five scope "
        "prefixes, so it is counted but kept out of the scope total."
    ),
}

# FALSE rows in the report whose current source state could not be decided
# automatically from this file alone. Empty: every FALSE row was classified.
UNCLASSIFIED = []

# ---------------------------------------------------------------------------
# SITES: only the FALSE rows that are still tagged on a data container today.
# ---------------------------------------------------------------------------
SITES = [
    (
        "crates/engine/src/command_queue.rs",
        "CommandQueue",
        {
            "FR-CIV-CORE-008": (
                "Spec docs/specs/CIV-0001-core-simulation-loop.md:902-905 requires "
                "\"Commands from multiple clients applied in deterministic order "
                "(priority queue)\", tested by \"Issue 10 commands from 3 clients, "
                "verify order matches priority + FIFO\". CommandQueue is a plain "
                "VecDeque<Command> FIFO with only a max_pending bound "
                "(command_queue.rs:31-34); Command (command_queue.rs:5-10) has no "
                "priority field, and push/pop (command_queue.rs:58,72) only "
                "push_back/pop_front. The real implementing symbol is "
                "CommandQueue::push / CommandQueue::pop -- a FIFO queue that cannot "
                "produce priority ordering; the test at "
                "crates/engine/tests/fr_fr_civ_core_008.rs:14 asserts FIFO only, and "
                "no production code constructs this type."
            ),
        },
        [
            "The tag sits directly above `pub struct CommandQueue {` at line 31 and "
            "is still live in the source today.",
            "CommandQueue is referenced only from its own unit tests and from "
            "integration tests; no production path in the workspace uses it.",
        ],
    ),
    (
        "crates/engine/src/command_queue.rs",
        "CommandQueue",
        {
            "FR-CIV-CORE-016": (
                "Spec docs/specs/CIV-0001-core-simulation-loop.md:942-945 requires "
                "\"Commands prioritized by (client_priority, tick_received)\", tested "
                "by \"Issue conflicting commands from different priority clients, "
                "verify higher priority wins\". No priority field exists: Command "
                "(command_queue.rs:5-10) carries only client_id, seq, kind and "
                "tick_issued, and client_priority has exactly one repo-wide "
                "occurrence, the doc comment of "
                "crates/engine/tests/fr_fr_civ_core_016.rs:6. No implementing symbol "
                "exists -- there are no priority tiers, no priority field and no "
                "sorting anywhere. The only ordering artifact is the FIFO "
                "CommandQueue::push / CommandQueue::pop pair, which returns the "
                "first-pushed command regardless of priority; the test at "
                "crates/engine/tests/fr_fr_civ_core_016.rs:14 asserts plain FIFO, the "
                "opposite of the required priority ordering."
            ),
        },
        [
            "The tag sits in the same block as the multi-client ordering tag, "
            "directly above `pub struct CommandQueue {` at line 31, and is still live.",
            "The report ranks this the single highest-confidence false tag: the only "
            "test asserting the tagged behavior asserts the opposite of the "
            "requirement.",
        ],
    ),
    (
        "crates/engine/src/engine.rs",
        "Simulation",
        {
            "FR-CIV-CORE-014": (
                "Spec docs/specs/CIV-0001-core-simulation-loop.md:932-935 requires "
                "\"Every state-mutating action emits event to log\", tested by "
                "\"Verify event count > 0 per tick; replay matches event log\". No "
                "unified event log exists: events are spread across many per-tick "
                "buffers, e.g. Simulation::last_tick_voxel_events "
                "(crates/engine/src/engine.rs:819, accessor at line 1911) and the "
                "chronicle (crates/engine/src/engine.rs:563). Simulation is a passive "
                "aggregate with no impl block appending to a single log, so the tagged "
                "struct merely holds the buffers and cannot discharge the requirement; "
                "the real implementing symbol is none."
            ),
        },
        [
            "The tag is listed in the 'Tags that remain and why they stay' block "
            "above `pub struct Simulation {` at crates/engine/src/engine.rs:746, so "
            "it is still live today.",
            "An earlier commit already unbound two other tags from this same "
            "declaration; this one survived that pass.",
        ],
    ),
    (
        "crates/engine/src/hash_chain.rs",
        "HashChainState",
        {
            "FR-CIV-CORE-015": (
                "Spec docs/specs/CIV-0001-core-simulation-loop.md:937-940 requires "
                "\"Every event includes hash of state that produced it\", tested by "
                "\"Replay event, verify state hash matches; mismatch -> error\". "
                "Hash-chain machinery is real but it is per-tick, not per-event: "
                "HashChainState holds a single running_hash "
                "(crates/engine/src/hash_chain.rs:44-46), "
                "HashChainState::advance (line 58) folds only tick_event_bytes, and "
                "chain_root_from_ticks (line 67) chains bare little-endian tick "
                "counters. The hash is read out as one whole-run root via "
                "Simulation::hash_chain_root (crates/engine/src/engine.rs:2940) and "
                "surfaced as a single snapshot field "
                "(crates/server/src/jsonrpc.rs:493), so no event object ever carries "
                "the hash of the state that produced it. The implementing symbol is "
                "HashChainState::advance / chain_root_from_ticks, a run-level chain "
                "that cannot stamp individual events."
            ),
        },
        [
            "The tag sits directly above `pub struct HashChainState {` at line 44 "
            "and is still live in the source today.",
            "A sibling requirement with nearly the same wording was independently "
            "unbound elsewhere in the workspace because the priority field does not "
            "exist anywhere.",
        ],
    ),
]


def self_check():
    """Return the self-check report as an ordered list of (label, count, detail)."""
    transcribed = sum(len(site[2]) for site in SITES)
    removed = len(ALREADY_REMOVED)
    accounted = transcribed + removed + len(UNCLASSIFIED)
    return [
        ("FALSE rows per the report's verdict summary table", SUMMARY_TABLE_FALSE_TOTAL,
         "the report claims 18 CONTAINER-ONLY + 14 NOT-IMPLEMENTED"),
        ("FALSE rows actually written in the report's row tables", ROW_TABLE_FALSE_TOTAL,
         "10 CONTAINER-ONLY + 23 NOT-IMPLEMENTED out of %d verdict rows" % ROW_TABLE_TOTAL),
        ("of those, in scope for this transcription", ROW_TABLE_FALSE_TOTAL - 1,
         "excludes the completeness-only row with no scope prefix"),
        ("already removed from source by an earlier commit", removed,
         "unbound from WorldState by the earlier pass"),
        ("transcribed as live tags in SITES", transcribed,
         "each sits above a named Rust declaration"),
        ("could not be classified", len(UNCLASSIFIED),
         "nothing was dropped silently"),
        ("accounted for (transcribed + removed + unclassified)", accounted,
         "must equal the row-table count of %d" % ROW_TABLE_FALSE_TOTAL),
    ]


def main():
    transcribed = sum(len(site[2]) for site in SITES)
    removed = len(ALREADY_REMOVED)
    accounted = transcribed + removed + len(UNCLASSIFIED)

    lines = [
        "container-binding triage: engine / core / social / protocol",
        "",
        "The report's verdict-summary table and its row tables disagree; both are",
        "reported below so the difference is visible rather than smoothed over.",
        "",
    ]
    for label, count, detail in self_check():
        lines.append("%-52s %3d   %s" % (label + ":", count, detail))
    lines.append("")
    if accounted != ROW_TABLE_FALSE_TOTAL:
        lines.append(
            "SELF-CHECK FAILED: %d accounted for but %d FALSE rows in the row tables"
            % (accounted, ROW_TABLE_FALSE_TOTAL)
        )
        lines.append("")
        return 1

    lines.append("(file, declaration) pairs transcribed:")
    for path, decl, tags, _notes in SITES:
        lines.append("  %-32s %-14s -> %s" % (path, decl, ", ".join(sorted(tags))))
    if UNCLASSIFIED:
        lines.append("")
        lines.append("rows that could not be classified:")
        for item in UNCLASSIFIED:
            lines.append("  %s" % (item,))
    lines.append("")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
