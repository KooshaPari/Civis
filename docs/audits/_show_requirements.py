"""Print the authoritative requirement text for the given requirement ids.

Usage:  python docs/audits/_show_requirements.py FR-ECON-005 FR-CIV-3D-011 ...

Looks only in the authoritative spec roots the detector also treats as normative
(agileplus-specs/, docs/specs/, docs/design/, docs/guides/, docs/traceability/),
so a hit in a generated audit report cannot be mistaken for a definition.
"""
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
ROOTS = ("agileplus-specs", "docs/specs", "docs/design", "docs/guides", "docs/traceability")
ID_RE = re.compile(r"\b(?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*\b")
PIPE = re.compile(r"^\s*\|")


def is_defn(line: str) -> bool:
    """True when the line reads as a requirement definition rather than prose."""
    ids = ID_RE.findall(line)
    if not ids:
        return False
    stripped = line.strip()
    # Markdown table row, or a bullet/number that leads with the id.
    if PIPE.match(line) or stripped.startswith(("-", "*", "#")):
        return True
    return stripped.startswith(ids[0])


for req in sys.argv[1:]:
    print("=" * 70)
    print(req)
    hits = 0
    for root in ROOTS:
        base = REPO / root
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*")):
            if path.suffix.lower() not in (".md", ".txt", ".yaml", ".yml"):
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace").split("\n")
            except OSError:
                continue
            for n, line in enumerate(text, 1):
                if req in line and is_defn(line):
                    rel = path.relative_to(REPO).as_posix()
                    print(f"  {rel}:{n}")
                    print("    " + " ".join(line.split())[:400])
                    hits += 1
                    break
    if not hits:
        print("  !! no authoritative definition found")
    print()
