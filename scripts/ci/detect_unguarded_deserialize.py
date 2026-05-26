#!/usr/bin/env python3
"""Unguarded-deserialize detector — Pattern #95 CI gate.

Pattern #95 ("Cross-FFI Deserialize Without Validation") is the failure
mode where C# code at a foreign-trust boundary (native-interop bridge,
CLI sub-process JSON, MCP tool argument decode, etc.) calls
``JsonSerializer.Deserialize<T>`` or ``JsonConvert.DeserializeObject<T>``
on a DTO that does NOT implement
:class:`DINOForge.SDK.Validation.IValidatable` and is NOT immediately
followed by a ``JsonGuard.ValidateOrThrow`` / ``JsonGuard.TryValidate``
call. The deserialized DTO is then trusted — required fields may be
``null``, enums may be out of range, paths may escape the pack root,
etc. — and that untrusted state propagates into pack loaders, asset
pipelines, and the runtime bridge.

The healthy pattern (already wired by #210 + #215 inside ``src/SDK/``):

    var dto = JsonSerializer.Deserialize<MyDto>(payload, options);
    JsonGuard.ValidateOrThrow(dto, source);   // <- IValidatable enforced

This gate scans the cross-FFI surface — ``src/Tools/``, ``src/Runtime/``,
and ``src/SDK/NativeInterop/`` — for every typed deserialize call and
classifies each site:

  * **HIGH** (``no_validation_dto``)  — DTO type ``T`` does NOT
    implement ``IValidatable``. The runtime cannot detect malformed
    payloads at all. This is the headline failure.
  * **MED**  (``no_jsonguard_call``)  — DTO type ``T`` IS validatable
    but the deserialize call site is not paired with a
    ``JsonGuard.ValidateOrThrow`` / ``JsonGuard.TryValidate`` within
    the next ~10 lines. The validation method exists but is never
    called at this site.
  * **LOW**  (``untyped_deserialize``) — Deserialize target is
    ``JsonElement`` / ``JsonNode`` / ``JObject`` / ``dynamic``. Neither
    the IValidatable-pairing nor the JsonGuard call applies; the site
    needs a different review (manual schema check on the raw tree).
    Reported for visibility only — does NOT fail the gate.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/unguarded-deserialize-allowlist.txt``. Two entry forms:

  1. ``severity|file|line|T_name`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_unguarded_deserialize.py
        [--root <repo>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes LOW
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_global_state_tests.py`` (#257) and
``scripts/ci/check_framework_version.py`` (#260). Pairs with
``src/SDK/Validation/JsonGuard.cs`` + ``IValidatable.cs`` (the API the
gate enforces).

This is task #265.
"""
from __future__ import annotations

import argparse
import json
import re
import sys

from regex_timeout import compile as _re_compile
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Generic identifier (used inside a generic type expression).
_IDENT = r"[A-Za-z_][A-Za-z0-9_]*"

# A type expression that can include generics, dotted namespaces, and
# nullable-? markers. Captures up to the matching ``>`` of the deserialize
# generic. This is a heuristic — we balance angle brackets manually.
def _capture_generic(text: str, start: int) -> tuple[str, int] | None:
    """Given a position right after ``Deserialize<`` (i.e. *start* points at
    the first character of the generic argument), return ``(t_name, end)``
    where ``end`` is the offset just past the matching ``>``. Returns None
    on parse failure (mismatched brackets / EOF).
    """
    depth = 1
    i = start
    end = len(text)
    while i < end:
        c = text[i]
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
            if depth == 0:
                return text[start:i].strip(), i + 1
        elif c == "\n" and depth >= 1 and i - start > 200:
            # Generic args don't span 200 chars — abort to avoid pathological
            # over-capture across method bodies.
            return None
        i += 1
    return None


# System.Text.Json: ``JsonSerializer.Deserialize<T>(...)`` /
# ``JsonSerializer.DeserializeAsync<T>(...)``. We anchor on the ``<`` after
# the method name and let _capture_generic walk the brackets. Module-level
# regex finds the candidate, the helper extracts T precisely.
STJ_DESERIALIZE_RE = _re_compile(
    r"\bJsonSerializer\s*\.\s*"
    r"(?:Deserialize|DeserializeAsync)\s*<"
)

# Newtonsoft: ``JsonConvert.DeserializeObject<T>(...)``.
NEWTONSOFT_DESERIALIZE_RE = _re_compile(
    r"\bJsonConvert\s*\.\s*"
    r"(?:DeserializeObject|DeserializeAnonymousType)\s*<"
)

# Generic ``ser\.Deserialize<T>(...)`` calls on a JsonSerializer-typed
# instance variable. We catch the non-static form too, but exclude
# ``_deserializer`` (YamlDotNet pattern in SDK content loaders, which is
# not at the FFI boundary and has its own validation surface).
INSTANCE_DESERIALIZE_RE = _re_compile(
    r"\b(?P<recv>" + _IDENT + r")\s*\.\s*Deserialize\s*<"
)

# ``IValidatable`` declaration on a class / record.
# Examples:
#   public sealed class MyDto : IValidatable, IComparable<MyDto>
#   internal record MyDto(int X) : IValidatable;
#   public partial class MyDto : SomeBase, DINOForge.SDK.Validation.IValidatable
IVALIDATABLE_DECL_RE = _re_compile(
    r"\b(?:class|record|struct)\s+(?P<name>" + _IDENT + r")\b"
    r"[^{;]*?:\s*[^{;]*?\bIValidatable\b",
    re.DOTALL,
)

# JsonGuard.* call (either ValidateOrThrow or TryValidate). Stops at end
# of statement.
JSONGUARD_CALL_RE = _re_compile(
    r"\bJsonGuard\s*\.\s*(?:ValidateOrThrow|TryValidate)\s*\("
)

# Parametric collection outer-type names. When T's outer type is one of these
# (or T ends with ``[]``), the validation contract attaches to the *elements*
# of the collection, not the collection itself. The detector looks for a
# per-element JsonGuard pairing rather than treating these as a missing-IV
# violation. Matches the ``_simple_t`` output (last identifier of the outer
# type).
PARAMETRIC_COLLECTION_OUTER = frozenset({
    "List",
    "IList",
    "ICollection",
    "IReadOnlyList",
    "IReadOnlyCollection",
    "IEnumerable",
    "HashSet",
    "ISet",
    "Dictionary",
    "IDictionary",
    "IReadOnlyDictionary",
    "SortedDictionary",
    "ConcurrentDictionary",
    "ConcurrentBag",
    "Queue",
    "Stack",
})

