use civ_engine::{step_with_budget, CaTickBudget};

#[test]
fn default_values() {
    let budget = CaTickBudget::default();

    assert_eq!(budget.max_chunks_per_step, 64);
    assert_eq!(budget.tick_hz, 2.0);
}

#[test]
fn step_within_budget() {
    let budget = CaTickBudget {
        max_chunks_per_step: 4,
        tick_hz: 2.0,
    };

    let outcome = step_with_budget(&budget, |i| i < 2);

    assert_eq!(outcome.chunks_stepped, 2);
    assert!(!outcome.budget_exhausted);
}

#[test]
fn step_exhausts_budget() {
    let budget = CaTickBudget {
        max_chunks_per_step: 3,
        tick_hz: 2.0,
    };

    let outcome = step_with_budget(&budget, |_| true);

    assert_eq!(outcome.chunks_stepped, 3);
    assert!(outcome.budget_exhausted);
}

#[test]
fn zero_budget_skips_all() {
    let budget = CaTickBudget {
        max_chunks_per_step: 0,
        tick_hz: 2.0,
    };
    let mut calls = 0;

    let outcome = step_with_budget(&budget, |_| {
        calls += 1;
        true
    });

    assert_eq!(outcome.chunks_stepped, 0);
    assert!(!outcome.budget_exhausted);
    assert_eq!(calls, 0);
}

#[test]
fn custom_hz() {
    let budget = CaTickBudget {
        max_chunks_per_step: 8,
        tick_hz: 7.5,
    };

    assert_eq!(budget.tick_hz, 7.5);
}
