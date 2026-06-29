//! Research focus allocation: weighted split of research points across active
//! projects, with completion at thresholds.
//!
//! FR-CIV-RESEARCH-FOCUS
//! =====================
//!
//! Each tick (or any cadence a caller chooses), a civilization can spend a
//! pool of `ResearchPoints` on several *active research projects* in parallel.
//! A `FocusAllocation` records the relative weight each project receives;
//! weights are integers, do not need to sum to any specific value, and may
//! change every call to [`ResearchFocus::set_weight`].
//!
//! Pure logic lives here so it can be reused by determinism-sensitive
//! subsystems (Bevy systems, server ticks, replay log validation, AI
//! advisors). All arithmetic is integer-only (no floats) so that the same
//! inputs always produce the same point distribution on every platform.
//!
//! Allocation algorithm
//! --------------------
//!
//! Given `points` total and weights `{w_i}`:
//!
//! 1. Floor-share: each active project i receives `floor(points * w_i / total_w)`.
//! 2. The `points` left over (remainder) are distributed one point at a time
//!    to the projects whose fractional part `(points * w_i) % total_w` is
//!    largest, breaking ties by largest `w_i` (then by project id, lexicographic).
//! 3. Each project accumulates its share. If accumulated points meet or
//!    exceed the project's `points_required`, it **completes** and is
//!    removed from the allocation set.
//! 4. Any leftover completed-project points in the same call are *not*
//!    recycled mid-tick — the caller can route them next tick via
//!    [`ResearchFocus::sweep_unallocated`]. This keeps the step boundary
//!    explicit and replay logs easy to read.
//!
//! Threading, async, and I/O are intentionally out of scope — this module is
//! pure and synchronous. The caller is responsible for serialising state into
//! snapshots / event logs.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// One research project the civ is currently investing points into.
///
/// `id` is opaque to this module; the caller decides (typically a stable
/// `TechCard.id` or a faction-specific project handle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchProject {
    /// Stable identifier (caller-defined).
    pub id: String,
    /// Total research points required to complete the project.
    pub points_required: u64,
    /// Points already invested. `points_invested <= points_required`
    /// is *not* an invariant: the caller may arbitrarily raise
    /// `points_required`, and the allocator caps investment at the
    /// threshold so over-invested points don't overflow the project.
    pub points_invested: u64,
}

/// A record that a project completed during one allocation call. Returned by
/// [`ResearchFocus::allocate_points`] so the caller can emit
/// `mod.loaded.v1`-style events, advance unlocks, and so on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedProject {
    /// The completed project's id.
    pub id: String,
    /// Project points **at completion** (i.e. `min(points_invested, points_required)`).
    pub points_at_completion: u64,
}

/// The set of projects a civilisation is focusing on, plus their weights.
///
/// Stored as `BTreeMap` so iteration order is deterministic across runs,
/// which keeps replay verification stable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResearchFocus {
    /// `(project_id, project)` for every active project.
    projects: BTreeMap<String, ResearchProject>,
    /// `(project_id, weight)` — the relative focus weight per project. A
    /// weight of `0` means "do not invest in this project" and the project
    /// is silently skipped during allocation. Removing the entry
    /// entirely is also fine and equivalent.
    weights: BTreeMap<String, u64>,
    /// Per-tick unallocated spillover from previously-completed projects.
    /// See [`ResearchFocus::sweep_unallocated`].
    unallocated_spool: u64,
}

impl ResearchFocus {
    /// Construct an empty focus set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a project in the active set. If `points_invested >=
    /// points_required` it is **already complete** and will be ignored on
    /// subsequent allocations (the caller can drop it via
    /// [`ResearchFocus::take_completed`]).
    ///
    /// Returns `true` if the project was newly added, `false` if it
    /// updated an existing entry.
    pub fn upsert_project(
        &mut self,
        id: impl Into<String>,
        points_required: u64,
        points_invested: u64,
    ) -> bool {
        let id = id.into();
        let is_new = !self.projects.contains_key(&id);
        self.projects.insert(
            id.clone(),
            ResearchProject {
                id,
                points_required,
                points_invested,
            },
        );
        is_new
    }

    /// Remove a project from the focus set entirely (e.g. because the
    /// civ canceled it). Returns the project, if it had been registered.
    #[must_use]
    pub fn remove_project(&mut self, id: &str) -> Option<ResearchProject> {
        self.weights.remove(id);
        self.projects.remove(id)
    }

