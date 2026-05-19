# Bridge Bypass Surface Audit

**Date**: 2026-04-25
**Auditor**: Infra audit subagent (iteration 37, parallel batch of 5)
**Scope**: `src/Tools/McpServer/GameBridgeServer.cs`, all bridge JSON-RPC
handlers, related test traits (`Category=E2E`, `Category=Journey`,
`Category=UserStory`), `FakeGameBridge` usage in tests
**Verdict**: SILENT-LIES

---

## Executive summary

`GameBridgeServer.cs` contains seven sites where the bridge silently bypasses
its own contract — returning a literal "success" or "running" value to a
caller while the underlying state is undefined, uninitialized, or actively
errored. This is the single most pernicious infra rot in the project:
upstream callers (MCP tools, prove-features gate, smoke tests) believe they
are talking to a healthy game and downstream judges accept the
self-reported state as ground truth.

In addition, two test classes labeled `[Trait("Category", "E2E")]` and
`[Trait("Category", "UserStory")]` actually use `FakeGameBridge` for their
entire body — making them trait-fraud. CI ran them under the "E2E" cohort
and they passed without ever talking to a real game.

Both surfaces were closed at source level in iteration 37 (#189 and #190),
but the audit findings are recorded here for the smart-contract proof system
spec to consume — the bridge bypass surface is precisely the class of
behavior that a "Bridge HMAC + signed status" wire would prevent.

---

## Findings table

| Surface | File:Line | Pre-fix behavior | Post-fix behavior |
|---------|-----------|-------------------|--------------------|
| `HandleStatus.Running` | GameBridgeServer.cs:557 | Literal `Running = true` regardless of state | `Running = worldReady && platformReady && sceneReady` |
| `HandleStatus` catch | GameBridgeServer.cs:614-616 | Swallow `catch (Exception)`, return success | Returns `Error="status_serialization_failed"`, `Running=false` |
| `HandleGetCatalog` empty | GameBridgeServer.cs ~700 | Empty fallback returns `IsValid=true` | Returns `Error="platform_not_initialized"`, `IsValid=false` |
| `HandleGetResources` empty | GameBridgeServer.cs ~770 | Empty fallback returns `IsValid=true` | Returns `Error="world_not_created"`, `IsValid=false` |
| `HandleDumpState` empty | GameBridgeServer.cs ~830 | Empty fallback returns `IsValid=true` | Returns `Error="resource_query_timeout"`, `IsValid=false` |
| `HandleApplyOverride.Success` | GameBridgeServer.cs:947-949 | `Success=true` regardless of count | `Success = modified > 0`; new `Enqueued` field |
| Trait-fraud `InGameAutomationTests` | InGameAutomationTests.cs:27 | `[Trait("Category","E2E")]` + `FakeGameBridge` body | Re-traited `Category=BridgeFake` (#195) |
| Trait-fraud `WorkflowE2ETests` | WorkflowE2ETests.cs:18-26 | `[Trait("Category","UserStory")]` + `FakeGameBridge` body | Re-traited `Category=BridgeFake` (#195) |
| Trait-fraud `BridgeRoundTripTests` | (added during scan) | `Category=Journey` + `FakeGameBridge` | Re-traited `Category=BridgeFake` (#195) |

---

## Concrete gaps

1. **`HandleStatus` returned literal `Running=true`.** Whether the world was
   created, the platform was initialized, or the scene had loaded — the
   handler returned `Running=true`. This is the single most-load-bearing
   silent lie in the bridge: every MCP `game_status` call, every
   prove-features gate, every smoke test that "checks the game is up" was
   satisfied by this literal. Fix in #189: `Running = worldReady &&
   platformReady && sceneReady`.

2. **`HandleStatus` swallowed exceptions.** `catch (Exception)` returned a
   default-constructed status with no error field. Any serialization or
   deeper-state fault was therefore invisible. Fix: return
   `Error="status_serialization_failed"`, `Running=false`.

3. **Empty-fallback paths in catalog/resources/dump.** Three handlers had
   "if not initialized, return empty success" branches. Empty success is
   indistinguishable from "everything is fine but the game has no entities".
   Fix: return explicit `Error="platform_not_initialized"`,
   `Error="world_not_created"`, `Error="resource_query_timeout"` with
   `IsValid=false`.

4. **`HandleApplyOverride` reported success without applying.** The handler
   ran the override pipeline, counted modified entities, and returned
   `Success=true` even when `modified=0`. Fix: `Success = modified > 0` plus
   a separate `Enqueued` field for the deferred-apply count.

5. **`CatalogSnapshot` / `ResourceSnapshot` DTOs lacked `IsValid` / `Error`
   fields.** Callers had no way to distinguish "real empty result" from
   "uninitialized". Fix: added both fields.

6. **`InGameAutomationTests` carried `Category=E2E`.** The class entirely
   uses `FakeGameBridge` for its body. `[Trait("Category", "E2E")]` is
   therefore false advertising. Fix in #195: re-trait to
   `Category=BridgeFake`.

7. **`WorkflowE2ETests` carried `Category=UserStory`.** Same pattern.
   Re-traited to `Category=BridgeFake`.

8. **`BridgeRoundTripTests` carried `Category=Journey`.** Same pattern.
   Re-traited.

9. **No CI guard against future trait-fraud.** Until #190 landed
   `scripts/analysis/check_trait_fraud.py`, nothing prevented a future test
   from declaring `Category=E2E` while using `FakeGameBridge`. The guard now
   runs in `policy-gate.yml` and rejects test classes with `Category=E2E |
   Journey | UserStory` and `FakeGameBridge` body unless
   `Trait("Bridge","Fake")` is also present.

---

## Tasks opened from this audit

- **#189 P1 INFRA** — Bridge silent-bypass surface (7 sites). Closed at source. 109/111 targeted tests pass; 2 follow-up failures tracked under #196.
- **#190 P1 INFRA** — Trait-fraud guard. Closed; wired into `policy-gate.yml`.
- **#195 P3** — Re-trait three offending classes. Closed; 42/42 affected tests pass.
- **#196 P3** — Two post-landing test failures (`GameClientFramingTests.ConnectAsync_WithCustomTimeout_UsesProvidedValue` regression family of #122; `UIContentLoader.LoadHudElementFile:115` YAML deserialization). Open.

Future: a Wave 2 (#191) "bridge HMAC + signed status" wire would make
"silent lies" cryptographically impossible — caller verifies a signature over
the status frame and rejects unsigned or stale frames. Currently the bridge
is unauthenticated loopback JSON-RPC.

---

## See also

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — Wave 1 owns #189/#190/#195
- `docs/sessions/2026-04-25-ci-manifest-gate-audit.md` — bridge HMAC is shared concern
- `docs/design/2026-04-25-smart-contract-proof-system.md` — Wave 2 spec covers bridge HMAC
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md` — empty DINOBox + lying status = false-green CI
- `docs/sessions/2026-04-25-sandbox-isolation-audit.md` — same dependency
- `docs/sessions/2026-04-25-ui-sdk-audit.md` — orphan UI registries report status via this bridge

---

## Verdict

The bridge surface was actively producing **silent lies** before iteration
37. Seven handlers returned success-shaped responses while underlying state
was undefined or errored, and three test classes claimed E2E / UserStory /
Journey traits while running entirely against `FakeGameBridge`. All seven
handlers and all three traits were corrected at source level. The remaining
risk is regression: the trait-fraud guard now blocks the test-side
recurrence, but the handler-side regressions need either Wave 2 bridge HMAC
or stricter unit tests on the new error fields. Until then, "bridge says
running" is honest but unsigned — trust degrades over time without the
smart-contract layer.
