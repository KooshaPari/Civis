//! Emergent building/architecture selection for settlements (FR-CIV-architecture).
//!
//! Settlements pick their next building from the **highest unmet need pressure**
//! gated on the locally-available materials in their stockpile. There is no
//! hand-authored city layout: building kinds, candidates, and material
//! preferences all flow from data the settlement has accumulated (needs decay
//! state + the resource ledger). The catalogue of *candidate buildings* is a
//! vocabulary supplied by the modding / SDK layer (see `civlab-sdk::building`);
//! the *selection* here is purely a function of pressure × availability.
//!
//! Determinism is sacred (ADR-008 + always-auto determinism): [`pick_next_building`]
//! is a pure function over its inputs. No RNG, no wall-clock, no environment
//! lookups. Two settlements with identical `(pressures, ledger, catalog, era,
//! weights)` always choose the same building.

use std::collections::BTreeMap;

use civ_build::ParcelKind;
use civ_voxel::MaterialId;
use serde::{Deserialize, Serialize};

use crate::Needs;

/// Schema version for the architecture-emergence module.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Per-`ParcelKind` need pressure derived from a settlement's aggregate
/// [`Needs`] and a [`PressureWeights`] policy vector.
///
/// Pressure is unit-less and lies in `[0, 1]` for every kind. The settlement
/// should next place the building whose pressure is highest and whose material
/// requirements are met by the local ledger.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeedPressure {
    /// Pressure to build residential (shelter / belonging deficit).
    pub residential: f32,
    /// Pressure to build commercial (belonging / food deficit).
    pub commercial: f32,
    /// Pressure to build industrial (food / tools deficit).
    pub industrial: f32,
    /// Pressure to build civic (safety / belonging deficit).
    pub civic: f32,
}

impl NeedPressure {
    /// Compute pressures from an aggregate [`Needs`] vector and per-kind
    /// weights. The result is clamped to `[0, 1]` per kind so that extreme
    /// need spikes never overflow downstream scoring.
    #[must_use]
    pub fn from_needs(needs: &Needs, weights: &PressureWeights) -> Self {
        // Residential is shelter- and belonging-driven; commercial is
        // belonging- and food-driven; industrial is food- and tools-driven;
        // civic is safety- and belonging-driven. The weights object makes the
        // mapping policy tunable without rewriting the schema.
        let residential =
            (needs.shelter * weights.residential_shelter + needs.belonging * weights.residential_belonging)
                .clamp(0.0, 1.0);
        let commercial =
            (needs.belonging * weights.commercial_belonging + needs.food * weights.commercial_food)
                .clamp(0.0, 1.0);
        let industrial =
            (needs.food * weights.industrial_food + needs.safety * weights.industrial_safety)
                .clamp(0.0, 1.0);
        let civic =
            (needs.safety * weights.civic_safety + needs.belonging * weights.civic_belonging)
                .clamp(0.0, 1.0);

        Self {
            residential,
            commercial,
            industrial,
            civic,
        }
    }

    /// Pressure value for `kind`.
    #[must_use]
    pub fn get(&self, kind: ParcelKind) -> f32 {
        match kind {
            ParcelKind::Residential => self.residential,
            ParcelKind::Commercial => self.commercial,
            ParcelKind::Industrial => self.industrial,
            ParcelKind::Civic => self.civic,
        }
    }

    /// The maximum pressure across all four kinds (used as the
    /// `BuildingDecision::urgency`).
    #[must_use]
    pub fn max(&self) -> f32 {
        self.residential
            .max(self.commercial)
            .max(self.industrial)
            .max(self.civic)
    }
}

/// Policy vector mapping each need channel into each `ParcelKind`'s pressure.
/// All weights lie in `[0, 1]`. Default weights mirror the spec's emergent
/// urban grammar (see `docs/design/emergent-systems-spec.md` §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PressureWeights {
    /// `shelter` contribution to residential pressure.
    pub residential_shelter: f32,
    /// `belonging` contribution to residential pressure.
    pub residential_belonging: f32,
    /// `belonging` contribution to commercial pressure.
    pub commercial_belonging: f32,
    /// `food` contribution to commercial pressure.
    pub commercial_food: f32,
    /// `food` contribution to industrial pressure.
    pub industrial_food: f32,
    /// `safety` contribution to industrial pressure.
    pub industrial_safety: f32,
    /// `safety` contribution to civic pressure.
    pub civic_safety: f32,
    /// `belonging` contribution to civic pressure.
    pub civic_belonging: f32,
}

impl Default for PressureWeights {
    fn default() -> Self {
        Self {
            // Residential is overwhelmingly a shelter concern with some
            // belonging (households cluster kin).
            residential_shelter: 0.7,
            residential_belonging: 0.3,
            // Commercial hubs trade in food and provide public meeting space.
            commercial_belonging: 0.5,
            commercial_food: 0.5,
            // Industrial extraction is driven by hunger (farm / mill) and a
            // pinch of safety (workshops inside palisades).
            industrial_food: 0.8,
            industrial_safety: 0.2,
            // Civic is a public-safety and gathering-space concern.
            civic_safety: 0.6,
            civic_belonging: 0.4,
        }
    }
}