    /// Set the focus weight for `id`. `weight == 0` is allowed (the project
    /// will be skipped but remains registered so accumulated points are
    /// preserved if it is later re-prioritized).
    ///
    /// If `id` is not currently a project it is added with the default
    /// `(points_required == 0, points_invested == 0)` so subsequent
    /// allocations can address it. Callers should follow up with
    /// [`ResearchFocus::upsert_project`] if they need real thresholds.
    pub fn set_weight(&mut self, id: impl Into<String>, weight: u64) {
        let id = id.into();
        self.projects.entry(id.clone()).or_insert(ResearchProject {
            id: id.clone(),
            points_required: 0,
            points_invested: 0,
        });
        if weight == 0 {
            self.weights.remove(&id);
        } else {
            self.weights.insert(id, weight);
        }
    }

    /// Read-only view over the active projects, in deterministic id order.
    #[must_use]
    pub fn projects(&self) -> Vec<ResearchProject> {
        self.projects.values().cloned().collect()
    }

    /// Current weight for `id`, or `0` if unset.
    #[must_use]
    pub fn weight_of(&self, id: &str) -> u64 {
        self.weights.get(id).copied().unwrap_or(0)
    }

    /// How many points are spooled up from previously-completed projects and
    /// have not yet been re-allocated.
    #[must_use]
    pub fn unallocated(&self) -> u64 {
        self.unallocated_spool
    }

    /// Move the current spool into `out` and reset the spool to zero.
    /// Callers typically route the drained points into a fresh
    /// [`ResearchFocus::allocate_points`] call.
    pub fn sweep_unallocated(&mut self, out: &mut u64) {
        *out = self.unallocated_spool;
        self.unallocated_spool = 0;
    }

    /// Distribute `points` across active projects by weight.
    ///
    /// Returns the list of projects that completed **during this call** in
    /// the order they finished (which is itself deterministic: largest
    /// fractional-remainder first; ties resolved by larger weight, then
    /// lexicographic id).
    ///
    /// The caller can then route any uninvested remainder back into the
    /// spool via [`ResearchFocus::record_unallocated`] (or by simply
    /// ignoring it — the caller owns the resource accounting).
    pub fn allocate_points(&mut self, points: u64) -> Vec<CompletedProject> {
        if points == 0 {
            return Vec::new();
        }

        // Snapshot the (id, weight) pairs and compute the total weight.
        // Snapshotting means changing weights mid-allocation can't race
        // the floor-share math.
        let weighted: Vec<(String, u64)> = self
            .weights
            .iter()
            .filter(|(id, w)| **w > 0 && self.is_investable(id))
            .map(|(id, w)| (id.clone(), *w))
            .collect();

        let total_weight: u64 = weighted.iter().map(|(_, w)| *w).sum();
        if total_weight == 0 {
            // Nobody to fund — return the whole pool as unallocated so the
            // caller can decide what to do with it.
            self.unallocated_spool = self.unallocated_spool.saturating_add(points);
            return Vec::new();
        }

        // Step 1: floor-share each project.
        let mut floors: BTreeMap<String, u64> = BTreeMap::new();
        let mut distributed: u64 = 0;
        for (id, w) in &weighted {
            let share = points
                .saturating_mul(*w)
                .checked_div(total_weight)
                .unwrap_or(0);
            floors.insert(id.clone(), share);
            distributed = distributed.saturating_add(share);
        }

        // Step 2: distribute the remainder one point at a time, deterministic.
        let mut remainder = points.saturating_sub(distributed);
        // Build a stable order for spillovers. We compare on (remainder
        // fractional part *inversely*, weight desc, id asc) so higher
        // fractional share wins, ties broken by larger weight.
        while remainder > 0 {
            let Some(pick_id) = weighted
                .iter()
                .filter(|(id, _)| !self.is_full(id))
                .max_by(|a, b| {
                    let ra = points
                        .saturating_mul(a.1)
                        .checked_rem(total_weight)
                        .unwrap_or(0);
                    let rb = points
                        .saturating_mul(b.1)
                        .checked_rem(total_weight)
                        .unwrap_or(0);
                    rb.cmp(&ra)
                        // tie-break: larger weight
                        .then_with(|| b.1.cmp(&a.1))
                        // tie-break: lexicographic id ascending
                        .then_with(|| a.0.cmp(&b.0))
                })
                .map(|(id, _)| id.clone())
            else {
                // No project left that can absorb more points (all at threshold).
                break;
            };

            let entry = floors.entry(pick_id.clone()).or_insert(0);
            *entry = entry.saturating_add(1);
            remainder = remainder.saturating_sub(1);
        }

        // Anything still unspent (e.g. all projects at threshold) goes to
        // the spool for the caller to route later.
        if remainder > 0 {
            self.unallocated_spool = self.unallocated_spool.saturating_add(remainder);
        }

        // Step 3: apply per-project investments, recording completions.
        let mut completed: Vec<CompletedProject> = Vec::new();
        for (id, share) in floors {
            if share == 0 {
                continue;
            }
            let Some(proj) = self.projects.get_mut(&id) else {
                continue;
            };
            // Cap investment so we never over-fill past threshold.
            let remaining_needed = proj.points_required.saturating_sub(proj.points_invested);
            let actual = share.min(remaining_needed);
            proj.points_invested = proj
                .points_invested
                .saturating_add(actual)
                .min(proj.points_required);

            // Anything we wanted to spend but couldn't (because threshold
            // already met) goes back to the spool.
            let overflow = share.saturating_sub(actual);
            if overflow > 0 {
                self.unallocated_spool = self.unallocated_spool.saturating_add(overflow);
            }

            if proj.points_invested >= proj.points_required && proj.points_required > 0 {
                completed.push(CompletedProject {
                    id: proj.id.clone(),
                    points_at_completion: proj.points_invested,
                });
            }
        }

        // Deterministic completion ordering: by completion order is
        // already deterministic (we already iterated in id-sorted order),
        // but we sort here as well in case callers want a stable view of
        // the returned vector.
        completed.sort_by(|a, b| a.id.cmp(&b.id));
        completed
    }

