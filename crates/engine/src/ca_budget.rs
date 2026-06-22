pub struct CaTickBudget {
    pub max_chunks_per_step: usize,
    pub tick_hz: f32,
}

impl Default for CaTickBudget {
    fn default() -> Self {
        Self {
            max_chunks_per_step: 64,
            tick_hz: 2.0,
        }
    }
}

pub struct StepOutcome {
    pub chunks_stepped: usize,
    pub budget_exhausted: bool,
}
