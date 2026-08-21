//! Core production system (FR-ECON-001).
//!
//! This module implements order-based resource production that integrates with
//! the existing [`Stocks`] inventory system. Producers submit [`ProductionOrder`]s
//! specifying what to produce and in what quantity; the [`ProductionQueue`]
//! processes all pending orders each tick, consuming input resources and
//! adding output resources to stocks.

use serde::{Deserialize, Serialize};

use crate::stocks::{Good, Stocks};

/// Resource types supported by the production system (FR-ECON-001).
///
/// Each variant maps to one or more [`Good`] categories in the stock system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Food production (farming, fishing, foraging).
    Food,
    /// Energy production (power generation, fuel processing).
    Energy,
    /// Raw and refined materials (mining, smelting, lumber).
    Materials,
    /// Research and technology development.
    Technology,
}

/// All resource types in deterministic iteration order.
pub const RESOURCE_TYPES: [ResourceType; 4] = [
    ResourceType::Food,
    ResourceType::Energy,
    ResourceType::Materials,
    ResourceType::Technology,
];

impl ResourceType {
    /// Maps a [`ResourceType`] to the corresponding [`Good`] for stock tracking.
    ///
    /// The mapping is:
    /// - `Food` → `Good::Food`
    /// - `Energy` → `Good::Wood` (fuel)
    /// - `Materials` → `Good::Metal`
    /// - `Technology` → `Good::Tools`
    pub fn to_good(self) -> Good {
        match self {
            ResourceType::Food => Good::Food,
            ResourceType::Energy => Good::Wood,
            ResourceType::Materials => Good::Metal,
            ResourceType::Technology => Good::Tools,
        }
    }
}

/// A pending production order (FR-ECON-001).
///
/// Represents a request to produce a quantity of a resource. The order includes
/// the resource type, desired quantity, and the producer's identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionOrder {
    /// Unique identifier for this order.
    pub order_id: u64,
    /// What resource to produce.
    pub resource_type: ResourceType,
    /// Number of units to produce.
    pub quantity: i64,
    /// ID of the producer (district, building, or actor).
    pub producer_id: u32,
    /// Simulation tick when the order was submitted.
    pub submitted_tick: u64,
}

/// Result of processing a production order (FR-ECON-001).
///
/// Contains the actual output quantity produced and whether the order was
/// fully satisfied. When input resources are insufficient, the output is
/// proportionally reduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionResult {
    /// The order that was processed.
    pub order: ProductionOrder,
    /// Actual quantity produced (may be less than ordered).
    pub produced_qty: i64,
    /// Whether the order was fully satisfied (produced_qty == quantity).
    pub fully_satisfied: bool,
    /// Input resources consumed (good, quantity consumed).
    pub inputs_consumed: Vec<(Good, i64)>,
}

/// Production queue that manages pending orders and processes them each tick (FR-ECON-001).
///
/// The queue maintains a list of pending orders. Each tick, the caller invokes
/// [`ProductionQueue::process_tick`] which drains all pending orders, attempts
/// to produce the requested resources (consuming inputs from [`Stocks`]), and
/// returns the results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductionQueue {
    /// Pending orders waiting to be processed.
    pending: Vec<ProductionOrder>,
    /// Results from the most recent `process_tick` call.
    completed: Vec<ProductionResult>,
    /// Auto-incrementing order ID counter.
    next_order_id: u64,
}

impl ProductionQueue {
    /// Create an empty production queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a new production order and return its assigned order ID.
    ///
    /// The order is queued for processing on the next `process_tick` call.
    pub fn submit(
        &mut self,
        resource_type: ResourceType,
        quantity: i64,
        producer_id: u32,
        submitted_tick: u64,
    ) -> u64 {
        let order_id = self.next_order_id;
        self.next_order_id = self.next_order_id.saturating_add(1);
        self.pending.push(ProductionOrder {
            order_id,
            resource_type,
            quantity: quantity.max(0),
            producer_id,
            submitted_tick,
        });
        order_id
    }

