# Tier-1 Scoring Wire - 2026-06-08

## Scope

- Wired `docs/specs/SPEC-TIER1-MVP.md` into the Autograder test surface through a focused acceptance-contract engine.
- Added one pytest that loads the canonical spec file and asserts the MVP payload.
- Added the traceability row required by the spec contract.

## Files Changed

- `src/Tools/DinoforgeMcp/dinoforge_mcp/acceptance_contract_engine.py`
- `src/Tools/DinoforgeMcp/tests/test_acceptance_contract_engine.py`
- `docs/specs/traceability-matrix.md`

## Exact Assertion Lines

The new autograder test asserts:

```python
assert result["tier"] == "mvp"
assert result["score"] == pytest.approx(1.0)
assert result["pass"] is True
assert result["gaps"] == []
```

## Validation

Targeted test run:

```bash
python -m pytest src/Tools/DinoforgeMcp/tests/test_acceptance_contract_engine.py -q
```

Expected outcome:

- `tier=mvp`
- `pass=true`
- `score=1.0`

## Notes

- The engine reads the canonical spec file at `docs/specs/SPEC-TIER1-MVP.md` and verifies the session pointer plus traceability row.
- The result stays narrow and deterministic so the MVP contract remains a single-file, single-test, single-row wire.
