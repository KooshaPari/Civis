# Autograder CI Wire - 2026-06-08

Scope:

- Built on the existing e2e VLM judge/scorer in `src/Tests/e2e/vlm_judge.py`.
- Added a machine-readable CLI surface for the scorer instead of duplicating the existing coverage-gate workflow.
- Kept the change narrow to the e2e judge path and its documentation.

Changes:

- Added `main()` and argument parsing to `src/Tests/e2e/vlm_judge.py`.
- The scorer now runs as `python src/Tests/e2e/vlm_judge.py <screenshot> <assertion> [--model MODEL]`.
- The CLI prints JSON verdicts and returns non-zero on failed assertions, making it suitable for CI smoke checks or local autograder use.
- Added focused CLI tests in `src/Tests/e2e/test_vlm_judge_cli.py`.
- Updated `docs/screenshots/prove-features-validation/VALIDATION_SUMMARY.md` to describe the machine-readable command surface.

Validation:

- `python -m pytest src/Tests/e2e/test_vlm_judge_cli.py -q` succeeded: `2 passed`.
- `python src/Tests/e2e/vlm_judge.py --help` succeeded and showed the new scorer CLI usage.

Notes:

- No coverage-gate workflow logic was duplicated or modified.
- The scorer still depends on the Anthropic-backed VLM path used by the existing e2e tests.
