use civ_engine::Fixed;

#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    pub waste_joules: f64,
    pub surplus_joules: f64,
    pub tyranny_index: f64,
    pub legitimacy_index: f64,
}

pub fn compute(energy_budget_joules: Fixed, consumption_joules: Fixed) -> Metrics {
    let energy_f64 = energy_budget_joules.to_f64();
    let consumption_f64 = consumption_joules.to_f64();
    
    let waste = (consumption_f64 * 0.1).max(0.0);
    let surplus = (energy_f64 - consumption_f64).max(0.0);
    let tyranny = (consumption_f64 / (energy_f64 + 1.0)).min(1.0);
    let legitimacy = (1.0 - tyranny).max(0.0);

    Metrics {
        waste_joules: waste,
        surplus_joules: surplus,
        tyranny_index: tyranny,
        legitimacy_index: legitimacy,
    }
}
