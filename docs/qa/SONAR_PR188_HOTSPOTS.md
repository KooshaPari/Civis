# SonarCloud PR 188 — Security Hotspot Manual Follow-up

Source: SonarCloud public API (`/api/hotspots/search`; `issues/search` with `types=SECURITY_HOTSPOT` is not supported).
Fetched: 2026-05-26T04:51Z UTC · project `KooshaPari_Dino` · pull request **188** · **44** hotspots (status `TO_REVIEW`).

## Checklist

| Rule | File | Line | Recommended action | Notes |
| --- | --- | ---: | --- | --- |
| `githubactions:S7637` | `.github/workflows/polyglot-build.yml` | 55 | **fix** | Pin actions to full commit SHA. |
| `githubactions:S7637` | `.github/workflows/polyglot-build.yml` | 280 | **fix** | Pin actions to full commit SHA. |
| `githubactions:S7635` | `.github/workflows/release-drafter.yml` | 16 | **review** | Minimize secrets passed to reusable workflow. |
| `githubactions:S7637` | `.github/workflows/release.yml` | 413 | **fix** | Pin actions to full commit SHA. |
| `python:S5852` | `docs/scripts/retired/audit_unsealed_concrete_classes.py` | 40 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `python:S5852` | `scripts/analysis/enumerate_mock_theater.py` | 24 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `python:S5852` | `scripts/analysis/enumerate_orphan_classes.py` | 59 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `python:S5852` | `scripts/analysis/enumerate_orphan_classes.py` | 67 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `python:S5852` | `scripts/analysis/enumerate_orphan_classes.py` | 69 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `python:S5852` | `scripts/ci/audit_empty_catch_blocks.py` | 31 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_public_fields.py` | 28 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_public_fields.py` | 81 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_static_init_side_effects.py` | 88 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_static_init_side_effects.py` | 95 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_static_init_side_effects.py` | 114 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_todo_without_ticket.py` | 17 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/audit_todo_without_ticket.py` | 24 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/changelog_lint.py` | 68 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/check_framework_version.py` | 206 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_catch_swallow_default.py` | 96 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_event_lifecycle_asymmetry.py` | 119 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_event_lifecycle_asymmetry.py` | 127 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_httpclient_per_instance.py` | 101 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_implicit_encoding.py` | 87 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_implicit_encoding.py` | 101 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_implicit_encoding.py` | 115 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_implicit_encoding.py` | 129 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_orphan_process_start.py` | 306 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_pattern_226.py` | 72 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_silent_catch.py` | 52 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_stringbuilder_no_capacity.py` | 55 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_stringbuilder_no_capacity.py` | 59 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_stringbuilder_no_capacity.py` | 60 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_stringly_enums.py` | 159 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_sync_over_async.py` | 84 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_sync_over_async.py` | 99 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_sync_over_async.py` | 103 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/detect_sync_over_async.py` | 104 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5852` | `scripts/ci/schema_drift_check.py` | 68 | **accept** | CI-only regex; multicriteria ignore in sonar-project.properties. |
| `python:S5332` | `scripts/ci/schema_drift_check.py` | 264 | **review** | Confirm localhost/test URL; use https if external. |
| `python:S5332` | `scripts/ci/schema_drift_check.py` | 340 | **review** | Confirm localhost/test URL; use https if external. |
| `csharpsquid:S6444` | `src/Domains/UI/Models/ThemeDefinition.cs` | 93 | **fix** | Add Process/command timeouts. |
| `python:S5852` | `src/Tools/DinoforgeMcp/dinoforge_mcp/journey_keyframe_tagger.py` | 147 | **review** | Make sure the regex used here, which is vulnerable to polynomial runtime due to backtracking, cannot lead to denial of service. |
| `csharpsquid:S6444` | `src/Tools/Installer/InstallerLib/InstallLifecycle.cs` | 490 | **fix** | Add Process/command timeouts. |

## Summary by recommended action

| Action | Count |
| --- | ---: |
| fix | 5 |
| review | 9 |
| accept | 30 |

## Follow-up

- **fix (6):** C# `S6444` timeouts (2), GitHub Actions `S7637` SHA pins (3).
- **accept (29):** `scripts/ci/**` ReDoS (`S5852`) — dev-only, bounded inputs; mark Safe in Sonar after scan exclusions land.
- **review (9):** non-CI regex, HTTP strings, workflow secret scope.

