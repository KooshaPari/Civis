"""Report which ids sit on each declaration a verdict module names as false.

`_apply_verdicts.py` refuses to touch a declaration carrying an id it has not
been told about, because an unclassified id is either a mistake in the verdict
module or a real tag that has not been adjudicated yet. This lists the unclassified
ones per site so each can be given an explicit verdict, kept or removed, rather
than being waved through by a blanket rule.
"""
import importlib.util
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
ID_RE = re.compile(r"\b(?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*\b")
TOKEN = "[unbound]"


def load(n):
    p = pathlib.Path("docs/audits/" + n + ".py")
    s = importlib.util.spec_from_file_location(p.stem, p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m


for name in sys.argv[1:]:
    mod = load(name)
    keep = set(getattr(mod, "KEEP", {}))
    print("#" * 20, name)
    for rel, decl, removed, _note in mod.SITES:
        path = REPO / rel
        lines = path.read_text(encoding="utf-8").split("\n")
        idx = next(
            (
                i
                for i, l in enumerate(lines)
                if re.match(
                    r"pub(?:\([^)]*\))?\s+(?:struct|enum|const|static|type)\s+"
                    + re.escape(decl)
                    + r"\b",
                    l.strip(),
                )
            ),
            None,
        )
        if idx is None:
            print(f"  {rel} :: {decl}  DECL NOT FOUND")
            continue
        # Walk the contiguous comment block above the declaration. The walk stops
        # at the `[unbound]` marker a removal pass leaves behind: everything from
        # that comment downward is rationale prose naming ids that are already
        # gone, not live tags, and counting them would report every removed id as
        # unclassified. The live block is only what sits between the end of that
        # rationale and the declaration.
        start = idx
        i = idx - 1
        while i >= 0:
            s = lines[i].strip()
            if not (s.startswith(("///", "//", "#["))):
                break
            start = i
            i -= 1
        # A removal pass leaves a marker comment naming how many tags it took off
        # this declaration. Everything from that marker down to the declaration
        # is rationale prose that repeats the ids it removed; counting them here
        # would report every already-unbound id as unclassified. The live block
        # is the contiguous comment run *above* the marker.
        marker = next(
            (n for n in range(start, idx) if TOKEN in lines[n]),
            None,
        )
        live_stop = marker if marker is not None else idx
        present = set(ID_RE.findall("\n".join(lines[start:live_stop])))
        unclassified = sorted(present - set(removed) - keep)
        if unclassified:
            print(f"  {rel} :: {decl}")
            print(f"     false:     {sorted(set(removed) - {'__keep__'})}")
            print(f"     UNCLASSIFIED: {unclassified}")
    print()
