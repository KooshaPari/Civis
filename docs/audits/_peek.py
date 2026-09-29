import importlib.util
import pathlib
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def load(n):
    p = pathlib.Path("docs/audits/" + n + ".py")
    s = importlib.util.spec_from_file_location(p.stem, p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m


for n in sys.argv[1:]:
    m = load(n)
    print("#" * 20, n, "sites:", len(m.SITES))
    n_sites = int(sys.argv[0] and 0) or 3
    for rel, decl, rem, note in m.SITES[:n_sites]:
        print("  --", rel, "::", decl)
        for k, v in list(rem.items())[:3]:
            flat = " ".join(v.split())
            print("     ", k, "->", flat[:340])
        for ln in note[:2]:
            print("      note:", " ".join(ln.split())[:150])
    print()
