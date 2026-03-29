# /eval-game-features

Evaluate DINOForge extended game features via MCP + VLM screenshots.

Validates 5 additional features beyond the core 3 (Mods button, F9 overlay, F10 menu):
- **Feature A-2**: Stat override (HP=999 applied dynamically)
- **Feature A-3**: Hot reload (YAML change → game picks up without restart)
- **Feature A-4**: Economy pack loaded + resources queryable
- **Feature A-5**: Scenario pack loaded + system active
- **Feature A-6**: Asset swap (clone trooper visual model visible)

Evidence: MCP tool responses + VLM-confirmed screenshots → `validate_extended_report.json`

## Prerequisites

- Game must be running with DINOForge Runtime loaded
- MCP server must be running on `http://127.0.0.1:8765` (or `/launch-game` if not started)
- Run `/prove-features` first to boot the game to stable state, OR verify game is on gameplay screen

---

## Orchestration

All steps run from repo root. Execute each step in order; stop on first validation failure.

---

### Step 1: Verify game bridge is ready

```
Call game_status MCP tool.
Assert: running=true, entityCount > 0, activeScene contains "gameplay" or similar.
Output: Capture the raw response JSON (needed for report).
```

**MCP Call**:
```
Tool: game_status
```

**Validation**:
- `running` must be `true`
- `entityCount` must be > 0 (indicates ECS world is populated)
- If either fails: STOP and ask user to run `/prove-features` to boot game properly

**On Success**: Continue to Feature A-2.

---

### Step 2A: Evaluate stat override (Feature A-2) — Apply HP override

**Objective**: Verify that `game_apply_override` can modify a unit's stat dynamically, and the game reflects it.

```
1. Call game_apply_override:
   - sdk_path: "unit.stats.hp"
   - value: 999.0
   - mode: "override"
   - description: "Extended feature test — increase unit HP to 999"

2. Wait 2 seconds for game to process the override.

3. Call game_get_stat:
   - sdk_path: "unit.stats.hp"
   - Validate response contains a numeric value close to 999 (within 1.0 tolerance).
   - Capture raw response JSON.

4. If stat query returns ~999: Mark A-2 as "stat_apply_success: true".
   If stat query does NOT return ~999: Mark as "stat_apply_success: false", continue anyway to screenshot.

5. Press F9 via game_input to open debug overlay (to show live stats panel if available).

6. Wait 1 second for overlay to render.

7. Call game_screenshot, save to: $env:TEMP\DINOForge\capture\validate_stat_override.png

8. Call game_analyze_screen with prompt:
   "Does this screenshot show a debug overlay or in-game HUD displaying a health/HP stat value of approximately 999 or higher? Look for numeric displays showing '999', 'HP: 999', or similar. If you see a stats panel, report what HP value is displayed."

   Capture full VLM response.

9. Press F9 again to close overlay.

10. Write to validate_extended_report.json:
    {
      "feature": "A-2_stat_override",
      "stat_applied": true/false,
      "stat_value_queried": <numeric>,
      "vlm_confirmed": true/false,
      "vlm_response": "...",
      "screenshot": "validate_stat_override.png",
      "timestamp": "<ISO 8601>"
    }

11. If VLM confirmed OR stat value is ~999: Continue to A-3.
    If both failed: Log failure but continue anyway (do not stop pipeline).
```

---

### Step 2B: Evaluate hot reload (Feature A-3) — Trigger pack reload

**Objective**: Verify that `game_reload_packs` triggers live reload without restarting the game.

```
1. Call log_tail:
   - lines: 50
   - Capture the output. Note the last timestamp/line count as "baseline".

2. Call game_reload_packs (no arguments).
   - This writes the HMR signal file and triggers soft reload in game.
   - Wait 5 seconds for reload to complete.

3. Call log_tail:
   - lines: 50
   - Search new output for keywords: "reload", "pack", "hot", "HMR", "swap".
   - If any keyword appears: Mark "hot_reload_triggered: true".
   - If no keywords: Mark "hot_reload_triggered: false".

4. Call game_screenshot, save to: $env:TEMP\DINOForge\capture\validate_hot_reload.png

5. Call game_analyze_screen with prompt:
   "Is the game still running and responsive? Look for gameplay elements (units, terrain, HUD). Is there any error dialog, crash message, or 'fatal error' text? Report whether the game appears stable and playable."

   Capture full VLM response.

6. Write to validate_extended_report.json:
   {
     "feature": "A-3_hot_reload",
     "reload_triggered": true/false,
     "game_stable_post_reload": true/false,
     "vlm_response": "...",
     "screenshot": "validate_hot_reload.png",
     "log_evidence": "<tail excerpt with reload keyword>",
     "timestamp": "<ISO 8601>"
   }

7. Continue to A-4 (do not stop on failure).
```

