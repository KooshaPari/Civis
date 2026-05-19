# First External Kimi Judge Receipt — Runbook

**Goal:** land DINOForge's first external (non-Anthropic) judge receipt in `docs/proof/judge-receipts/`. Until this exists, every "verified" claim in the project is self-grading.

**Time budget:** ~5 minutes once the prerequisites are met.

## Prerequisites

1. **MOONSHOT_API_KEY** — get one at <https://platform.moonshot.cn/console/api-keys>. The free tier is enough for proof bundles.
2. **MCP server running** at `http://127.0.0.1:8765`. Check via `Invoke-RestMethod http://127.0.0.1:8765/health`. If down, run `pwsh scripts/start-mcp.ps1 -Action start -Detached`.
3. **DINOForge-modded DINO running**. Either main install or a DINOBox via `pwsh scripts/game/Launch-DINOBoxInstance.ps1 -BoxPath G:\dino_boxes\box_1`. The runbook only needs a screenshot of *something* the feature affects.
4. **`httpx` installed in the MCP server's Python env**. Verify: `python -c "import httpx"`.

## Steps

### 1. Set the API key in the current shell

```powershell
$env:MOONSHOT_API_KEY = "<your key>"
```

Confirm with `[Environment]::GetEnvironmentVariable('MOONSHOT_API_KEY', 'Process')`. **Do NOT set it system-wide.** The MCP server inherits process env when it spawns subprocesses; if it was started before you set the key, restart it (`pwsh scripts/start-mcp.ps1 -Action restart -Detached`).

### 2. Take a screenshot via MCP

```powershell
$shot = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:8765/tools/game_screenshot -Body '{}' -ContentType 'application/json'
$shotPath = $shot.path
"Screenshot at $shotPath"
```

Pick something in the game window that's tied to a feature you want to verify — a HUD element after a stat override, a Star Wars unit visual after the AssetSwap fix, a menu after pack hot-reload, etc.

### 3. Call game_analyze_screen with external_judge=True

```powershell
$body = @{
    screenshot_path = $shotPath
    prompt = "Does this screenshot show <your specific feature claim>? Answer pass or fail with one sentence of reasoning."
    external_judge = $true
} | ConvertTo-Json

$result = Invoke-RestMethod -Method Post -Uri http://127.0.0.1:8765/tools/game_analyze_screen -Body $body -ContentType 'application/json'
$result | ConvertTo-Json -Depth 5
```

Expected outcome on success: the response includes `external_verdict`, `external_receipt_path`, and (if the local CLIP tier also ran) `disputed`. The receipt JSON itself is at `docs/proof/judge-receipts/<utc-timestamp>-<sha8>.json` in the repo.

### 4. Confirm the receipt is on disk

```powershell
Get-ChildItem docs\proof\judge-receipts\*.json | Sort-Object LastWriteTime -Descending | Select-Object -First 3 | Format-Table Name, Length, LastWriteTime
```

Open the most recent: `code docs/proof/judge-receipts/<file>.json`. Verify:
- `model` field starts with `moonshot-` (NOT `claude-` or `codex-` — those are rejected by the gate).
- `screenshot_sha256` matches what `Get-FileHash $shotPath -Algorithm SHA256` returns.
- `verdict` is `pass` / `fail` / `uncertain`.
- `raw_response.choices[0].message.content` has the model's reasoning verbatim.

### 5. Add it to TRUTH_TABLE.md

Edit `docs/TRUTH_TABLE.md` and update the row for the feature you just judged. Replace the ❌ or 🟡 with ✅ REAL and cite the receipt path in the Evidence column.

### 6. Bind the gate

`prove-features-gate.ps1` already rejects `claude-*` / `codex-*` model fields. Confirm by running it once with `-ExternalJudge` flag — it should pass when a receipt exists, fail when none was produced or all are Anthropic-family.

## Troubleshooting

- **`ExternalJudgeUnavailable: MOONSHOT_API_KEY not set`** — env var didn't propagate to the MCP server process. Restart MCP after setting the key.
- **`KeyError: 'choices'`** — Moonshot API may have rate-limited or returned an error. Print `result.raw_response` and read the error.
- **Receipt has `model: "claude-haiku-..."`** — the call accidentally took the Anthropic fallback path. That should not happen with the no-silent-fallback invariant; if it does, check `external_judge.py` for regression and that `external_judge=True` was passed correctly.
- **`disputed: true`** — external Kimi judge disagrees with local CLIP tier. This is a feature, not a bug. Do not promote the bundle. Investigate the actual screenshot.

## Why this matters

Every `prove-features` bundle before this runbook ran was Claude grading Claude. After this lands, DINOForge has at least one feature with verifiable, replayable, external-judged proof on disk. That's the artifact "verified" claims are supposed to point at.

Replay command: any reviewer can re-run this runbook with the same screenshot, get a fresh receipt, and compare verdicts. That's what makes the receipt load-bearing instead of a vibe.