# Per-element validation pattern: ``foreach (...) { ... JsonGuard.* ... }`` or
# ``.ForEach(... => JsonGuard.*)`` or a direct keyed-lookup
# ``JsonGuard.ValidateOrThrow(dict[key], ...)``. We use ``re.DOTALL`` to span
# multiple lines inside the loop body. ``JSONGUARD_CALL_RE`` is reused after
# we confirm the foreach scaffolding.
PER_ELEMENT_FOREACH_RE = _re_compile(
    r"\bforeach\s*\([^)]*\)\s*"
    r"(?:\{[^{}]*?\bJsonGuard\s*\.\s*(?:ValidateOrThrow|TryValidate)\s*\("
    r"|[^{;]*?\bJsonGuard\s*\.\s*(?:ValidateOrThrow|TryValidate)\s*\()",
    re.DOTALL,
)
PER_ELEMENT_LINQ_FOREACH_RE = _re_compile(
    r"\.\s*ForEach\s*\([^)]*=>\s*[^)]*?"
    r"\bJsonGuard\s*\.\s*(?:ValidateOrThrow|TryValidate)\s*\(",
    re.DOTALL,
)

# Per-element lookahead window — wider than the flat JSONGUARD_LOOKAHEAD_CHARS
# because dict/list ingestion code often does several lines of bookkeeping
# (null-check, length-check, key-existence-check) before iterating.
PER_ELEMENT_LOOKAHEAD_CHARS = 1600   # ~25 lines @ 80 cols, generous


# Tokens that signal an untyped-deserialize target (LOW severity bucket).
UNTYPED_TARGET_TOKENS = {
    "JsonElement",
    "JsonNode",
    "JsonDocument",
    "JsonArray",
    "JsonObject",
    "JObject",
    "JArray",
    "JToken",
    "dynamic",
    "object",
}

# YamlDotNet receiver names — these are content loaders inside src/SDK
# that have their own JsonGuard pairing already; we don't want to flag
# their YAML deserialize calls as Pattern #95 hits even though the AST
# shape matches. (Belt-and-suspenders: we already exclude src/SDK from
# the FFI scan root, but if someone moves a YamlDotNet call into Tools
# or Runtime, this prevents a noisy false-positive.)
YAML_RECEIVER_NAMES = {
    "_deserializer",
    "deserializer",
    "_yamlDeserializer",
    "yamlDeserializer",
    "_yaml",
    "yaml",
    "Deserializer",  # YamlDotNet.Serialization.Deserializer static-ish access
}

# ``IValidatable`` lookahead window after a deserialize call (in chars,
# roughly mirroring "10 lines" given typical C# line widths).
JSONGUARD_LOOKAHEAD_CHARS = 800   # ~10 lines @ 80 cols, generous

# FFI boundary heuristic — these path fragments (POSIX-relative, lowercased)
# put a deserialize site into the HIGH bucket when the DTO is missing
# IValidatable. Sites OUTSIDE these directories still get flagged but
# remain MED-level.
HIGH_BOUNDARY_PARTS = (
    "src/sdk/nativeinterop/",
    "src/tools/cli/",
    "src/tools/assetpipelinerust/",
    "src/tools/assetpipelinezig/",
    "src/tools/dependencyresolver/",
    "src/tools/mcpserver/",
    "src/tools/gamecontrolcli/",
    "src/tools/desktopcompanion/",
    "src/runtime/",
)

# Default scan roots (POSIX-relative to repo root).
DEFAULT_SCAN_ROOTS = (
    "src/Tools",
    "src/Runtime",
    "src/SDK/NativeInterop",
)

# Tests are noisy and irrelevant — they often deserialize fixture DTOs
# without validation deliberately to exercise the validator itself.
EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "Tests"}

SEV_HIGH = "HIGH"
SEV_MED = "MED"
SEV_LOW = "LOW"


# ----------------------------------------------------------------------------
# IO helpers
# ----------------------------------------------------------------------------


