//! FR-SOCI-005 / FR-SOCI-006 — Health index and health crisis.
//!
//! The health index is computed from food Joules, clean water, and medical
//! infrastructure. Each input is weighted and the result is a continuous
//! score in `[0, 1_000]` basis points. A health crisis occurs when the
//! index falls below a configured threshold, emitting an event and
//! reducing labor productivity.

use crate::events::{Event, EventType};
use serde::{Deserialize, Serialize};

/// Basis-point denominator for health index range `[0, 1_000]`.
pub const MAX_HEALTH_BP: i64 = 1_000;

/// Default weights for health inputs (must sum to 100).
pub const DEFAULT_FOOD_WEIGHT: i64 = 40;
pub const DEFAULT_WATER_WEIGHT: i64 = 30;
pub const DEFAULT_MEDICAL_WEIGHT: i64 = 30;

/// Default crisis threshold (basis points).
pub const DEFAULT_CRISIS_THRESHOLD_BP: i64 = 300;

/// Default labor productivity multiplier during crisis (basis points, 0-1000).
pub const DEFAULT_CRISIS_LABOR_FACTOR_BP: i64 = 600;

/// Configuration for health computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Weight for food input (out of 100).
    pub food_weight: i64,
    /// Weight for water input (out of 100).
    pub water_weight: i64,
    /// Weight for medical input (out of 100).
    pub medical_weight: i64,
    /// Crisis threshold in basis points (below = crisis).
    pub crisis_threshold_bp: i64,
    /// Labor productivity multiplier during crisis (bp of normal).
    pub crisis_labor_factor_bp: i64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            food_weight: DEFAULT_FOOD_WEIGHT,
            water_weight: DEFAULT_WATER_WEIGHT,
            medical_weight: DEFAULT_MEDICAL_WEIGHT,
            crisis_threshold_bp: DEFAULT_CRISIS_THRESHOLD_BP,
            crisis_labor_factor_bp: DEFAULT_CRISIS_LABOR_FACTOR_BP,
        }
    }
}

/// Health input values, each in `[0, 1_000]` basis points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthInputs {
    /// Food availability in basis points.
    pub food_bp: i64,
    /// Clean water availability in basis points.
    pub water_bp: i64,
    /// Medical infrastructure availability in basis points.
    pub medical_bp: i64,
}

/// Result of a health computation for one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResult {
    /// Weighted health index in `[0, 1_000]` basis points.
    pub index_bp: i64,
    /// Whether a health crisis is active.
    pub crisis: bool,
    /// Labor productivity multiplier (bp of 1_000 = 100 %).
    pub labor_factor_bp: i64,
    /// Event to emit if a crisis just started; `None` otherwise.
    pub crisis_event: Option<Event>,
}

/// Compute the health index from food, water, and medical inputs.
///
/// `index = (food * food_w + water * water_w + medical * medical_w) / total_w`
///
/// All inputs are clamped to `[0, 1_000]`.
#[must_use]
pub fn compute_health(
    inputs: HealthInputs,
    config: &HealthConfig,
    crisis_active: bool,
    tick: u64,
) -> HealthResult {
    let food = inputs.food_bp.clamp(0, MAX_HEALTH_BP);
    let water = inputs.water_bp.clamp(0, MAX_HEALTH_BP);
    let medical = inputs.medical_bp.clamp(0, MAX_HEALTH_BP);

    let total_weight = config.food_weight + config.water_weight + config.medical_weight;
    let weight_denom = if total_weight > 0 { total_weight } else { 1 };

    let index = (food * config.food_weight
        + water * config.water_weight
        + medical * config.medical_weight)
        / weight_denom;

    let index = index.clamp(0, MAX_HEALTH_BP);
    let in_crisis = index < config.crisis_threshold_bp;
    let labor = if in_crisis {
        config.crisis_labor_factor_bp
    } else {
        MAX_HEALTH_BP
    };

    // Only emit a NEW crisis event if crisis just started this tick.
    let crisis_event = if in_crisis && !crisis_active {
        Some(Event {
            event_type: EventType::HealthCrisis,
            tick,
        })
    } else {
        None
    };

    HealthResult {
        index_bp: index,
        crisis: in_crisis,
        labor_factor_bp: labor,
        crisis_event,
    }
}
