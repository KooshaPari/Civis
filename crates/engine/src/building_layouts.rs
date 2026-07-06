/// Emergent building layouts module (FR-CIV-ARCH)
///
/// Provides settlement-aware layout generation for buildings that emerges
/// from population, resources, and cultural factors rather than using
/// fixed predefined grids.
///
/// # Layout Strategies
///
/// - **Organic**: Chaotic, high-culture settlements favor curved/irregular patterns
/// - **Grid**: Low-culture, low-population settlements favor rigid orthogonal layouts
/// - **Mixed**: Intermediate states blend both patterns

use crate::culture::CultureSnapshot;
use crate::demographics::DemographicsSnapshot;
use serde::{Deserialize, Serialize};

/// Describes the structural layout pattern for a settlement (FR-CIV-ARCH).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutStrategy {
    /// Rigid orthogonal grid (0..33% culture score).
    Grid,
    /// Balanced mix of grid and organic (33..67% culture score).
    Mixed,
    /// Curved, irregular, organic pattern (67..100% culture score).
    Organic,
}

/// Computed layout properties for a settlement, emerged from its state (FR-CIV-ARCH).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergentLayout {
    /// Primary layout strategy chosen by emergence conditions.
    pub strategy: LayoutStrategy,
    /// Estimated building density (0..1) based on population/resource ratio.
    pub density: f32,
    /// Preferred inter-building spacing in tile units; lower = tighter (organic),
    /// higher = looser (grid).
    pub spacing: f32,
    /// Rotation/angular variation allowed in placement (radians);
    /// organic layouts permit more variation.
    pub angular_variation: f32,
}

impl EmergentLayout {
    /// Compute an emergent layout from settlement state (FR-CIV-ARCH).
    ///
    /// # Parameters
    ///
    /// - `culture`: Current cultural development level (0..100)
    /// - `population`: Total population in the settlement
    /// - `food_surplus`: Food resources available (influences density)
    /// - `metal_available`: Industrial capacity (influences clustering)
    ///
    /// # Returns
    ///
    /// An `EmergentLayout` with strategy, density, and spacing tuned to
    /// the settlement's emergent properties.
    pub fn compute(
        culture: f32,
        population: u32,
        food_surplus: f32,
        metal_available: f32,
    ) -> Self {
        // Normalize culture to 0..1 range for strategy selection.
        let culture_normalized = (culture / 100.0).clamp(0.0, 1.0);

        // Strategy selection based on culture development (FR-CIV-ARCH).
        let strategy = match culture_normalized {
            c if c < 0.33 => LayoutStrategy::Grid,
            c if c < 0.67 => LayoutStrategy::Mixed,
            _ => LayoutStrategy::Organic,
        };

        // Density emerges from population relative to sustainable carrying capacity.
        // Higher population + good food supply = denser settlement.
        let density = {
            let carrying_capacity = 500.0;
            let base_density = (population as f32 / carrying_capacity).clamp(0.0, 1.0);
            let food_factor = (food_surplus / 1000.0).clamp(0.0, 1.0);
            (base_density * 0.7 + food_factor * 0.3).clamp(0.1, 0.95)
        };

        // Spacing inversely related to density and culture level.
        // High culture + high population = tight spacing; low culture = loose grid spacing.
        let base_spacing = 3.0;
        let spacing = base_spacing / (1.0 + density * 2.0 * culture_normalized);

        // Angular variation: organic layouts permit rotation; grid layouts are rigid.
        let angular_variation = match strategy {
            LayoutStrategy::Grid => 0.0,      // No rotation
            LayoutStrategy::Mixed => 0.15,    // Slight rotation (~8.6 degrees)
            LayoutStrategy::Organic => 1.0,   // Full rotation freedom
        };

        EmergentLayout {
            strategy,
            density,
            spacing,
            angular_variation,
        }
    }