    /// Caller-reported unallocated points (e.g. points that were assigned
    /// to a project the caller just deleted). Spools them for the next
    /// allocation.
    pub fn record_unallocated(&mut self, points: u64) {
        self.unallocated_spool = self.unallocated_spool.saturating_add(points);
    }

    /// Drop completed projects from the active set. Returns the projects
    /// that were dropped, in id order.
    pub fn take_completed(&mut self) -> Vec<CompletedProject> {
        let mut done: Vec<CompletedProject> = self
            .projects
            .values()
            .filter(|p| p.points_required > 0 && p.points_invested >= p.points_required)
            .map(|p| CompletedProject {
                id: p.id.clone(),
                points_at_completion: p.points_invested,
            })
            .collect();
        done.sort_by(|a, b| a.id.cmp(&b.id));

        // Remove both the project and any stale weight entries so the
        // caller doesn't accidentally re-focus a finished project.
        let done_ids: Vec<String> = done.iter().map(|c| c.id.clone()).collect();
        for id in &done_ids {
            self.projects.remove(id);
            self.weights.remove(id);
        }
        done
    }

    /// `true` iff `id` exists, is unfunded (zero weight) ⇒ we skip, OR is
    /// at/above threshold ⇒ we skip. Used by allocation to filter.
    fn is_investable(&self, id: &str) -> bool {
        match self.projects.get(id) {
            None => false,
            Some(p) => {
                if p.points_required == 0 || p.points_invested < p.points_required {
                    true
                } else {
                    false
                }
            }
        }
    }

