//! FR-ECON-002 — Joule consumption SHALL be deducted from district reserves
//! before regional distribution.
//!
//! Matrix check: `consumption::deducted_before_distribution`.

use civ_economy::{
    deduct_consumption, distribute_surplus, step_distribution, DistributionConfig, DistrictEnergyState,
    DistrictGraph,
};

/// Deduction runs strictly before distribution, so a donor's giveaway is
/// measured from its post-consumption balance.
#[test]
fn deducted_before_distribution() {
    let mut districts = vec![
        DistrictEnergyState::new(1, 100),
        DistrictEnergyState::new(2, 0),
    ];
    let mut graph = DistrictGraph::new();
    graph.connect(1, 2);

    // Step order is explicit: deduct, then distribute.
    let deducted = deduct_consumption(&mut districts, 30);
    assert_eq!(deducted, 60, "30 charged to each of the two districts");
    assert_eq!(districts[0].balance_joules, 70);
    assert_eq!(districts[1].balance_joules, -30);

    let report = distribute_surplus(&mut districts, &graph, &DistributionConfig::default());
    assert_eq!(report.transferred_joules, 30, "post-deduction deficit is relieved");
    assert_eq!(districts[0].balance_joules, 40);
    assert_eq!(districts[1].balance_joules, 0);

    // The combined helper performs the same order and reports both steps.
    let mut again = vec![
        DistrictEnergyState::new(1, 100),
        DistrictEnergyState::new(2, 0),
    ];
    let combined = step_distribution(&mut again, &graph, &DistributionConfig::default(), 30);
    assert_eq!(combined.deducted_joules, 60);
    assert_eq!(combined.transferred_joules, 30);
    assert_eq!(again, districts, "step_distribution matches the manual order");

    // Counterfactual: if distribution ran first there would be no deficit to
    // relieve (both districts are non-negative), so nothing would move. The
    // 30-unit transfer above is therefore proof of the required ordering.
    let mut reversed = vec![
        DistrictEnergyState::new(1, 100),
        DistrictEnergyState::new(2, 0),
    ];
    let pre = distribute_surplus(&mut reversed, &graph, &DistributionConfig::default());
    assert_eq!(pre.transferred_joules, 0, "distribution first moves nothing");
    assert!(pre.transfers.is_empty());
}

/// Deduction applies to every district, including those already in deficit.
#[test]
fn deduction_applies_to_deficit_districts_too() {
    let mut districts = vec![
        DistrictEnergyState::new(1, 10),
        DistrictEnergyState::new(2, -50),
    ];
    let deducted = deduct_consumption(&mut districts, 25);
    assert_eq!(deducted, 50);
    assert_eq!(districts[0].balance_joules, -15);
    assert_eq!(districts[1].balance_joules, -75);
}

/// A non-positive consumption rate is a no-op.
#[test]
fn zero_consumption_deducts_nothing() {
    let mut districts = vec![DistrictEnergyState::new(1, 10)];
    assert_eq!(deduct_consumption(&mut districts, 0), 0);
    assert_eq!(deduct_consumption(&mut districts, -5), 0);
    assert_eq!(districts[0].balance_joules, 10);
}
