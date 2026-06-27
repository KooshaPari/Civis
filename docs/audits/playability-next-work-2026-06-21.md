# Playability next work backlog

**Date:** 2026-06-21  
**Scope:** Low-risk backlog from local docs/code signals only. No code changes.

## Reading model

- Prioritize work that removes blockers for actually playing, attaching, rendering, and validating the client.
- Source signals came from `AGENTS.md`, `docs/guides/agent-smoke.md`, `docs/development-guide/fr-ax-dx-ux-maturity-audit.md`, and the Bevy/Unreal client sources.
- Keep this as a task queue, not a design doc.

## P0. Playability gates and smoke reliability

1. Harden `scripts/agent-smoke.ps1` failure reporting for the playable block so it names the exact failed surface (`civ-server`, `civ-watch`, Unreal preflight/full build).
2. Make the playable block in `agent-smoke` emit a short pass/fail summary with the checked contract names.
3. Add a dedicated smoke note for the `civ_pins[].job` assertion so job shape regressions are obvious at first failure.
4. Expand the agent smoke doc to show which checks are required before a user can claim “playable” on each client.
5. Add a CI-facing smoke gate that runs the same playable sequence without depending on interactive client startup.
6. Split protocol drift from runtime playability in the verification docs so a failing catalog check does not mask a broken attach path.
7. Add a fast preflight for missing Unreal toolchain pieces before any expensive build attempt is queued.
8. Make the attach matrix explicitly call out which URLs and ports are mandatory for the playable path.

## P0. Renderer and frame-budget stability

9. Investigate the Bevy renderer’s `bevy_render.rs` chunk culling and frame setup for any avoidable per-frame allocation.
10. Reduce redundant mesh buffer conversions in the Bevy render path where the same data is reshaped multiple times.
11. Audit `voxel_stream.rs` for chunk churn that can cause visible hitches when the camera crosses stream boundaries.
12. Add a renderer-side frame budget note for smooth vs cubic meshing so the streaming path has an explicit target.
13. Tighten the chunk pre-pass in `voxel_stream.rs` so back-facing or out-of-range chunks are rejected earlier.
14. Revisit the smooth mesher path in `voxel_smooth_mesher.rs` for expensive blur/density work that can be amortized.
15. Add a guardrail for large `Vec::with_capacity` allocations in the hot render path when they scale with chunk count.
16. Validate the native HAL probe and Bevy render startup path for crash-only failures that stop the world before first frame.

## P1. Core client UI playability

17. Review `diplomacy_ui.rs` for control density and discoverability so the panel stays usable at normal window sizes.
18. Review `info_views.rs` overlay toggles and labels for first-time readability and hotkey clarity.
19. Normalize the left-panel UI flow so the user does not need to discover multiple competing overlay entry points.
20. Add a playability pass for `civ_history.rs` to ensure it is readable, bounded, and does not obscure the live game view.
21. Tighten camera defaults in `camera.rs` so the initial framing supports immediate map reading instead of manual recovery.
22. Recheck `settings_ui` feature-gated controls for exposed graphics and gameplay tabs so the main UI has no hidden dead ends.
23. Add an explicit on-screen indicator when egui-gated menus are compiled out, so “no menus” is not mistaken for a broken UI.
24. Make the UI theme and panel spacing in the Bevy client consistent across the major overlays to reduce cognitive load.

## P1. Unreal and attach-path parity

25. Confirm `CivWsClient` binary-frame handling stays aligned with the server’s frame kinds and does not silently drop newer payloads.
26. Audit `CivShowGameMode.cpp` for stale assumptions about snapshot shape when terrain, pins, or building diffs arrive.
27. Recheck `CivProtocolClient.cpp` control actions for spawn and damage so the Unreal path still mirrors the documented playable actions.
28. Validate `CivMinimapWidget.cpp` against the documented click-to-focus convention and keep the coordinate mapping obvious.
29. Add a parity note for the dense `voxels` path in Unreal so 16³ mesh fallback and marker-only fallback are easy to distinguish.

## P2. CI and regression coverage

30. Add a CI job that runs the Bevy client tests used by the playability path, not just the server and protocol checks.
31. Add a CI note for `just civis-3d-verify` so the web, catalog, scenario, and mod-host checks are presented as one contract.
32. Add a lightweight regression check for renderer startup that can catch first-frame failures before full client attach.
33. Add a frame-budget regression sentinel for the Bevy streaming path so chunk bursts cannot drift upward unnoticed.
34. Add a sanity check for `agent-smoke.ps1 -FullUnreal` output when UE is available, so the optional path remains trustworthy.
35. Add a docs-only checklist linking the smoke gates to the affected client surfaces: Bevy, Unreal, web dashboard, and server attach.

## P2. Documentation and operator clarity

36. Merge the multiple playability entry points into one short “start here” sequence for agents and developers.
37. Add a single paragraph that distinguishes “can attach” from “is playably rendered” for the three primary clients.
38. Add a direct mapping from local failure names to likely remediation surfaces (`renderer`, `streaming`, `attach`, `UI`, `toolchain`).
39. Capture the known optional-vs-required gate behavior so playability regressions fail loudly instead of becoming ambiguous warnings.
40. Add a compact backlog reference in the maturity audit that points to the next playability actions instead of only status labels.

## Suggested execution order

1. Finish P0 smoke and gate reliability first.
2. Then fix renderer and frame-budget issues that most affect first-frame and sustained play.
3. Then close UI and Unreal parity gaps.
4. Finally, codify the remaining checks in CI and docs so regressions stay visible.

## Signals used

- `AGENTS.md`
- `docs/guides/agent-smoke.md`
- `docs/development-guide/fr-ax-dx-ux-maturity-audit.md`
- `clients/bevy-ref/src/bevy_render.rs`
- `clients/bevy-ref/src/voxel_stream.rs`
- `clients/bevy-ref/src/voxel_smooth_mesher.rs`
- `clients/bevy-ref/src/diplomacy_ui.rs`
- `clients/bevy-ref/src/info_views.rs`
- `clients/bevy-ref/src/camera.rs`
- `clients/bevy-ref/src/civ_history.rs`
- `clients/unreal-show/Source/CivShow/CivWsClient.cpp`
- `clients/unreal-show/Source/CivShow/CivShowGameMode.cpp`
- `clients/unreal-show/Source/CivShow/CivProtocolClient.cpp`
- `clients/unreal-show/Source/CivShow/CivMinimapWidget.cpp`
