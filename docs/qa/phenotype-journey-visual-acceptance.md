# Phenotype Journey Visual Acceptance Gate

**Date**: 2026-05-23  
**Scope**: phenotype journey screenshots, screenshot-backed proof, visual acceptance for journey evidence  
**Status**: **PARTIAL** — gate criteria defined; multi-step indexed PNGs captured (`live-bridge-journey_2026-05-23/steps/`); human or judge review still required before **ACCEPTED**

Use this gate when reviewing phenotype journey screenshots or screenshot-driven proof artifacts.

## Minimum manifest requirements

A journey promoted through this gate must satisfy the [phenotype manifest schema](../../tools/phenotype-journeys/schema/manifest.schema.json) minimums:

| Field | Requirement |
|-------|-------------|
| `id`, `intent` | Required identifiers |
| `steps[]` | At least **two** steps for visual acceptance (lighting or camera contrast across the set) |
| Per step | `index`, `slug`, `intent`, `screenshot_path` — each path must resolve to an on-disk PNG from a live capture |
| `keyframe_count` | Must equal `steps.length` |
| `passed` | Set `true` only after human or judge review against pass/reject criteria below |

**Optional capture layers** (viewer overlays; not required for gate review): `keyframes[]`, top-level `screenshots[]`, `captures.screenshots[]`, `recording_mp4` / `recording_gif`, `media_timeline`, OCR/SVG blocks. See [example manifest](../journeys/manifests/example-visual-evidence/manifest.json) and `tools/phenotype-journeys/npm/journey-viewer/fixtures/example-journey.json`.

**Production capture:** `phenotype-journey record` / `verify`, or `GameControlCli` demo + `phenotype-journey sync --from <artefact-dir> --to docs/journeys`.

## Minimum capture requirements (visual)

Before promoting evidence to **ACCEPTED**:

1. **Multi-frame set** — PNGs for every `steps[].screenshot_path`, captured from a live bridge (not mock placeholders).
2. **Contrast across frames** — At least one visible difference in lighting, camera angle, or material/shader state (per pass criteria).
3. **3D fidelity** — Frames show authored terrain/world content with depth; reject slab placeholders, flat water, black billboards, static noon-only sun with no variation.
4. **Verification** — `verification.mode` of `mock` or `api` with `all_intents_passed: true` when using automated judge; otherwise explicit human sign-off in the audit trail.

## Interim evidence (bridge smoke — not visual acceptance)

When full phenotype record is not yet available, use the live-bridge smoke path to prove bridge I/O only. **Do not** promote smoke output to visual **ACCEPTED** status.

```powershell
pwsh -File scripts/qa/live-bridge-journey-smoke.ps1
```

| Artifact | Path |
|----------|------|
| Step receipt (`overall_pass`, pipe/MCP/status/screenshot steps) | `docs/qa/evidence/live-bridge-journey_<date>/smoke-receipt.json` |
| Single bridge screenshot | `docs/qa/evidence/live-bridge-journey_<date>/bridge-status-screenshot.png` |

Prerequisites: game running with bridge plugin; `\\.\pipe\dinoforge-game-bridge` present; MCP at `http://127.0.0.1:8765/health`. `DINO_GAME_PATH` is informational — screenshot capture requires a live bridge response.

Latest smoke: [live-bridge-journey_2026-05-23](evidence/live-bridge-journey_2026-05-23/) (`smoke-receipt.json` — prior run `overall_pass: true`). See [Live Bridge / Journey Evidence Blocker Report](live-bridge-journey-evidence-blocker_2026-05-23.md).

## Multi-step capture (indexed steps — gate prep, not ACCEPTED)

Use when you need **two or more** indexed PNGs for manifest `steps[].screenshot_path` resolution (example: [example-visual-evidence manifest](../journeys/manifests/example-visual-evidence/manifest.json) → `docs/qa/evidence/live-bridge-journey_<date>/steps/step-NNN.png`).

```powershell
pwsh -File scripts/qa/live-bridge-journey-capture.ps1
pwsh -File scripts/qa/live-bridge-journey-capture.ps1 -EvidenceDate 2026-05-23
```

| Artifact | Path |
|----------|------|
| Capture receipt (`captured_count`, per-step slugs) | `docs/qa/evidence/live-bridge-journey_<date>/capture-receipt.json` |
| Indexed step PNGs | `docs/qa/evidence/live-bridge-journey_<date>/steps/step-000.png`, `step-001.png`, … |

Capture sequence (when bridge status passes): baseline frame → `toggle-ui debug` + frame → close debug + frame. Exit **0** when `captured_count >= 2`; exit **1** when skipped.

**Skip when game unavailable:** If the named pipe is absent or `GameControlCli status` does not report `Connected to game bridge`, the script writes `capture-receipt.json` with `capture_skipped` and **does not** create step PNGs. Re-run with the game and bridge plugin loaded.

**2026-05-23 run:** Capture **complete** — `live-bridge-journey-capture.ps1` wrote `step-000.png`, `step-001.png`, `step-002.png` under [live-bridge-journey_2026-05-23/steps](evidence/live-bridge-journey_2026-05-23/steps/) via `GameControlCli screenshot` (`captured_count: 3`, `overall_pass: true` in `capture-receipt.json`). Manifest paths in [example-visual-evidence](../journeys/manifests/example-visual-evidence/manifest.json) resolve on disk; visual acceptance remains **PARTIAL** until pass/reject criteria review (not promoted to **ACCEPTED**).

## Pass Criteria

- The screenshot shows authored final content, not placeholder geometry or fallback rendering.
- Lighting, shader, and material state are visibly exercised in the frame set.
- The scene reads as a real 3D environment with depth, terrain continuity, and active surface response.
- The screenshot set is consistent with the journey intent and the broad-change audit status.

## Reject Criteria

Reject the evidence if any of the following are present:

- Black billboard artifacts or other flat black card-like placeholders.
- Missing slope smoothing where terrain or ground transitions should be blended.
- 2.5D slab placeholders presented as final evidence.
- Flat water with no mesh, wave, or fluid visual.
- Shader or lighting toggles that do not produce any observable change.
- A static noon-only sun presentation that never changes across the evidence set.
- Flat unlit colors that do not appear to receive sun or scene lighting.

## Evidence Notes

- Prefer screenshot sets that show a visible difference between lighting states, camera angles, or material settings.
- If the live bridge cannot produce trustworthy journey evidence, do not promote the screenshot set to acceptance status.
- Treat the blocker in [Live Bridge / Journey Evidence Blocker Report](live-bridge-journey-evidence-blocker_2026-05-23.md) as **partially cleared** for bridge I/O; full phenotype manifests remain pending.

## Related References

- [Example visual-evidence manifest](../journeys/manifests/example-visual-evidence/manifest.json)
- [Current Broad-Change Completion Audit](current-broad-change-audit-2026-05-23.md)
- [Live Bridge / Journey Evidence Blocker Report](live-bridge-journey-evidence-blocker_2026-05-23.md)
- [Rendering Audit Blocker](rendering_audit_blocker_2026-05-23.md)
- [Asset Visual Acceptance Criteria](../reference/asset-visual-acceptance.md)
- [Journey viewer integration](../../tools/phenotype-journeys/npm/journey-viewer/INTEGRATION.md)
