//! TODO(FR): Scale budget and LOD ring planning module stub.

/// Cohort totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CohortTotals;

/// Extent budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtentBudget;

/// Extent error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtentError;

/// Gestalt configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gestalt;

/// LOD ring plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodRingPlan;

/// MVP resident budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MvpResidentBudget;

/// MVP resident configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MvpResidentConfig;

/// Plan error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanError;

/// Ring role identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingRole {
    /// TODO
    Inner,
}

/// Simulation LOD aggregator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimLodAggregator;

/// Stream configuration lite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfigLite;
