//! Market specialization system for economic regions.
//!
//! Implements FR-ECON-003: Each economic region specializes in certain
//! goods based on natural advantages, infrastructure, and historical trade patterns.

use std::collections::HashMap;

/// A type of good that can be produced and traded.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoodType {
    /// Identifier used to associate this good with regional advantages and trade.
    pub id: String,
    /// Display name of the good.
    pub name: String,
    /// Broad economic category of the good.
    pub category: GoodCategory,
}

/// Broad categories of goods produced and traded by regions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoodCategory {
    /// Food and agricultural produce.
    Food,
    /// Raw and processed materials.
    Materials,
    /// Technological goods and equipment.
    Technology,
    /// Luxury goods.
    Luxury,
    /// Military supplies and equipment.
    Military,
}

/// Natural advantage of a region for producing a good.
#[derive(Debug, Clone)]
pub struct RegionAdvantage {
    /// Natural resource abundance, conventionally between 0.0 and 2.0.
    pub resource_multiplier: f64,
    /// Infrastructure quality, conventionally between 0.0 and 1.0.
    pub infrastructure_level: f64,
    /// Workforce skill, conventionally between 0.0 and 1.0.
    pub skill_base: f64,
    /// Trade network effects, conventionally between 0.0 and 1.0.
    pub historical_bonus: f64,
}

/// A region's specialization profile.
#[derive(Debug, Clone)]
pub struct RegionSpecialization {
    /// Identifier of the region described by this profile.
    pub region_id: String,
    /// Production advantages indexed by good identifier.
    pub advantages: HashMap<String, RegionAdvantage>,
    /// Top good identifiers in production priority order.
    pub production_focus: Vec<String>,
    /// Surplus amounts indexed by good identifier.
    pub trade_surplus: HashMap<String, f64>,
}

/// Calculate the production efficiency for a good in a region.
pub fn production_efficiency(advantage: &RegionAdvantage, demand_pressure: f64) -> f64 {
    let base =
        advantage.resource_multiplier * advantage.infrastructure_level * advantage.skill_base;
    let historical = 1.0 + advantage.historical_bonus * 0.2;
    let demand_boost = 1.0 + demand_pressure * 0.1;
    base * historical * demand_boost
}

/// Determine a region's top specializations based on advantages.
pub fn determine_specialization(
    region_id: &str,
    advantages: &HashMap<String, RegionAdvantage>,
    max_focus: usize,
) -> RegionSpecialization {
    let mut scored: Vec<(String, f64)> = advantages
        .iter()
        .map(|(good_id, adv)| {
            let score = production_efficiency(adv, 0.5);
            (good_id.clone(), score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let production_focus: Vec<String> = scored
        .iter()
        .take(max_focus)
        .map(|(id, _)| id.clone())
        .collect();

    RegionSpecialization {
        region_id: region_id.to_string(),
        advantages: advantages.clone(),
        production_focus,
        trade_surplus: HashMap::new(),
    }
}

/// Calculate comparative advantage between two regions for a good.
pub fn comparative_advantage(region_a: &RegionAdvantage, region_b: &RegionAdvantage) -> f64 {
    let eff_a = production_efficiency(region_a, 0.5);
    let eff_b = production_efficiency(region_b, 0.5);
    if eff_b == 0.0 {
        return f64::INFINITY;
    }
    eff_a / eff_b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_advantage(resource: f64, infra: f64, skill: f64, hist: f64) -> RegionAdvantage {
        RegionAdvantage {
            resource_multiplier: resource,
            infrastructure_level: infra,
            skill_base: skill,
            historical_bonus: hist,
        }
    }

    #[test]
    fn test_production_efficiency_scales_with_resources() {
        let low = make_advantage(0.5, 0.5, 0.5, 0.0);
        let high = make_advantage(1.5, 0.5, 0.5, 0.0);
        assert!(production_efficiency(&high, 0.0) > production_efficiency(&low, 0.0));
    }

    #[test]
    fn test_specialization_picks_top_goods() {
        let mut advantages = HashMap::new();
        advantages.insert("wheat".into(), make_advantage(1.5, 0.8, 0.7, 0.5));
        advantages.insert("iron".into(), make_advantage(0.3, 0.4, 0.3, 0.1));
        advantages.insert("silk".into(), make_advantage(1.2, 0.9, 0.8, 0.6));

        let spec = determine_specialization("region_1", &advantages, 2);
        assert_eq!(spec.production_focus.len(), 2);
        assert_eq!(spec.region_id, "region_1");
    }

    #[test]
    fn test_comparative_advantage() {
        let a = make_advantage(2.0, 1.0, 1.0, 0.0);
        let b = make_advantage(1.0, 1.0, 1.0, 0.0);
        let ratio = comparative_advantage(&a, &b);
        assert!((ratio - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_demand_pressure_increases_efficiency() {
        let adv = make_advantage(1.0, 1.0, 1.0, 0.0);
        let low_demand = production_efficiency(&adv, 0.0);
        let high_demand = production_efficiency(&adv, 1.0);
        assert!(high_demand > low_demand);
    }
}
