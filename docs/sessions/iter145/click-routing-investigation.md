# Iter-145 Click Routing Investigation Retrospective

**Date**: 2026-05-22  
**Status**: Unresolved. The in-game click path is still broken after the deployed fix set.

## Timeline

1. The session opened on a persistent gameplay regression: the **MODS button** and **F10 menu** were both uninteractable in-game, while **F-keys still worked**. The UI also showed **0 packs** even though **9 packs were loaded**.
2. A fix bundle was deployed at hash **`4fbca9f8`**. It included Pattern #235 EventSystem ordering, bridge framing to NDJSON, the LoadChicken defer-init coroutine, MODS hover-sprite preservation, `ModMenuPanel` rect expansion from `133` to `2496x1376`, `SetPacks-after-load` wiring, `DFCanvas` `raycaster.enabled=true`, and a `DFCanvas Update()` EventSystem watchdog.
3. After deployment, user verification still failed. The buttons remained uninteractable and the pack counter still reported **0**.
4. Bridge `ui-click` telemetry reported `actionable=true`, which initially looked like progress, but that signal was misleading. The actual mouse path was still broken because `ExecuteEvents.Execute` bypasses the EventSystem chain.
5. That mismatch explained several false-positive “verified” claims during the session. The bridge was proving the synthetic route, not the real in-game route.
6. A full diagnostic agent, **`b98e72l4`**, is now in flight with three hypotheses: **H1** dual EventSystem, **H2** CanvasGroup cascade, **H3** `sortingOrder` priority plus a 0-packs trace.

## Symptoms

- MODS button uninteractable in-game.
- F10 menu uninteractable in-game.
- F-keys still function, so keyboard input is not globally dead.
- 0 packs displayed despite 9 loaded packs.
- Bridge click telemetry can report `actionable=true` even when the real mouse path fails.

## Failed Approaches

- Treating bridge `ui-click` output as proof of a working user interaction path.
- Assuming `ExecuteEvents.Execute` validation covered the same chain as a real mouse click.
- Relying on the deployed fix bundle alone to restore interactivity.
- Assuming the pack-count problem was solved by the same fixes that targeted UI click routing.
- Marking any step “verified” before the live in-game mouse path was checked.

## What Worked

- The deployed hash **`4fbca9f8`** established a clear, shared baseline of attempted fixes.
- User verification caught the remaining failure instead of letting telemetry-only success stand in for reality.
- The false-positive mechanism is now understood: `actionable=true` from the bridge is advisory, not proof.
- The in-flight `b98e72l4` diagnosis gives a concrete next step instead of another unfocused patch pass.

## Governance Updates

- Saved **`feedback_telemetry_first_before_iteration.md`**.
- New rule: always capture **in-game telemetry first** before iterating code fixes.
- Bridge or synthetic click reports must be treated as supporting evidence only.
- A fix is not “verified” until the real mouse path and the in-game UI state both match the intended result.

## Open Path

- Continue the **`b98e72l4`** full diagnostic run.
- Test the three current hypotheses in order:
  - H1: dual EventSystem
  - H2: CanvasGroup cascade
  - H3: `sortingOrder` priority plus the 0-packs trace
- Trace the pack-count failure separately if needed; do not assume it is the same root cause as the click path.
- Capture in-game telemetry before the next iteration so the next fix cycle starts from ground truth, not bridge-only success.
