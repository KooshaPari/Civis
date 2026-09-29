# Validation Record

Last validated: 2026-09-29, against `2253de28` (14 commits ahead of `origin/main` = `b3cd62a7`).

## Full workspace test run

```
cargo test --workspace --no-fail-fast -- --skip ten_thousand_ticks_under_budget
```

| Metric | Value |
|--------|------:|
| Passed  | 5731 |
| Failed  | 15 |
| Ignored | 32 |

`ten_thousand_ticks_under_budget` is excluded because it runs longer than the
600s execution cap and gets killed mid-run, which hides the rest of the suite.
It is a perf regression guard, not an FR-requirement assertion, and it was not
run in either the audit or the baseline below, so the comparison stays fair.

## The 15 failures are all pre-existing

Every one of the 15 reproduces identically on untouched `origin/main`. This was
verified by building a throwaway worktree at `b3cd62a7` with a separate
`CARGO_TARGET_DIR` and running the same targets.

| Crate / suite | Failing test | Fails on `origin/main`? |
|---|---|---|
| `civ-engine` lib | `caravan::tests::complementary_profiles_form_organic_route_and_cargo_loads` | yes |
| `civ-engine` lib | `emergence::tests::nfr_c_04_emergence_rng_is_seeded_and_deterministic` | yes |
| `civ-engine` lib | `emergent_migration::tests::migration_tick_conserves_population_and_is_deterministic` | yes |
| `civ-engine` lib | `engine::engine_tests::tests::fr_civ_cohesion_001_phase_aggregates_kinship_trust_into_settlement_fabric` | yes |
| `civ-engine` | `fr_civ_rts_client_perf_cluster::fr_sess_005_speed_domain_and_event_live_outside_the_engine` | yes |
| `civ-engine` | `fr_fr_civ_inspect_902::snapshot_exposes_entity_counts` | yes |
| `civ-engine` | `fr_fr_civ_vehicle_024::verify_fr_civ_vehicle_024_basic` | yes |
| `civ-engine` | `fr_fr_net_001::world_state_initializes_at_tick_zero` | yes |
| `civ-engine` | `fr_fr_prot_005::factions_populated_after_auth` | yes |
| `civ-engine` | `fr_fr_sess_002::hotseat_multiple_factions` | yes |
| `civ-genetics` lib | `seeds::tests::fr_civ_genetics_seed_003_divergence_dial_is_monotone` | yes |
| `civ-i18n` lib | `tests::all_locales_have_bundles` | yes |
| `civ-i18n` lib | `tests::string_table_infrastructure_serves_every_locale` | yes |
| `civ-i18n` lib | `tests::zh_cn_bundle_coverage_meets_threshold` | yes |
| `civ-emergence-oracle` lib | `oracles::desert_caravan::tests::desert_caravan_oracle_tick_zero_passes_with_fr_id` | yes |

Two independent lines of evidence support this conclusion:

1. `_check_comment_only.py` reports that every changed line under `crates/`
   across the full audit range `cd6a6cf9~1..HEAD` is a comment or blank, so
   the removals cannot have altered behaviour.
2. The baseline runs above show the same 15 failures before the audit existed.

## Structural audit invariants

| Check | Result |
|---|---|
| `cargo build --workspace` | passes |
| Container bindings | 3, in 3 files |
| Container-binding IDs lacking an authoritative definition | 0 |
| `_apply_verdicts.py` second pass | 0 removals (idempotent), working tree clean |
| `_check_comment_only.py` | PASS |

## Coverage totals

1423 unique IDs scanned by `scripts/traceability/gen-fr-audit.py`.

| Status | Count | % |
|---|---:|---:|
| `COVERED` | 1057 | 74.3 |
| `TEST-NO-CODE-REF` | 154 | 10.8 |
| `SPEC-ONLY` | 202 | 14.2 |
| `IMPL-NO-TEST` | 9 | 0.6 |
| `CODE-ONLY-no-spec` | 1 | 0.1 |
| `STUB-TEST-ONLY` | 0 | 0.0 |

## Honest caveats

- A test that passes tells us nothing on its own about whether the requirement
  it names is implemented. 154 IDs are `TEST-NO-CODE-REF`: a real test asserts
  something, but no source file carries the ID, so the implementation cannot be
  located from the ID alone. These are not audited-clean.
- 202 IDs remain `SPEC-ONLY` with no implementing code. They are recorded, not
  fixed, and are tracked in `spec-only-deferrals.json`.
- The three surviving container bindings were kept only after reading the
  authoritative requirement text and confirming the symbol implements that exact
  behaviour: `ConstraintState` for `FR-CIV-0104-003`, `ReplayLog` for
  `FR-SAVE-009`, `FogOfWar` for `FR-CIV-FOG-001`.
- No new requirement coverage was manufactured. Removals only ever reduced the
  set of IDs asserted against a symbol.
