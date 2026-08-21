//! Multi-Agent Coordination (FR-AI-004).
//!
//! Provides primitives for coordinating multiple agents: group management,
//! negotiation protocols, and consensus building.

use crate::social_graph::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Errors arising from coordination operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum CoordinationError {
    /// Attempted to operate on an empty agent group.
    #[error("agent group is empty")]
    EmptyGroup,
    /// The specified agent is not a member of the group.
    #[error("agent {0} is not a member of the group")]
    NotAMember(AgentId),
    /// Consensus was not reached within the allowed rounds.
    #[error("consensus not reached after {0} rounds")]
    ConsensusNotReached(usize),
    /// No coordinator has been set for the group.
    #[error("no coordinator set for the group")]
    NoCoordinator,
}

/// Coordination protocol to use for negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationProtocol {
    /// Synchronous: all agents vote simultaneously.
    Sync,
    /// Asynchronous: agents propose in priority order.
    Async,
    /// Leader-follower: coordinator decides, others ratify.
    LeaderFollower,
}

/// A proposal from an agent.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    /// The agent making the proposal.
    pub agent_id: AgentId,
    /// The proposed action.
    pub action: String,
    /// Weight/priority of this proposal.
    pub weight: f64,
}

/// Outcome of a negotiation round.
#[derive(Debug, Clone, PartialEq)]
pub enum NegotiationOutcome {
    /// Consensus reached on an action with given supporters.
    Consensus {
        /// The agreed-upon action.
        action: String,
        /// Agents that supported this action.
        supporters: Vec<AgentId>,
    },
    /// No consensus; shows how each agent voted.
    Deadlock {
        /// Maps action -> list of agent IDs that voted for it.
        votes: HashMap<String, Vec<AgentId>>,
    },
}

/// A group of agents that coordinate via a chosen protocol.
#[derive(Debug, Clone)]
pub struct AgentGroup {
    /// Human-readable name for this group.
    pub name: String,
    /// Member agent IDs.
    pub members: Vec<AgentId>,
    /// Coordination protocol.
    pub protocol: CoordinationProtocol,
    /// Minimum number of members required for quorum.
    pub quorum: usize,
    /// Optional coordinator agent.
    pub coordinator: Option<AgentId>,
    /// Per-agent priority weights (higher = more influence).
    pub priorities: HashMap<AgentId, f64>,
}

impl AgentGroup {
    /// Create a new empty group with the given protocol.
    pub fn new(name: impl Into<String>, protocol: CoordinationProtocol) -> Self {
        Self {
            name: name.into(),
            members: Vec::new(),
            protocol,
            quorum: 1,
            coordinator: None,
            priorities: HashMap::new(),
        }
    }

    /// Add an agent to the group.
    pub fn add_member(&mut self, id: AgentId) {
        if !self.members.contains(&id) {
            self.members.push(id);
            self.priorities.entry(id).or_insert(1.0);
        }
    }

    /// Remove an agent from the group.
    pub fn remove_member(&mut self, id: AgentId) {
        self.members.retain(|m| *m != id);
        self.priorities.remove(&id);
        if self.coordinator == Some(id) {
            self.coordinator = None;
        }
    }

    /// Set the priority weight for an agent.
    pub fn set_priority(&mut self, id: AgentId, weight: f64) {
        if self.members.contains(&id) {
            self.priorities.insert(id, weight);
        }
    }

    /// Set the coordinator for this group.
    pub fn set_coordinator(&mut self, id: AgentId) -> Result<(), CoordinationError> {
        if !self.members.contains(&id) {
            return Err(CoordinationError::NotAMember(id));
        }
        self.coordinator = Some(id);
        Ok(())
    }

    /// Number of members.
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Check if an agent is a member.
    #[must_use]
    pub fn is_member(&self, id: AgentId) -> bool {
        self.members.contains(&id)
    }

    /// Return a reference to the member list.
    #[must_use]
    pub fn members(&self) -> &[AgentId] {
        &self.members
    }
}

