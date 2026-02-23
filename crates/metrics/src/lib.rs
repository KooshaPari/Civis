#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    pub waste_joules: f64,
    pub surplus_joules: f64,
    pub tyranny_index: f64,
    pub legitimacy_index: f64,
}

pub fn compute(energy_budget_joules: f64, consumption_joules: f64) -> Metrics {
    let waste = (consumption_joules * 0.1).max(0.0);
    let surplus = (energy_budget_joules - consumption_joules).max(0.0);
    let tyranny = (consumption_joules / (energy_budget_joules + 1.0)).min(1.0);
    let legitimacy = (1.0 - tyranny).max(0.0);

    Metrics {
        waste_joules: waste,
        surplus_joules: surplus,
        tyranny_index: tyranny,
        legitimacy_index: legitimacy,
    }
}
