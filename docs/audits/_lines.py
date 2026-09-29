"""Print a line range of a file:  python docs/audits/_lines.py <file> <start> <end>"""
import pathlib
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
path, a, b = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
for n, line in enumerate(REPO.joinpath(path).read_text(encoding="utf-8").split("\n")[a - 1 : b], a):
    print(f"{n:5d}| {line}")
