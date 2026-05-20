# Pattern #231 — Static Init Side Effect — CLOSURE REPORT (v0.26.0)

**Date:** 2026-05-19
**Status:** RETIRED — HIGH count: 0 in NuGet-published surface
**Detector:** `scripts/ci/detect_static_init_side_effect.py`

## Final Detector Output

```
=== Pattern #231: Static Init Side Effect ===

HIGH (NuGet surface): 0
MED (Internal):       0
```

## Resolution of the 11 Historical Violations (per prep notes)

The historical `pattern-231-v0.26.0-prep-notes.md` enumerated 11 HIGH violations. Audit during the v0.26.0 sweep confirms each is now either:

1. **`src/Bridge/Client/GameClient.cs:101`** — was `static readonly` field init; refactored to `InitializeLogger()` private static *method* (no class-load I/O). Method-body I/O is not a Pattern #231 violation. Closed.

2-4. **`src/SDK/Dependencies/PackSubmoduleManager.cs:266, 266, 290`** — These line numbers now point at `private static async Task<>` *method declarations* (`RunGitCommandWithOutputAsync`, `RunGitCommandAsync`). They were never static field initializers in the post-refactor codebase. Earlier detector regex produced false positives on method signatures starting with `static`. Closed.

5-8. **`src/SDK/IO/SafeFileIO.cs:16, 16, 19, 19`** — These lines are expression-bodied *instance helper methods* (`ReadText`, `ReadAllLines`). The only true static field is `StrictUtf8 = new UTF8Encoding(...)` on line 13, which is a pure constructor call with no I/O — exempt by governance. Closed.

9. **`src/SDK/NativeInterop/GoDependencyResolver.cs:177`** — Line points at `private static string? FindResolverBinary()` method declaration. The `Environment.GetEnvironmentVariable` call is inside the method body, executed on demand (not at class-load). Closed.

10. **`src/SDK/NativeInterop/RustAssetPipeline.cs:32`** — Carries explicit `// static-init-ok: Pattern #115 canonical HttpClient singleton ...` governance marker. Detector correctly skips. Closed (governance-approved).

11. **`src/SDK/NativeInterop/RustAssetPipeline.cs:33`** — The `HttpClient` field. Same marker applies (preceding-line marker accepted by detector). Closed.

## Governance Going Forward

- Detector remains wired in CI (`pattern-gates.yml`).
- Allowlist file `docs/qa/pattern-231-static-init-allowlist.txt` is preserved for future opt-in exemptions.
- Any new `static readonly X = <I/O expression>` in NuGet-published assemblies MUST:
  - Use `Lazy<T>` for deferred initialization, OR
  - Be refactored into an explicit `Initialize()` method, OR
  - Carry an inline `// static-init-ok: <reason>` marker (allowlist-tracked).

## Detector Enhancements (v0.26.0)

The detector was reviewed for false positives. The current regex `static\s+readonly\s+\w+\s+\w+\s*=` correctly excludes method declarations (which lack `readonly` and `=`). No detector change required.

## Next Audit

Re-run quarterly during pattern-rotation. Trigger an immediate re-audit if any new static-field-init regression is suspected (PR review or static analysis flag).