def is_excluded_path(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def load_allowlist(path: Path) -> set[str]:
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


# ----------------------------------------------------------------------------
# IValidatable index — built across the whole repo so cross-project T types
# resolve. Returns the set of simple type names known to implement
# IValidatable.
# ----------------------------------------------------------------------------


def _strip_cs_comments(text: str) -> str:
    """Strip ``//`` line comments and ``/* */`` block comments from *text*
    by replacing comment bytes with spaces (preserves byte offsets — useful
    if a caller wants to keep line numbers stable). Used by the IValidatable
    index so that XML doc-comments containing the word ``record``/``class``
    don't pollute the detected type set (Pattern #95 #272 finding: the
    docstring above ``DownloadUrlEntry`` contained "A record for ..." which
    falsely matched the ``record\s+(?P<name>\w+)`` regex).
    """
    out: list[str] = []
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            # Line comment — replace until newline.
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
        elif c == "/" and nxt == "*":
            # Block comment — replace until ``*/``.
            out.append("  ")
            i += 2
            while i < n and not (text[i] == "*" and i + 1 < n and text[i + 1] == "/"):
                out.append(" " if text[i] != "\n" else "\n")
                i += 1
            if i < n:
                out.append("  ")
                i += 2
        elif c == '"':
            # String literal — copy through but don't try to find type decls
            # inside. Walks past escaped quotes.
            out.append(c)
            i += 1
            while i < n and text[i] != '"':
                if text[i] == "\\" and i + 1 < n:
                    out.append(text[i])
                    out.append(text[i + 1])
                    i += 2
                else:
                    out.append(text[i])
                    i += 1
            if i < n:
                out.append(text[i])
                i += 1
        else:
            out.append(c)
            i += 1
    return "".join(out)


def build_ivalidatable_index(repo_root: Path) -> set[str]:
    out: set[str] = set()
    src_root = repo_root / "src"
    if not src_root.exists():
        return out
    for cs in src_root.rglob("*.cs"):
        if is_excluded_path(cs):
            continue
        text = read_text_safe(cs)
        if "IValidatable" not in text:
            continue
        # Strip comments+strings so that doc-comment phrases like "A record
        # for deserializing X" don't false-positive into the index.
        cleaned = _strip_cs_comments(text)
        for m in IVALIDATABLE_DECL_RE.finditer(cleaned):
            out.add(m.group("name"))
    return out


# ----------------------------------------------------------------------------
# Scan
# ----------------------------------------------------------------------------


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    t_name: str          # captured generic argument as written
    t_simple: str        # last identifier token of T (used for IValidatable lookup)
    severity: str
    rule: str            # no_validation_dto / no_jsonguard_call / untyped_deserialize
    detail: str
    boundary: str        # high|med — coarse FFI-boundary tag
    has_jsonguard: bool
    has_ivalidatable: bool
    untyped: bool
    receiver: str = ""   # for instance.Deserialize<T> form (else "")
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _simple_t(t_name: str) -> str:
    """Return the right-most identifier of *t_name* (strips namespaces and
    generic argument decoration)."""
    t = t_name.strip()
    # Strip nullable suffix.
    while t.endswith("?"):
        t = t[:-1]
    # If T has angle brackets (e.g. List<MyDto>), look at the OUTER name —
    # the validation contract attaches to the OUTER type. List<MyDto> is
    # almost never IValidatable, so this is the right lookup.
    if "<" in t:
        t = t.split("<", 1)[0]
    # Strip dotted namespace qualifier.
    if "." in t:
        t = t.rsplit(".", 1)[1]
    return t


def _is_parametric_collection(t_name: str) -> bool:
    """Return True when *t_name* is a parametric collection type whose
    validation contract attaches to its elements (not the collection itself).

    Recognized forms:
      - ``T[]`` (any rank, including jagged via trailing ``[]``)
      - ``List<X>``, ``IList<X>``, ``IEnumerable<X>``, etc. (see
        :data:`PARAMETRIC_COLLECTION_OUTER`)
      - ``Dictionary<K,V>`` and the IDictionary family
    """
    t = t_name.strip()
    while t.endswith("?"):
        t = t[:-1]
    # Array form: anything ending with ``]`` after a ``[``.
    # ``BatchManifestItem[]`` -> True. ``List<int>`` doesn't end with ``]``.
    if t.endswith("]") and "[" in t:
        return True
    # Generic form: outer simple name in the parametric set.
    outer = _simple_t(t)
    return outer in PARAMETRIC_COLLECTION_OUTER


def _inner_t(t_name: str) -> str:
    """Return the element/value type of a parametric collection *t_name*.

    For ``List<Foo>`` -> ``Foo``. For ``Dictionary<string,Bar>`` -> ``Bar``
    (the value side, since we validate per-value not per-key — string keys
    are not DTOs). For ``Foo[]`` -> ``Foo``. Returns the input unchanged if
    *t_name* is not parametric.
    """
    t = t_name.strip()
    while t.endswith("?"):
        t = t[:-1]
    # Array: strip trailing ``[]`` runs.
    while t.endswith("]") and "[" in t:
        t = t[: t.rfind("[")].strip()
    if "<" not in t:
        return _simple_t(t)
    # Generic: take the args between the outer angle brackets.
    args = t[t.find("<") + 1 : t.rfind(">")].strip()
    # Dictionary-style: ``string,DownloadUrlEntry`` — value is the LAST
    # comma-separated arg at depth 0.
    depth = 0
    last_comma = -1
    for i, c in enumerate(args):
        if c == "<":
            depth += 1
        elif c == ">":
            depth -= 1
        elif c == "," and depth == 0:
            last_comma = i
    if last_comma >= 0:
        inner = args[last_comma + 1 :].strip()
    else:
        inner = args
    return _simple_t(inner)


def _has_per_element_validation(text: str, after_offset: int) -> bool:
    """Return True if the next few lines after a parametric-collection
    deserialize contain a per-element ``JsonGuard.ValidateOrThrow`` /
    ``JsonGuard.TryValidate`` call inside a ``foreach`` body, a LINQ
    ``.ForEach(... => JsonGuard.*)`` lambda, or a keyed-lookup form like
    ``JsonGuard.ValidateOrThrow(dict[key], ...)``.
    """
    end = min(len(text), after_offset + PER_ELEMENT_LOOKAHEAD_CHARS)
    window = text[after_offset:end]

    # 1. ``foreach (...) ...`` followed somewhere later by a ``JsonGuard`` call
    #    in window. We allow nested braces inside the foreach body (regex-
    #    balancing curlies is fragile), so we just look for the foreach
    #    keyword and ANY subsequent JsonGuard within the same window.
    fe = re.search(r"\bforeach\s*\(", window)
    if fe is not None:
        post_fe = window[fe.end():]
        if JSONGUARD_CALL_RE.search(post_fe):
            return True

    # 2. LINQ ``.ForEach(... => JsonGuard.*)`` lambda form.
    if PER_ELEMENT_LINQ_FOREACH_RE.search(window):
        return True

    # 3. Keyed-lookup form, two flavors:
    #    a) ``JsonGuard.ValidateOrThrow(dict[key], ...)`` — direct indexer
    #       inside the JsonGuard argument list.
    direct_keyed = re.search(
        r"\bJsonGuard\s*\.\s*(?:ValidateOrThrow|TryValidate)\s*\(\s*"
        r"[A-Za-z_][A-Za-z0-9_\.]*\s*\[",
        window,
    )
    if direct_keyed is not None:
        return True

    #    b) ``var X = dict[key]; ... JsonGuard.ValidateOrThrow(X, ...)`` —
    #       a local assigned from an indexer is then validated. We pattern-
    #       match ``= <ident>[...]`` then a later JsonGuard call in window.
    indirect_keyed = re.search(
        r"=\s*[A-Za-z_][A-Za-z0-9_\.]*\s*\[[^\]]+\]",
        window,
    )
    if indirect_keyed is not None:
        post_assign = window[indirect_keyed.end():]
        if JSONGUARD_CALL_RE.search(post_assign):
            return True

    return False


def _is_untyped(t_name: str) -> bool:
    simple = _simple_t(t_name)
    return simple in UNTYPED_TARGET_TOKENS


def _is_high_boundary(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in HIGH_BOUNDARY_PARTS)


def _has_nearby_jsonguard(text: str, after_offset: int) -> bool:
    end = min(len(text), after_offset + JSONGUARD_LOOKAHEAD_CHARS)
    window = text[after_offset:end]
    return JSONGUARD_CALL_RE.search(window) is not None


def _scan_one_call_site(
    text: str,
    rel: str,
    method_match: re.Match,
    receiver: str,
    ivalidatable: set[str],
) -> Hit | None:
    """Given a regex match that ends at the ``<`` of ``Deserialize<``,
    capture T and produce a Hit (or None if we couldn't parse T)."""
    gen_start = method_match.end()
    parsed = _capture_generic(text, gen_start)
    if parsed is None:
        return None
    t_name, gen_end = parsed
    if not t_name:
        return None
    t_simple = _simple_t(t_name)

    line = line_of(text, method_match.start())
    untyped = _is_untyped(t_name)
    has_iv = (not untyped) and (t_simple in ivalidatable)
    has_guard = _has_nearby_jsonguard(text, gen_end)
    high_boundary = _is_high_boundary(rel)

    # Parametric-collection escape hatch (#272 refinement). When T is a
    # ``List<X>`` / ``T[]`` / ``Dictionary<K,V>`` etc., the validation
    # contract attaches to the elements, not the collection. If the inner
    # element type X implements IValidatable AND the call site has a
    # per-element ``foreach (...) JsonGuard.ValidateOrThrow(item, ...)``
    # (or LINQ ForEach / keyed-lookup) within ~25 lines, we treat the site
    # as healthy. Without per-element validation we still flag (HIGH at FFI).
    parametric = _is_parametric_collection(t_name)
    if parametric:
        inner = _inner_t(t_name)
        inner_is_iv = inner in ivalidatable
        inner_is_primitive = inner in {
            "string", "int", "long", "short", "byte", "bool", "double",
            "float", "decimal", "char", "uint", "ulong", "ushort", "sbyte",
            "Guid", "DateTime", "DateTimeOffset", "TimeSpan",
            "String", "Int32", "Int64", "Boolean", "Double", "Single",
        }
        per_element = _has_per_element_validation(text, gen_end)
        if inner_is_iv and per_element:
            # Healthy: parametric collection of IValidatable elements with
            # per-element JsonGuard pairing. No hit.
            return None
        if inner_is_primitive and not high_boundary:
            # Collection of primitives at non-FFI boundary — nothing to
            # validate, no hit.
            return None
        # Otherwise fall through to the standard rules below, but enrich
        # the detail with parametric-specific guidance.

    if untyped:
        rule = "untyped_deserialize"
        sev = SEV_LOW
        detail = (
            f"Deserialize<{t_name}> at FFI boundary returns an untyped tree; "
            f"consider validating individual fields manually or migrating "
            f"to a typed DTO + IValidatable."
        )
    elif parametric and has_guard:
        # Parametric collection with a JsonGuard call somewhere in window
        # but NOT the per-element foreach pattern. The guard is operating
        # on the collection itself (no-op for List/Dictionary, neither
        # implements IValidatable) — flag as collection-guard-only.
        rule = "no_validation_dto"
        sev = SEV_HIGH if high_boundary else SEV_MED
        inner = _inner_t(t_name)
        detail = (
            f"Deserialize<{t_name}> targets a parametric collection. The "
            f"nearby JsonGuard call appears to validate the collection "
            f"itself (no-op) rather than each element. Wrap iteration in "
            f"``foreach (var item in result) JsonGuard.ValidateOrThrow"
            f"(item, ...)`` and make {inner} : IValidatable."
        )
    elif parametric and not has_guard:
        # Parametric collection with no JsonGuard at all in window.
        rule = "no_validation_dto"
        sev = SEV_HIGH if high_boundary else SEV_MED
        inner = _inner_t(t_name)
        detail = (
            f"Deserialize<{t_name}> at FFI boundary has no per-element "
            f"validation. Add ``foreach (var item in result) "
            f"JsonGuard.ValidateOrThrow(item, ...)`` and make {inner} : "
            f"IValidatable."
        )
    elif not has_iv:
        rule = "no_validation_dto"
        sev = SEV_HIGH if high_boundary else SEV_MED
        detail = (
            f"Deserialize<{t_name}> targets a DTO without IValidatable. "
            f"Make {t_simple} : IValidatable and pair the call with "
            f"JsonGuard.ValidateOrThrow."
        )
    elif not has_guard:
        rule = "no_jsonguard_call"
        sev = SEV_MED
        detail = (
            f"Deserialize<{t_name}> targets validatable {t_simple} but the "
            f"call site is not paired with JsonGuard.ValidateOrThrow within "
            f"~10 lines."
        )
    else:
        # Healthy: validatable + paired guard. Don't emit a hit.
        return None

    return Hit(
        file=rel,
        line=line,
        t_name=t_name,
        t_simple=t_simple,
        severity=sev,
        rule=rule,
        detail=detail,
        boundary="high" if high_boundary else "med",
        has_jsonguard=has_guard,
        has_ivalidatable=has_iv,
        untyped=untyped,
        receiver=receiver,
    )


def scan_file(path: Path, repo_root: Path, ivalidatable: set[str]) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    hits: list[Hit] = []

    # System.Text.Json static surface.
    for m in STJ_DESERIALIZE_RE.finditer(text):
        h = _scan_one_call_site(text, rel, m, "JsonSerializer", ivalidatable)
        if h is not None:
            hits.append(h)

    # Newtonsoft.
    for m in NEWTONSOFT_DESERIALIZE_RE.finditer(text):
        h = _scan_one_call_site(text, rel, m, "JsonConvert", ivalidatable)
        if h is not None:
            hits.append(h)

    # Instance ``recv.Deserialize<T>(...)`` form. Skip YamlDotNet receivers
    # — those are SDK content loaders with their own pairing.
    for m in INSTANCE_DESERIALIZE_RE.finditer(text):
        recv = m.group("recv")
        if recv in YAML_RECEIVER_NAMES:
            continue
        # Avoid double-counting `JsonSerializer.Deserialize<T>` (already
        # caught by STJ_DESERIALIZE_RE) — INSTANCE regex would match the
        # token ``JsonSerializer`` as the receiver, but that's the static
        # form, which the STJ regex covers more precisely.
        if recv in ("JsonSerializer", "JsonConvert"):
            continue
        h = _scan_one_call_site(text, rel, m, recv, ivalidatable)
        if h is not None:
            hits.append(h)

    return hits


def enumerate_target_files(repo_root: Path, roots: list[str]) -> list[Path]:
    out: list[Path] = []
    seen: set[Path] = set()
    for r in roots:
        rp = (repo_root / r).resolve()
        if not rp.exists():
            continue
        for cs in rp.rglob("*.cs"):
            if is_excluded_path(cs):
                continue
            if cs in seen:
                continue
            seen.add(cs)
            out.append(cs)
    out.sort()
    return out


def scan_roots(repo_root: Path, roots: list[str]) -> tuple[list[Hit], int, set[str]]:
    ivalidatable = build_ivalidatable_index(repo_root)
    files = enumerate_target_files(repo_root, roots)
    hits: list[Hit] = []
    for f in files:
        hits.extend(scan_file(f, repo_root, ivalidatable))
    return hits, len(files), ivalidatable


# ----------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------


def _hit_key(h: Hit) -> str:
    """Stable allowlist key: ``severity|file|line|t_simple``."""
    return f"{h.severity}|{h.file}|{h.line}|{h.t_simple}"


def apply_allowlist(hits: list[Hit], allowlist: set[str]) -> list[Hit]:
    new_hits: list[Hit] = []
    for h in hits:
        key = _hit_key(h)
        in_allow = (
            key in allowlist
            or h.file in allowlist
        )
        h.allowlist_key = key
        h.in_allowlist = in_allow
        if not in_allow:
            new_hits.append(h)
    return new_hits


def build_report(
    hits: list[Hit],
    allowlist: set[str],
    ivalidatable_count: int,
    files_scanned: int,
    strict: bool = False,
) -> dict:
    new_hits = apply_allowlist(hits, allowlist)

    def _sev_bucket(rule: str) -> list[Hit]:
        return [h for h in new_hits if h.rule == rule]

    no_validation_dto = sorted(
        _sev_bucket("no_validation_dto"),
        key=lambda h: (h.boundary != "high", h.file, h.line),
    )
    no_jsonguard_call = sorted(
        _sev_bucket("no_jsonguard_call"),
        key=lambda h: (h.file, h.line),
    )
    untyped_deserialize = sorted(
        _sev_bucket("untyped_deserialize"),
        key=lambda h: (h.file, h.line),
    )

    high_count = sum(1 for h in new_hits if h.severity == SEV_HIGH)
    med_count = sum(1 for h in new_hits if h.severity == SEV_MED)
    low_count = sum(1 for h in new_hits if h.severity == SEV_LOW)
    fail = high_count > 0 or med_count > 0 or (strict and low_count > 0)
    exit_code = 1 if fail else 0

    def _h2d(h: Hit) -> dict:
        return {
            "file": h.file,
            "line": h.line,
            "t_name": h.t_name,
            "t_simple": h.t_simple,
            "severity": h.severity,
            "rule": h.rule,
            "boundary": h.boundary,
            "has_jsonguard": h.has_jsonguard,
            "has_ivalidatable": h.has_ivalidatable,
            "untyped": h.untyped,
            "receiver": h.receiver,
            "detail": h.detail,
            "allowlist_key": h.allowlist_key,
            "in_allowlist": h.in_allowlist,
        }

    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "files_scanned": files_scanned,
        "ivalidatable_types_known": ivalidatable_count,
        "total_hits": len(hits),
        "new_hits": len(new_hits),
        "allowlist_size": len(allowlist),
        "strict": strict,
        "high_count": high_count,
        "med_count": med_count,
        "low_count": low_count,
        "no_validation_dto": [_h2d(h) for h in no_validation_dto],
        "no_jsonguard_call": [_h2d(h) for h in no_jsonguard_call],
        "untyped_deserialize": [_h2d(h) for h in untyped_deserialize],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("unguarded-deserialize gate (Pattern #95)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  IValidatable types     : {report['ivalidatable_types_known']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (no IValidatable, FFI) : {report['high_count']}")
    print(f"    MED  (no JsonGuard / no IV) : {report['med_count']}")
    print(f"    LOW  (untyped tree)         : {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW unguarded-deserialize sites:")
        if report["no_validation_dto"]:
            print("  -- no_validation_dto (DTO lacks IValidatable) --")
            for h in report["no_validation_dto"]:
                print(
                    f"    [{h['severity']}] {h['t_simple']:<32}"
                    f" {h['file']}:{h['line']}"
                )
        if report["no_jsonguard_call"]:
            print("  -- no_jsonguard_call (IValidatable but no guard call) --")
            for h in report["no_jsonguard_call"]:
                print(
                    f"    [{h['severity']}] {h['t_simple']:<32}"
                    f" {h['file']}:{h['line']}"
                )
        if report["untyped_deserialize"]:
            print("  -- untyped_deserialize (informational; --strict to fail) --")
            for h in report["untyped_deserialize"]:
                print(
                    f"    [{h['severity']}] {h['t_simple']:<32}"
                    f" {h['file']}:{h['line']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect cross-FFI deserialize sites without IValidatable + "
            "JsonGuard pairing (Pattern #95). HIGH = DTO at FFI boundary "
            "without IValidatable; MED = IValidatable DTO without "
            "JsonGuard call; LOW = untyped tree (info)."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help=(
            "Repo root (default: auto-detected from the script location). "
            "Scan roots are always src/Tools, src/Runtime, "
            "src/SDK/NativeInterop relative to this root."
        ),
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/unguarded-deserialize-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line|T`` key or bare "
            "relative path per line; ``#`` for comments "
            "(default: docs/qa/unguarded-deserialize-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/unguarded-deserialize-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW (untyped_deserialize) findings to fail the gate. "
            "Default: only HIGH+MED fail."
        ),
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    repo = Path(args.root).resolve() if args.root else repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return repo, _abs(args.allowlist), _abs(args.output)


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


_FIXTURE_VALIDATABLE = """
namespace DINOForge.Fixture
{
    using DINOForge.SDK.Validation;

    public sealed class GoodDto : IValidatable
    {
        public ValidationResult Validate() => ValidationResult.Success;
    }

    public sealed class StaleDto : IValidatable
    {
        public ValidationResult Validate() => ValidationResult.Success;
    }

    public sealed class PlainDto
    {
        public string? Name { get; set; }
    }
}
"""

_FIXTURE_HEALTHY = """
namespace DINOForge.Fixture.Healthy
{
    using System.IO;
    using System.Text.Json;
    using DINOForge.SDK.Validation;

    public static class Loader
    {
        public static GoodDto Load(string text)
        {
            var dto = JsonSerializer.Deserialize<GoodDto>(text)!;
            JsonGuard.ValidateOrThrow(dto, "fixture");
            return dto;
        }
    }
}
"""

_FIXTURE_NO_VALIDATION = """
namespace DINOForge.Fixture.Bad
{
    using System.Text.Json;

    public static class BadLoader
    {
        public static PlainDto Load(string text)
        {
            // PlainDto is not IValidatable -> HIGH (when scanned at FFI path).
            return JsonSerializer.Deserialize<PlainDto>(text)!;
        }
    }
}
"""

_FIXTURE_NO_GUARD = """
namespace DINOForge.Fixture.Stale
{
    using System.Text.Json;

    public static class StaleLoader
    {
        public static StaleDto Load(string text)
        {
            // StaleDto is IValidatable but no JsonGuard call follows.
            var dto = JsonSerializer.Deserialize<StaleDto>(text)!;
            return dto;
        }
    }
}
"""

_FIXTURE_UNTYPED = """
namespace DINOForge.Fixture.Untyped
{
    using System.Text.Json;

    public static class UntypedLoader
    {
        public static JsonElement Load(string text)
        {
            return JsonSerializer.Deserialize<JsonElement>(text);
        }
    }
}
"""

_FIXTURE_NEWTONSOFT = """
namespace DINOForge.Fixture.Newt
{
    using Newtonsoft.Json;

    public static class NewtLoader
    {
        public static PlainDto Load(string text)
        {
            return JsonConvert.DeserializeObject<PlainDto>(text)!;
        }
    }
}
"""

# ---- Parametric-collection fixtures (Pattern #95 #272 refinement) ----

# List<GoodDto> with per-element JsonGuard inside foreach — should NOT flag.
_FIXTURE_LIST_PER_ELEMENT_VALIDATED = """
namespace DINOForge.Fixture.ListGood
{
    using System.Collections.Generic;
    using System.Text.Json;
    using DINOForge.SDK.Validation;

    public static class ListGoodLoader
    {
        public static List<GoodDto> Load(string text)
        {
            var result = JsonSerializer.Deserialize<List<GoodDto>>(text)
                         ?? new List<GoodDto>();
            foreach (var item in result)
            {
                JsonGuard.ValidateOrThrow(item, "ListGoodLoader.Load");
            }
            return result;
        }
    }
}
"""

# T[] with per-element JsonGuard inside foreach — should NOT flag.
_FIXTURE_ARRAY_PER_ELEMENT_VALIDATED = """
namespace DINOForge.Fixture.ArrayGood
{
    using System.Text.Json;
    using DINOForge.SDK.Validation;

    public static class ArrayGoodLoader
    {
        public static GoodDto[] Load(string text)
        {
            var result = JsonSerializer.Deserialize<GoodDto[]>(text)
                         ?? System.Array.Empty<GoodDto>();
            foreach (var item in result)
            {
                JsonGuard.ValidateOrThrow(item, "ArrayGoodLoader.Load");
            }
            return result;
        }
    }
}
"""

# Dictionary<K,V> with keyed-lookup JsonGuard — should NOT flag.
_FIXTURE_DICT_KEYED_VALIDATED = """
namespace DINOForge.Fixture.DictGood
{
    using System.Collections.Generic;
    using System.Text.Json;
    using DINOForge.SDK.Validation;

    public static class DictGoodLoader
    {
        public static GoodDto LoadOne(string text, string key)
        {
            var result = JsonSerializer.Deserialize<Dictionary<string, GoodDto>>(text)
                         ?? new Dictionary<string, GoodDto>();
            JsonGuard.ValidateOrThrow(result[key], "DictGoodLoader.LoadOne");
            return result[key];
        }
    }
}
"""

# List<PlainDto> with NO per-element validation at HIGH boundary — should flag.
_FIXTURE_LIST_NO_GUARD = """
namespace DINOForge.Fixture.ListBad
{
    using System.Collections.Generic;
    using System.Text.Json;

    public static class ListBadLoader
    {
        public static List<PlainDto> Load(string text)
        {
            return JsonSerializer.Deserialize<List<PlainDto>>(text)
                   ?? new List<PlainDto>();
        }
    }
}
"""

# List<PlainDto> with a JsonGuard on the COLLECTION ITSELF (no foreach) — must
# still flag, because list-as-IValidatable is meaningless.
_FIXTURE_LIST_COLLECTION_GUARD_ONLY = """
namespace DINOForge.Fixture.ListBadGuard
{
    using System.Collections.Generic;
    using System.Text.Json;
    using DINOForge.SDK.Validation;

    public static class ListBadGuardLoader
    {
        public static List<PlainDto> Load(string text)
        {
            var result = JsonSerializer.Deserialize<List<PlainDto>>(text)
                         ?? new List<PlainDto>();
            JsonGuard.ValidateOrThrow(result, "WRONG-collection-not-element");
            return result;
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a fake repo under *td* matching the gate's expected structure
    so scanning + the IValidatable index work end-to-end."""
    repo = td / "repo"
    sdk_validation = repo / "src" / "SDK" / "Validation"
    ffi = repo / "src" / "SDK" / "NativeInterop"
    tools_cli = repo / "src" / "Tools" / "Cli"
    tools_mcp = repo / "src" / "Tools" / "McpServer"
    runtime = repo / "src" / "Runtime"
    tests = repo / "src" / "Tests"
    for d in (sdk_validation, ffi, tools_cli, tools_mcp, runtime, tests):
        d.mkdir(parents=True, exist_ok=True)

    # Stub IValidatable + ValidationResult so the index sees Good/StaleDto.
    (sdk_validation / "IValidatable.cs").write_text(
        "namespace DINOForge.SDK.Validation { public interface IValidatable { "
        "ValidationResult Validate(); } "
        "public sealed class ValidationResult { public static readonly "
        "ValidationResult Success = new(); } }",
        encoding="utf-8",
    )
    # The fixture DTOs (some validatable, one not).
    (sdk_validation / "FixtureDtos.cs").write_text(
        _FIXTURE_VALIDATABLE, encoding="utf-8"
    )

    # Healthy site under FFI — should NOT produce any hit.
    (ffi / "HealthyLoader.cs").write_text(_FIXTURE_HEALTHY, encoding="utf-8")
    # No-validation DTO at HIGH boundary (Tools/Cli).
    (tools_cli / "BadLoader.cs").write_text(
        _FIXTURE_NO_VALIDATION, encoding="utf-8"
    )
    # No-jsonguard at MED boundary (Runtime — also high boundary, so MED rule
    # but the rule itself is no_jsonguard_call which is always MED).
    (runtime / "StaleLoader.cs").write_text(_FIXTURE_NO_GUARD, encoding="utf-8")
    # Untyped at FFI — LOW.
    (tools_mcp / "UntypedLoader.cs").write_text(
        _FIXTURE_UNTYPED, encoding="utf-8"
    )
    # Newtonsoft variant — should also trip HIGH.
    (tools_cli / "NewtLoader.cs").write_text(
        _FIXTURE_NEWTONSOFT, encoding="utf-8"
    )

    # Parametric-collection refinement fixtures (#272). These all live at
    # FFI boundaries.
    (tools_cli / "ListGoodLoader.cs").write_text(
        _FIXTURE_LIST_PER_ELEMENT_VALIDATED, encoding="utf-8"
    )
    (tools_cli / "ArrayGoodLoader.cs").write_text(
        _FIXTURE_ARRAY_PER_ELEMENT_VALIDATED, encoding="utf-8"
    )
    (tools_cli / "DictGoodLoader.cs").write_text(
        _FIXTURE_DICT_KEYED_VALIDATED, encoding="utf-8"
    )
    (tools_cli / "ListBadLoader.cs").write_text(
        _FIXTURE_LIST_NO_GUARD, encoding="utf-8"
    )
    (tools_cli / "ListBadGuardLoader.cs").write_text(
        _FIXTURE_LIST_COLLECTION_GUARD_ONLY, encoding="utf-8"
    )

    # Tests directory should be SKIPPED — write a deliberately bad file there
    # and confirm the gate ignores it.
    (tests / "ShouldBeSkipped.cs").write_text(
        _FIXTURE_NO_VALIDATION, encoding="utf-8"
    )
    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Type-name simplification.
    assert _simple_t("MyDto") == "MyDto"
    assert _simple_t("Some.NS.MyDto") == "MyDto"
    assert _simple_t("MyDto?") == "MyDto"
    assert _simple_t("List<MyDto>") == "List"
    assert _is_untyped("JsonElement")
    assert _is_untyped("System.Text.Json.JsonElement")
    assert _is_untyped("dynamic")
    assert not _is_untyped("MyDto")

    # 1b) Parametric-collection helpers.
    assert _is_parametric_collection("List<Foo>")
    assert _is_parametric_collection("IList<Foo>")
    assert _is_parametric_collection("Foo[]")
    assert _is_parametric_collection("Foo[][]")
    assert _is_parametric_collection("Dictionary<string,Bar>")
    assert _is_parametric_collection("Dictionary<string, Bar>")
    assert _is_parametric_collection("IReadOnlyList<Foo>")
    assert not _is_parametric_collection("Foo")
    assert not _is_parametric_collection("MyDto<TParam>")  # custom generic, not in set
    assert _inner_t("List<Foo>") == "Foo"
    assert _inner_t("Foo[]") == "Foo"
    assert _inner_t("Dictionary<string, Bar>") == "Bar"
    assert _inner_t("Dictionary<string, List<int>>") == "List"
    assert _inner_t("Foo") == "Foo"

    # 2) Generic capture handles nested brackets.
    snippet = "JsonSerializer.Deserialize<Dictionary<string,List<int>>>(payload)"
    m = STJ_DESERIALIZE_RE.search(snippet)
    assert m is not None
    parsed = _capture_generic(snippet, m.end())
    assert parsed is not None
    t_name, end = parsed
    assert t_name == "Dictionary<string,List<int>>", t_name
    assert snippet[end] == "(", snippet[end]

    # 3) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        ivalidatable = build_ivalidatable_index(repo)
        assert "GoodDto" in ivalidatable, ivalidatable
        assert "StaleDto" in ivalidatable, ivalidatable
        assert "PlainDto" not in ivalidatable, ivalidatable

        hits, n_files, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        # Tests/ deliberately bad file must be excluded.
        files_seen = {h.file for h in hits}
        assert not any(
            "src/Tests/" in f for f in files_seen
        ), f"src/Tests must be excluded: {files_seen}"

        # Categorize.
        by_rule = {SEV_HIGH: [], SEV_MED: [], SEV_LOW: []}
        for h in hits:
            by_rule[h.severity].append(h)

        # HIGH: BadLoader (PlainDto, STJ) + NewtLoader (PlainDto, Newtonsoft).
        high_files = {h.file for h in by_rule[SEV_HIGH]}
        assert any(
            "BadLoader.cs" in f for f in high_files
        ), f"missing HIGH for BadLoader.cs: {high_files}"
        assert any(
            "NewtLoader.cs" in f for f in high_files
        ), f"missing HIGH for NewtLoader.cs: {high_files}"

        # MED: StaleLoader (IValidatable but no JsonGuard pairing).
        med_files = {h.file for h in by_rule[SEV_MED]}
        assert any(
            "StaleLoader.cs" in f for f in med_files
        ), f"missing MED for StaleLoader.cs: {med_files}"

        # LOW: UntypedLoader.
        low_files = {h.file for h in by_rule[SEV_LOW]}
        assert any(
            "UntypedLoader.cs" in f for f in low_files
        ), f"missing LOW for UntypedLoader.cs: {low_files}"

        # Healthy site (HealthyLoader.cs in NativeInterop) must NOT appear.
        all_files = {h.file for h in hits}
        assert not any(
            "HealthyLoader.cs" in f for f in all_files
        ), f"healthy site flagged: {all_files}"

        # Parametric-collection fixtures (#272 refinement):
        #   ListGoodLoader / ArrayGoodLoader / DictGoodLoader — per-element
        #   JsonGuard inside foreach (or keyed lookup for Dictionary). MUST
        #   NOT flag.
        for good_name in (
            "ListGoodLoader.cs",
            "ArrayGoodLoader.cs",
            "DictGoodLoader.cs",
        ):
            assert not any(
                good_name in f for f in all_files
            ), f"per-element-validated site falsely flagged: {good_name}"

        #   ListBadLoader — List<PlainDto> with NO per-element validation.
        #   MUST flag at HIGH (FFI boundary, parametric of non-validatable).
        bad_files = {h.file for h in by_rule[SEV_HIGH]}
        assert any(
            "ListBadLoader.cs" in f for f in bad_files
        ), f"ListBadLoader (no per-element validation) not flagged: {bad_files}"

        #   ListBadGuardLoader — JsonGuard on the COLLECTION ITSELF, no
        #   foreach. MUST flag (collection-as-IValidatable is meaningless).
        assert any(
            "ListBadGuardLoader.cs" in f for f in bad_files
        ), f"ListBadGuardLoader (collection-only guard) not flagged: {bad_files}"

        # 4) Allowlist suppression — pick the BadLoader hit, allowlist it.
        bad_hit = next(
            h for h in hits
            if h.rule == "no_validation_dto" and "BadLoader.cs" in h.file
        )
        report_pre = build_report(list(hits), set(), len(ivalidatable), n_files)
        target_key = bad_hit.allowlist_key
        # Re-scan to reset state, then build report with allowlist set.
        hits2, _, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        report_post = build_report(
            list(hits2), {target_key}, len(ivalidatable), n_files,
        )
        post_keys = {h["allowlist_key"] for h in report_post["no_validation_dto"]}
        assert target_key not in post_keys, (
            f"allowlist did not suppress {target_key}; remaining: {post_keys}"
        )
        assert report_post["new_hits"] < report_pre["new_hits"], (
            f"allowlist did not reduce new_hits: pre={report_pre['new_hits']} "
            f"post={report_post['new_hits']}"
        )

        # 5) Bare-path allowlist form — listing a relative file path drops
        #    every hit in that file.
        hits3, _, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        bare_path = "src/Tools/Cli/BadLoader.cs"
        report_bare = build_report(
            list(hits3), {bare_path}, len(ivalidatable), n_files,
        )
        bare_files = {h["file"] for h in report_bare["no_validation_dto"]}
        assert bare_path not in bare_files, (
            f"bare-path allowlist did not suppress {bare_path}"
        )

        # 6) Strict mode promotes LOW to fail.
        hits4, _, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        non_strict = build_report(list(hits4), set(), len(ivalidatable), n_files,
                                  strict=False)
        strict = build_report(list(hits4), set(), len(ivalidatable), n_files,
                              strict=True)
        assert non_strict["exit_code"] == 1  # HIGH+MED already fail
        assert strict["exit_code"] == 1
        # If we suppress all HIGH+MED, strict still fails because of LOW.
        suppress = {
            h.allowlist_key for h in hits4
            if h.severity in (SEV_HIGH, SEV_MED)
        }
        # apply_allowlist mutates h in-place; rescan to re-derive keys.
        hits5, _, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        rpt_strict_only_low = build_report(
            list(hits5), suppress, len(ivalidatable), n_files, strict=True,
        )
        # All HIGH+MED suppressed; only LOW remain.
        assert rpt_strict_only_low["high_count"] == 0
        assert rpt_strict_only_low["med_count"] == 0
        assert rpt_strict_only_low["low_count"] >= 1
        assert rpt_strict_only_low["exit_code"] == 1, rpt_strict_only_low

        rpt_lax_only_low = build_report(
            list(hits5), suppress, len(ivalidatable), n_files, strict=False,
        )
        # Note: hits5 had the suppress set applied above, but we got fresh
        # hit list ordering from rescan. Re-rescan for a clean state.
        hits6, _, _ = scan_roots(
            repo,
            ["src/Tools", "src/Runtime", "src/SDK/NativeInterop"],
        )
        rpt_lax_only_low = build_report(
            list(hits6), suppress, len(ivalidatable), n_files, strict=False,
        )
        assert rpt_lax_only_low["exit_code"] == 0, rpt_lax_only_low

    print("self-test: OK")
    return 0


# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    if len(argv) >= 1 and argv[0] == "--self-test":
        return _self_test()

    args = parse_args(argv)
    if args.quiet and args.verbose:
        print(
            "ERROR: --quiet and --verbose are mutually exclusive",
            file=sys.stderr,
        )
        return 2

    repo, allow_path, output = resolve_paths(args)

    # Validate scan roots exist (any subset is fine; warn on missing).
    missing = [r for r in DEFAULT_SCAN_ROOTS if not (repo / r).exists()]
    if len(missing) == len(DEFAULT_SCAN_ROOTS):
        print(
            f"ERROR: no scan roots found under {repo}: {missing}",
            file=sys.stderr,
        )
        return 2

    if args.verbose:
        print(
            f"scanning {repo} roots={list(DEFAULT_SCAN_ROOTS)} "
            f"(missing={missing})",
            file=sys.stderr,
        )

    hits, n_files, ivalidatable = scan_roots(repo, list(DEFAULT_SCAN_ROOTS))
    allowlist = load_allowlist(allow_path)
    report = build_report(
        hits, allowlist, len(ivalidatable), n_files, strict=args.strict,
    )
    report["scan_root"] = repo.as_posix()
    report["scan_paths"] = list(DEFAULT_SCAN_ROOTS)
    report["allowlist_path"] = allow_path.as_posix()

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "unguarded-deserialize: "
            f"files={report['files_scanned']} "
            f"total={report['total_hits']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={report['new_hits']} "
            f"HIGH={report['high_count']} "
            f"MED={report['med_count']} "
            f"LOW={report['low_count']} "
            f"strict={'on' if args.strict else 'off'} "
            f"-> {output}"
        )
    else:
        print_summary(report, output)

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