/// Run a single negotiation round within a group.
///
/// `make_proposals` is called with the group and should return one
/// [`Proposal`] per member agent.
pub fn negotiate(
    group: &AgentGroup,
    proposals: &[Proposal],
) -> Result<NegotiationOutcome, CoordinationError> {
    if group.members.is_empty() {
        return Err(CoordinationError::EmptyGroup);
    }
    match group.protocol {
        CoordinationProtocol::Sync => negotiate_sync(group, proposals),
        CoordinationProtocol::Async => negotiate_async(group, proposals),
        CoordinationProtocol::LeaderFollower => negotiate_leader_follower(group, proposals),
    }
}

/// Synchronous majority: action with most weighted votes wins.
fn negotiate_sync(
    group: &AgentGroup,
    proposals: &[Proposal],
) -> Result<NegotiationOutcome, CoordinationError> {
    let mut votes: HashMap<String, f64> = HashMap::new();
    let mut voters: HashMap<String, Vec<AgentId>> = HashMap::new();
    for p in proposals {
        let weight = group.priorities.get(&p.agent_id).copied().unwrap_or(1.0);
        *votes.entry(p.action.clone()).or_insert(0.0) += weight;
        voters.entry(p.action.clone()).or_default().push(p.agent_id);
    }
    let total_weight: f64 = group
        .members
        .iter()
        .map(|m| group.priorities.get(m).copied().unwrap_or(1.0))
        .sum();
    if let Some((best_action, best_weight)) = votes
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        if *best_weight > total_weight / 2.0 {
            return Ok(NegotiationOutcome::Consensus {
                action: best_action.to_string(),
                supporters: voters.remove(best_action).unwrap_or_default(),
            });
        }
    }
    Ok(NegotiationOutcome::Deadlock { votes: voters })
}

