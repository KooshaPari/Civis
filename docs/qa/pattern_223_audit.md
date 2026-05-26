# Pattern #223 Audit: TODO/FIXME/HACK Comments Without Ticket Reference

## Executive Summary

**Total unreferenced comments: 34**
**All occurrences are NOTE markers (educational comments, not action items)**
**Tier: LOW — handle as you touch the file**

## Definition

Pattern #223 detects comment markers (TODO, FIXME, HACK, XXX, NOTE) lacking ticket references, owner names, or dates. Unbounded lifecycle without tracking means these comments rot and become stale. The audit exempts comments that include:
- Ticket refs (`#\d+`, e.g. `#479`)
- GitHub URLs (`/issues/` or `/pull/`)
- Owner markers (`@username` or `(koosha)`)
- Date anchors (`202\d-*`)

## Detection Logic

Script: `scripts/ci/audit_todo_without_ticket.py` (124 LOC)
1. Walks `src/` recursively (excludes bin/, obj/, .Generated)
2. Regex-matches `// TODO|FIXME|HACK|XXX|NOTE` (case-insensitive)
3. Checks each match for ticket/URL/owner/date patterns
4. Reports violations by kind, file, line

## Breakdown by Kind

| Kind | Count | Severity |
|------|-------|----------|
| NOTE | 34    | LOW      |
| TODO | 0     | —        |
| FIXME| 0     | —        |
| HACK | 0     | —        |
| XXX  | 0     | —        |

## Top 15 Violations

| # | File | Line | Kind | Excerpt |
|---|------|------|------|---------|
| 1 | Bridge/Client/CanonicalJson.cs | 26 | NOTE | The deterministic JSON canonicalizer formerly defined in thi... |
| 2 | Runtime/Bridge/ComponentMap.cs | 233 | NOTE | These primary mappings are used by ResourceReader's fallback... |
| 3 | Runtime/Bridge/KeyInputSystem.cs | 214 | NOTE | Input.GetKey() uses MonoBehaviour.Update() which NEVER fires... |
| 4 | Runtime/Bridge/ProjectileVFXSystem.cs | 49 | NOTE | EntityQueries is in the same namespace (DINOForge.Runtime.Br... |
| 5 | Runtime/UI/DebugPanel.cs | 146 | NOTE | MonoBehaviour.Update() NEVER fires in DINO (Unity PlayerLoop... |
| 6 | Runtime/UI/DFCanvas.cs | 160 | NOTE | F9/F10 key handling has been intentionally moved to RuntimeD... |
| 7 | Runtime/UI/ModMenuOverlay.cs | 91 | NOTE | F10 toggling has been moved to RuntimeDriver.Update() so tha... |
| 8 | Runtime/UI/UiGridHarmonyPatch.cs | 19 | NOTE | Patching <c>TMP_Text.set_text</c> (virtual) fails on Mono/Ha... |
| 9 | Tests/EndToEndUserJourneysTests.cs | 458 | NOTE | YAML must not have leading whitespace on each line (verbatim... |
| 10 | Tests/GameClientCoverageTests.cs | 701 | NOTE | ConnectAsync_WhenAlreadyConnecting test is not applicable be... |
| 11 | Tests/InstallerCoverageTests.cs | 1154 | NOTE | On Windows with actual Steam, this might return a valid path... |
| 12 | Tests/Integration/ParallelGameTestsWithHarness.cs | 26 | NOTE | This test class is skipped in CI/CD environments where DINOB... |
| 13 | Tests/Integration/Tests/GameWorkflowTests.cs | 187 | NOTE | This test documents the expected behavior. The real WaitForW... |
| 14 | Tests/Integration/Tests/InGameAutomationTests.cs | 253 | NOTE | DumpState is not implemented in FakeGameBridge... |
| 15 | Tests/Integration/Tests/InGameAutomationTests.cs | 265 | NOTE | game_navigate_to has a known bug with Spectre.Console "cat"... |

## Directory Heat-Map

| Directory | Count | Role |
|-----------|-------|------|
| Bridge | 1 | JSON-RPC protocol layer |
| Runtime | 7 | Plugin bootstrap + ECS bridge |
| Tests | 26 | Unit + integration test suites |

**Observation**: 76% of violations are in Tests/ and explain test skip behavior, test-only APIs, and integration constraints. These are educational (NOT action items) hence categorized as NOTE rather than TODO.

## Tier Classification

**LOW (34 violations)**

All 34 markers are NOTE comments documenting:
- ECS execution-context constraints (`MonoBehaviour.Update() NEVER fires`)
- Test skip conditions (`skipped in CI/CD environments`)
- Known limitations (`game_navigate_to has a known bug`)
- API constraints (`test is not applicable`)

None are action TODOs awaiting completion. Cleanup scope: handle as you touch the file (no scheduled sweep required).

## Promotion Judgment

NOTE markers are informational documentation, not deferred work — no Pattern Catalog entry required; retire this audit as informational-only.
