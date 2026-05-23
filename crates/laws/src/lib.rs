//! civ-laws — versioned physics-law database.
//!
//! Pure data + validator. Defines the canonical set of laws (conservation,
//! material properties, era unlock prereqs) plus a typed mechanism for
//! futurism extensions that still expose measurable inputs / outputs /
//! losses / dependencies. The validator is the gate every
//! `civ-research`-proposed tech card must pass before becoming canon
//! (ADR-006).
//!
//! All laws live in RON files so they are mod-friendly out of the box.
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-LAWS-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Schema version for the RON law DB.
pub const SCHEMA_VERSION: u32 = 0;

/// Kinds of law the DB recognises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LawKind {
    /// Hard conservation law (energy, mass, momentum, …).
    Conservation,
    /// Material property (density, tensile strength, conductivity, …).
    Material,
    /// Futurism / fictional-physics extension. Must still expose at least one
    /// non-empty member of `{inputs, outputs, losses}` so the cost model
    /// behaves consistently.
    FictionalExtension,
}

/// One law entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Law {
    /// Stable, unique identifier.
    pub id: String,
    /// What kind of law this is.
    pub kind: LawKind,
    /// Earliest era this law is unlocked at (0 = prehistoric).
    pub era_min: u16,
    /// Required inputs (resource IDs).
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Outputs (resource IDs).
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Byproducts / waste heat / pollutants.
    #[serde(default)]
    pub losses: Vec<String>,
    /// Other law IDs that must be present for this law to apply.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Top-level law database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawDb {
    /// Versioned for hashable replay determinism.
    pub version: u32,
    /// The laws themselves. Order in the file is preserved.
    pub laws: Vec<Law>,
}

/// Errors the validator may report.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// A law references a dependency that does not exist in the DB.
    #[error("law `{law}` references missing dependency `{dep}`")]
    MissingDependency {
        /// The law that has the bad dependency.
        law: String,
        /// The missing dependency ID.
        dep: String,
    },
    /// A `FictionalExtension` law omitted all of `inputs`, `outputs`, `losses`.
    #[error("fictional-extension law `{law}` must declare at least one of inputs/outputs/losses")]
    FictionalExtensionUnderspecified {
        /// The offending law.
        law: String,
    },
    /// Two laws share the same `id`.
    #[error("duplicate law id `{id}`")]
    DuplicateId {
        /// The duplicated ID.
        id: String,
    },
    /// RON parsing failed.
    #[error("RON parse error: {0}")]
    RonParse(String),
}

impl LawDb {
    /// Parse a RON document into a `LawDb`. Does not run validation; call
    /// [`LawDb::validate`] separately.
    pub fn load_ron(s: &str) -> Result<Self, ValidationError> {
        ron::from_str(s).map_err(|e| ValidationError::RonParse(e.to_string()))
    }

    /// Run all validation passes. Returns the full list of errors found.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors: Vec<ValidationError> = Vec::new();

        // 1) Duplicate IDs.
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for law in &self.laws {
            if !seen.insert(law.id.as_str()) {
                errors.push(ValidationError::DuplicateId { id: law.id.clone() });
            }
        }

        // 2) Missing dependencies.
        let known: BTreeSet<&str> = self.laws.iter().map(|l| l.id.as_str()).collect();
        for law in &self.laws {
            for dep in &law.dependencies {
                if !known.contains(dep.as_str()) {
                    errors.push(ValidationError::MissingDependency {
                        law: law.id.clone(),
                        dep: dep.clone(),
                    });
                }
            }
        }

