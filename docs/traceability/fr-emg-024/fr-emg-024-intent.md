# Intent: FR-EMG-024 -- I18n lookup safety & powers registry oracles

> Date: 2026-09-20
> FR: FR-EMG-024
> Epic: FR-EMG

## User Intent

Two oracles share the FR-EMG-024 contract:

- **I18n oracle**: confirms `Bundle::get_or_key` is panic-free for
  arbitrary keys across every supported locale. Every probe (empty
  string, nonexistent key, app.title, unicode, padded key) must
  succeed for every locale after tick > 0.
- **Powers oracle**: confirms the default god-tools power registry is
  structurally consistent — every registered power has a unique id
  and non-empty `label` + `coupling_note` fields.

### What This FR Achieves

Verifies that the i18n escape hatch used by the UI when a
translation is missing is safe, and that the powers registry is a
valid catalogue.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, full
coverage thereafter.

## Acceptance Signal

- Library unit test
  `get_or_key_is_panic_free_for_common_probes` in
  `crates/emergence-oracle/src/oracles/i18n.rs` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/i18n.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/i18n.rs:46` |
| Code | `crates/emergence-oracle/src/oracles/powers.rs:1` |
| Implementing crate | `crates/emergence-oracle/src/` |