    /// Process all pending orders for this tick.
    ///
    /// For each order, consumes the input resources from `stocks` and adds the
    /// output resources. When inputs are insufficient, output is proportionally
    /// reduced.
    ///
    /// Returns the list of production results.
    pub fn process_tick(&mut self, stocks: &mut Stocks) -> Vec<ProductionResult> {
        let orders: Vec<ProductionOrder> = self.pending.drain(..).collect();
        self.completed.clear();

        for order in orders {
            let result = produce(stocks, &order);
            self.completed.push(result);
        }

        self.completed.clone()
    }

    /// Query the current inventory for a resource type.
    ///
    /// Returns the quantity of the resource's corresponding [`Good`] in `stocks`.
    pub fn inventory(stocks: &Stocks, resource_type: ResourceType) -> i64 {
        stocks.get(resource_type.to_good())
    }

    /// Returns the number of pending orders.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Returns results from the most recent `process_tick` call.
    pub fn last_results(&self) -> &[ProductionResult] {
        &self.completed
    }

    /// Returns all pending orders.
    pub fn pending_orders(&self) -> &[ProductionOrder] {
        &self.pending
    }
}

/// Produce a single order, consuming inputs and adding outputs to stocks (FR-ECON-001).
///
/// Each unit of production consumes one unit of an input resource. The production
/// is deterministic and integer-only: when inputs are insufficient, output is
/// reduced proportionally.
///
/// # Production Chain
///
/// The system implements a transformation chain where each production step
/// consumes one good and produces a different one:
///
/// - **Food**: No inputs required (farming from land) → produces `Good::Food`.
/// - **Energy**: Consumes `Good::Food` (biomass fuel) → produces `Good::Wood`.
/// - **Materials**: Consumes `Good::Wood` (lumber) → produces `Good::Metal`.
/// - **Technology**: Consumes `Good::Metal` (refined components) → produces `Good::Tools`.
///
/// This chain ensures inputs and outputs are always different goods,
/// preventing circular production.
pub fn produce(stocks: &mut Stocks, order: &ProductionOrder) -> ProductionResult {
    if order.quantity <= 0 {
        return ProductionResult {
            order: order.clone(),
            produced_qty: 0,
            fully_satisfied: true,
            inputs_consumed: Vec::new(),
        };
    }

    let output_good = order.resource_type.to_good();
    let (input_good, input_per_unit) = match order.resource_type {
        ResourceType::Food => (None, 0),
        ResourceType::Energy => (Some(Good::Food), 1),
        ResourceType::Materials => (Some(Good::Wood), 1),
        ResourceType::Technology => (Some(Good::Metal), 1),
    };

    // Calculate how many units we can actually produce based on available inputs.
    let max_from_inputs = if let Some(ig) = input_good {
        let available = stocks.get(ig);
        available / input_per_unit
    } else {
        order.quantity
    };

    let produced_qty = order.quantity.min(max_from_inputs);

    // Consume inputs.
    let mut inputs_consumed = Vec::new();
    if let Some(ig) = input_good {
        let consumed = produced_qty * input_per_unit;
        if consumed > 0 {
            stocks.add(ig, -consumed);
            inputs_consumed.push((ig, consumed));
        }
    }

    // Add output.
    if produced_qty > 0 {
        stocks.add(output_good, produced_qty);
    }

    ProductionResult {
        order: order.clone(),
        produced_qty,
        fully_satisfied: produced_qty == order.quantity,
        inputs_consumed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_type_to_good_mapping() {
        assert_eq!(ResourceType::Food.to_good(), Good::Food);
        assert_eq!(ResourceType::Energy.to_good(), Good::Wood);
        assert_eq!(ResourceType::Materials.to_good(), Good::Metal);
        assert_eq!(ResourceType::Technology.to_good(), Good::Tools);
    }

    #[test]
    fn resource_types_constant_has_all_variants() {
        assert_eq!(RESOURCE_TYPES.len(), 4);
        assert!(RESOURCE_TYPES.contains(&ResourceType::Food));
        assert!(RESOURCE_TYPES.contains(&ResourceType::Energy));
        assert!(RESOURCE_TYPES.contains(&ResourceType::Materials));
        assert!(RESOURCE_TYPES.contains(&ResourceType::Technology));
    }

    #[test]
    fn production_order_fields_are_accessible() {
        let order = ProductionOrder {
            order_id: 42,
            resource_type: ResourceType::Food,
            quantity: 100,
            producer_id: 7,
            submitted_tick: 5,
        };
        assert_eq!(order.order_id, 42);
        assert_eq!(order.resource_type, ResourceType::Food);
        assert_eq!(order.quantity, 100);
        assert_eq!(order.producer_id, 7);
        assert_eq!(order.submitted_tick, 5);
    }

    #[test]
    fn produce_food_has_no_input_cost() {
        let mut stocks = Stocks::default();
        let order = ProductionOrder {
            order_id: 1,
            resource_type: ResourceType::Food,
            quantity: 50,
            producer_id: 1,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        assert_eq!(result.produced_qty, 50);
        assert!(result.fully_satisfied);
        assert!(result.inputs_consumed.is_empty());
        assert_eq!(stocks.get(Good::Food), 50);
    }

    #[test]
    fn produce_energy_consumes_food() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 10);
        let order = ProductionOrder {
            order_id: 2,
            resource_type: ResourceType::Energy,
            quantity: 8,
            producer_id: 1,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        assert_eq!(result.produced_qty, 8);
        assert!(result.fully_satisfied);
        assert_eq!(result.inputs_consumed, vec![(Good::Food, 8)]);
        assert_eq!(stocks.get(Good::Food), 2);
        assert_eq!(stocks.get(Good::Wood), 8); // Energy maps to Wood
    }

    #[test]
    fn produce_materials_consumes_wood() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Wood, 15);
        let order = ProductionOrder {
            order_id: 3,
            resource_type: ResourceType::Materials,
            quantity: 10,
            producer_id: 2,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        assert_eq!(result.produced_qty, 10);
        assert!(result.fully_satisfied);
        assert_eq!(result.inputs_consumed, vec![(Good::Wood, 10)]);
        assert_eq!(stocks.get(Good::Wood), 5);
        assert_eq!(stocks.get(Good::Metal), 10); // Materials maps to Metal
    }

    #[test]
    fn produce_technology_consumes_metal() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Metal, 3);
        let order = ProductionOrder {
            order_id: 4,
            resource_type: ResourceType::Technology,
            quantity: 5,
            producer_id: 3,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        // Only 3 Metal available, so can only produce 3 Technology.
        assert_eq!(result.produced_qty, 3);
        assert!(!result.fully_satisfied);
        assert_eq!(result.inputs_consumed, vec![(Good::Metal, 3)]);
        assert_eq!(stocks.get(Good::Metal), 0);
    }

    #[test]
    fn produce_clamps_at_zero_when_no_inputs() {
        let mut stocks = Stocks::default();
        let order = ProductionOrder {
            order_id: 5,
            resource_type: ResourceType::Energy,
            quantity: 10,
            producer_id: 1,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        assert_eq!(result.produced_qty, 0);
        assert!(!result.fully_satisfied);
        assert!(result.inputs_consumed.is_empty());
    }

    #[test]
    fn produce_zero_quantity_is_noop() {
        let mut stocks = Stocks::default();
        let order = ProductionOrder {
            order_id: 6,
            resource_type: ResourceType::Food,
            quantity: 0,
            producer_id: 1,
            submitted_tick: 0,
        };

        let result = produce(&mut stocks, &order);

        assert_eq!(result.produced_qty, 0);
        assert!(result.fully_satisfied);
        assert!(result.inputs_consumed.is_empty());
        assert_eq!(stocks.total(), 0);
    }

    #[test]
    fn production_queue_submit_assigns_incrementing_ids() {
        let mut queue = ProductionQueue::new();
        let id1 = queue.submit(ResourceType::Food, 10, 1, 0);
        let id2 = queue.submit(ResourceType::Energy, 5, 1, 0);
        let id3 = queue.submit(ResourceType::Materials, 3, 2, 0);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(queue.pending_count(), 3);
    }

    #[test]
    fn production_queue_process_tick_drains_pending() {
        let mut queue = ProductionQueue::new();
        queue.submit(ResourceType::Food, 10, 1, 0);
        queue.submit(ResourceType::Food, 20, 2, 0);

        let mut stocks = Stocks::default();
        let results = queue.process_tick(&mut stocks);

        assert_eq!(results.len(), 2);
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(stocks.get(Good::Food), 30);
    }

    #[test]
    fn production_queue_inventory_queries_stock_level() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 42);
        stocks.add(Good::Wood, 17);

        assert_eq!(ProductionQueue::inventory(&stocks, ResourceType::Food), 42);
        assert_eq!(
            ProductionQueue::inventory(&stocks, ResourceType::Energy),
            17
        );
        assert_eq!(
            ProductionQueue::inventory(&stocks, ResourceType::Materials),
            0
        );
    }

    #[test]
    fn production_queue_last_results_returns_previous_tick() {
        let mut queue = ProductionQueue::new();
        let mut stocks = Stocks::default();

        // No orders: empty results.
        let results = queue.process_tick(&mut stocks);
        assert!(results.is_empty());

        // Submit and process.
        queue.submit(ResourceType::Food, 5, 1, 0);
        let results = queue.process_tick(&mut stocks);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].produced_qty, 5);

        // last_results still returns the previous tick's results.
        assert_eq!(queue.last_results().len(), 1);
    }

    #[test]
    fn production_queue_mixed_orders() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 10); // For Energy production
        stocks.add(Good::Wood, 4); // For Materials production
        stocks.add(Good::Metal, 6); // For Technology production

        let mut queue = ProductionQueue::new();
        queue.submit(ResourceType::Food, 10, 1, 0); // Free
        queue.submit(ResourceType::Energy, 3, 1, 0); // Needs 3 Food → produces 3 Wood
        queue.submit(ResourceType::Materials, 2, 2, 0); // Needs 2 Wood → produces 2 Metal

        let results = queue.process_tick(&mut stocks);

        assert_eq!(results.len(), 3);

        // Food: 10 produced, no inputs.
        assert_eq!(results[0].produced_qty, 10);
        assert!(results[0].fully_satisfied);
        assert_eq!(stocks.get(Good::Food), 10 + 10 - 3); // initial 10 + produced 10 - consumed 3 = 17

        // Energy: 3 produced, consumed 3 Food → produces 3 Wood.
        assert_eq!(results[1].produced_qty, 3);
        assert!(results[1].fully_satisfied);
        assert_eq!(stocks.get(Good::Wood), 4 + 3 - 2); // initial 4 + produced 3 - consumed 2 = 5

        // Materials: 2 produced, consumed 2 Wood → produces 2 Metal.
        assert_eq!(results[2].produced_qty, 2);
        assert!(results[2].fully_satisfied);
        assert_eq!(stocks.get(Good::Metal), 6 + 2); // initial 6 + produced 2 = 8
    }

    #[test]
    fn production_queue_insufficient_inputs_reduces_output() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 3); // Only 3 Food available for Energy.

        let mut queue = ProductionQueue::new();
        queue.submit(ResourceType::Energy, 10, 1, 0); // Wants 10, needs 10 Food.

        let results = queue.process_tick(&mut stocks);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].produced_qty, 3); // Only 3 can be produced.
        assert!(!results[0].fully_satisfied);
        assert_eq!(stocks.get(Good::Food), 0);
    }
}
