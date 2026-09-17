//! FR traceability tests for the economy crate.
//!
//! Covers: FR-CIV-ECON-001-MARKET, FR-CIV-ECON-004

use civ_economy::{
    LedgerEntry, ACCOUNT_ENERGY_BUDGET, ACCOUNT_CONSUMPTION, SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// FR-CIV-ECON-001-MARKET — Market system: schema version exists
// ---------------------------------------------------------------------------

/// FR-CIV-ECON-001-MARKET — Schema version is a valid positive integer.
#[test]
fn fr_civ_econ_001_market_schema_version_valid() {
    assert!(SCHEMA_VERSION > 0, "SCHEMA_VERSION must be positive");
}

// ---------------------------------------------------------------------------
// FR-CIV-ECON-004 — Economic allocation: ledger entry construction
// ---------------------------------------------------------------------------

/// FR-CIV-ECON-004 — Ledger entry fields are accessible and consistent.
#[test]
fn fr_civ_econ_004_ledger_entry_construction() {
    let entry = LedgerEntry {
        tick: 42,
        debit: 1000,
        credit: 0,
        account: ACCOUNT_ENERGY_BUDGET,
    };
    assert_eq!(entry.tick, 42);
    assert_eq!(entry.debit, 1000);
    assert_eq!(entry.account, ACCOUNT_ENERGY_BUDGET);
}

/// FR-CIV-ECON-004 — Account constants are distinct.
#[test]
fn fr_civ_econ_004_account_constants_distinct() {
    assert_ne!(
        ACCOUNT_ENERGY_BUDGET, ACCOUNT_CONSUMPTION,
        "budget and consumption accounts must differ"
    );
}

/// FR-CIV-ECON-004 — LedgerEntry can be constructed with different accounts.
#[test]
fn fr_civ_econ_004_ledger_entry_different_accounts() {
    let e1 = LedgerEntry {
        tick: 1,
        debit: 100,
        credit: 0,
        account: ACCOUNT_ENERGY_BUDGET,
    };
    let e2 = LedgerEntry {
        tick: 1,
        debit: 0,
        credit: 100,
        account: ACCOUNT_CONSUMPTION,
    };
    assert_eq!(e1.account, ACCOUNT_ENERGY_BUDGET);
    assert_eq!(e2.account, ACCOUNT_CONSUMPTION);
    assert_ne!(e1.account, e2.account);
}
