//! Wave-based parallel agent dispatch.
//!
//! Reads [`crate::model::Plan::waves()`] (topological wave plan) and
//! fans each wave out across configured lanes. Concurrency is bounded
//! per-lane via [`tokio::sync::Semaphore`]; mutex-protected lanes
//! acquire an in-memory mutex before dispatch so two exclusive lanes
//! (build-retry, oracle-gate, ...) cannot overlap.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinHandle;

use crate::model::{Lane, NodeId, NodeState, Plan, PlanMeta};
use crate::reporter::{DagEvent, Reporter};

/// Runner-return payload per node.
#[derive(Debug, Clone)]
pub struct NodeReport {
    pub node_id: String,
    pub summary: String,
    pub elapsed_ms: u64,
}

/// Pluggable agent runner: shell out, call a model, run cargo, ...
#[async_trait]
pub trait AgentRunner: Send + Sync + std::fmt::Debug {
    fn runner_label(&self) -> &str { "agent" }
    async fn run(&self, node: &crate::model::Node, _lane: &Lane) -> Result<NodeReport>;
}

/// Borrow/no-borrow wallet attached to each invocation.
#[derive(Debug, Default, Clone)]
pub struct Wallet {
    pub tick_id: String,
    pub budget_ms: u64,
}

#[derive(Clone)]
pub struct WaveContext {
    pub plan: Arc<Plan>,
    pub tick_id: String,
    pub reporter: Arc<dyn Reporter>,
    pub wallet: Wallet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}
impl Default for RetryPolicy {
    fn default() -> Self { Self { max_attempts: 3, backoff_ms: 250 } }
}

pub struct WaveExecutor {
    plan: Arc<Plan>,
    runners: HashMap<String, Arc<dyn AgentRunner>>,
    default_runner: Option<Arc<dyn AgentRunner>>,
    concurrency_per_lane: HashMap<String, usize>,
    active_mutexes: Arc<Mutex<HashSet<String>>>,
    /// Nodes that have completed in this executor's lifetime. Used by
    /// `ready_in_wave` to evaluate cross-wave deps that were not Done
    /// when the plan was loaded.
    completed: Arc<Mutex<HashSet<String>>>,
    current_tick: Arc<AtomicU64>,
    retry_policy: RetryPolicy,
}

impl Default for WaveExecutor {
    fn default() -> Self {
        Self {
            plan: Arc::new(Plan {
                id: String::new(),
                version: String::new(),
                layers: Vec::new(),
                lanes: Vec::new(),
                ticks: Vec::new(),
                pillars: Vec::new(),
                queued_pillars: Vec::new(),
                phases: Vec::new(),
                nodes: Vec::new(),
                metadata: PlanMeta {
                    total_nodes: 0,
                    total_ticks: 0,
                    last_updated: chrono::Utc::now(),
                    source_path: std::path::PathBuf::new(),
                },
            }),
            runners: HashMap::new(),
            default_runner: None,
            concurrency_per_lane: HashMap::new(),
            active_mutexes: Arc::new(Mutex::new(HashSet::new())),
            completed: Arc::new(Mutex::new(HashSet::new())),
            current_tick: Arc::new(AtomicU64::new(0)),
            retry_policy: RetryPolicy::default(),
        }
    }
}

impl std::fmt::Debug for WaveExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveExecutor")
            .field("lanes", &self.runners.keys().collect::<Vec<_>>())
            .field("tick", &self.current_tick.load(Ordering::Relaxed))
            .finish()
    }
}