    /// `true` iff `id` cannot absorb more points without exceeding its
    /// threshold. Used by the remainder-distribution loop.
    fn is_full(&self, id: &str) -> bool {
        match self.projects.get(id) {
            None => true,
            Some(p) => p.points_invested >= p.points_required,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Covers FR-CIV-RESEARCH-FOCUS.
    //!
    //! FR-CIV-RESEARCH-FOCUS — weighted allocation of research points
    //! completes the higher-weight project first when both are eligible
    //! for completion within the same allocation call.

    use super::*;

    fn focus_with(weights: &[(&str, u64)], projects: &[(&str, u64, u64)]) -> ResearchFocus {
        let mut f = ResearchFocus::new();
        for (id, req, invested) in projects {
            f.upsert_project(*id, *req, *invested);
        }
        for (id, w) in weights {
            f.set_weight(*id, *w);
        }
        f
    }

    /// FR-CIV-RESEARCH-FOCUS — weighted allocation completes the
    /// higher-weight project first.
    ///
    /// Two projects both need 3 points to complete. Project A has weight 3
    /// and project B has weight 1, so A should pull in 75 % of every point
    /// pool. With a 4-point pool A hits its threshold on the first share
    /// (3 points) and the call must report A as completed.
    #[test]
    fn weighted_allocation_completes_higher_weight_first() {
        let mut focus = focus_with(
            &[("A", 3), ("B", 1)],
            &[("A", 3, 0), ("B", 3, 0)],
        );

        let completed = focus.allocate_points(4);

        // A (weight 3, 75% of 4 = 3) must be reported as completed.
        // B (weight 1, 25% of 4 = 1) must NOT.
        assert_eq!(
            completed,
            vec![CompletedProject {
                id: "A".into(),
                points_at_completion: 3,
            }],
            "higher-weight project must complete before the lower-weight one",
        );

        // A has been advanced to its threshold; B still needs 2 more.
        let proj_a = focus
            .projects()
            .into_iter()
            .find(|p| p.id == "A")
            .expect("A still registered until take_completed");
        assert_eq!(proj_a.points_invested, 3);

        let proj_b = focus
            .projects()
            .into_iter()
            .find(|p| p.id == "B")
            .expect("B remains active");
        assert_eq!(proj_b.points_invested, 1);

        // One more 3-point call closes B as well.
        let next = focus.allocate_points(3);
        assert_eq!(
            next,
            vec![CompletedProject {
                id: "B".into(),
                points_at_completion: 3,
            }]
        );
    }

    /// FR-CIV-RESEARCH-FOCUS — allocation is deterministic: the same
    /// inputs always produce the same per-project shares.
    #[test]
    fn allocation_is_deterministic() {
        // Two projects with equal weight must split evenly; with a 1-point
        // remainder, the tie-break (id ascending) chooses deterministically.
        let weights = &[("A", 1), ("B", 1)];
        let projects = &[("A", 10, 0), ("B", 10, 0)];

        let mut f1 = focus_with(weights, projects);
        let mut f2 = focus_with(weights, projects);

        let out1 = f1.allocate_points(7);
        let out2 = f2.allocate_points(7);

        assert_eq!(out1, out2);
        // A gets floor(7 * 1/2) = 3 plus one remainder point (ties on
        // fractional = 0, weight = 1, then id asc → A wins). B gets 3.
        let proj_a = f1.projects().into_iter().find(|p| p.id == "A").unwrap();
        let proj_b = f1.projects().into_iter().find(|p| p.id == "B").unwrap();
        assert_eq!(proj_a.points_invested, 4);
        assert_eq!(proj_b.points_invested, 3);
    }

    /// FR-CIV-RESEARCH-FOCUS — a zero-weight entry is skipped but the
    /// underlying project (if any) is preserved.
    #[test]
    fn zero_weight_entry_is_skipped() {
        let mut focus = focus_with(
            &[("A", 0), ("B", 1)],
            &[("A", 5, 2), ("B", 5, 0)],
        );

        let completed = focus.allocate_points(2);
        assert!(completed.is_empty(), "A at threshold is not investable, B should not yet complete");

        let proj_a = focus
            .projects()
            .into_iter()
            .find(|p| p.id == "A")
            .expect("A still registered");
        assert_eq!(
            proj_a.points_invested, 2,
            "zero-weighted project must not receive investment"
        );

        let proj_b = focus
            .projects()
            .into_iter()
            .find(|p| p.id == "B")
            .expect("B registered");
        assert_eq!(proj_b.points_invested, 2);
    }

    /// FR-CIV-RESEARCH-FOCUS — unallocated pool grows when all projects
    /// are at threshold.
    #[test]
    fn unallocated_spool_when_all_full() {
        let mut focus = focus_with(&[("A", 1)], &[("A", 3, 3)]);

        let completed = focus.allocate_points(10);
        assert!(completed.is_empty());
        assert_eq!(focus.unallocated(), 10);
    }

    /// FR-CIV-RESEARCH-FOCUS — `take_completed` removes finished projects
    /// from the active set and returns them deterministically.
    #[test]
    fn take_completed_drops_finished_projects() {
        let mut focus = focus_with(
            &[("A", 3), ("B", 1)],
            &[("A", 3, 0), ("B", 3, 0)],
        );

        // Drive A to threshold; B has 1 point.
        let _ = focus.allocate_points(4);
        // Now finish B.
        let _ = focus.allocate_points(3);

        let done = focus.take_completed();
        assert_eq!(
            done,
            vec![
                CompletedProject {
                    id: "A".into(),
                    points_at_completion: 3,
                },
                CompletedProject {
                    id: "B".into(),
                    points_at_completion: 3,
                },
            ],
        );
        assert!(focus.projects().is_empty());
        assert_eq!(focus.weight_of("A"), 0);
        assert_eq!(focus.weight_of("B"), 0);
    }
}
