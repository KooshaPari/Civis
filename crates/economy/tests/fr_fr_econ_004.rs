//! FR-ECON-004 — Surplus Joules SHALL flow to adjacent districts via the
//! distribution graph each tick.
//!
//! Matrix check: `distribution::surplus_flows_adjacent`.

use civ_economy::{
    distribute_surplus, DistributionConfig, DistrictEnergyState, DistrictGraph,
};

fn districts(pairs: &[(u32, i64)]) -> Vec<DistrictEnergyState> {
    pairs
        .iter()
        .map(|&(id, bal)| DistrictEnergyState::new(id, bal))
        .collect()
}

/// Surplus reaches adjacent districts and only adjacent districts, without
/// creating or destroying Joules.
#[test]
fn surplus_flows_adjacent() {
    let mut d = districts(&[
        (1, 500), // donor
        (2, -40), // adjacent deficit -> receives
        (3, -80), // not adjacent -> untouched
        (4, 60),  // connected but not in deficit -> untouched
    ]);
    let mut graph = DistrictGraph::new();
    graph.connect(1, 2);
    graph.connect(1, 4);

    let before: i64 = d.iter().map(|x| x.balance_joules).sum();
    let report = distribute_surplus(&mut d, &graph, &DistributionConfig::default());
    let after: i64 = d.iter().map(|x| x.balance_joules).sum();

    assert_eq!(report.transferred_joules, 40, "exactly the adjacent deficit");
    assert_eq!(d[0].balance_joules, 460);
    assert_eq!(d[1].balance_joules, 0, "deficit fully relieved");
    assert_eq!(d[2].balance_joules, -80, "unconnected district does not receive");
    assert_eq!(d[3].balance_joules, 60, "no transfer to a non-deficit peer");
    assert_eq!(before, after, "distribution conserves total Joules");

    assert_eq!(report.transfers.len(), 1);
    assert_eq!(report.transfers[0].from, 1);
    assert_eq!(report.transfers[0].to, 2);

    // Multi-hop: surplus can only advance one adjacency ring per tick, so the
    // unconnected district never benefits.
    assert!(!report.transfers.iter().any(|t| t.to == 3));
}

/// Surplus spreads across several adjacent deficits in deterministic id order.
#[test]
fn surplus_spreads_across_adjacent_deficits() {
    let mut d = districts(&[(1, 300), (2, -30), (3, -20), (4, -10)]);
    let mut graph = DistrictGraph::new();
    graph.connect(1, 2);
    graph.connect(1, 3);
    graph.connect(1, 4);

    let report = distribute_surplus(&mut d, &graph, &DistributionConfig::default());

    assert_eq!(report.transferred_joules, 60);
    for i in 1..4 {
        assert_eq!(d[i].balance_joules, 0, "every adjacent deficit relieved");
    }
    assert_eq!(d[0].balance_joules, 240);
    let order: Vec<u32> = report.transfers.iter().map(|t| t.to).collect();
    assert_eq!(order, vec![2, 3, 4], "recipients visited in ascending id order");
}

/// A district keeps its reserve floor, losing at most the per-tick cap.
#[test]
fn surplus_respects_floor_and_cap() {
    let mut d = districts(&[(1, 500), (2, -100)]);
    let mut graph = DistrictGraph::new();
    graph.connect(1, 2);
    let cfg = DistributionConfig {
        reserve_floor: 400,
        max_transfer_per_tick: 25,
    };

    let report = distribute_surplus(&mut d, &graph, &cfg);
    assert_eq!(report.transferred_joules, 25);
    assert_eq!(d[0].balance_joules, 475, "donor never dips below floor + give");
    assert_eq!(d[1].balance_joules, -75);
}

/// No deficit anywhere means the graph moves nothing.
#[test]
fn balanced_graph_is_a_no_op() {
    let mut d = districts(&[(1, 100), (2, 50)]);
    let mut graph = DistrictGraph::new();
    graph.connect(1, 2);
    let report = distribute_surplus(&mut d, &graph, &DistributionConfig::default());
    assert_eq!(report.transferred_joules, 0);
    assert!(report.transfers.is_empty());
    assert_eq!(d[0].balance_joules, 100);
    assert_eq!(d[1].balance_joules, 50);
}

/// Adjacency is undirected: a district can also receive from its neighbour.
#[test]
fn adjacency_is_undirected() {
    let mut graph = DistrictGraph::new();
    graph.connect(7, 9);
    assert_eq!(graph.neighbours(7), &[9]);
    assert_eq!(graph.neighbours(9), &[7]);
}
