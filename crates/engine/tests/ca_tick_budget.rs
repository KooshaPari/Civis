#[path = "../src/ca_budget.rs"]
mod ca_budget;

use ca_budget::CaTickBudget;

#[test]
fn default_max_chunks_per_step_is_64() {
    let budget = CaTickBudget::default();
    assert_eq!(budget.max_chunks_per_step, 64);
}

#[test]
fn default_tick_hz_is_2_0() {
    let budget = CaTickBudget::default();
    assert_eq!(budget.tick_hz, 2.0);
}

#[test]
fn default_budget_constructs_expected_fields() {
    let budget = CaTickBudget::default();
    assert_eq!(budget.max_chunks_per_step, 64);
    assert_eq!(budget.tick_hz, 2.0);
}