/// Asynchronous: highest-priority agent wins if no tie.
fn negotiate_async(
    group: &AgentGroup,
    proposals: &[Proposal],
) -> Result<NegotiationOutcome, CoordinationError> {
    let mut sorted: Vec<&Proposal> = proposals.iter().collect();
    sorted.sort_by(|a, b| {
        let wa = group.priorities.get(&a.agent_id).copied().unwrap_or(1.0);
        let wb = group.priorities.get(&b.agent_id).copied().unwrap_or(1.0);
        wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(top) = sorted.first() {
        let supporters: Vec<AgentId> = sorted
            .iter()
            .filter(|p| p.action == top.action)
            .map(|p| p.agent_id)
            .collect();
        Ok(NegotiationOutcome::Consensus {
            action: top.action.clone(),
            supporters,
        })
    } else {
        Err(CoordinationError::EmptyGroup)
    }
}

/// Leader-follower: coordinator decides, others ratify.
fn negotiate_leader_follower(
    group: &AgentGroup,
    proposals: &[Proposal],
) -> Result<NegotiationOutcome, CoordinationError> {
    let coordinator = group.coordinator.ok_or(CoordinationError::NoCoordinator)?;
    let leader_proposal = proposals
        .iter()
        .find(|p| p.agent_id == coordinator)
        .ok_or(CoordinationError::NotAMember(coordinator))?;
    let action = leader_proposal.action.clone();
    let supporters: Vec<AgentId> = proposals
        .iter()
        .filter(|p| p.action == action)
        .map(|p| p.agent_id)
        .collect();
    if supporters.len() >= group.quorum {
        Ok(NegotiationOutcome::Consensus { action, supporters })
    } else {
        let mut votes: HashMap<String, Vec<AgentId>> = HashMap::new();
        for p in proposals {
            votes.entry(p.action.clone()).or_default().push(p.agent_id);
        }
        Ok(NegotiationOutcome::Deadlock { votes })
    }
}

/// Iteratively negotiate until consensus or max rounds.
///
/// `make_proposals` is called each round with the current group and should
/// return one [`Proposal`] per member agent.
pub fn reach_consensus<F>(
    group: &AgentGroup,
    max_rounds: usize,
    mut make_proposals: F,
) -> Result<NegotiationOutcome, CoordinationError>
where
    F: FnMut(&AgentGroup) -> Vec<Proposal>,
{
    if group.members.is_empty() {
        return Err(CoordinationError::EmptyGroup);
    }
    for _ in 0..max_rounds {
        let proposals = make_proposals(group);
        let outcome = negotiate(group, &proposals)?;
        match outcome {
            NegotiationOutcome::Consensus { .. } => return Ok(outcome),
            NegotiationOutcome::Deadlock { .. } => continue,
        }
    }
    Err(CoordinationError::ConsensusNotReached(max_rounds))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_group(name: &str, protocol: CoordinationProtocol, ids: &[AgentId]) -> AgentGroup {
        let mut g = AgentGroup::new(name, protocol);
        for &id in ids {
            g.add_member(id);
        }
        g
    }

    #[test]
    fn group_add_and_remove() {
        let mut g = make_group("test", CoordinationProtocol::Sync, &[1, 2, 3]);
        assert_eq!(g.member_count(), 3);
        assert!(g.is_member(2));
        g.remove_member(2);
        assert_eq!(g.member_count(), 2);
        assert!(!g.is_member(2));
    }

    #[test]
    fn group_set_priority() {
        let mut g = make_group("test", CoordinationProtocol::Sync, &[1, 2]);
        g.set_priority(1, 5.0);
        assert!((g.priorities[&1] - 5.0).abs() < f64::EPSILON);
        // Non-member cannot have priority set
        g.set_priority(99, 10.0);
        assert!(g.priorities.get(&99).is_none());
    }

    #[test]
    fn group_set_coordinator() {
        let mut g = make_group("test", CoordinationProtocol::LeaderFollower, &[1, 2]);
        assert!(g.set_coordinator(1).is_ok());
        assert_eq!(g.coordinator, Some(1));
    }

    #[test]
    fn group_set_coordinator_non_member_fails() {
        let mut g = make_group("test", CoordinationProtocol::LeaderFollower, &[1]);
        assert_eq!(
            g.set_coordinator(99),
            Err(CoordinationError::NotAMember(99))
        );
    }

    #[test]
    fn group_remove_coordinator_clears() {
        let mut g = make_group("test", CoordinationProtocol::LeaderFollower, &[1, 2]);
        g.set_coordinator(1).unwrap();
        g.remove_member(1);
        assert_eq!(g.coordinator, None);
    }

    #[test]
    fn group_members_ref() {
        let g = make_group("test", CoordinationProtocol::Sync, &[10, 20]);
        assert_eq!(g.members(), &[10, 20]);
    }

    #[test]
    fn sync_majority() {
        let mut g = make_group("council", CoordinationProtocol::Sync, &[1, 2, 3]);
        g.set_priority(1, 1.0);
        g.set_priority(2, 1.0);
        g.set_priority(3, 1.0);
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 2,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 3,
                action: "retreat".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        match outcome {
            NegotiationOutcome::Consensus { action, supporters } => {
                assert_eq!(action, "attack");
                assert_eq!(supporters.len(), 2);
            }
            _ => panic!("expected consensus"),
        }
    }

    #[test]
    fn sync_tie_produces_deadlock() {
        let mut g = make_group("council", CoordinationProtocol::Sync, &[1, 2]);
        g.set_priority(1, 1.0);
        g.set_priority(2, 1.0);
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 2,
                action: "retreat".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        assert!(matches!(outcome, NegotiationOutcome::Deadlock { .. }));
    }
    #[test]
    fn async_priority_wins() {
        let mut g = make_group("council", CoordinationProtocol::Async, &[1, 2]);
        g.set_priority(1, 10.0);
        g.set_priority(2, 1.0);
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 10.0,
            },
            Proposal {
                agent_id: 2,
                action: "retreat".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        match outcome {
            NegotiationOutcome::Consensus { action, .. } => assert_eq!(action, "attack"),
            _ => panic!("expected consensus"),
        }
    }

    #[test]
    fn leader_follower_quorum() {
        let mut g = make_group("council", CoordinationProtocol::LeaderFollower, &[1, 2, 3]);
        g.quorum = 2;
        g.set_coordinator(1).unwrap();
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 2,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 3,
                action: "retreat".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        match outcome {
            NegotiationOutcome::Consensus { action, supporters } => {
                assert_eq!(action, "attack");
                assert!(supporters.len() >= 2);
            }
            _ => panic!("expected consensus"),
        }
    }

    #[test]
    fn leader_follower_no_quorum() {
        let mut g = make_group("council", CoordinationProtocol::LeaderFollower, &[1, 2, 3]);
        g.quorum = 3;
        g.set_coordinator(1).unwrap();
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 2,
                action: "retreat".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 3,
                action: "retreat".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        assert!(matches!(outcome, NegotiationOutcome::Deadlock { .. }));
    }

    #[test]
    fn empty_group_error() {
        let g = AgentGroup::new("empty", CoordinationProtocol::Sync);
        let result = negotiate(&g, &[]);
        assert_eq!(result, Err(CoordinationError::EmptyGroup));
    }

    #[test]
    fn non_member_proposal_still_works() {
        // A proposal from a non-member is simply counted in the vote.
        let g = make_group("council", CoordinationProtocol::Sync, &[1, 2]);
        let proposals = vec![
            Proposal {
                agent_id: 1,
                action: "attack".into(),
                weight: 1.0,
            },
            Proposal {
                agent_id: 99,
                action: "attack".into(),
                weight: 1.0,
            },
        ];
        let outcome = negotiate(&g, &proposals).unwrap();
        // Non-member proposal contributes to the vote tally
        assert!(matches!(outcome, NegotiationOutcome::Consensus { .. }));
    }

    #[test]
    fn no_coordinator_error() {
        let g = make_group("council", CoordinationProtocol::LeaderFollower, &[1, 2]);
        let proposals = vec![Proposal {
            agent_id: 1,
            action: "attack".into(),
            weight: 1.0,
        }];
        let result = negotiate(&g, &proposals);
        assert_eq!(result, Err(CoordinationError::NoCoordinator));
    }

    #[test]
    fn reach_consensus_converges() {
        let mut g = make_group("council", CoordinationProtocol::Sync, &[1, 2, 3]);
        g.set_priority(1, 1.0);
        g.set_priority(2, 1.0);
        g.set_priority(3, 1.0);
        let mut round = 0;
        let result = reach_consensus(&g, 10, |_| {
            round += 1;
            if round >= 3 {
                vec![
                    Proposal {
                        agent_id: 1,
                        action: "unite".into(),
                        weight: 1.0,
                    },
                    Proposal {
                        agent_id: 2,
                        action: "unite".into(),
                        weight: 1.0,
                    },
                    Proposal {
                        agent_id: 3,
                        action: "unite".into(),
                        weight: 1.0,
                    },
                ]
            } else {
                vec![
                    Proposal {
                        agent_id: 1,
                        action: "attack".into(),
                        weight: 1.0,
                    },
                    Proposal {
                        agent_id: 2,
                        action: "retreat".into(),
                        weight: 1.0,
                    },
                    Proposal {
                        agent_id: 3,
                        action: "defend".into(),
                        weight: 1.0,
                    },
                ]
            }
        });
        assert!(result.is_ok());
        assert_eq!(round, 3);
    }

    #[test]
    fn reach_consensus_timeout() {
        let g = make_group("council", CoordinationProtocol::Sync, &[1, 2]);
        let result = reach_consensus(&g, 3, |_| {
            vec![
                Proposal {
                    agent_id: 1,
                    action: "attack".into(),
                    weight: 1.0,
                },
                Proposal {
                    agent_id: 2,
                    action: "retreat".into(),
                    weight: 1.0,
                },
            ]
        });
        assert_eq!(result, Err(CoordinationError::ConsensusNotReached(3)));
    }

    #[test]
    fn coordination_protocol_serde() {
        let p = CoordinationProtocol::LeaderFollower;
        let json = serde_json::to_string(&p).unwrap();
        let de: CoordinationProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(p, de);
    }
}
