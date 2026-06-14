# Autograder Spec Shape - 2026-06-08

Scope:

- Extended the external-judge autograder tests to assert the spec-shaped JSON payload uses snake_case keys.
- Kept the existing tick10/tick11 CLI coverage work untouched.
- Limited the change to the autograder judge surface in `src/Tools/DinoforgeMcp`.

Changes:

- Added `JudgeReceipt.to_autograder_dict()` in `src/Tools/DinoforgeMcp/dinoforge_mcp/external_judge.py`.
- The autograder payload now exposes the required top-level keys:
  - `tier`
  - `score`
  - `pass`
  - `gaps`
  - `evidence`
- Extended `src/Tools/DinoforgeMcp/tests/test_external_judge.py` with a focused contract test that asserts the exact key set and basic value shape.

Validation:

- `python -m pytest src/Tools/DinoforgeMcp/tests/test_external_judge.py -q` succeeded.

Notes:

- This intentionally does not duplicate the tick10/tick11 work in `src/Tests/e2e`.
- The new contract is additive: the existing receipt persistence and verdict parsing tests still stand.