/// A candidate building the settlement can construct.
///
/// Candidates are *vocabulary*: they are supplied by the modding / SDK layer
/// (or by a server-side definition file) and indexed by stable id. Selection
/// picks at most one candidate per tick; the candidate's preferred materials
/// are checked against the settlement's [`MaterialLedger`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingCandidate {
    /// Stable candidate id (mirrors `BuildingBlueprint::id` in `civlab-sdk`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Building kind — drives which need-pressure channel gates this candidate.
    pub kind: ParcelKind,
    /// Preferred materials in priority order. The settlement needs at least
    /// one of these in its [`MaterialLedger`] to build it.
    pub preferred_materials: Vec<MaterialId>,
    /// Earliest era this candidate is buildable. Candidates below `era` are
    /// filtered out before scoring.
    pub era_min: u16,
}

/// A settlement's local material stockpile, indexed by [`MaterialId`].
///
/// The ledger is intentionally integer-quantity so callers can drain it on
/// construction; `pick_next_building` only reads it. There is no currency or
/// "balance" — materials are just voxel-grade counts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialLedger {
    quantities: BTreeMap<MaterialId, u32>,
}

impl MaterialLedger {
    /// Empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a ledger from explicit `(material, quantity)` pairs. Negative or
    /// zero quantities are clamped to zero.
    #[must_use]
    pub fn from_pairs<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (MaterialId, u32)>,
    {
        let mut ledger = Self::default();
        for (material, qty) in pairs {
            ledger.quantities.insert(material, qty);
        }
        ledger
    }

    /// Current quantity of `material`. Zero when absent.
    #[must_use]
    pub fn get(&self, material: MaterialId) -> u32 {
        self.quantities.get(&material).copied().unwrap_or(0)
    }

    /// Whether `material` is present in the ledger in any non-zero quantity.
    #[must_use]
    pub fn contains(&self, material: MaterialId) -> bool {
        self.get(material) > 0
    }

    /// The first preferred material from `candidates` that the ledger holds.
    /// Returns `None` when no preferred material is on hand.
    #[must_use]
    pub fn first_available<'a>(
        &self,
        candidates: &'a [MaterialId],
    ) -> Option<&'a MaterialId> {
        candidates.iter().find(|m| self.contains(**m))
    }
}

/// Outcome of one settlement tick of emergent architecture selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingDecision {
    /// Stable candidate id of the chosen building.
    pub candidate_id: String,
    /// Display name of the chosen building.
    pub name: String,
    /// Building kind chosen.
    pub kind: ParcelKind,
    /// Material the settlement will use (must be present in the ledger).
    pub material: MaterialId,
    /// Era this candidate will appear in.
    pub era_min: u16,
    /// Pressure score (0..=1) that gated this choice. Useful for HUDs and
    /// for downstream consumers that want to weight allocation further.
    pub urgency: f32,
}

