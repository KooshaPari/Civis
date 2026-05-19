#!/usr/bin/env python3
"""
Pattern #233 enforcement: BepInEx plugins must multi-target netstandard2.0;net8.0 (or
netstandard2.1;net8.0 WITH inline marker).

Detects:
- csproj with BepInEx PackageReference or InternalsVisibleTo to BepInEx
- AND has [BepInPlugin] attribute somewhere in the project's .cs files
- AND TargetFramework is not multi-target (netstandard2.0;net8.0) or (netstandard2.1;net8.0)

Accepts netstandard2.1 ONLY if marked with:
- C#: // pattern-233-ok: netstandard2.1 in a .cs file
- XML: <!-- pattern-233-ok: netstandard2.1 --> in csproj

Reports HIGH for violations, MED for unmarked netstandard2.1.
"""
import sys
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
SRC = REPO / "src"
PREFERRED_TFM = "netstandard2.0;net8.0"
ACCEPTABLE_TFM = "netstandard2.1;net8.0"
MARKER_CS = "// pattern-233-ok: netstandard2.1"
MARKER_CSPROJ = "<!-- pattern-233-ok: netstandard2.1 -->"

def main():
    violations = []
    for csproj in SRC.rglob("*.csproj"):
        content = csproj.read_text(encoding="utf-8", errors="replace")
        # Find csproj files that look like BepInEx plugins
        is_bepinex_plugin = (
            "BepInEx" in content
            and any(
                "[BepInPlugin]" in cs.read_text(encoding="utf-8", errors="replace")
                for cs in csproj.parent.rglob("*.cs")
                if cs.exists()
            )
        )
        if not is_bepinex_plugin:
            continue
        # Check TFM
        tfs = re.search(r"<TargetFrameworks>([^<]+)</TargetFrameworks>", content)
        tf = re.search(r"<TargetFramework>([^<]+)</TargetFramework>", content)
        actual = tfs.group(1) if tfs else (tf.group(1) if tf else "<missing>")
        if actual == PREFERRED_TFM:
            continue
        if actual == ACCEPTABLE_TFM:
            # Check for marker in csproj or any .cs file
            has_marker = (
                MARKER_CSPROJ in content
                or any(
                    MARKER_CS in cs.read_text(encoding="utf-8", errors="replace")
                    for cs in csproj.parent.rglob("*.cs")
                    if cs.exists()
                )
            )
            if not has_marker:
                violations.append((csproj.relative_to(REPO), actual, "MED"))
            continue
        violations.append((csproj.relative_to(REPO), actual, "HIGH"))
    if violations:
        high_count = sum(1 for _, _, sev in violations if sev == "HIGH")
        med_count = len(violations) - high_count
        print(f"Pattern #233 violations ({high_count} HIGH, {med_count} MED):")
        for path, tfm, sev in violations:
            if sev == "HIGH":
                print(f"  HIGH  {path}: TFM={tfm} (required: {PREFERRED_TFM})")
            else:
                print(f"  MED   {path}: TFM={tfm} (accepted with marker)")
        sys.exit(1 if high_count > 0 else 0)
    print("Pattern #233: 0 violations.")
    sys.exit(0)

if __name__ == "__main__":
    main()
