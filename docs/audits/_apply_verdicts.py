"""Apply false-binding verdicts recorded in docs/audits/_verdicts_*.py.

Each verdict module exposes `SITES`, a list of

    (repo-relative file, Rust declaration, {id: reason}, [note lines])

Every id in the mapping is a false binding on that declaration. The reason is
copied verbatim into a comment at the site so the binding can be re-derived
later rather than silently lost. Optional `KEEP` maps an id to the reason it is
deliberately retained.

This is the same engine _unbind_protocol_modhost.py grew, extracted so the
engine-core, sim-domain, and hand-audited verdicts all run through one
implementation. Three properties are load-bearing and are each covered by a real
defect found during this audit:

1. Ids are removed individually, not line-wise. A tag line can mix a false and a
   true id (`// FR-CIV-RTS-015, FR-SAVE-009`); removing the line would discard
   the true tag and keeping it would leave the false one asserted.

2. Lines this tool writes are marked, and marked lines are not re-read as tags.
   Without the marker, the explanatory reasons are themselves scanned as live
   bindings on the next run.

3. The idempotence marker is the exact header line that gets written, and it is
   checked after classification so the count in it is real.

Idempotence is asserted at the end of a --apply run, and the exit status is
non-zero if a second pass would change anything.
"""
import argparse
import importlib.util
import pathlib
import re
import sys

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
AUDITS = REPO / "docs/audits"
ID_RE = re.compile(r"\b(?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*\b")
TOKEN = "[unbound]"


def load(name: str):
    path = AUDITS / name
    spec = importlib.util.spec_from_file_location(path.stem, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def is_doc_line(line: str) -> bool:
    s = line.strip()
    return s.startswith("///") or s.startswith("//!")


def is_tag_line(line: str) -> bool:
    s = line.strip()
    if not (s.startswith("//") and ID_RE.search(s)):
        return False
    return TOKEN not in s


def is_attr_line(line: str) -> bool:
    return line.strip().startswith("#[")


def collect_ids(text: str):
    return set(ID_RE.findall(text))


def find_decl(lines, decl: str):
    for i, line in enumerate(lines):
        s = line.strip()
        if not s.startswith("pub"):
            continue
        m = re.match(
            r"pub(?:\([^)]*\))?\s+(?:struct|enum|const|static|type|fn|trait)\s+"
            r"([A-Za-z0-9_]+)",
            s,
        )
        if m and m.group(1) == decl:
            return i
    return None


def strip_ids(line: str, removed: set) -> str:
    """Delete only the false ids from a line; keep the legitimate ones."""
    ids = collect_ids(line)
    if not ids or not (ids & removed):
        return line
    for bad in sorted(ids & removed):
        line = re.sub(rf"\b{re.escape(bad)}\b", "", line)
    if not line.lstrip().startswith("//"):
        return line
    body = re.sub(r",\s*(?=,)", "", line)
    body = re.sub(r"^(\s*//\s*)[,;]\s*", r"\1", body)
    body = re.sub(r"[,;]\s*$", "", body)
    body = re.sub(r"\(\s*\)|\[\s*\]", "", body)
    body = re.sub(r"[,;]{2,}", ",", body)
    body = body.rstrip()
    return "" if not collect_ids(body) else body


def process(rel: str, decl: str, removed: dict, note: list, keep: set, apply: bool):
    path = REPO / rel
    lines = path.read_text(encoding="utf-8").split("\n")

    idx = find_decl(lines, decl)
    if idx is None:
        return f"  MISSING declaration {decl} in {rel}", 0

    tags_start = None
    i = idx - 1
    while i >= 0 and (is_doc_line(lines[i]) or is_tag_line(lines[i]) or is_attr_line(lines[i])):
        if is_tag_line(lines[i]):
            tags_start = i
        i -= 1
    if tags_start is None:
        return f"  no tag block above {rel} :: {decl}", 0

    block = lines[tags_start:idx]
    present = collect_ids("\n".join(block))
    removable = sorted(present & set(removed))
    unknown = sorted(present - set(removed) - keep)
    if unknown:
        return f"  WARNING {rel} :: {decl} unclassified ids {unknown}", 0
    if not removable:
        return f"  nothing removable at {rel} :: {decl}", 0

    marker = f"// The following {len(removable)} requirement tags were removed from {decl}."
    if any(marker in l for l in lines):
        return f"  already processed: {rel} :: {decl}", 0

    kept = [s for s in (strip_ids(l, set(removed)) for l in block) if s]
    reasons = [f"// {TOKEN} {tag}: {removed[tag]}" for tag in removable]

    trailing = {k for k, l in enumerate(kept) if is_doc_line(l) or is_attr_line(l)}
    split = min(trailing) if trailing else len(kept)
    leading, tail = kept[:split], kept[split:]

    header = [
        f"// The following {len(removable)} requirement tags were removed from {decl}.",
        "// They are not discharged by this symbol. The tag named a requirement whose",
        "// behavior lives elsewhere, or a requirement with no implementation at all, so",
        "// leaving the tag here asserted coverage that this declaration does not provide.",
    ]
    header += [f"// {n}" for n in note]
    header += ["//", "// Removed, with the reason each cannot be discharged here:"]
    out = lines[:tags_start] + header + reasons + leading + tail + lines[idx:]

    if apply:
        path.write_text("\n".join(out), encoding="utf-8", newline="\n")
    return f"  {rel} :: {decl}: removed {len(removable)} tag(s)", len(removable)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument(
        "modules",
        nargs="*",
        help="verdict module names without the .py suffix; default all found",
    )
    args = ap.parse_args()

    names = args.modules or sorted(
        p.stem for p in AUDITS.glob("_verdicts_*.py")
    )
    total = 0
    for name in names:
        mod = load(f"{name}.py")
        keep = set(getattr(mod, "KEEP", {}))
        print(f"### {name}: {len(mod.SITES)} site(s)")
        for rel, decl, removed, note in mod.SITES:
            msg, n = process(rel, decl, removed, note, keep, args.apply)
            print(msg)
            total += n
    print(f"\ntotal removed: {total}")
    if not args.apply:
        print("(preview only; pass --apply to write)")
        return 0

    # Idempotence: a second pass must find nothing left to do.
    recheck = 0
    for name in names:
        mod = load(f"{name}.py")
        keep = set(getattr(mod, "KEEP", {}))
        for rel, decl, removed, note in mod.SITES:
            _, n = process(rel, decl, removed, note, keep, False)
            recheck += n
    if recheck:
        print(f"NOT IDEMPOTENT: a second pass would remove {recheck} more tag(s)")
        return 1
    print("idempotent: second pass would remove 0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
