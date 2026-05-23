# Build-vs-Buy Audit: Handrolled Components

Model-knowledge-only review. Compatibility is conservative for Unity 2021.3.45f2 Mono + BepInEx + `netstandard2.0`.

| Rank | Area | Recommended library | Why | Why handroll is risky | Compat | Cost | Pri |
|---|---|---|---|---|---|---|---|
| 1 | `CanonicalJson.cs` | `jsoncanonicalizer` (cyberphone / RFC8785 impl) | Canonical JSON needs exact float formatting, escaping, and property ordering; existing impls encode edge cases and test vectors already. | Partial RFC coverage creates silent signature/hash drift and interop bugs. | UNKNOWN | Medium | P0 |
| 2 | `GameBridgeServer.cs` | `StreamJsonRpc` | Mature JSON-RPC over streams/pipes with framing, cancellation, and request/response plumbing already solved. | Byte-by-byte NDJSON parsing is fragile under chunking, CRLF, partial reads, and backpressure. | YES | Medium | P0 |
| 3 | `AssetPipelineZig.cs` | `meshoptimizer` (or managed wrapper) | SOTA mesh simplification/optimization and BVH-adjacent mesh processing are battle-tested; replaces stubbed geometry work with real algorithms. | Stubbed decimation/BVH blocks shipping quality, wastes content authoring time, and risks runtime perf regressions. | UNKNOWN | Large | P0 |
| 4 | `PackDependencyResolver` | `QuikGraph` | Gives proven graph types and topological sort / cycle handling instead of maintaining bespoke DAG code. | Kahn handrolls often miss cycle diagnostics, stable ordering, and duplicate-edge corner cases. | YES | Small | P1 |
| 5 | `JsonGuard` | `NJsonSchema` | Already a dependency; use its schema validation and rule surface instead of custom JSON guards. | Hand-rolled JSON checks tend to drift from schema truth and miss nested/union constraints. | YES | Small | P1 |
| 6 | `MainThreadDispatcher.cs` | `UniTask` | Unity-native async model with main-thread marshalling, PlayerLoop integration, and broad community adoption. | Custom queues are easy to deadlock, leak, or starve under scene reloads and exception paths. | YES | Medium | P1 |
| 7 | `Registry.cs` case-collision index | Prefer `Dictionary<string, ...>(StringComparer.OrdinalIgnoreCase)` plus a collision report, or a small vetted helper; no heavy library needed | The bug is mostly data-structure policy, not a library gap. Built-ins are enough if collision detection is explicit. | Shadow indexes can desync from source-of-truth and hide duplicate-key behavior until content load. | YES | Small | P2 |

## Migration plan

1. **P0 first:** replace canonical JSON and pipe RPC framing, because they affect correctness, protocol stability, and cross-process failures.
2. **Parallelize asset work:** validate `meshoptimizer` integration in a throwaway branch; if native binding friction is high, keep the stub isolated behind an interface until the wrapper is proven.
3. **Then remove utility handrolls:** swap topo sort to `QuikGraph`, wire `JsonGuard` to `NJsonSchema`, and replace the dispatcher with `UniTask`-backed main-thread marshaling.
4. **Keep the registry fix simple:** use built-in case-insensitive lookup plus explicit duplicate reporting rather than introducing a library dependency.
5. **Gate each swap with regression tests:** canonical JSON vectors, pipe framing fuzz tests, DAG cycle cases, schema negative cases, and Unity playmode checks for dispatcher behavior.
