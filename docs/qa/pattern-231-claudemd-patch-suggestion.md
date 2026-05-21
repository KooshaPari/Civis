# Pattern #231 CLAUDE.md Patch Suggestion

**Status**: USER-GATED — do not apply without explicit user approval.
**Date**: 2026-05-21
**Context**: Pattern #231 (Static Constructor / Static Field Initializer with I/O Side Effect) was closed in v0.26.0 with HIGH=0 confirmed. The current CLAUDE.md entry still reads as an active pattern with open violations ("Exit 1 if HIGH > 0"), implying a live gate against current violations rather than a retired RAII guard preventing regression.

## Location
`CLAUDE.md` lines 1021–1029 (section heading `### Pattern #231: Static Constructor / Static Field Initializer with I/O Side Effect`).

## Verbatim BEFORE-text

```markdown
### Pattern #231: Static Constructor / Static Field Initializer with I/O Side Effect

**Smell**: Static field initializer (`static readonly Foo = ...`) or static constructor body (`static { ... }`) performing file I/O, process spawn, environment-variable read, HttpClient instantiation, or other blocking operations at class-load time.

**Why bad**: Load-order dependent and untestable. Class-load triggers at JIT compilation (unpredictable timing in .NET). Exceptions during static init become `TypeInitializationException`, hiding the root cause. I/O failures block entire assembly load. Audit (a74aaa4) found 11 HIGH violations in NuGet-published surfaces (SDK, Bridge.Client, Bridge.Protocol, Domains).

**Detection**: `scripts/ci/detect_static_init_side_effect.py` — scans NuGet-published assemblies for `static readonly ... = File|Process|Environment|HttpClient|Directory|Path\.` and `static { ... }` blocks containing I/O. Severity: HIGH in NuGet surface, MED elsewhere. Exit 1 if HIGH > 0. Allowlist: `docs/qa/pattern-231-static-init-allowlist.txt`.

**Governance**: Refactor static I/O to lazy initialization using `Lazy<T>` or explicit `Initialize()` method. For unavoidable static setup (rare), document with `// static-init-ok: <reason>` and add to allowlist. Examples: `static readonly Logger = LoggerFactory.CreateLogger()` (acceptable—logger setup), `static readonly Foo = File.ReadAllText(...)` (unacceptable—replace with lazy property).
```

## Suggested AFTER-text

```markdown
### Pattern #231: Static Constructor / Static Field Initializer with I/O Side Effect

**Status**: RETIRED (v0.26.0) — HIGH=0 confirmed; detector retained as regression gate.

**Smell**: Static field initializer (`static readonly Foo = ...`) or static constructor body (`static { ... }`) performing file I/O, process spawn, environment-variable read, HttpClient instantiation, or other blocking operations at class-load time.

**Why bad**: Load-order dependent and untestable. Class-load triggers at JIT compilation (unpredictable timing in .NET). Exceptions during static init become `TypeInitializationException`, hiding the root cause. I/O failures block entire assembly load. Original audit (a74aaa4) found 11 HIGH violations in NuGet-published surfaces (SDK, Bridge.Client, Bridge.Protocol, Domains); all remediated by v0.26.0.

**Detection**: `scripts/ci/detect_static_init_side_effect.py` — scans NuGet-published assemblies for `static readonly ... = File|Process|Environment|HttpClient|Directory|Path\.` and `static { ... }` blocks containing I/O. Severity: HIGH in NuGet surface, MED elsewhere. Detector remains wired as a regression gate (Exit 1 if HIGH > 0); current HIGH count is 0 as of v0.26.0. Allowlist: `docs/qa/pattern-231-static-init-allowlist.txt`.

**Governance**: Continue to refactor any new static I/O to lazy initialization using `Lazy<T>` or explicit `Initialize()` method. For unavoidable static setup (rare), document with `// static-init-ok: <reason>` and add to allowlist. Examples: `static readonly Logger = LoggerFactory.CreateLogger()` (acceptable—logger setup), `static readonly Foo = File.ReadAllText(...)` (unacceptable—replace with lazy property).
```

## Diff Summary
1. Add `**Status**: RETIRED (v0.26.0) — HIGH=0 confirmed; detector retained as regression gate.` immediately after the heading.
2. Append `; all remediated by v0.26.0` to the audit sentence in **Why bad** so the 11 HIGH count reads as historical.
3. Reframe the `Exit 1 if HIGH > 0` clause in **Detection** as a regression gate ("current HIGH count is 0 as of v0.26.0") and soften **Governance** opener from "Refactor static I/O" to "Continue to refactor any new static I/O" to match retired status.

## Justification
- Pattern #231 closure in v0.26.0 with HIGH=0 means the entry is currently misleading — readers see it grouped with active patterns (#530, #235) and may assume remediation is still required.
- Other retired patterns in the catalog (e.g., #234) carry an explicit `**Status**: CLOSED/RETIRED` marker; #231 should match that convention for consistency.
- The detector itself is correctly kept in place to prevent regression, which is preserved by the rewording ("regression gate") rather than removed.
- No behavioral change to CI or code — documentation hygiene only.

## Apply Instructions (for user)
This file is a suggestion only. To apply, the user (or an explicitly authorized agent) should edit `CLAUDE.md` lines 1021–1029 in place with the AFTER-text above. CLAUDE.md is user-gated per project conventions; agents must not edit it autonomously.
