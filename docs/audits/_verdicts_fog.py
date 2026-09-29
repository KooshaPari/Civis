"""Verdicts for the FR-CIV-FOG-* block on `FogOfWar` (crates/tactics/src/fog_of_war.rs).

Five ids sit in one tag block above `pub struct FogOfWar`, and only the first is
discharged by the type and its `impl`. The other four describe behavior in three
other subsystems. Each was checked against the source before being written here,
and the verdict differs per id: two are implemented elsewhere, one is partially
implemented, and one is not implemented at all. The point of the per-id split is
that "misplaced" and "false" are different findings and only the last justifies
removing the tag without a replacement.

Spec: agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47-64.
All five are unchecked `- [ ]` boxes in that spec, which is a separate matter
from where the code lives; see the note lines.
"""

SITES = [
    (
        "crates/tactics/src/fog_of_war.rs",
        "FogOfWar",
        {
            "FR-CIV-FOG-002": (
                "MIS-BOUND, IMPLEMENTED ELSEWHERE. The requirement is that the war "
                "bridge SHALL NOT queue DamageEvents for engagements where the "
                "attacker has no friendly unit with line of sight to the defender "
                "(civ-015 spec:51-53). That is implemented and tested, at "
                "WarBridge::resolve_combat in crates/tactics/src/war_bridge.rs, which "
                "gates engagement on fog.is_visible(shooter.faction_id, cell) at "
                "war_bridge.rs:151, with tests attack_blocked_when_no_los, "
                "attack_succeeds_with_clear_los, and fog_blocks_engagement_beyond_vision. "
                "The behavior carries FR-CIV-TACTICS-042, the id the spec names for the "
                "same gate, so the coverage is not lost when the tag moves off "
                "FogOfWar. A visibility grid cannot decide which damage to queue."
            ),
            "FR-CIV-FOG-003": (
                "MIS-BOUND, IMPLEMENTED ELSEWHERE. The requirement is that scenario "
                "fog_vision_radius SHALL be respected per-civilisation and that the "
                "default baseline scenario set it to 4 hex for at least one side "
                "(civ-015 spec:54-56). ScenarioConfig.fog_vision_radius is parsed at "
                "crates/engine/src/scenario.rs:101 and wired into the war bridge via "
                "sim.configure_military_fog at scenario.rs:326, with round-trip tests at "
                "scenario.rs:545 and :579. The per-civilisation and default-baseline-4 "
                "halves of the requirement are not separately exercised, so this is "
                "partial coverage, but the field is not a FogOfWar field and the "
                "scenario does the work."
            ),
            "FR-CIV-FOG-004": (
                "NOT IMPLEMENTED. The requirement is that the web dashboard SHALL "
                "provide a tactics panel with unit selection, an at-a-glance fog "
                "overlay, and a jump-to-engagement action, reading from sim.snapshot "
                "JSON-RPC only (civ-015 spec:57-60). The only tactics reference under "
                "web/dashboard/src is a formation POST at bottom_bar.tsx:806; there is "
                "no tactics panel, no unit selection, no fog overlay, and no "
                "jump-to-engagement action. A grid of visibility bits is not a UI."
            ),
            "FR-CIV-FOG-005": (
                "NOT IMPLEMENTED. The requirement is fog-of-war observer mode applying "
                "server-side visibility filtering before transmission, with a stable "
                "per-nation filter (civ-015 spec:61-64). crates/server/src/session.rs "
                "records the opposite at lines 46-50: no observer flag exists, no "
                "observer RPCs exist, and there is no omniscient mode field. The session "
                "layer therefore cannot apply the filter the requirement describes, and "
                "there is no other observer-mode code."
            ),
        },
        [
            "Five ids were stacked in one tag block above the struct. Only the first is",
            "discharged by the struct and its impl. The verdicts differ per id on purpose:",
            "two are implemented in other subsystems and only the binding is wrong, one is",
            "partially implemented, and one does not exist anywhere. Collapsing those into",
            "a single 'false tag' verdict would have thrown away real coverage.",
            "",
            "All five ids are unchecked boxes in the civ-015 spec. That is a coverage fact",
            "worth recording separately, not a reason to drop a tag whose implementation is",
            "real and tagged elsewhere.",
        ],
    ),
]

# Deliberately kept, with the reason.
KEEP = {
    "FR-CIV-FOG-001": (
        "Visibility is genuinely a function of unit position, vision radius, and terrain "
        "line of sight, recomputed from scratch on every update, and FogOfWar::update and "
        "FogOfWar::is_visible (fog_of_war.rs:66,128) are the artifacts that do it. The tag "
        "is correct."
    ),
}