---

### Step 3: Evaluate economy pack (Feature A-4) — Verify economy pack is loaded

**Objective**: Verify that the economy-balanced pack is loaded and resources are queryable.

```
1. Call game_status.
   - Check the "packsLoaded" array in response.
   - If "economy-balanced" appears in array: Mark "economy_pack_loaded: true".
   - If NOT in array: Mark "economy_pack_loaded: false", continue anyway.

2. Call game_get_resources (no arguments).
   - Captures current resource state (gold, food, wood, etc.).
   - Validate response is valid JSON with at least one resource field (gold/food/wood/stone/etc.).
   - Capture raw response JSON. If response has no resource fields: Mark "resources_queryable: false".
   - If response has >= 1 resource field: Mark "resources_queryable: true".

3. Call game_screenshot, save to: $env:TEMP\DINOForge\capture\validate_economy.png

4. Call game_analyze_screen with prompt:
   "Does this screenshot show a game HUD with visible resource counters or economy indicators? Look for labels like 'Gold:', 'Food:', 'Wood:', 'Stone:' with numeric values displayed. If you see resource counters, report what resources are shown and their approximate values."

   Capture full VLM response.

5. Write to validate_extended_report.json:
   {
     "feature": "A-4_economy_pack",
     "pack_loaded": true/false,
     "resources_queryable": true/false,
     "resources_sample": {<captured from game_get_resources response>},
     "vlm_confirmed": true/false,
     "vlm_response": "...",
     "screenshot": "validate_economy.png",
     "timestamp": "<ISO 8601>"
   }

6. Continue to A-5 (do not stop on failure).
```

---

### Step 4: Evaluate scenario pack (Feature A-5) — Verify scenario pack is loaded

**Objective**: Verify that the scenario-tutorial pack is loaded and the scenario system is active.

```
1. Call game_status.
   - Check "packsLoaded" array.
   - If "scenario-tutorial" appears: Mark "scenario_pack_loaded: true".
   - If NOT: Mark "scenario_pack_loaded: false", continue anyway.

2. Call log_tail:
   - lines: 100
   - Search for keywords: "ScenarioRunner", "VictoryCondition", "scenario", "difficulty".
   - If any keyword appears: Mark "scenario_system_active: true".
   - If no keywords: Mark "scenario_system_active: false".

3. Call game_screenshot, save to: $env:TEMP\DINOForge\capture\validate_scenario.png

4. Call game_analyze_screen with prompt:
   "Is this screenshot showing an active game session in progress? Look for gameplay elements like units on a map, terrain, resource HUD, and faction indicators. Is the game actively playing (not on a menu or loading screen)?"

   Capture full VLM response.

5. Write to validate_extended_report.json:
   {
     "feature": "A-5_scenario_pack",
     "pack_loaded": true/false,
     "scenario_system_active": true/false,
     "vlm_gameplay_active": true/false,
     "vlm_response": "...",
     "screenshot": "validate_scenario.png",
     "log_evidence": "<excerpt with scenario keyword>",
     "timestamp": "<ISO 8601>"
   }

6. Continue to A-6 (do not stop on failure).
```

---

### Step 5: Evaluate asset swap (Feature A-6) — Verify Star Wars clone trooper visual

**Objective**: Verify that the asset swap system has replaced unit models with Star Wars clone trooper visuals.

```
1. Call log_swap_status (no arguments).
   - Captures current asset swap status and completed swap count.
   - Parse response for "completed_swaps" count.
   - If count > 0: Mark "asset_swaps_completed: true".
   - If count == 0: Mark "asset_swaps_completed: false".

2. Call game_dump_state:
   - category: "unit"
   - Dump the ECS unit entity data to a temporary file.
   - Search dump output for entities with prefab name or visual_asset containing "clone", "trooper", "rep", "sw" (Star Wars prefix).
   - If found: Mark "clone_trooper_entities_present: true".
   - If NOT found: Mark "clone_trooper_entities_present: false".

3. Call game_screenshot, save to: $env:TEMP\DINOForge\capture\validate_asset_swap.png

4. Call game_analyze_screen with prompt:
   "Does this screenshot show any unit models that look like futuristic armored soldiers or Star Wars-style clone troopers (rather than default medieval fantasy units)? Look for sleek white/grey armor, helmet designs, or futuristic uniforms. If you see Star Wars-style units, describe their appearance."

   Capture full VLM response.

5. Write to validate_extended_report.json:
   {
     "feature": "A-6_asset_swap",
     "swaps_completed": <count>,
     "clone_trooper_entities_found": true/false,
     "vlm_starwars_visual_detected": true/false,
     "vlm_response": "...",
     "screenshot": "validate_asset_swap.png",
     "timestamp": "<ISO 8601>"
   }

6. Continue to final bundling step (do not stop on failure).
```

