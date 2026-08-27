//! Emergent caravan and trade route formation.
//!
//! Implements FR-CIV-CARAVAN-001: Trade routes form organically between
//! settlements with complementary resource profiles.
//!
//! The caravan system evaluates every settlement pair each tick and
//! decides whether a trade route is profitable enough to spawn. Caravans
//! travel along routes, are vulnerable to raiding, and increase trade
//! trust between settlements on success.

use std::collections::BTreeMap;

/// Resource good identifier.
pub type GoodId = u32;

/// Fixed-point type alias (using i64 with SCALE=1000 for now).
pub type Fixed = i64;

/// Scale factor for fixed-point arithmetic.
pub const SCALE: i64 = 1000;

/// Maximum number of active caravans per settlement.
pub const MAX_CARAVANS_PER_SETTLEMENT: usize = 3;

/// Minimum commodity gap to trigger a caravan (in SCALE units).
pub const MIN_COMMODITY_GAP: i64 = 50;

/// Minimum safety level for caravan to depart.
pub const MIN_SAFETY_THRESHOLD: i64 = 300; // 30% safety

/// Raider encounter probability per tick (in SCALE units, 0.02 = 2%).
pub const RAIDER_PROBABILITY_PER_TICK: i64 = 20;

/// Cargo capacity of a single caravan (in SCALE units).
pub const CARAVAN_CAPACITY: i64 = 1000;

/// A settlement's resource profile for trade evaluation.
#[derive(Debug, Clone)]
pub struct SettlementProfile {
    /// Settlement identifier.
    pub id: u32,
    /// Resource stock: good_id -> quantity (in SCALE units).
    pub stock: BTreeMap<GoodId, i64>,
    /// Resource production rate: good_id -> quantity per tick.
    pub production: BTreeMap<GoodId, i64>,
    /// Population (affects demand).
    pub population: i64,
    /// Safety level: 0.0 to 1.0 (in SCALE units).
    pub safety: i64,
    /// Number of active caravans.
    pub active_caravans: usize,
}

/// A commodity gap between two settlements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommodityGap {
    /// Good identifier.
    pub good: GoodId,
    /// Surplus in source settlement.
    pub source_surplus: i64,
    /// Deficit in target settlement.
    pub target_deficit: i64,
    /// Estimated profit (surplus * deficit / scale).
    pub estimated_profit: i64,
}

/// A trade route between two settlements.
#[derive(Debug, Clone)]
pub struct TradeRoute {
    /// Source settlement (has surplus).
    pub source: u32,
    /// Target settlement (has deficit).
    pub target: u32,
    /// Goods being traded.
    pub goods: Vec<CommodityGap>,
    /// Total estimated profit per trip.
    pub total_profit: i64,
    /// Trust bonus on success.
    pub trust_bonus: i64,
}

/// A spawned caravan entity.
#[derive(Debug, Clone)]
pub struct Caravan {
    /// Unique identifier.
    pub id: u32,
    /// Source settlement.
    pub source: u32,
    /// Target settlement.
    pub target: u32,
    /// Goods being carried: good_id -> quantity.
    pub cargo: BTreeMap<GoodId, i64>,
    /// Ticks remaining to arrival.
    pub ticks_remaining: i64,
    /// Total travel distance (ticks).
    pub travel_time: i64,
    /// Whether the caravan was raided.
    pub raided: bool,
}

/// Result of evaluating a single caravan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaravanOutcome {
    /// Caravan successfully arrived.
    Arrived {
        /// Total goods delivered.
        delivered: BTreeMap<GoodId, i64>,
    },
    /// Caravan was raided en route.
    Raided {
        /// Goods lost.
        lost: BTreeMap<GoodId, i64>,
    },
    /// Caravan is still in transit.
    InTransit { ticks_remaining: i64 },
}

/// Configuration for the caravan system.
#[derive(Debug, Clone)]
pub struct CaravanConfig {
    /// Minimum safety to spawn a caravan.
    pub min_safety: i64,
    /// Minimum commodity gap (SCALE units).
    pub min_commodity_gap: i64,
    /// Maximum caravans per settlement.
    pub max_per_settlement: usize,
    /// Raider probability per tick (0-SCALE).
    pub raider_probability: i64,
    /// Travel time multiplier (ticks per distance unit).
    pub travel_time_multiplier: i64,
    /// Trust bonus per successful trip (SCALE units).
    pub trust_bonus: i64,
}

impl Default for CaravanConfig {
    fn default() -> Self {
        Self {
            min_safety: MIN_SAFETY_THRESHOLD,
            min_commodity_gap: MIN_COMMODITY_GAP,
            max_per_settlement: MAX_CARAVANS_PER_SETTLEMENT,
            raider_probability: RAIDER_PROBABILITY_PER_TICK,
            travel_time_multiplier: 1,
            trust_bonus: 50, // 5% trust increase per trip
        }
    }
}

