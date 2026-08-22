//! Economy panel — aggregated economic indicators for the HUD dashboard.
//!
//! Provides structured data types for GDP, trade balance, employment,
//! and resource stock levels. All fields are serialisable to JSON for
//! wire transport to any client.
//!
//! Design contract:
//! 1. **Pure data, no engine.**
//! 2. **Additive only.**
//! 3. **Serialisation-safe.**

use serde::{Deserialize, Serialize};

/// A single resource stock entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceStock {
    /// Human-readable resource name (e.g. "grain", "iron").
    pub name: String,
    /// Current stock quantity (arbitrary units).
    pub quantity: f64,
    /// Production rate per tick.
    pub production_rate: f64,
    /// Consumption rate per tick.
    pub consumption_rate: f64,
}

impl ResourceStock {
    /// Net flow: production minus consumption. Positive = surplus.
    #[must_use]
    pub fn net_flow(&self) -> f64 {
        self.production_rate - self.consumption_rate
    }

    /// Is this resource in surplus?
    #[must_use]
    pub fn is_surplus(&self) -> bool {
        self.net_flow() > 0.0
    }
}

/// Employment sector summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmploymentSector {
    /// Sector name (e.g. "agriculture", "military").
    pub sector: String,
    /// Workers assigned to this sector.
    pub workers: u32,
    /// Output efficiency multiplier (1.0 = baseline).
    pub efficiency: f32,
}

/// Aggregated economy panel snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomyPanel {
    /// Gross domestic product (arbitrary currency units per tick).
    pub gdp: f64,
    /// Trade balance: exports minus imports (positive = net exporter).
    pub trade_balance: f64,
    /// Total employed civilians.
    pub employment: u32,
    /// Total eligible workforce (employed + unemployed).
    pub workforce: u32,
    /// Per-sector employment breakdown.
    pub sectors: Vec<EmploymentSector>,
    /// Per-resource stock levels.
    pub resources: Vec<ResourceStock>,
}

impl EconomyPanel {
    /// Construct an empty panel with zero GDP.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            gdp: 0.0,
            trade_balance: 0.0,
            employment: 0,
            workforce: 0,
            sectors: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Unemployment rate as a fraction (0.0-1.0).
    #[must_use]
    pub fn unemployment_rate(&self) -> f32 {
        if self.workforce == 0 {
            return 0.0;
        }
        (self.workforce - self.employment) as f32 / self.workforce as f32
    }

    /// Number of tracked resource types.
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Total surplus resources (those with positive net flow).
    #[must_use]
    pub fn surplus_resource_count(&self) -> usize {
        self.resources.iter().filter(|r| r.is_surplus()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel() -> EconomyPanel {
        EconomyPanel {
            gdp: 5000.0,
            trade_balance: 120.0,
            employment: 800,
            workforce: 1000,
            sectors: vec![
                EmploymentSector { sector: "agriculture".into(), workers: 400, efficiency: 1.2 },
                EmploymentSector { sector: "military".into(), workers: 200, efficiency: 0.9 },
            ],
            resources: vec![
                ResourceStock { name: "grain".into(), quantity: 500.0, production_rate: 50.0, consumption_rate: 40.0 },
                ResourceStock { name: "iron".into(), quantity: 200.0, production_rate: 10.0, consumption_rate: 25.0 },
            ],
        }
    }

    #[test]
    fn empty_panel_has_zero_gdp() {
        let p = EconomyPanel::empty();
        assert_eq!(p.gdp, 0.0);
        assert_eq!(p.employment, 0);
        assert_eq!(p.workforce, 0);
    }

    #[test]
    fn unemployment_rate_computes_correctly() {
        let p = sample_panel();
        // (1000 - 800) / 1000 = 0.2
        assert!((p.unemployment_rate() - 0.2).abs() < 1e-6);
    }

    #[test]
    fn unemployment_rate_returns_zero_for_empty_panel() {
        let p = EconomyPanel::empty();
        assert!((p.unemployment_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn resource_surplus_detection() {
        let grain = &sample_panel().resources[0];
        assert!(grain.is_surplus());
        assert!((grain.net_flow() - 10.0).abs() < 1e-6);

        let iron = &sample_panel().resources[1];
        assert!(!iron.is_surplus());
        assert!((iron.net_flow() - (-15.0)).abs() < 1e-6);
    }

    #[test]
    fn surplus_resource_count() {
        let p = sample_panel();
        assert_eq!(p.surplus_resource_count(), 1);
    }

    #[test]
    fn panel_round_trips_via_serde_json() {
        let p = sample_panel();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: EconomyPanel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p.gdp, back.gdp);
        assert_eq!(p.sectors.len(), back.sectors.len());
        assert_eq!(p.resources.len(), back.resources.len());
    }
}
