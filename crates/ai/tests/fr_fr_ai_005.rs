//! FR-AI-005 — AI SHALL never exceed a configurable MilliCredit/Joule
//! expenditure per tick (fair-play cap).

#[test]
fn cap_enforced_per_tick() {
    use civ_ai::FairPlayCap;

    let mut cap = FairPlayCap::with_limits(1000, 500);

    // Spend within limits — should succeed.
    assert!(cap.try_spend_mc(800));
    assert_eq!(cap.mc_spent, 800);
    assert!(cap.try_use_joules(300));
    assert_eq!(cap.joules_used, 300);

    // Exceed MC limit — should fail.
    assert!(!cap.try_spend_mc(300), "should reject over-budget MC spend");
    assert_eq!(cap.mc_spent, 800, "MC spent should not change on reject");

    // Exceed Joule limit — should fail.
    assert!(!cap.try_use_joules(300), "should reject over-budget Joule use");
    assert_eq!(cap.joules_used, 300, "Joule used should not change on reject");
}

#[test]
fn remaining_budget_calculated() {
    use civ_ai::FairPlayCap;

    let mut cap = FairPlayCap::with_limits(1000, 500);
    cap.try_spend_mc(400);
    cap.try_use_joules(200);

    assert_eq!(cap.mc_remaining(), 600);
    assert_eq!(cap.joule_remaining(), 300);
}

#[test]
fn reset_clears_accumulators() {
    use civ_ai::FairPlayCap;

    let mut cap = FairPlayCap::with_limits(1000, 500);
    cap.try_spend_mc(800);
    cap.try_use_joules(400);

    cap.reset();
    assert_eq!(cap.mc_spent, 0);
    assert_eq!(cap.joules_used, 0);
    assert_eq!(cap.mc_remaining(), 1000);
    assert_eq!(cap.joule_remaining(), 500);
}

#[test]
fn zero_or_negative_spend_always_allowed() {
    use civ_ai::FairPlayCap;

    let mut cap = FairPlayCap::with_limits(100, 100);
    assert!(cap.try_spend_mc(0));
    assert!(cap.try_spend_mc(-50));
    assert!(cap.try_use_joules(0));
    assert!(cap.try_use_joules(-100));
}