        // 3) Fictional extension underspecification.
        for law in &self.laws {
            if law.kind == LawKind::FictionalExtension
                && law.inputs.is_empty()
                && law.outputs.is_empty()
                && law.losses.is_empty()
            {
                errors.push(ValidationError::FictionalExtensionUnderspecified {
                    law: law.id.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Look up a law by id. Linear scan — fine for the law-DB scale we expect.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Law> {
        self.laws.iter().find(|l| l.id == id)
    }

    /// Laws unlocked at era `era` or earlier.
    pub fn unlocked_at_era(&self, era: u16) -> impl Iterator<Item = &Law> {
        self.laws.iter().filter(move |l| l.era_min <= era)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ron() -> &'static str {
        r#"(
            version: 0,
            laws: [
                (
                    id: "mass_conservation",
                    kind: Conservation,
                    era_min: 0,
                    inputs: [],
                    outputs: [],
                    losses: [],
                    dependencies: [],
                ),
                (
                    id: "steel",
                    kind: Material,
                    era_min: 4,
                    inputs: ["iron_ore", "coal"],
                    outputs: ["steel_ingot"],
                    losses: ["slag"],
                    dependencies: ["mass_conservation"],
                ),
                (
                    id: "fusion_power",
                    kind: FictionalExtension,
                    era_min: 9,
                    inputs: ["deuterium"],
                    outputs: ["energy"],
                    losses: ["helium_4"],
                    dependencies: ["mass_conservation"],
                ),
            ],
        )"#
    }

    /// FR-CIV-LAWS-001 — versioned RON schema round-trips.
    #[test]
    fn ron_roundtrips() {
        let db = LawDb::load_ron(sample_ron()).expect("parse");
        assert_eq!(db.version, 0);
        assert_eq!(db.laws.len(), 3);
        let s = ron::to_string(&db).expect("serialize");
        let back = LawDb::load_ron(&s).expect("reparse");
        assert_eq!(db, back);
    }

    /// FR-CIV-LAWS-002 — validator rejects fictional extensions with no
    /// inputs/outputs/losses.
    #[test]
    fn validator_rejects_underspecified_fictional() {
        let db = LawDb {
            version: 0,
            laws: vec![Law {
                id: "void_drive".into(),
                kind: LawKind::FictionalExtension,
                era_min: 10,
                inputs: vec![],
                outputs: vec![],
                losses: vec![],
                dependencies: vec![],
            }],
        };
        let errs = db.validate().unwrap_err();
        assert!(matches!(
            errs[0],
            ValidationError::FictionalExtensionUnderspecified { .. }
        ));
    }

    /// FR-CIV-LAWS-003 — missing-dependency detection.
    #[test]
    fn validator_detects_missing_dependency() {
        let db = LawDb {
            version: 0,
            laws: vec![Law {
                id: "steel".into(),
                kind: LawKind::Material,
                era_min: 4,
                inputs: vec!["iron_ore".into()],
                outputs: vec!["steel_ingot".into()],
                losses: vec![],
                dependencies: vec!["mass_conservation".into()],
            }],
        };
        let errs = db.validate().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| matches!(e, ValidationError::MissingDependency { .. })));
    }

    /// FR-CIV-LAWS-004 — duplicate-id detection.
    #[test]
    fn validator_detects_duplicate_id() {
        let dup = Law {
            id: "x".into(),
            kind: LawKind::Conservation,
            era_min: 0,
            inputs: vec![],
            outputs: vec![],
            losses: vec![],
            dependencies: vec![],
        };
        let db = LawDb {
            version: 0,
            laws: vec![dup.clone(), dup],
        };
        let errs = db.validate().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateId { .. })));
    }

    /// FR-CIV-LAWS-005 — era filter only returns unlocked laws.
    #[test]
    fn unlocked_at_era_filters_correctly() {
        let db = LawDb::load_ron(sample_ron()).expect("parse");
        let early: Vec<_> = db.unlocked_at_era(3).map(|l| l.id.as_str()).collect();
        assert_eq!(early, vec!["mass_conservation"]);
        let modern: Vec<_> = db.unlocked_at_era(5).map(|l| l.id.as_str()).collect();
        assert_eq!(modern, vec!["mass_conservation", "steel"]);
    }
}