/// Evaluate commodity gaps between two settlements.
///
/// Returns a list of profitable trade opportunities.
pub fn evaluate_commodity_gaps(
    source: &SettlementProfile,
    target: &SettlementProfile,
    min_gap: i64,
) -> Vec<CommodityGap> {
    let mut gaps = Vec::new();

    // Check all goods that source produces
    for (&good_id, &surplus) in &source.production {
        if surplus <= 0 {
            continue;
        }

        // Target's stock of this good
        let target_stock = target.stock.get(&good_id).copied().unwrap_or(0);

        // Target's production of this good
        let target_prod = target.production.get(&good_id).copied().unwrap_or(0);

        // Deficit = what target consumes minus what it produces
        // Consumption is proportional to population (each person needs some of each good)
        let target_consumption = target.population; // Simplified: 1 unit per person per tick
        let target_deficit = target_consumption.saturating_sub(target_prod);

        if target_deficit < min_gap {
            continue;
        }

        // Source surplus is what it can spare
        let source_spare = source
            .stock
            .get(&good_id)
            .copied()
            .unwrap_or(0)
            .min(surplus);

        if source_spare < min_gap {
            continue;
        }

        let shipment = source_spare.min(target_deficit);
        let estimated_profit = (shipment * 8) / 10; // 80% profit margin

        gaps.push(CommodityGap {
            good: good_id,
            source_surplus: source_spare,
            target_deficit,
            estimated_profit,
        });
    }

    gaps
}

/// Score a trade route based on commodity gaps.
///
/// Higher scores indicate more profitable routes.
pub fn score_route(gaps: &[CommodityGap]) -> i64 {
    if gaps.is_empty() {
        return 0;
    }

    let total_profit: i64 = gaps.iter().map(|g| g.estimated_profit).sum();
    let diversity_bonus = (gaps.len() as i64) * 10; // Bonus for multiple goods
    let max_single = gaps.iter().map(|g| g.estimated_profit).max().unwrap_or(0);

    total_profit + diversity_bonus + max_single / 4
}

/// Evaluate whether a caravan should be spawned.
///
/// Returns a TradeRoute if profitable, None otherwise.
pub fn evaluate_caravan_spawn(
    source: &SettlementProfile,
    target: &SettlementProfile,
    config: &CaravanConfig,
) -> Option<TradeRoute> {
    // Safety check
    if source.safety < config.min_safety || target.safety < config.min_safety {
        return None;
    }

    // Caravan capacity check
    if source.active_caravans >= config.max_per_settlement {
        return None;
    }

    // Evaluate commodity gaps
    let gaps = evaluate_commodity_gaps(source, target, config.min_commodity_gap);

    if gaps.is_empty() {
        return None;
    }

    // Score the route
    let total_profit = gaps.iter().map(|g| g.estimated_profit).sum();
    let route_score = score_route(&gaps);

    // Only spawn if profitable enough
    if route_score < config.min_commodity_gap {
        return None;
    }

    // Travel time is proportional to population difference (larger = further)
    let pop_diff = (source.population - target.population).abs();
    let travel_time = 2 + (pop_diff / (500 * SCALE)) * config.travel_time_multiplier;

    Some(TradeRoute {
        source: source.id,
        target: target.id,
        goods: gaps,
        total_profit,
        trust_bonus: config.trust_bonus,
    })
}

/// Tick a caravan: advance travel, check for raiding.
pub fn tick_caravan(
    caravan: &mut Caravan,
    rng_value: i64,
    config: &CaravanConfig,
) -> CaravanOutcome {
    if caravan.ticks_remaining <= 0 {
        // Already arrived
        return CaravanOutcome::Arrived {
            delivered: caravan.cargo.clone(),
        };
    }

    caravan.ticks_remaining -= 1;

    if caravan.ticks_remaining <= 0 {
        // Just arrived
        if caravan.raided {
            return CaravanOutcome::Raided {
                lost: caravan.cargo.clone(),
            };
        }
        return CaravanOutcome::Arrived {
            delivered: caravan.cargo.clone(),
        };
    }

    // Check for raider encounter (probability per tick)
    if !caravan.raided && rng_value < config.raider_probability {
        caravan.raided = true;
        // Lose half the cargo
        for qty in caravan.cargo.values_mut() {
            *qty /= 2;
        }
    }

    CaravanOutcome::InTransit {
        ticks_remaining: caravan.ticks_remaining,
    }
}

/// Create a Caravan from a TradeRoute.
pub fn spawn_caravan(
    route: &TradeRoute,
    source: &SettlementProfile,
    id: u32,
    travel_time: i64,
) -> Caravan {
    let mut cargo = BTreeMap::new();
    for gap in &route.goods {
        // Load half the source surplus (keep half for local consumption)
        let load = gap.source_surplus / 2;
        cargo.insert(gap.good, load.min(CARAVAN_CAPACITY));
    }

    Caravan {
        id,
        source: route.source,
        target: route.target,
        cargo,
        ticks_remaining: travel_time,
        travel_time,
        raided: false,
    }
}

