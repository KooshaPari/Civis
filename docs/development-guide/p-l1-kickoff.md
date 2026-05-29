# P-L1 physics-law DB — kickoff

**Phase:** P-L1 (`crates/laws`)
**Depends on:** (independent; feeds P-R1 hybrid research)
**Branch:** `feat/p-l1-kickoff`

## FR status (`docs/traceability/fr-3d-matrix.md`)

| FR ID | Status | Notes |
|-------|--------|-------|
| FR-CIV-LAWS-000 | implemented | schema version stub |
| FR-CIV-LAWS-001 | implemented | RON round-trip |
| FR-CIV-LAWS-002 | implemented | fictional extension validator |
| FR-CIV-LAWS-003 | implemented | missing dependency detection |
| FR-CIV-LAWS-004 | implemented | duplicate id detection |
| FR-CIV-LAWS-005 | implemented | `unlocked_at_era` filter |
| FR-CIV-LAWS-006 | implemented | `unlockable_at_era` + `dependency_order` |
| FR-CIV-LAWS-007 | implemented | `merge_overlay` for mod RON |
| FR-CIV-LAWS-008 | implemented | embedded `laws/default.ron` + `default_canon` |
| FR-CIV-LAWS-009 | implemented | `load_with_mod_overlays` scans `mods/*/laws.ron` |

## Kickoff slices

1. **Embedded canon RON + mod overlay** — **done** (item 1): `crates/laws/laws/default.ron`, `DEFAULT_LAW_RON`, `load_path`, `merge_overlay`, `civ-watch` uses `LawDb::default_canon()`.
2. **Era unlock graph** — **done** (item 2): `unlockable_at_era`, `dependency_order` with cycle error.
3. **Mod directory loader** — **done** (item 3): `load_with_mod_overlays`, `civ-watch` `load_law_db` at startup; sample `mods/example-policy/laws.ron`.
4. **Research gate integration** — **done** (item 4): `civ-research::validate` + tech tree use `unlockable_at_era` dependency closure.

## Run

```bash
cargo test -p civ-laws
cargo test -p civ-research
cargo test -p civ-watch
```