---

### Step 6: Bundle validation results

```
1. Aggregate all feature results into a single JSON report:
   - Path: $env:TEMP\DINOForge\capture\validate_extended_report.json
   - Structure:
     {
       "timestamp": "<ISO 8601>",
       "game_status_baseline": {<game_status response from Step 1>},
       "features": [
         {<A-2 result>},
         {<A-3 result>},
         {<A-4 result>},
         {<A-5 result>},
         {<A-6 result>}
       ],
       "overall_summary": {
         "total_features_evaluated": 5,
         "confirmed_count": <count of features with confirmed=true>,
         "failed_count": <count of features with confirmed=false>
       }
     }

2. Count "confirmed" features (any result with confirmed: true or a major assertion passing).
   - If >= 3 features confirmed: Overall status is "PASS".
   - If < 3 features confirmed: Overall status is "PARTIAL PASS".

3. Write $env:TEMP\DINOForge\capture\validate_extended_report.json to disk.

4. Copy all validation artifacts to persistent bundle:
   $ts = Get-Date -Format "yyyyMMdd_HHmmss"
   $bundle = "docs/proof-of-features/extended_$ts"
   Create $bundle directory.
   Copy all .png screenshots from $env:TEMP\DINOForge\capture\ to $bundle\.
   Copy validate_extended_report.json to $bundle\.

5. Generate $bundle\EXTENDED_EVAL_REPORT.md from the JSON:

   # DINOForge Extended Feature Evaluation — <timestamp>

   ## Summary
   | Feature | Result | Evidence |
   |---------|--------|----------|
   | A-2: Stat Override | PASS/FAIL | validate_stat_override.png |
   | A-3: Hot Reload | PASS/FAIL | validate_hot_reload.png |
   | A-4: Economy Pack | PASS/FAIL | validate_economy.png |
   | A-5: Scenario Pack | PASS/FAIL | validate_scenario.png |
   | A-6: Asset Swap | PASS/FAIL | validate_asset_swap.png |

   **Overall**: <X/5 features confirmed>

   ## Details
   For each feature, include the VLM response and key assertions (stat value, pack loaded, etc.).

   ## Artifacts
   - Screenshots: 5× PNG (one per feature)
   - Report: validate_extended_report.json
   - Build: docs/proof-of-features/extended_<timestamp>/

6. Print success message:
   "Extended feature evaluation complete: <X/5 features confirmed>.
    Bundle: docs/proof-of-features/extended_<timestamp>/
    Report: EXTENDED_EVAL_REPORT.md"
```

---

## Error Handling

- **If game is not running** at Step 1: Print "Game not running. Start with /launch-game or /prove-features." Exit with status code 1.
- **If MCP server is unreachable**: Print "MCP server not responding on http://127.0.0.1:8765. Start with 'python src/Tools/DinoforgeMcp/server.py'" Exit with status code 1.
- **If any individual feature fails validation**: Log the failure, but **continue to the next feature** (do not stop pipeline). Mark the feature as `confirmed: false`.
- **If < 3 features confirm**: Return overall status PARTIAL PASS, do not fail the pipeline (continue to bundling).

---

## VLM Model Selection

For `game_analyze_screen` VLM calls, use the weakest capable model (fastest + lowest cost):
1. Claude Opus (if available and budget allows)
2. Claude Sonnet 4 (preferred balance)
3. Claude Haiku (fallback, fast mode OK)

`game_analyze_screen` handles the actual screenshot capture; the model only needs to describe what it sees.

---

## Cleanup / Revert

**After all steps complete**:
- If any YAML files were modified during hot reload testing: **REVERT THEM** (do not leave test changes in repo).
- Leave screenshots and reports in `docs/proof-of-features/extended_<timestamp>/` for review.
- Game may remain running (do not force kill unless asked).

---

## Success Criteria

✓ All 5 features evaluated (even if some fail validation)
✓ >= 3 features confirmed (stat override, hot reload, economy, scenario, asset swap)
✓ `validate_extended_report.json` written to disk
✓ Evidence bundle created in `docs/proof-of-features/extended_<timestamp>/`
✓ VLM screenshots captured and analyzed for each feature