    /// Test helper: adjust layout based on resource scarcity (FR-CIV-ARCH).
    ///
    /// When resources are scarce, settlements cluster more tightly
    /// regardless of cultural development.
    pub fn with_resource_pressure(mut self, scarcity_factor: f32) -> Self {
        // Clamp scarcity to 0..1 (0 = abundant, 1 = critically scarce).
        let scarcity = scarcity_factor.clamp(0.0, 1.0);

        // High scarcity overrides density, forcing tighter clustering.
        self.density = (self.density * (1.0 - scarcity * 0.5)).clamp(0.1, 0.95);

        // Tighter spacing under pressure.
        self.spacing = self.spacing / (1.0 + scarcity);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-ARCH Test 1: Low culture → Grid layout
    #[test]
    fn test_low_culture_grid_layout() {
        let layout = EmergentLayout::compute(
            20.0,  // Low culture
            100.0, // Small population
            500.0, // Moderate food
            100.0, // Low metal
        );

        assert_eq!(layout.strategy, LayoutStrategy::Grid);
        assert_eq!(layout.angular_variation, 0.0, "Grid should have no rotation");
    }

    /// FR-CIV-ARCH Test 2: High culture → Organic layout
    #[test]
    fn test_high_culture_organic_layout() {
        let layout = EmergentLayout::compute(
            85.0,  // High culture
            800.0, // Large population
            2000.0, // Abundant food
            1500.0, // High metal
        );

        assert_eq!(layout.strategy, LayoutStrategy::Organic);
        assert!(
            layout.angular_variation > 0.5,
            "Organic layouts should permit significant rotation"
        );
    }

    /// FR-CIV-ARCH Test 3: Medium culture → Mixed layout
    #[test]
    fn test_medium_culture_mixed_layout() {
        let layout = EmergentLayout::compute(
            50.0,  // Medium culture
            400.0, // Medium population
            1000.0, // Adequate food
            800.0, // Moderate metal
        );

        assert_eq!(layout.strategy, LayoutStrategy::Mixed);
        assert!(layout.angular_variation > 0.0 && layout.angular_variation < 1.0);
    }

    /// FR-CIV-ARCH Test 4: Density differs by input conditions
    #[test]
    fn test_density_varies_with_population() {
        let low_pop = EmergentLayout::compute(50.0, 100.0, 500.0, 200.0);
        let high_pop = EmergentLayout::compute(50.0, 1000.0, 2000.0, 1200.0);

        assert!(
            high_pop.density > low_pop.density,
            "Higher population should produce denser layout"
        );
    }

    /// FR-CIV-ARCH Test 5: Resource pressure forces clustering
    #[test]
    fn test_resource_pressure_tightens_spacing() {
        let normal = EmergentLayout::compute(50.0, 300.0, 800.0, 400.0);
        let pressured = normal.with_resource_pressure(0.8);

        assert!(
            pressured.spacing < normal.spacing,
            "Resource pressure should tighten spacing"
        );
        assert!(
            pressured.density < normal.density,
            "Resource pressure should reduce density"
        );
    }

    /// FR-CIV-ARCH Test 6: Boundary conditions
    #[test]
    fn test_extreme_conditions() {
        // Minimum viable settlement
        let minimal = EmergentLayout::compute(0.0, 10.0, 50.0, 10.0);
        assert_eq!(minimal.strategy, LayoutStrategy::Grid);
        assert!(minimal.density >= 0.1, "Density should have floor");

        // Maximum development
        let maximal = EmergentLayout::compute(100.0, 2000.0, 5000.0, 5000.0);
        assert_eq!(maximal.strategy, LayoutStrategy::Organic);
        assert!(maximal.density <= 0.95, "Density should have ceiling");
    }

    /// FR-CIV-ARCH Test 7: Identical inputs produce identical layouts
    #[test]
    fn test_deterministic_layout_generation() {
        let input = (50.0, 400.0, 1000.0, 500.0);

        let layout1 = EmergentLayout::compute(input.0, input.1, input.2, input.3);
        let layout2 = EmergentLayout::compute(input.0, input.1, input.2, input.3);

        assert_eq!(layout1, layout2, "Layout generation must be deterministic");
    }
}
