# WGC Capture Backend End-to-End Verification Plan (#537)

Date: 2026-05-19

## Purpose
Validate that the WGC capture backend reliably captures DINO screenshots under edge cases where prior approaches are fragile.

This plan verifies capture success for all of these states:
1. DINO in DXGI exclusive fullscreen
2. DINO process hung/unresponsive
3. DINO window minimized
4. Baseline foreground responsive behavior

## Test scenarios

### 1) Foreground responsive
- Start DINO normally and ensure WGC capture target window is active.
- Trigger a single capture per second for 5 seconds.
- Save output as `artifacts/wgc/fg-responsive-###.png`.
- For baseline, run same capture with legacy `GameControlCli` named-pipe.

### 2) Foreground hung (force-stuck via Sleep)
- Start DINO normally, then force a hang with an injected `System.Threading.Thread.Sleep(...)` pause long enough to emulate unresponsive behavior.
- During hang window, trigger `game_screenshot_wgc` capture.
- Save output as `artifacts/wgc/fg-hung-###.png`.
- For baseline, execute same window/process state using `GameControlCli` screenshot command and confirm expected failure/timeouts.

### 3) Background minimized
- Start DINO, then minimize the window.
- Trigger capture and save `artifacts/wgc/background-minimized-###.png`.
- For baseline, attempt capture with legacy `GameControlCli` under identical minimized state.

### 4) Exclusive fullscreen
- Launch DINO in DXGI exclusive fullscreen mode.
- Capture one frame and save `artifacts/wgc/exclusive-fullscreen-###.png`.
- For baseline, attempt matching capture with legacy pipeline.

## Pass criteria (WGC)
Each scenario must satisfy all three checks:

1. **Non-trivial artifact size**
   - `Get-Item` file size strictly greater than `10KB`.

2. **Hash reproducibility and integrity**
   - Capture tool reports non-empty, deterministic output for retry window and writes valid image bytes.
   - Persist hash for audit: `Get-FileHash -Path <png> -Algorithm SHA256`.

3. **Visual sanity check via pixel sampling**
   - Must not be all-black (`R=0,G=0,B=0` for sampled pixels).
   - Must not be a fullscreen WorldBox bleed image (uniform/incorrect-content signature).
   - Sample strategy:
     - Read several spatially distributed pixels.
     - Confirm at least one high-entropy pixel cluster exists.
     - Confirm dominant color ratio is below an all-black threshold.

## Tooling

- `game_screenshot_wgc` (MCP tool)
  - Primary capture path under test.
- `scripts/diag/health-summary.ps1`
  - Environment sanity check before each scenario (processes, window visibility, fullscreen/minimized state breadcrumbs).
- `capture_wgc.py`
  - Orchestrates repeated captures, writes files under `artifacts/wgc`, and appends hash + pixel-sample metadata.

## Baseline comparison (legacy GameControlCli named-pipe)

Run the same 4 scenarios using the prior named-pipe capture path.

Expected baseline behavior:
- **Foreground responsive**: typically passes.
- **Foreground hung (Sleep forced unresponsive)**: expected fail (no usable response or timeout).
- **Background minimized**: mixed/unstable; capture path may stall or return stale image.
- **Exclusive fullscreen**: often fails depending on window transition/path coverage.

Capture matrix to record:
- Scenario
- Timestamp
- `game_screenshot_wgc` result + SHA256 + sample verdict
- `GameControlCli` result + timeout/error details

## Expected outcome chart

```mermaid
flowchart TD
  subgraph WGC[WGC backend]
    FG1[Foreground responsive] --> W1[PASS]
    FG2[Foreground hung (Sleep)] --> W2[PASS]
    BG[Background minimized] --> W3[PASS]
    FS[Exclusive fullscreen] --> W4[PASS]
  end

  subgraph Legacy[Legacy GameControlCli named-pipe]
    LB1[Foreground responsive] --> L1[PASS]
    LB2[Foreground hung (Sleep)] --> L2[FAIL]
    LB3[Background minimized] --> L3[FLAKY/FAIL]
    LB4[Exclusive fullscreen] --> L4[FAIL]
  end

  classDef pass fill:#0f7c2b,color:#fff;
  classDef fail fill:#9a1f1f,color:#fff;
  classDef flaky fill:#8f6b00,color:#fff;
  class W1,W2,W3,W4,L1 pass;
  class L2,L4 fail;
  class L3 flaky;
```

## Acceptance criteria
- **4/4 WGC scenarios captured successfully**.
- Each captured image is non-trivial (>10KB), hashed, and passes pixel-sampling sanity checks.
- Baseline comparison documented for all 4 scenarios.
- Artifacts and logs attached to this plan run record.