/// Pick the next building the settlement should construct.
///
/// The selection algorithm is deterministic and side-effect-free:
///
/// 1. Compute `pressures = NeedPressure::from_needs(needs, weights)` from the
///    settlement's aggregate needs vector and the policy weights.
/// 2. Filter `catalog` to candidates whose `era_min <= era` and whose
///    preferred materials include at least one item the ledger holds.
/// 3. Sort the survivors by `(urgency DESC, candidate_id ASC)`.
/// 4. Return the first survivor. If no survivor matches, fall back to the
///    single highest-pressure kind, choosing its lexicographically-first
///    available candidate (still gated on materials). Returns `None` only
///    when the catalog is empty.
///
/// The catalog is iterated in its existing order; [`BuildingCandidate::id`]
/// is used as a tie-break key so two settlements with identical pressure but
/// different catalog orderings pick the same building given identical
/// catalog contents and ordering.
#[must_use]
pub fn pick_next_building(
    needs: &Needs,
    weights: &PressureWeights,
    ledger: &MaterialLedger,
    catalog: &[BuildingCandidate],
    era: u16,
) -> Option<BuildingDecision> {
    if catalog.is_empty() {
        return None;
    }

    let pressures = NeedPressure::from_needs(needs, weights);

    // (1) Filter to buildable candidates (era gate + material availability).
    let buildable: Vec<&BuildingCandidate> = catalog
        .iter()
        .filter(|c| c.era_min <= era)
        .filter(|c| ledger.first_available(&c.preferred_materials).is_some())
        .collect();

    if let Some(chosen) = buildable.into_iter().max_by(|a, b| {
        let pa = pressures.get(a.kind);
        let pb = pressures.get(b.kind);
        // Primary key: pressure DESC. Secondary key: candidate id ASC for
        // determinism when two kinds share the same pressure.
        pb.partial_cmp(&pa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    }) {
        let material = *ledger
            .first_available(&chosen.preferred_materials)
            .expect("buildable candidate must have at least one material");
        return Some(BuildingDecision {
            candidate_id: chosen.id.clone(),
            name: chosen.name.clone(),
            kind: chosen.kind,
            material,
            era_min: chosen.era_min,
            urgency: pressures.get(chosen.kind),
        });
    }

    // (2) Fallback: relax the material gate. Pick the highest-pressure kind's
    // first candidate in catalog order so the settlement still logs *intent*
    // to build something it cannot yet afford — the higher-level economy
    // can use this to dispatch a procurement task.
    let fallback = catalog
        .iter()
        .min_by(|a, b| {
            let pa = pressures.get(a.kind);
            let pb = pressures.get(b.kind);
            pb.partial_cmp(&pa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        })
        .expect("catalog is non-empty");

    let material = fallback
        .preferred_materials
        .first()
        .copied()
        .expect("candidate must declare at least one preferred material");

    Some(BuildingDecision {
        candidate_id: fallback.id.clone(),
        name: fallback.name.clone(),
        kind: fallback.kind,
        material,
        era_min: fallback.era_min,
        urgency: pressures.get(fallback.kind),
    })
}

#[cfg(test)]
mod tests {
    //! Two unit tests covering the FR-CIV-architecture contract:
    //! (1) pressure-driven selection over available materials, and
    //! (2) material availability gating. Both are deterministic pure-function
    //! checks; no RNG, no wall-clock, no I/O.

    use super::*;

    fn wood() -> MaterialId {
        MaterialId(2)
    }
    fn stone() -> MaterialId {
        MaterialId(3)
    }
    fn mud() -> MaterialId {
        MaterialId(1)
    }

    fn sample_catalog() -> Vec<BuildingCandidate> {
        vec![
            BuildingCandidate {
                id: "longhouse".to_owned(),
                name: "Longhouse".to_owned(),
                kind: ParcelKind::Residential,
                preferred_materials: vec![wood()],
                era_min: 0,
            },
            BuildingCandidate {
                id: "cottage".to_owned(),
                name: "Cottage".to_owned(),
                kind: ParcelKind::Residential,
                preferred_materials: vec![mud(), wood()],
                era_min: 0,
            },
            BuildingCandidate {
                id: "market".to_owned(),
                name: "Market".to_owned(),
                kind: ParcelKind::Commercial,
                preferred_materials: vec![wood()],
                era_min: 1,
            },
            BuildingCandidate {
                id: "mill".to_owned(),
                name: "Mill".to_owned(),
                kind: ParcelKind::Industrial,
                preferred_materials: vec![stone(), wood()],
                era_min: 1,
            },
            BuildingCandidate {
                id: "palisade".to_owned(),
                name: "Palisade".to_owned(),
                kind: ParcelKind::Civic,
                preferred_materials: vec![wood()],
                era_min: 0,
            },
        ]
    }

    /// FR-CIV-architecture (test 1 of 2): settlement picks the building whose
    /// need-pressure dominates AND whose required material is in the local
    /// ledger. With shelter need dominant and wood on hand, a Residential
    /// building is chosen; the other kinds (commercial / industrial / civic)
    /// have lower pressure and must NOT be picked even though they are
    /// buildable.
    #[test]
    fn picks_highest_pressure_kind_with_materials_on_hand() {
        let needs = Needs {
            food: 0.2,
            shelter: 0.95, // dominant → residential pressure
            safety: 0.1,
            belonging: 0.2,
        };
        let ledger = MaterialLedger::from_pairs([(wood(), 10), (mud(), 5)]);
        let decision = pick_next_building(
            &needs,
            &PressureWeights::default(),
            &ledger,
            &sample_catalog(),
            /* era */ 1,
        )
        .expect("non-empty catalog must yield a decision");

        assert_eq!(decision.kind, ParcelKind::Residential);
        assert_eq!(decision.material, wood());
        assert!(decision.urgency > 0.5);
        // Both residential candidates are valid; either may win on the
        // candidate-id tie-break, but no non-residential candidate may.
        assert!(
            decision.candidate_id == "longhouse" || decision.candidate_id == "cottage",
            "expected a residential candidate, got {}",
            decision.candidate_id
        );
    }

    /// FR-CIV-architecture (test 2 of 2): a candidate whose preferred
    /// material is NOT in the ledger is filtered out — the settlement must
    /// not pick something it cannot build. With only `mud` on hand and wood
    /// absent, the longhouse (wood-only) is dropped but the cottage
    /// (mud-then-wood) still matches. This is the material-availability
    /// gate that makes emergent architecture *grounded* in local resources.
    #[test]
    fn unavailable_materials_filter_candidates() {
        let needs = Needs {
            food: 0.2,
            shelter: 0.9,
            safety: 0.1,
            belonging: 0.2,
        };
        // Only mud on hand — no wood. Wood-preferring candidates
        // (longhouse, market, mill, palisade) must be filtered out.
        let ledger = MaterialLedger::from_pairs([(mud(), 5)]);
        let decision = pick_next_building(
            &needs,
            &PressureWeights::default(),
            &ledger,
            &sample_catalog(),
            /* era */ 1,
        )
        .expect("cottage (mud-preferring) must still be buildable");

        // Cottage is the only buildable candidate and it wins.
        assert_eq!(decision.candidate_id, "cottage");
        assert_eq!(decision.kind, ParcelKind::Residential);
        assert_eq!(decision.material, mud());
    }
}
