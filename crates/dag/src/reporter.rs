//! Sponsor-format reporter.
//!
//! Emits the same shape used in the latest sponsor messages:
//! - Progress bar(s) per epic, with N/M counts and ETA
//! - DAG tree in pseudo‑ASCII
//! - Agent | task | state | summary table
//! - Bullet list of next steps
//!
//! The reporter is renderer‑free (it accumulates snapshots); the CLI
//! binary in `bin/civ-dagctl` is what actually prints them using
//! `ratatui`-compatible block glyphs.

use anyhow::Result;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::Mutex;

use crate::model::{Node, NodeState, Plan};

/// Stream of DAG events produced by [`crate::wave::WaveExecutor`].
#[derive(Debug, Clone)]
pub enum DagEvent {
    WaveStart { wave_id: usize, total: usize },
    NodeStarted { node: String, agent: String },
    NodeFinished { node: String, duration_ms: u64, summary: String },
    WaveEnd { wave_id: usize, succeeded: usize, failed: usize },
}

pub trait Reporter: Send + Sync {
    fn event(&self, event: DagEvent);
    fn snapshot(&self, plan: &Plan) -> ReportSnapshot;
}

/// One frozen view at a point in time.
#[derive(Debug, Default, Clone)]
pub struct ReportSnapshot {
    pub progress_line: String,
    pub epics: Vec<EpicBar>,
    pub dag_tree: String,
    pub agents: Vec<AgentRow>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EpicBar {
    pub name: String,
    pub ratio: f64,
    pub done: usize,
    pub total: usize,
    pub elapsed_secs: u64,
    pub eta_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub agent: String,
    pub task: String,
    pub state: String,
    pub summary: String,
}

/// Default reporter — owns no I/O. It snapshots on demand.
#[derive(Debug, Default)]
pub struct NoopReporter;
impl Reporter for NoopReporter {
    fn event(&self, _: DagEvent) {}
    fn snapshot(&self, plan: &Plan) -> ReportSnapshot {
        ReportSnapshot::from_plan(plan)
    }
}

/// Test/probe recorder — captures every event for assertions.
#[derive(Debug, Default)]
pub struct RecorderReporter {
    pub events: Mutex<Vec<DagEvent>>,
}
impl Reporter for RecorderReporter {
    fn event(&self, e: DagEvent) {
        self.events.lock().unwrap().push(e);
    }
    fn snapshot(&self, plan: &Plan) -> ReportSnapshot {
        ReportSnapshot::from_plan(plan)
    }
}

impl ReportSnapshot {
    pub fn from_plan(plan: &Plan) -> Self {
        let done = plan.nodes.iter().filter(|n| matches!(n.state, NodeState::Done | NodeState::Merged | NodeState::Verified)).count();
        let total = plan.nodes.len();
        let ratio = if total == 0 { 1.0 } else { done as f64 / total as f64 };
        let bar = Self::bar(10, ratio);
        let progress_line = format!(
            "Progress █{}░░░ ~{:.0}% {}/{} FRs",
            bar.chars().take((ratio * 10.0).round() as usize).collect::<String>().len().min(10),
            ratio * 100.0,
            done,
            total
        );

        let epics = plan
            .pillars
            .iter()
            .take(8)
            .map(|p| {
                let total = p.prs.len().max(1);
                let done = (total as f64 * ratio) as usize;
                EpicBar {
                    name: p.fr_id.clone(),
                    ratio,
                    done,
                    total,
                    elapsed_secs: 0,
                    eta_secs: if ratio > 0.0 { Some(((total - done) as f64 * 60.0) as u64) } else { None },
                }
            })
            .collect();

        let dag_tree = Self::render_dag(plan, 80);
        let agents: Vec<AgentRow> = plan.nodes.iter()
            .filter(|n| n.agent.is_some())
            .map(|n| AgentRow {
                agent: n.agent.clone().unwrap_or_default(),
                task: n.title.clone(),
                state: format!("{:?}", n.state),
                summary: n.summary.clone().unwrap_or_default(),
            })
            .collect();
        let next_steps = plan.queued_pillars.iter().take(5).map(|p| format!("▶ {}", p.fr_id)).collect();

        Self { progress_line, epics, dag_tree, agents, next_steps }
    }

