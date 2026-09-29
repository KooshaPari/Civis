"""Rename test files that claim to cover requirement IDs no spec defines.

`FR-CIV-RTS-RENDER-*`, `FR-CIV-RTS-ZOOM-*` and `FR-CIV-RTS-NATION-*` exist only
in the middle column of CIV-0600's §14 traceability table, which uses them as
"verification owner" labels. No spec defines them: CIV-0300 §12.1 owns
`FR-CIV-RTS-001..015` and never mentions the RENDER/ZOOM/NATION sub-namespaces.

The assertions in these files are real and are preserved verbatim. Only the
names change, so a reader no longer believes an ID with no requirement behind it
is covered. The new file name and module describe the behavior under test.

Usage:  python docs/audits/_rename_unverifiable_tests.py [--apply]
"""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
TEST_DIR = REPO / "crates/engine/tests"

# old stem -> (new stem, what the test actually asserts)
RENAMES = {
    "fr_fr_civ_rts_render_001": ("rts_supersampling_render_size", "SsConfig render dimensions"),
    "fr_fr_civ_rts_render_002": ("rts_supersampling_4x_ratio", "4x supersampling dimension ratio"),
    "fr_fr_civ_rts_render_003": ("rts_nation_hex_parsing", "NationColor hex parsing"),
    "fr_fr_civ_rts_render_004": ("rts_atlas_power_of_two", "atlas config dimensions are pow2"),
    "fr_fr_civ_rts_render_005": ("rts_uv_rect_containment", "UvRect fit and overlap predicates"),
    "fr_fr_civ_rts_zoom_001": ("rts_sprite_handle_zoom", "SpriteHandle::set_zoom state update"),
    "fr_fr_civ_rts_nation_001": ("rts_nation_color_rgba", "NationColor parse to RGBA"),
    "fr_fr_civ_rts_nation_002": ("rts_nation_shader_tolerance", "color_matches tolerance"),
}

# The IDs each file used to claim, in the order they appear in its header.
CLAIMED = {
    "fr_fr_civ_rts_render_001": "FR-CIV-RTS-RENDER-001",
    "fr_fr_civ_rts_render_002": "FR-CIV-RTS-RENDER-002",
    "fr_fr_civ_rts_render_003": "FR-CIV-RTS-RENDER-003",
    "fr_fr_civ_rts_render_004": "FR-CIV-RTS-RENDER-004",
    "fr_fr_civ_rts_render_005": "FR-CIV-RTS-RENDER-005",
    "fr_fr_civ_rts_zoom_001": "FR-CIV-RTS-ZOOM-001",
    "fr_fr_civ_rts_nation_001": "FR-CIV-RTS-NATION-001",
    "fr_fr_civ_rts_nation_002": "FR-CIV-RTS-NATION-002",
}

BANNER = """\
//! Behavior tests for `{desc}`.
//!
//! These assertions are real and were previously filed under `{claimed}`.
//! That ID is not a requirement: the only place it appears in the repository
//! is the middle column of CIV-0600's §14 traceability table
//! (`docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3205-3224`), which uses it
//! as a "verification owner" label for a test file that does not exist.
//! CIV-0300 §12.1 owns `FR-CIV-RTS-001..015` and never mentions the
//! RENDER/ZOOM/NATION sub-namespaces.
//!
//! So the ID was dropped rather than satisfied: there is no requirement text to
//! implement, and inventing one would be the same defect in a new place. The
//! behavior is still worth a test, so the test stays under a name that says
//! what it checks.
"""


def rewrite_header(text: str, desc: str, claimed: str) -> str:
    """Replace the old //! header block with the honest banner."""
    lines = text.splitlines()
    i = 0
    while i < len(lines) and (lines[i].startswith("//!") or not lines[i].strip()):
        i += 1
    body = "\n".join(lines[i:])
    return BANNER.format(desc=desc, claimed=claimed) + "\n" + body


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="write changes to disk")
    args = ap.parse_args()

    for old_stem, (new_stem, desc) in RENAMES.items():
        src = TEST_DIR / f"{old_stem}.rs"
        if not src.exists():
            print(f"SKIP  {src.name} (missing)")
            continue
        text = src.read_text(encoding="utf-8")
        claimed = CLAIMED[old_stem]
        new_text = rewrite_header(text, desc, claimed)
        # Rename the inner module and the test fn so nothing still says the ID.
        new_text = re.sub(rf"\bmod {re.escape(old_stem)}\b", f"mod {new_stem}", new_text)
        new_text = re.sub(
            rf"fn verify_{re.escape(old_stem)}_basic\b",
            f"fn {new_stem}_behavior",
            new_text,
        )
        # Drop the now-duplicated `/// FR-...` line above the test.
        new_text = re.sub(rf"^\s*/// {re.escape(claimed)} --.*\n", "", new_text, flags=re.M)
        dst = TEST_DIR / f"{new_stem}.rs"

        if args.apply:
            dst.write_text(new_text, encoding="utf-8")
            src.unlink()
        print(f"{'RENAMED' if args.apply else 'WOULD RENAME'}  {src.name} -> {dst.name}  ({desc})")

    leftover = subprocess.run(
        ["git", "grep", "-l", "-E", r"FR-CIV-RTS-(RENDER|ZOOM|NATION)-[0-9]+", "--", "crates", "clients"],
        capture_output=True,
        text=True,
        cwd=REPO,
    ).stdout.split()
    print(f"\nfiles still referencing the invented sub-namespaces: {len(leftover)}")
    for f in leftover:
        print(f"  {f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
