//! Emergent cluster membership for civilian agents.
//!
//! This module intentionally stays engine-free: it models emergent co-location
//! clusters and a payoff gate for joining or leaving them, but it does not
//! encode any economy, safety, or faction logic itself.

use std::collections::HashMap;

use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};

use crate::Position3d;

/// Stable emergent cluster identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterId(pub u64);

/// Component marking the cluster a civilian currently belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterMember {
    /// The current cluster identifier.
    pub cluster: ClusterId,
}

/// Payoff abstraction for cluster membership decisions.
///
/// Implementations are expected to be deterministic. Positive values indicate
/// net benefit from membership; negative values indicate net cost.
pub trait MembershipPayoff {
    /// Returns the payoff for an agent in a given cluster.
    fn payoff(&self, agent_id: u64, cluster: ClusterId) -> f32;
}

fn squared_distance(a: &WorldCoord, b: &WorldCoord) -> i128 {
    let dx = i128::from(a.x) - i128::from(b.x);
    let dy = i128::from(a.y) - i128::from(b.y);
    let dz = i128::from(a.z) - i128::from(b.z);
    dx * dx + dy * dy + dz * dz
}

fn find(parent: &mut [usize], mut idx: usize) -> usize {
    let mut root = idx;
    while parent[root] != root {
        root = parent[root];
    }
    while parent[idx] != idx {
        let next = parent[idx];
        parent[idx] = root;
        idx = next;
    }
    root
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        if ra < rb {
            parent[rb] = ra;
        } else {
            parent[ra] = rb;
        }
    }
}

/// Deterministically clusters agents by single-link co-location.
///
/// Agents whose pairwise squared distance is within `radius_fp^2` are connected
/// into the same emergent cluster. The cluster ID is the minimum agent ID in
/// that connected component.
pub fn cluster_by_colocation(
    positions: &[(u64, Position3d)],
    radius_fp: i64,
) -> Vec<(u64, ClusterId)> {
    // Sort by agent id so connected-component traversal and component id
    // assignment are independent of input ordering (FR-CIV-LIFE-035).
    let mut agents: Vec<(u64, Position3d)> = positions.to_vec();
    agents.sort_by_key(|(agent_id, _)| *agent_id);

    let count = agents.len();
    let mut parent: Vec<usize> = (0..count).collect();
    let radius_sq = i128::from(radius_fp) * i128::from(radius_fp);

    for i in 0..count {
        for j in (i + 1)..count {
            if squared_distance(&agents[i].1.coord, &agents[j].1.coord) <= radius_sq {
                union(&mut parent, i, j);
            }
        }
    }

    // ClusterId = minimum agent id within each connected component. Keyed on the
    // canonical root *index*; because indices follow the agent-id sort, the same
    // component always yields the same min id regardless of input order.
    let mut cluster_min_id: HashMap<usize, u64> = HashMap::new();
    for (idx, _) in agents.iter().enumerate().take(count) {
        let root = find(&mut parent, idx);
        let agent_id = agents[idx].0;
        cluster_min_id
            .entry(root)
            .and_modify(|min_id| *min_id = (*min_id).min(agent_id))
            .or_insert(agent_id);
    }

    let mut result: Vec<(u64, ClusterId)> = (0..count)
        .map(|idx| {
            let root = find(&mut parent, idx);
            let cluster_id = ClusterId(*cluster_min_id.get(&root).expect("cluster root missing"));
            (agents[idx].0, cluster_id)
        })
        .collect();
    result.sort_by_key(|(agent_id, _)| *agent_id);
    result
}

/// Returns `true` when membership payoff meets or exceeds the threshold.
pub fn should_join(
    payoff: &impl MembershipPayoff,
    agent_id: u64,
    cluster: ClusterId,
    threshold: f32,
) -> bool {
    payoff.payoff(agent_id, cluster) >= threshold
}

/// Returns `true` when membership payoff is below the threshold.
pub fn should_leave(
    payoff: &impl MembershipPayoff,
    agent_id: u64,
    cluster: ClusterId,
    threshold: f32,
) -> bool {
    payoff.payoff(agent_id, cluster) < threshold
}

