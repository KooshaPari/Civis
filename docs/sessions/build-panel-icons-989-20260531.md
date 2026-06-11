# DINOForge #989 — Build-Panel Icons Regression

## Investigation
- Branch: `feat/bldicons-20260531`
- Files reviewed: `src/Runtime/UI/CanvasReskinner.cs`, `src/Runtime/UI/NativeMenuInjector.cs`, `src/Runtime/UI/NativeUiHelper.cs`, `src/Runtime/UI/ModMenuPanel.cs`
- No active sprite-null assignments were found in `CanvasReskinner.cs` and `ModMenuPanel.cs`.
- In `NativeUiHelper.CopySelectableVisualState`, we found an unconditional assignment that could clear visuals:
  - `src/Runtime/UI/NativeUiHelper.cs:245` → `resolvedImage.sprite = donorImage.sprite;`
- In `NativeMenuInjector.CopyImageVisualStyle`, we found another unconditional assignment that could clear visuals:
  - `src/Runtime/UI/NativeMenuInjector.cs:1696` → `target.sprite = source.sprite;`

## Fix
- Guarded both sprite copies to only write when the donor sprite is non-null.
- This keeps existing cloned/background sprites intact when source sprites are missing.
- If a donor sprite is available, existing copy behavior remains unchanged.

## Build status
- Command: `dotnet build src/Runtime`
- Result: exited 0 (success)