    /// Render a progress bar of `width` blocks filled by `ratio` (0..=1).
    pub fn bar(width: usize, ratio: f64) -> String {
        let full = (ratio * width as f64).round().clamp(0.0, width as f64) as usize;
        let mut s = String::with_capacity(width * 3);
        for _ in 0..full { s.push('█'); }
        for _ in full..width { s.push('░'); }
        s
    }

    /// Render a textual DAG using branches for parallel lanes.
    pub fn render_dag(plan: &Plan, _width: usize) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "DAG (waves={})", plan.waves().len());
        for (i, wave) in plan.waves().iter().enumerate() {
            let _ = writeln!(out, "  wave {i}");
            let mut q: VecDeque<&Node> = wave.iter()
                .filter_map(|id| plan.nodes.iter().find(|n| n.id == id.0))
                .collect();
            while let Some(n) = q.front() {
                let prefix = if q.len() > 1 { "├── " } else { "└── " };
                let _ = writeln!(out, "    {prefix}{} [{:?}]", n.id, n.state);
                let _ = q.pop_front();
            }
        }
        out
    }

    /// Emit a sponsor-style tick report.
    pub fn render_first(&self) -> Result<String> {
        let mut out = String::new();
        let _ = writeln!(out, "{}", self.progress_line);
        for e in &self.epics {
            let _ = writeln!(out,
                "  {name:20} {bar} {ratio:.0}% {done}/{total} elapsed={elapsed}s eta={eta:?}",
                name = e.name,
                bar = Self::bar(10, e.ratio),
                ratio = e.ratio * 100.0,
                done = e.done,
                total = e.total,
                elapsed = e.elapsed_secs,
                eta = e.eta_secs,
            );
        }
        if !self.dag_tree.is_empty() {
            let _ = writeln!(out, "\n{}", self.dag_tree);
        }
        if !self.agents.is_empty() {
            let _ = writeln!(out, "agents:");
            for a in &self.agents {
                let _ = writeln!(out, "  · {} | {} | {} | {}", a.agent, a.task, a.state, a.summary);
            }
        }
        if !self.next_steps.is_empty() {
            let _ = writeln!(out, "\nNext steps");
            for s in &self.next_steps {
                let _ = writeln!(out, "  - {}", s);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::path::PathBuf;
    use chrono::Utc;

    fn empty_plan() -> Plan {
        Plan {
            id: "p".into(),
            version: "v0".into(),
            layers: vec![],
            lanes: vec![],
            ticks: vec![],
            pillars: vec![],
            queued_pillars: vec![],
            phases: vec![],
            nodes: vec![],
            metadata: PlanMeta {
                total_nodes: 0,
                total_ticks: 0,
                last_updated: Utc::now(),
                source_path: PathBuf::from("p.md"),
            },
        }
    }

    #[test]
    fn bar_full_and_empty() {
        assert_eq!(ReportSnapshot::bar(10, 1.0), "██████████");
        assert_eq!(ReportSnapshot::bar(10, 0.0), "░░░░░░░░░░");
    }

    #[test]
    fn from_plan_empty_emits_progress() {
        let snap = ReportSnapshot::from_plan(&empty_plan());
        assert!(snap.progress_line.contains("Progress"));
        assert!(snap.render_first().unwrap().contains("Progress"));
    }

    #[test]
    fn recorder_captures_events() {
        let rec = RecorderReporter::default();
        rec.event(DagEvent::WaveStart { wave_id: 0, total: 1 });
        rec.event(DagEvent::WaveEnd { wave_id: 0, succeeded: 1, failed: 0 });
        let evs = rec.events.lock().unwrap();
        assert_eq!(evs.len(), 2);
    }
}
