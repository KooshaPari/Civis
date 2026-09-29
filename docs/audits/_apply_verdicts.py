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

    # A site entry may name ids it deliberately keeps on this declaration, which
    # is how a block of N ids ends up with some removed and some retained. This
    # has to be per-site: a module-level KEEP applies to every declaration in the
    # module, and would silence a genuinely unclassified id on a different one.
    keep = set(keep) | set(removed.get("__keep__", ()))

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
    removable = sorted(present & (set(removed) - {"__keep__"}))
    unknown = sorted(present - set(removed) - keep)
    if unknown:
        return f"  WARNING {rel} :: {decl} unclassified ids {unknown}", 0
    if not removable:
        return f"  nothing removable at {rel} :: {decl}", 0

    # The idempotence marker must be read only from the comment block that this
    # tool would rewrite, i.e. the lines immediately above the declaration. An
    # earlier version scanned the whole file for the marker string, so once two
    # verdict modules had both claimed `BiomeKind` in geology.rs, the first
    # module's marker made the second module's site report "already processed"
    # and its removal was silently skipped -- while the run still printed a
    # non-zero total for the other sites. Scoping the scan to the block is what
    # makes "total removed" an honest count.
    marker = f"// The following {len(removable)} requirement tags were removed from {decl}."
    if any(marker in l for l in block):
        return f"  already processed: {rel} :: {decl}", 0

    # "1 requirement tags" is what the template above emits, but it is wrong
    # English and this string is compared for idempotence, so it is fixed in
    # both places rather than leaving the grammar bug in the output.
    if len(removable) == 1:
        marker = f"// The following 1 requirement tag was removed from {decl}."
        if any(marker in l for l in block):
            return f"  already processed: {rel} :: {decl}", 0

    kept = [s for s in (strip_ids(l, set(removed)) for l in block) if s]
    reasons = [f"// {TOKEN} {tag}: {removed[tag]}" for tag in removable]

    # leading = the id-stripped tag lines that belong above the new reasons;
    # tail = the doc/attribute lines that trailed the original block and must
    # stay below them, so the declaration keeps its rustdoc and derives.
    trailing = {k for k, l in enumerate(kept) if is_doc_line(l) or is_attr_line(l)}
    split = min(trailing) if trailing else len(kept)
    leading, tail = kept[:split], kept[split:]

    header = [
        marker,
        "// It is not discharged by this symbol. The tag named a requirement whose"
        if len(removable) == 1
        else "// They are not discharged by this symbol. The tag named a requirement whose",
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
    mods = [(n, load(f"{n}.py")) for n in names]

    # An id counts as adjudicated if *any* verdict module names it, not just the
    # one that happens to own the site. Two modules routinely split one
    # declaration -- the sim/domain report takes a subset of a block and this
    # module decides the rest -- and a per-module view reports the other's ids
    # as unexamined. The union is the honest set of "a human looked at this".
    known = set()
    for _, mod in mods:
        known |= set(getattr(mod, "KEEP", {}))
        for _rel, _decl, removed, _note in mod.SITES:
            known |= set(removed)
    known.discard("__keep__")

    total = 0
    warnings = 0
    for name, mod in mods:
        keep = set(getattr(mod, "KEEP", {}))
        print(f"### {name}: {len(mod.SITES)} site(s)")
        for rel, decl, removed, note in mod.SITES:
            msg, n = process(rel, decl, removed, note, keep | known, args.apply)
            print(msg)
            if "unclassified" in msg or "MISSING" in msg:
                warnings += 1
            total += n
    print(f"\ntotal removed: {total}")
    if not args.apply:
        print("(preview only; pass --apply to write)")
        return 0

    # Idempotence: a second pass must find nothing left to do.
    recheck = 0
    for name, mod in mods:
        keep = set(getattr(mod, "KEEP", {}))
        for rel, decl, removed, note in mod.SITES:
            _, n = process(rel, decl, removed, note, keep | known, False)
            recheck += n
    if recheck:
        print(f"NOT IDEMPOTENT: a second pass would remove {recheck} more tag(s)")
        return 1
    print("idempotent: second pass would remove 0")
    if warnings:
        print(f"WARNING: {warnings} site(s) still carry an unclassified id")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