/// Calculate the total trust gained from successful caravans.
pub fn calculate_trust_gain(successful_trips: usize, base_bonus: i64) -> i64 {
    // Diminishing returns: each trip adds less
    let mut total = 0i64;
    for i in 0..successful_trips {
        let bonus = base_bonus / (1 + i as i64 / 3);
        total += bonus;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source() -> SettlementProfile {
        let mut stock = BTreeMap::new();
        stock.insert(1, 500); // food
        stock.insert(2, 200); // wood
        let mut production = BTreeMap::new();
        production.insert(1, 100); // produces food
        production.insert(2, 80); // produces wood

        SettlementProfile {
            id: 1,
            stock,
            production,
            population: 200,
            safety: 600, // 60% safe
            active_caravans: 0,
        }
    }

    fn make_target() -> SettlementProfile {
        let mut stock = BTreeMap::new();
        stock.insert(1, 100); // low food
        stock.insert(3, 300); // iron surplus
        let mut production = BTreeMap::new();
        production.insert(1, 20); // low food production
        production.insert(3, 100); // produces iron

        SettlementProfile {
            id: 2,
            stock,
            production,
            population: 150,
            safety: 500, // 50% safe
            active_caravans: 1,
        }
    }

    #[test]
    fn commodity_gaps_detected() {
        let source = make_source();
        let target = make_target();
        let gaps = evaluate_commodity_gaps(&source, &target, MIN_COMMODITY_GAP);
        assert!(!gaps.is_empty(), "Should detect food gap");
        assert!(gaps.iter().any(|g| g.good == 1), "Food gap should exist");
    }

    #[test]
    fn route_scoring_prefers_large_gaps() {
        let mut small_gaps = vec![CommodityGap {
            good: 1,
            source_surplus: 100,
            target_deficit: 100,
            estimated_profit: 80,
        }];

        let mut large_gaps = vec![CommodityGap {
            good: 1,
            source_surplus: 500,
            target_deficit: 500,
            estimated_profit: 400,
        }];

        let small_score = score_route(&small_gaps);
        let large_score = score_route(&large_gaps);
        assert!(large_score > small_score, "Large gaps should score higher");
    }

    #[test]
    fn safety_blocks_caravan() {
        let mut source = make_source();
        source.safety = 100; // 10% safe - below threshold
        let target = make_target();
        let config = CaravanConfig::default();

        let route = evaluate_caravan_spawn(&source, &target, &config);
        assert!(route.is_none(), "Unsafe source should not spawn caravan");
    }

    #[test]
    fn capacity_blocks_caravan() {
        let mut source = make_source();
        source.active_caravans = 3; // At max
        let target = make_target();
        let config = CaravanConfig::default();

        let route = evaluate_caravan_spawn(&source, &target, &config);
        assert!(route.is_none(), "Full capacity should not spawn caravan");
    }

    #[test]
    fn caravan_spawned_on_profitable_route() {
        let source = make_source();
        let target = make_target();
        let config = CaravanConfig::default();

        let route = evaluate_caravan_spawn(&source, &target, &config);
        assert!(route.is_some(), "Profitable route should spawn caravan");
        let route = route.unwrap();
        assert_eq!(route.source, 1);
        assert_eq!(route.target, 2);
        assert!(!route.goods.is_empty());
    }

    #[test]
    fn caravan_ticks_to_arrival() {
        let route = TradeRoute {
            source: 1,
            target: 2,
            goods: vec![CommodityGap {
                good: 1,
                source_surplus: 200,
                target_deficit: 200,
                estimated_profit: 160,
            }],
            total_profit: 160,
            trust_bonus: 50,
        };
        let source = make_source();
        let mut caravan = spawn_caravan(&route, &source, 1, 5);
        let config = CaravanConfig::default();

        for _ in 0..4 {
            let outcome = tick_caravan(&mut caravan, 100, &config); // rng > raider prob
            assert!(matches!(outcome, CaravanOutcome::InTransit { .. }));
        }
        let outcome = tick_caravan(&mut caravan, 100, &config);
        assert!(
            matches!(outcome, CaravanOutcome::Arrived { .. }),
            "Should arrive after travel_time ticks"
        );
    }

    #[test]
    fn raider_reduces_cargo() {
        let route = TradeRoute {
            source: 1,
            target: 2,
            goods: vec![CommodityGap {
                good: 1,
                source_surplus: 200,
                target_deficit: 200,
                estimated_profit: 160,
            }],
            total_profit: 160,
            trust_bonus: 50,
        };
        let source = make_source();
        let mut caravan = spawn_caravan(&route, &source, 1, 3);
        let config = CaravanConfig::default();

        // Tick with low rng (below raider probability)
        let _ = tick_caravan(&mut caravan, 0, &config);
        assert!(caravan.raided, "Caravan should be raided with rng=0");

        // Cargo should be halved
        for &qty in caravan.cargo.values() {
            assert!(qty <= 100, "Cargo should be reduced after raid");
        }
    }

    #[test]
    fn trust_gain_diminishing_returns() {
        let t1 = calculate_trust_gain(1, 50);
        let t3 = calculate_trust_gain(3, 50);
        let t5 = calculate_trust_gain(5, 50);

        // Each additional trip adds less
        assert!(t1 > 0);
        assert!(t3 > t1);
        assert!(t5 > t3);
        // But not linearly
        assert!(t5 < t1 * 5, "Diminishing returns should apply");
    }
}
