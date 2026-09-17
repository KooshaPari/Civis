//! Inter-district Joule distribution (FR-ECON-002, FR-ECON-004).
//!
//! Two ordered steps run every tick:
//!
//! 1. **FR-ECON-002** — joule consumption is deducted from each district's own
//!    reserve *before* any regional redistribution happens. A district pays for
//!    its own consumption first; only what is left over is eligible to move.
//! 2. **FR-ECON-004** — surplus Joules then flow to *adjacent* districts along
//!    the distribution graph. A district whose balance exceeds
//!    [`DistributionConfig::reserve_floor`] donates its surplus to neighbouring
//!    districts that are in deficit, capped by
//!    [`DistributionConfig::max_transfer_per_tick`].
//!
//! Every quantity is an integer, and distribution is purely conservative: a
//! transfer moves Joules between two balances without creating or destroying
//! any. Ordering is deterministic (districts and neighbours are visited in
//! ascending id order), so the same inputs always yield the same transfers.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::district::DistrictEnergyState;

/// Default Joule reserve a district keeps before donating surplus.
pub const DEFAULT_RESERVE_FLOOR: i64 = 0;

/// Default per-tick cap on how much a single district may donate.
pub const DEFAULT_MAX_TRANSFER_PER_TICK: i64 = i64::MAX;

/// Undirected adjacency graph over district ids.
///
/// Only listed neighbours receive surplus; districts that are not connected
/// never exchange Joules.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictGraph {
    adjacency: BTreeMap<u32, Vec<u32>>,
}

impl DistrictGraph {
    /// Create an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a district with no neighbours yet.
    pub fn add_district(&mut self, id: u32) {
        self.adjacency.entry(id).or_default();
    }

    /// Connect two districts. The edge is undirected and idempotent.
    ///
    /// Self-loops are ignored.
    pub fn connect(&mut self, a: u32, b: u32) {
        if a == b {
            return;
        }
        self.add_district(a);
        self.add_district(b);
        let ea = self.adjacency.get_mut(&a).expect("a registered");
        if !ea.contains(&b) {
            ea.push(b);
            ea.sort_unstable();
        }
        let eb = self.adjacency.get_mut(&b).expect("b registered");
        if !eb.contains(&a) {
            eb.push(a);
            eb.sort_unstable();
        }
    }

    /// Neighbours of `id`, in ascending id order.
    #[must_use]
    pub fn neighbours(&self, id: u32) -> &[u32] {
        self.adjacency.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Number of districts in the graph.
    #[must_use]
    pub fn district_count(&self) -> usize {
        self.adjacency.len()
    }
}

/// Tuning for [`distribute_surplus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// Balance a district keeps before it may donate.
    pub reserve_floor: i64,
    /// Maximum Joules a single district may donate in one tick.
    pub max_transfer_per_tick: i64,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            reserve_floor: DEFAULT_RESERVE_FLOOR,
            max_transfer_per_tick: DEFAULT_MAX_TRANSFER_PER_TICK,
        }
    }
}

/// One inter-district Joule movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    /// Donating district.
    pub from: u32,
    /// Receiving district.
    pub to: u32,
    /// Joules moved (always positive).
    pub joules: i64,
}

/// Outcome of one distribution tick.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionReport {
    /// Joules removed from district reserves by the consumption step
    /// (FR-ECON-002). Sum of per-district deductions actually applied.
    pub deducted_joules: i64,
    /// Joules moved between districts by the distribution step (FR-ECON-004).
    pub transferred_joules: i64,
    /// Individual transfers, in the order they were applied.
    pub transfers: Vec<Transfer>,
}

/// Step 1 (FR-ECON-002): deduct each district's own consumption from its
/// reserve before any distribution occurs.
///
/// Each district's balance is reduced by `consumption_per_district`. Balances
/// may go negative (that is a deficit, which [`distribute_surplus`] can then
/// relieve). Returns the total Joules deducted.
pub fn deduct_consumption(
    districts: &mut [DistrictEnergyState],
    consumption_per_district: i64,
) -> i64 {
    if consumption_per_district <= 0 {
        return 0;
    }
    let mut total = 0i64;
    for d in districts.iter_mut() {
        d.balance_joules = d.balance_joules.saturating_sub(consumption_per_district);
        total = total.saturating_add(consumption_per_district);
    }
    total
}

/// Step 2 (FR-ECON-004): flow surplus Joules to adjacent districts in deficit.
///
/// Donors are visited in ascending district id order; each donates up to
/// `reserve_floor`-exceeding surplus, capped at `max_transfer_per_tick`, to its
/// neighbours in ascending id order. A neighbour never receives more than it
/// needs to reach a zero balance. The operation is conservative: the sum of all
/// balances is unchanged.
pub fn distribute_surplus(
    districts: &mut [DistrictEnergyState],
    graph: &DistrictGraph,
    config: &DistributionConfig,
) -> DistributionReport {
    let mut report = DistributionReport::default();

    // Snapshot order of districts for deterministic iteration.
    let mut order: Vec<usize> = (0..districts.len()).collect();
    order.sort_by_key(|&i| districts[i].district_id);

    for &donor_idx in &order {
        let floor = config.reserve_floor.max(0);
        let available = districts[donor_idx]
            .balance_joules
            .saturating_sub(floor)
            .min(config.max_transfer_per_tick.max(0));
        if available <= 0 {
            continue;
        }

        let donor_id = districts[donor_idx].district_id;
        let neighbours = graph.neighbours(donor_id);
        if neighbours.is_empty() {
            continue;
        }

        // Deterministic recipient order: ascending id, deficits first.
        let mut recipients: Vec<usize> = districts
            .iter()
            .enumerate()
            .filter(|(_, d)| neighbours.contains(&d.district_id) && d.balance_joules < 0)
            .map(|(i, _)| i)
            .collect();
        recipients.sort_by_key(|&i| districts[i].district_id);
        if recipients.is_empty() {
            continue;
        }

        let mut remaining = available;
        for &recipient_idx in &recipients {
            if remaining <= 0 {
                break;
            }
            let need = districts[recipient_idx].balance_joules.saturating_neg();
            let move_amount = need.min(remaining);
            if move_amount <= 0 {
                continue;
            }
            districts[donor_idx].balance_joules =
                districts[donor_idx].balance_joules.saturating_sub(move_amount);
            districts[recipient_idx].balance_joules =
                districts[recipient_idx].balance_joules.saturating_add(move_amount);
            remaining -= move_amount;
            report.transferred_joules = report.transferred_joules.saturating_add(move_amount);
            report.transfers.push(Transfer {
                from: donor_id,
                to: districts[recipient_idx].district_id,
                joules: move_amount,
            });
        }
    }

    report
}

