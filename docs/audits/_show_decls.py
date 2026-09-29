"""Print the comment block above each of the given declarations.

Usage:  python docs/audits/_show_decls.py path/to/file.rs Name1 Name2

The point is to judge a tag against the symbol it sits on, so this shows the tag
lines themselves and a generous slice of the body.
"""
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
DECL = r"pub(?:\([^)]*\))?\s+(?:struct|enum|const|static|type|fn)\s+{}\b"


def show(path, name, body=30):
    lines = (REPO / path).read_text(encoding="utf-8").split("\n")
    pat = re.compile(DECL.format(re.escape(name)))
    idx = next((i for i, l in enumerate(lines) if pat.match(l.strip())), None)
    if idx is None:
        print(f"!! {path} :: {name} not found")
        return
    start = idx
    i = idx - 1
    while i >= 0:
        s = lines[i].strip()
        if s.startswith(("//", "#[", "pub", "    ")) or s == "":
            start = i
            i -= 1
            continue
        break
    print(f"--- {path} :: {name} (line {idx + 1})")
    for n in range(start, min(len(lines), idx + body)):
        print(f"{n + 1:5d}| {lines[n]}")
    print()


if __name__ == "__main__":
    show(sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 30)
