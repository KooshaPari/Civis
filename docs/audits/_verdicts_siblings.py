"""Verdicts for sibling ids that appeared on blocks already being edited.

The three cluster auditors each surfaced a tag they were not asked about, sitting
on the same declaration as an id they were. In every case the sibling turned out
to be either a real binding or a family-level id rather than a per-symbol one,
and in neither case should it come off alongside the id that was in scope.

  FR-CIV-DIPLOMACY  on DiplomacyEvent. Used in 20+ places across the diplomacy
                    crate and engine as a family label for the whole emergent
                    diplomacy feature. It is not a per-symbol requirement, so a
                    tag on any one struct is not a false claim so much as a
                    category marker. Removing it from this struct would not make
                    the family id any more or less true, and the block reads
                    better with the category present. KEEP.

  NFR-C-03          on Fixed. Has no authoritative definition: it appears only
                    in docs/traceability/index.md:1153, in a generated per-id
                    folder, and in a gap report. The NFR doc that does exist
                    defines the same enforcement under NFR-CIV-DET-003, which is
                    the id the engine-core auditor is removing. The doc comment
                    at fixed_math.rs:8-11 describes NFR-C-03's content in full,
                    so the reference is intelligible even though the id has no
                    spec of its own. KEEP, and flagged: an undefined id that
                    describes itself accurately in a doc comment is a traceability
                    gap, not a false binding.
"""

KEEP = {
    "FR-CIV-DIPLOMACY": (
        "family label for the emergent-diplomacy feature, used in 20+ places "
        "across crates/diplomacy and crates/engine; a category marker rather "
        "than a per-symbol requirement, so it is not false on DiplomacyEvent"
    ),
    "NFR-C-03": (
        "no authoritative spec of its own; the enforcement it describes is "
        "defined as NFR-CIV-DET-003, and the doc comment at fixed_math.rs:8-11 "
        "states the content accurately, so this is a traceability gap rather "
        "than a false binding"
    ),
}

SITES = []
