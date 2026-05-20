# Pattern #231 v0.26.0 Prep Notes — Static Constructor / Static Field Initializer with I/O Side Effect

**Date:** 2026-05-19  
**Scope:** NuGet-published assemblies under `src/SDK/`, `src/Bridge/Client/`, `src/Bridge/Protocol/`, `src/Domains/`

## Baseline Inputs
- `scripts/ci/detect_static_init_side_effect.py` exists and was used.
- `docs/qa/pattern-231-static-init-allowlist.txt` exists and was used.
- If either file were missing, the detector should be treated as the source of truth and the v0.26.0 sweep should run the detector to enumerate violations from a clean working tree.

## Known HIGH Violations to Track in v0.26.0 Prep
(11 entries, as tracked in the historical Pattern #231 audit trail)

1. `src/Bridge/Client/GameClient.cs:101` — **static field init I/O** (`Environment.GetEnvironmentVariable`)  
   Refactor: move environment capture into a `Lazy<T>` or explicit `Initialize(GameOptions options)` call (avoid class-load side effects).

2. `src/SDK/Dependencies/PackSubmoduleManager.cs:266` — **static constructor I/O** (`Process.Start`)  
   Refactor: defer process launch behind an explicit `Initialize`/`Run` entrypoint and keep static constructor empty.

3. `src/SDK/Dependencies/PackSubmoduleManager.cs:266` — **static constructor I/O** (`Process.Start` variant)  
   Refactor: split helper that performs the spawn into a non-static method and call from explicit lifecycle point.

4. `src/SDK/Dependencies/PackSubmoduleManager.cs:290` — **static constructor I/O** (`Process.Start`)  
   Refactor: replace with lazy-backed process factory or deferred initialization method.

5. `src/SDK/IO/SafeFileIO.cs:16` — **static field init I/O** (`File.*` during type init)  
   Refactor: replace `static readonly` I/O-backed values with `Lazy<string>`/`Lazy<T[]>` or an explicit warm-up call.

6. `src/SDK/IO/SafeFileIO.cs:16` — **static field init I/O** (duplicate historical hit in detector output)  
   Refactor: same as above; consolidate file-backed init behind `GetOrCreate()` method and cache via `Lazy<T>`.

7. `src/SDK/IO/SafeFileIO.cs:19` — **static field init I/O** (`File.*`)  
   Refactor: convert to on-demand initialization with `Lazy<T>` and avoid IO during type load.

8. `src/SDK/IO/SafeFileIO.cs:19` — **static field init I/O** (duplicate historical hit in detector output)  
   Refactor: same pattern; keep pure constants as `static readonly`, move IO to `Initialize` flow.

9. `src/SDK/NativeInterop/GoDependencyResolver.cs:177` — **static field init I/O** (`Environment.GetEnvironmentVariable` in resolver bootstrap path)  
   Refactor: cache resolver path during first call via `Lazy<string?>`, not in static initializer.

10. `src/SDK/NativeInterop/RustAssetPipeline.cs:32` — **static field init I/O** (`new HttpClient`)  
   Refactor: per governance, consider keeping singleton `HttpClient` as `Lazy<HttpClient>` only if startup ordering matters; otherwise use `Initialize` + explicit lifetime setup.

11. `src/SDK/NativeInterop/RustAssetPipeline.cs:33` — **governance marker needed** (`static-init-ok` exemption)  
   Refactor: add governance marker only when approved (`static-init-ok: Pattern #115`), otherwise defer client construction through `Lazy<HttpClient>`.

## Sweep Strategy (v0.26.0)
- **Phase 1: Identify**  
  Re-run `python scripts/ci/detect_static_init_side_effect.py` on a clean tree and pin the output as the starting baseline.

- **Phase 2: Refactor each site**  
  For each HIGH entry, apply one of:
  - `Lazy<T>` for expensive/IO-backed static dependencies  
  - explicit `Initialize(...)` for startup-ordered dependencies  
  - static constructor extraction + lifecycle wiring (no I/O in type init)

- **Phase 3: Confirm**  
  Re-run detector; verify `HIGH (NuGet surface): 0`.

- **Phase 4: Governance marker for true exemptions**  
  If a site is intentionally kept static-initialized for runtime policy/compliance reasons, document with a scoped marker comment and add to allowlist with explicit rationale.

## Notes
- Current detector run on the checked-in tree currently reports **0 HIGH / 0 MED** for Pattern #231, but this prep set should still be used to drive the v0.26.0 cleanup and closure review process.
- `Bridge.Protocol` and `Domains` are part of the NuGet-surface scope for this pattern, even though the current detector snapshot does not produce active HIGH entries there.