impl WaveExecutor {
    pub fn new(plan: Arc<Plan>) -> Self {
        Self {
            plan,
            runners: HashMap::new(),
            default_runner: Some(Arc::new(NoopRunner)),
            concurrency_per_lane: HashMap::new(),
            active_mutexes: Arc::new(Mutex::new(HashSet::new())),
            completed: Arc::new(Mutex::new(HashSet::new())),
            current_tick: Arc::new(AtomicU64::new(0)),
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn register_lane_runner(&mut self, lane_id: &str, runner: Arc<dyn AgentRunner>) {
        self.runners.insert(lane_id.to_string(), runner);
    }

    pub fn set_concurrency(&mut self, lane_id: &str, n: usize) {
        self.concurrency_per_lane.insert(lane_id.to_string(), n);
    }

    fn lane_concurrency(&self, lane_id: &str) -> usize {
        self.concurrency_per_lane.get(lane_id).copied().unwrap_or(4)
    }

    /// Ready nodes within a wave considering deps. Treats nodes already
    /// marked Done in the loaded plan, plus nodes that this executor has
    /// completed in earlier waves, as "satisfied".
    pub async fn ready_in_wave(&self, wave: &[NodeId]) -> Vec<crate::model::Node> {
        let wave_ids: HashSet<&str> = wave.iter().map(|n| n.0.as_str()).collect();
        let plan_done: HashSet<&str> = self.plan.nodes.iter()
            .filter(|n| matches!(n.state, NodeState::Done | NodeState::Merged | NodeState::Verified))
            .map(|n| n.id.as_str())
            .collect();
        let runtime_done: HashSet<String> = {
            let g = self.completed.lock().await;
            g.iter().cloned().collect()
        };
        self.plan.nodes.iter()
            .filter(|n| wave_ids.contains(n.id.as_str()))
            .filter(|n| n.state == NodeState::Queued)
            .filter(|n| {
                n.deps.iter().all(|d| {
                    plan_done.contains(d.as_str()) || runtime_done.contains(d.as_str())
                })
            })
            .cloned()
            .collect()
    }

    /// Run one wave by index.
    pub async fn run_wave(&self, wave_idx: usize, ctx: WaveContext) -> Result<Vec<NodeReport>> {
        let total = self.plan.waves().len();
        let wave = self.plan.waves().into_iter().nth(wave_idx)
            .ok_or_else(|| anyhow!("wave {} out of range", wave_idx))?;
        ctx.reporter.event(DagEvent::WaveStart { wave_id: wave_idx, total });

        let ready = self.ready_in_wave(&wave).await;
        if ready.is_empty() {
            ctx.reporter.event(DagEvent::WaveEnd { wave_id: wave_idx, succeeded: 0, failed: 0 });
            return Ok(Vec::new());
        }

        let mut by_lane: HashMap<String, Vec<crate::model::Node>> = HashMap::new();
        for n in ready { by_lane.entry(n.lane.clone()).or_default().push(n); }

        let mut handles: Vec<JoinHandle<Result<NodeReport>>> = Vec::new();
        for (lane_id, nodes) in by_lane {
            let lane_obj = self.plan.lanes.iter()
                .find(|l| l.id == lane_id)
                .cloned()
                .unwrap_or(Lane {
                    id: lane_id.clone(),
                    purpose: "(default)".into(),
                    scope: "default".into(),
                    mutex: None,
                    default_concurrency: 1,
                    partition_by: None,
                });

            // (mutex acquisition moved into each per-node spawn to give
            //  per-task concurrency the same retry visibility)
            let runner = self.runners.get(&lane_id)
                .cloned()
                .or_else(|| self.default_runner.clone())
                .unwrap_or_else(|| Arc::new(NoopRunner));
            let cap = self.lane_concurrency(&lane_id);
            let sem = Arc::new(Semaphore::new(cap));
            let ctx_h = ctx.clone();
            let active_mutexes_h = Arc::clone(&self.active_mutexes);
            let mutex_lane_h = lane_obj.mutex.clone();
            let completed_h = Arc::clone(&self.completed);
            for n in nodes {
                let permit = Arc::clone(&sem).acquire_owned().await
                    .map_err(|_| anyhow!("sem closed"))?;
                let runner = Arc::clone(&runner);
                let reporter = Arc::clone(&ctx_h.reporter);
                let lane = lane_obj.clone();
                let active_h = Arc::clone(&active_mutexes_h);
                let mutex_lane = mutex_lane_h.clone();
                let completed_t = Arc::clone(&completed_h);
                handles.push(tokio::spawn(async move {
                    let _p = permit;
                    if let Some(ref m) = mutex_lane.clone() {
                        let mut spins = 0u32;
                        loop {
                            let g = active_h.lock().await;
                            if !g.contains(m) { break; }
                            drop(g);
                            if spins > 240 {
                                return Err(anyhow!("mutex {} busy too long", m));
                            }
                            tokio::time::sleep(Duration::from_millis(250)).await;
                            spins += 1;
                        }
                        let mut g = active_h.lock().await;
                        g.insert(m.clone());
                    }
                    reporter.event(DagEvent::NodeStarted {
                        node: n.id.clone(),
                        agent: runner.runner_label().to_string(),
                    });
                    let start = Instant::now();
                    let mut attempt = 0u32;
                    let res: Result<NodeReport> = loop {
                        attempt += 1;
                        match runner.run(&n, &lane).await {
                            Ok(mut r) => {
                                r.elapsed_ms = start.elapsed().as_millis() as u64;
                                reporter.event(DagEvent::NodeFinished {
                                    node: n.id.clone(),
                                    duration_ms: r.elapsed_ms,
                                    summary: r.summary.clone(),
                                });
                                break Ok(r);
                            }
                            Err(e) => {
                                if attempt >= 3 { break Err(e); }
                                tokio::time::sleep(Duration::from_millis(250)).await;
                            }
                        }
                    };
                    if let Ok(ref r) = res {
                        let mut g = completed_t.lock().await;
                        g.insert(r.node_id.clone());
                    }
                    if let Some(ref m) = mutex_lane {
                        if !m.is_empty() {
                            let mut g = active_h.lock().await;
                            g.remove(m);
                        }
                    }
                    res
                }));
            }
        }

        let mut reports = Vec::new();
        let mut failed = 0usize;
        for h in handles {
            let r = h.await.map_err(|e| anyhow!("join: {e}"))??;
            if r.summary.starts_with("FAILED") { failed += 1; }
            reports.push(r);
        }
        let succeeded = reports.len() - failed;
        ctx.reporter.event(DagEvent::WaveEnd { wave_id: wave_idx, succeeded, failed });
        Ok(reports)
    }

    /// Drive every wave to completion, in order.
    pub async fn run_all(&self) -> Result<Vec<NodeReport>> {
        let reporter: Arc<dyn Reporter> = Arc::new(crate::reporter::NoopReporter);
        let n = self.plan.waves().len();
        let mut total = Vec::new();
        for i in 0..n {
            let ctx = WaveContext {
                plan: Arc::clone(&self.plan),
                tick_id: format!("T{}", self.current_tick.fetch_add(1, Ordering::Relaxed)),
                reporter: Arc::clone(&reporter),
                wallet: Wallet::default(),
            };
            self.run_wave(i, ctx).await
                .map_err(|e| anyhow!("wave {} failed: {e}", i))?
                .into_iter()
                .for_each(|r| total.push(r));
        }
        Ok(total)
    }
}

/// Default no-op runner for lanes lacking a registered agent.
#[derive(Debug, Default)]
pub struct NoopRunner;
#[async_trait]
impl AgentRunner for NoopRunner {
    fn runner_label(&self) -> &str { "noop" }
    async fn run(&self, node: &crate::model::Node, _lane: &Lane) -> Result<NodeReport> {
        Ok(NodeReport {
            node_id: node.id.clone(),
            summary: format!("noop:{}", node.id),
            elapsed_ms: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::reporter::RecorderReporter;
    use std::path::PathBuf;

    fn n(id: &str, state: NodeState, deps: Vec<&str>, lane: &str) -> Node {
        Node {
            id: id.into(), layer: "P0".into(), pr: None,
            title: format!("t-{id}"), scope: "scope".into(),
            state, lane: lane.into(),
            deps: deps.into_iter().map(String::from).collect(),
            fr_ref: None, adr_ref: None, eta: None,
            started_at: None, finished_at: None,
            summary: None, agent: None, task: None,
        }
    }

    fn plan() -> Arc<Plan> {
        Arc::new(Plan {
            id: "t".into(), version: "v0".into(),
            layers: vec![], lanes: vec![], ticks: vec![],
            pillars: vec![], queued_pillars: vec![], phases: vec![],
            nodes: vec![
                n("root", NodeState::Queued, vec![], "core"),
                n("left", NodeState::Queued, vec!["root"], "core"),
                n("right", NodeState::Queued, vec!["root"], "core"),
                n("done", NodeState::Done, vec![], "core"),
            ],
            metadata: PlanMeta {
                total_nodes: 4, total_ticks: 0,
                last_updated: chrono::Utc::now(),
                source_path: PathBuf::from("t.md"),
            },
        })
    }

    #[tokio::test]
    async fn ready_in_wave_filters() {
        let p = plan();
        let ex = WaveExecutor::new(p.clone());
        let waves = p.waves();
        let mut found_ready = false;
        for w in &waves {
            let r = ex.ready_in_wave(w).await;
            if !r.is_empty() { found_ready = true; }
        }
        assert!(found_ready);
    }

    #[tokio::test]
    async fn run_all_emits_reports_and_events() {
        let p = plan();
        let rec = Arc::new(RecorderReporter::default());
        let ex = WaveExecutor::new(p.clone());
        let waves_n = p.waves().len();
        let mut total_reports = 0usize;
        for i in 0..waves_n {
            let ctx = WaveContext {
                plan: Arc::clone(&p),
                tick_id: format!("T{i}"),
                reporter: Arc::clone(&rec) as Arc<dyn Reporter>,
                wallet: Wallet::default(),
            };
            let rs = ex.run_wave(i, ctx).await.unwrap();
            total_reports += rs.len();
        }
        assert_eq!(total_reports, 3, "three Queued nodes total");
        let evs = rec.events.lock().unwrap();
        assert!(evs.iter().any(|e| matches!(e, DagEvent::WaveStart { .. })));
    }
}
