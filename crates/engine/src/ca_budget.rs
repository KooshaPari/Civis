use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CaTickBudget {
    pub max_chunks_per_step: usize,
    pub tick_hz: f32,
}

impl Default for CaTickBudget {
    fn default() -> Self { Self { max_chunks_per_step: 64, tick_hz: 2.0 } }
}

#[derive(Debug, Default)]
pub struct StepOutcome {
    pub chunks_stepped: usize,
    pub budget_exhausted: bool,
}

pub fn step_with_budget<F>(budget: &CaTickBudget, mut step_fn: F) -> StepOutcome
where
    F: FnMut(usize) -> bool,
{
    let mut outcome = StepOutcome::default();
    for i in 0..budget.max_chunks_per_step {
        if !step_fn(i) {
            break;
        }
        outcome.chunks_stepped += 1;
        if outcome.chunks_stepped >= budget.max_chunks_per_step {
            outcome.budget_exhausted = true;
            break;
        }
    }
    outcome
}