/// Run a full distribution tick in the required FR-ECON-002 order:
/// deduct consumption first, then redistribute surplus.
///
/// Returns a report whose `deducted_joules` and `transferred_joules` reflect
/// the two steps in that order.
pub fn step_distribution(
    districts: &mut [DistrictEnergyState],
    graph: &DistrictGraph,
    config: &DistributionConfig,
    consumption_per_district: i64,
) -> DistributionReport {
    let deducted = deduct_consumption(districts, consumption_per_district);
    let mut report = distribute_surplus(districts, graph, config);
    report.deducted_joules = deducted;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn district(id: u32, balance: i64) -> DistrictEnergyState {
        DistrictEnergyState::new(id, balance)
    }

    #[test]
    fn connect_is_undirected_and_idempotent() {
        let mut g = DistrictGraph::new();
        g.connect(1, 2);
        g.connect(1, 2);
        g.connect(2, 1);
        assert_eq!(g.neighbours(1), &[2]);
        assert_eq!(g.neighbours(2), &[1]);
        assert_eq!(g.district_count(), 2);
    }

    #[test]
    fn deduction_happens_before_distribution() {
        // District 1 is flush; district 2 starts empty. Consumption is charged
        // to each district first, so district 1's donation is measured from its
        // post-consumption balance.
        let mut districts = vec![district(1, 100), district(2, 0)];
        let mut g = DistrictGraph::new();
        g.connect(1, 2);

        let report = step_distribution(&mut districts, &g, &DistributionConfig::default(), 30);

        assert_eq!(report.deducted_joules, 60, "30 charged to each district");
        // After deduction: d1 = 70, d2 = -30. Distribution then sends 30 to d2.
        assert_eq!(districts[0].balance_joules, 40);
        assert_eq!(districts[1].balance_joules, 0);
        assert_eq!(report.transferred_joules, 30);

        // If distribution had run first, d1 (100) would have covered d2's 0
        // deficit and nothing would have moved; the 30-unit move proves the
        // deduction ordered ahead of distribution.
        assert_eq!(report.transfers.len(), 1);
        assert_eq!(report.transfers[0].from, 1);
        assert_eq!(report.transfers[0].to, 2);
    }

    #[test]
    fn surplus_flows_to_adjacent_deficit_only() {
        let mut districts = vec![
            district(1, 500), // donor
            district(2, -40), // adjacent, deficit
            district(3, -80), // not adjacent -> must not receive
        ];
        let mut g = DistrictGraph::new();
        g.connect(1, 2);

        let report = distribute_surplus(&mut districts, &g, &DistributionConfig::default());

        assert_eq!(report.transferred_joules, 40);
        assert_eq!(districts[1].balance_joules, 0);
        assert_eq!(districts[2].balance_joules, -80, "unconnected district untouched");
        assert_eq!(report.transfers.len(), 1);
        assert_eq!(report.transfers[0].to, 2);
    }

    #[test]
    fn distribution_is_conservative() {
        let mut districts = vec![district(1, 300), district(2, -120), district(3, 70)];
        let mut g = DistrictGraph::new();
        g.connect(1, 2);
        g.connect(2, 3);

        let before: i64 = districts.iter().map(|d| d.balance_joules).sum();
        distribute_surplus(&mut districts, &g, &DistributionConfig::default());
        let after: i64 = districts.iter().map(|d| d.balance_joules).sum();

        assert_eq!(before, after, "no Joules created or destroyed");
    }

    #[test]
    fn reserve_floor_and_transfer_cap_are_respected() {
        let mut districts = vec![district(1, 500), district(2, -100)];
        let mut g = DistrictGraph::new();
        g.connect(1, 2);
        let cfg = DistributionConfig {
            reserve_floor: 400,
            max_transfer_per_tick: 50,
        };

        let report = distribute_surplus(&mut districts, &g, &cfg);

        // Surplus above floor is 100, but the per-tick cap is 50.
        assert_eq!(report.transferred_joules, 50);
        assert_eq!(districts[0].balance_joules, 450);
        assert_eq!(districts[1].balance_joules, -50);
    }

    #[test]
    fn no_deficit_means_no_transfer() {
        let mut districts = vec![district(1, 500), district(2, 100)];
        let mut g = DistrictGraph::new();
        g.connect(1, 2);
        let report = distribute_surplus(&mut districts, &g, &DistributionConfig::default());
        assert!(report.transfers.is_empty());
        assert_eq!(report.transferred_joules, 0);
        assert_eq!(districts[0].balance_joules, 500);
    }
}