/// Reconciles membership against colocated clusters and payoff.
///
/// The input list is updated in deterministic agent-id order. Agents join the
/// colocated cluster when payoff is net-positive at the threshold and leave
/// when it is not.
pub fn reconcile_membership(
    members: &mut [(u64, Option<ClusterId>)],
    colocated: &[(u64, ClusterId)],
    payoff: &impl MembershipPayoff,
    threshold: f32,
) {
    let mut cluster_by_agent: HashMap<u64, ClusterId> = HashMap::with_capacity(colocated.len());
    for &(agent_id, cluster) in colocated {
        cluster_by_agent.insert(agent_id, cluster);
    }

    members.sort_by_key(|(agent_id, _)| *agent_id);
    for (agent_id, membership) in members.iter_mut() {
        if let Some(cluster) = cluster_by_agent.get(agent_id).copied() {
            if should_join(payoff, *agent_id, cluster, threshold) {
                *membership = Some(cluster);
            } else if should_leave(payoff, *agent_id, cluster, threshold) {
                *membership = None;
            }
        } else {
            *membership = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct StubPayoff {
        value: f32,
    }

    impl MembershipPayoff for StubPayoff {
        fn payoff(&self, _agent_id: u64, _cluster: ClusterId) -> f32 {
            self.value
        }
    }

    #[test]
    fn co_located_agents_cluster_together() {
        let positions = vec![
            (
                20,
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ),
            (
                10,
                Position3d {
                    coord: WorldCoord { x: 3, y: 4, z: 0 },
                },
            ),
        ];

        let clusters = cluster_by_colocation(&positions, 5);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].1, ClusterId(10));
        assert_eq!(clusters[1].1, ClusterId(10));
    }

    #[test]
    fn far_apart_agents_get_different_clusters() {
        let positions = vec![
            (
                1,
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ),
            (
                7,
                Position3d {
                    coord: WorldCoord { x: 100, y: 0, z: 0 },
                },
            ),
        ];

        let clusters = cluster_by_colocation(&positions, 5);
        assert_eq!(clusters, vec![(1, ClusterId(1)), (7, ClusterId(7))]);
    }

    #[test]
    fn cluster_id_is_min_id_and_deterministic() {
        let positions = vec![
            (
                9,
                Position3d {
                    coord: WorldCoord { x: 1, y: 0, z: 0 },
                },
            ),
            (
                4,
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ),
            (
                12,
                Position3d {
                    coord: WorldCoord { x: 2, y: 0, z: 0 },
                },
            ),
        ];

        let a = cluster_by_colocation(&positions, 2);
        let b = cluster_by_colocation(&positions, 2);
        assert_eq!(a, b);
        assert!(a.iter().all(|(_, cluster)| *cluster == ClusterId(4)));
    }

    #[test]
    fn join_and_leave_respect_threshold() {
        let payoff_join = StubPayoff { value: 1.0 };
        let payoff_leave = StubPayoff { value: -1.0 };
        let cluster = ClusterId(42);

        assert!(should_join(&payoff_join, 7, cluster, 0.0));
        assert!(!should_join(&payoff_leave, 7, cluster, 0.0));
        assert!(should_leave(&payoff_leave, 7, cluster, 0.0));
        assert!(!should_leave(&payoff_join, 7, cluster, 0.0));
    }

    #[test]
    fn reconcile_joins_positive_and_leaves_negative() {
        let payoff = StubPayoff { value: 0.5 };
        let colocated = vec![(2, ClusterId(2)), (5, ClusterId(2))];
        let mut members = vec![(5, None), (2, None)];

        reconcile_membership(&mut members, &colocated, &payoff, 0.0);
        assert_eq!(
            members,
            vec![(2, Some(ClusterId(2))), (5, Some(ClusterId(2)))]
        );

        let payoff_negative = StubPayoff { value: -0.5 };
        reconcile_membership(&mut members, &colocated, &payoff_negative, 0.0);
        assert_eq!(members, vec![(2, None), (5, None)]);
    }

    #[test]
    fn clustering_is_order_independent() {
        let a = vec![
            (
                3,
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ),
            (
                8,
                Position3d {
                    coord: WorldCoord { x: 1, y: 0, z: 0 },
                },
            ),
            (
                1,
                Position3d {
                    coord: WorldCoord { x: 100, y: 0, z: 0 },
                },
            ),
        ];
        let mut b = a.clone();
        b.reverse();

        assert_eq!(cluster_by_colocation(&a, 2), cluster_by_colocation(&b, 2));
    }
}
