"""Print the reasons recorded in a verdict module, read-only.

Usage:  python docs/audits/_peek.py _verdicts_tactics [n_sites]

A reason is either a single string or a tuple of lines forming one paragraph;
verdict modules use both, so this accepts either instead of assuming today's
shape. `__keep__` is printed as the ids that stay rather than a removal.
"""
import importlib.util
import pathlib
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def load(name):
    p = pathlib.Path("docs/audits/" + name + ".py")
    spec = importlib.util.spec_from_file_location(p.stem, p)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def as_text(value):
    return value if isinstance(value, str) else " ".join(value)


def main():
    names = sys.argv[1:]
    limit = 3
    if names and names[-1].isdigit():
        limit = int(names.pop())
    for name in names:
        mod = load(name)
        keep = getattr(mod, "KEEP", {})
        print("#" * 20, name, f"sites: {len(mod.SITES)}  keeps: {len(keep)}")
        for rel, decl, rem, note in mod.SITES[:limit]:
            print("  --", rel, "::", decl)
            for k, v in rem.items():
                if k == "__keep__":
                    print("      KEEPS:", list(v))
                    continue
                print("     ", k, "->", " ".join(as_text(v).split())[:400])
            for ln in note[:2]:
                print("      note:", " ".join(as_text(ln).split())[:180])
        print()


if __name__ == "__main__":
    main()
