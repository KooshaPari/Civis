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

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod enforcement;
pub use enforcement::{LawEnforcement, LawPenalty};

/// Schema version for the RON law DB.
pub const SCHEMA_VERSION: u32 = 0;

/// Embedded canonical law database shipped with the game (mod-friendly RON source).
pub const DEFAULT_LAW_RON: &str = include_str!("../laws/default.ron");

/// Filename for per-mod law overlays inside each mod directory.
pub const MOD_LAW_FILENAME: &str = "laws.ron";

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
    /// Dependency graph contains a cycle.
    #[error("cyclic dependency detected among laws")]
    CyclicDependency,
    /// I/O error reading a law file from disk.
    #[error("failed to read law file: {0}")]
    Io(String),
}

impl LawDb {
    /// Parse a RON document into a `LawDb`. Does not run validation; call
    /// [`LawDb::validate`] separately.
    pub fn load_ron(s: &str) -> Result<Self, ValidationError> {
        ron::from_str(s).map_err(|e| ValidationError::RonParse(e.to_string()))
    }

    /// Load a RON law database from a filesystem path (mods / data dir).
    pub fn load_path(path: &Path) -> Result<Self, ValidationError> {
        let s = std::fs::read_to_string(path).map_err(|e| ValidationError::Io(e.to_string()))?;
        Self::load_ron(&s)
    }

    /// Parse and validate the embedded [`DEFAULT_LAW_RON`] canon database.
    pub fn default_canon() -> Result<Self, ValidationError> {
        let db = Self::load_ron(DEFAULT_LAW_RON)?;
        db.validate().map_err(|mut errs| errs.remove(0))?;
        Ok(db)
    }

    /// Load embedded canon, then merge validated `mods/*/laws.ron` overlays (if present).
    pub fn load_with_mod_overlays(mods_dir: &Path) -> Result<Self, ValidationError> {
        let mut db = Self::default_canon()?;
        let entries = match std::fs::read_dir(mods_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(db),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let law_path = path.join(MOD_LAW_FILENAME);
            if !law_path.is_file() {
                continue;
            }
            let overlay = Self::load_path(&law_path)?;
            db = db.merge_overlay(overlay);
        }
        db.validate().map_err(|mut errs| errs.remove(0))?;
        Ok(db)
    }

    /// Overlay `other` onto `self`: laws with the same `id` are replaced; new ids append.
    pub fn merge_overlay(mut self, other: Self) -> Self {
        for law in other.laws {
            if let Some(existing) = self.laws.iter_mut().find(|entry| entry.id == law.id) {
                *existing = law;
            } else {
                self.laws.push(law);
            }
        }
        self
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

    /// Laws whose `era_min` is at or before `era` (ignores dependency closure).
    pub fn unlocked_at_era(&self, era: u16) -> impl Iterator<Item = &Law> {
        self.laws.iter().filter(move |l| l.era_min <= era)
    }

    /// Laws unlockable at `era`: satisfies `era_min` and all dependencies are also unlockable.
    pub fn unlockable_at_era(&self, era: u16) -> Vec<&Law> {
        let mut unlocked_ids: BTreeSet<String> = BTreeSet::new();
        loop {
            let mut changed = false;
            for law in &self.laws {
                if unlocked_ids.contains(&law.id) || law.era_min > era {
                    continue;
                }
                if law
                    .dependencies
                    .iter()
                    .all(|dep| unlocked_ids.contains(dep))
                {
                    unlocked_ids.insert(law.id.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        self.laws
            .iter()
            .filter(|law| unlocked_ids.contains(&law.id))
            .collect()
    }

    /// Topological dependency order (file order tie-break). Errors on cycles.
    pub fn dependency_order(&self) -> Result<Vec<&str>, ValidationError> {
        let mut order = Vec::with_capacity(self.laws.len());
        let mut placed: HashSet<&str> = HashSet::new();
        loop {
            let mut progressed = false;
            for law in &self.laws {
                if placed.contains(law.id.as_str()) {
                    continue;
                }
                if law
                    .dependencies
                    .iter()
                    .all(|dep| placed.contains(dep.as_str()))
                {
                    order.push(law.id.as_str());
                    placed.insert(law.id.as_str());
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        if order.len() != self.laws.len() {
            return Err(ValidationError::CyclicDependency);
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers FR-CIV-LAWS-000.
    /// FR-CIV-LAWS-000 — crate compiles and exposes a schema version.
    #[test]
    fn schema_version_stub() {
        assert_eq!(SCHEMA_VERSION, 0);
    }

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

    /// Covers FR-CIV-LAWS-001.
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

    /// Covers FR-CIV-LAWS-002.
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

    /// FR-CIV-LAWS-006 — `LawDb::get` returns the correct law by id, and
    /// returns `None` for ids that do not exist.
    #[test]
    fn get_finds_existing_law_and_returns_none_for_missing() {
        let db = LawDb::load_ron(sample_ron()).expect("parse");

        // Existing law: "mass_conservation"
        let law = db.get("mass_conservation");
        assert!(law.is_some(), "expected 'mass_conservation' to be found");
        let law = law.unwrap();
        assert_eq!(law.id, "mass_conservation");
        assert_eq!(law.kind, LawKind::Conservation);
        assert_eq!(law.era_min, 0);

        // Existing law: "steel"
        let steel = db.get("steel");
        assert!(steel.is_some(), "expected 'steel' to be found");
        assert_eq!(steel.unwrap().id, "steel");

        // Missing law
        assert!(
            db.get("nonexistent").is_none(),
            "expected None for missing id"
        );
    }

    /// FR-CIV-LAWS-006 — `LawDb::get` returns the correct law when multiple
    /// laws are present, verifying linear scan does not return the first item
    /// unconditionally.
    #[test]
    fn get_returns_correct_law_not_first_item() {
        let db = LawDb::load_ron(sample_ron()).expect("parse");

        let first = db.get("mass_conservation").unwrap();
        assert_eq!(first.id, "mass_conservation");

        let last = db.get("fusion_power").unwrap();
        assert_eq!(last.id, "fusion_power");
        assert_eq!(last.kind, LawKind::FictionalExtension);
        assert_eq!(last.era_min, 9);
    }

    /// FR-CIV-LAWS-006 — `LawDb::get` on an empty database always returns `None`.
    #[test]
    fn get_on_empty_db_returns_none() {
        let db = LawDb {
            version: 0,
            laws: vec![],
        };
        assert!(db.get("anything").is_none());
    }

    /// FR-CIV-LAWS-006 — dependency-aware era unlock excludes laws waiting on prereqs.
    #[test]
    fn unlockable_at_era_respects_dependencies() {
        let db = LawDb {
            version: 0,
            laws: vec![
                Law {
                    id: "base".into(),
                    kind: LawKind::Conservation,
                    era_min: 0,
                    inputs: vec![],
                    outputs: vec![],
                    losses: vec![],
                    dependencies: vec![],
                },
                Law {
                    id: "early_child".into(),
                    kind: LawKind::Material,
                    era_min: 0,
                    inputs: vec![],
                    outputs: vec![],
                    losses: vec![],
                    dependencies: vec!["late_parent".into()],
                },
                Law {
                    id: "late_parent".into(),
                    kind: LawKind::Conservation,
                    era_min: 5,
                    inputs: vec![],
                    outputs: vec![],
                    losses: vec![],
                    dependencies: vec![],
                },
            ],
        };
        let era_3: Vec<_> = db
            .unlockable_at_era(3)
            .into_iter()
            .map(|law| law.id.as_str())
            .collect();
        assert_eq!(era_3, vec!["base"]);
        let era_5: Vec<_> = db
            .unlockable_at_era(5)
            .into_iter()
            .map(|law| law.id.as_str())
            .collect();
        assert_eq!(era_5, vec!["base", "early_child", "late_parent"]);
    }

    /// FR-CIV-LAWS-006 — era unlock graph returns dependency-respecting order.
    #[test]
    fn dependency_order_respects_prereqs() {
        let db = LawDb::load_ron(sample_ron()).expect("parse");
        let order = db.dependency_order().expect("acyclic");
        assert_eq!(order, vec!["mass_conservation", "steel", "fusion_power"]);
    }

    /// FR-CIV-LAWS-007 — mod overlay merge replaces laws by id.
    #[test]
    fn merge_overlay_replaces_by_id() {
        let base = LawDb::load_ron(sample_ron()).expect("parse");
        let overlay = LawDb {
            version: 0,
            laws: vec![Law {
                id: "steel".into(),
                kind: LawKind::Material,
                era_min: 6,
                inputs: vec!["iron_ore".into()],
                outputs: vec!["steel_ingot".into()],
                losses: vec![],
                dependencies: vec!["mass_conservation".into()],
            }],
        };
        let merged = base.merge_overlay(overlay);
        assert_eq!(merged.get("steel").expect("steel").era_min, 6);
        assert_eq!(merged.laws.len(), 3);
    }

    /// FR-CIV-LAWS-008 — embedded default RON loads and validates.
    #[test]
    fn default_canon_loads_embedded_ron() {
        let db = LawDb::default_canon().expect("default canon");
        assert_eq!(db.laws.len(), 3);
        assert!(db.get("mass_conservation").is_some());
    }

    /// FR-CIV-LAWS-009 — mod directory loader merges child `laws.ron` overlays.
    #[test]
    fn load_with_mod_overlays_merges_child_laws_ron() {
        let temp =
            std::env::temp_dir().join(format!("civ-laws-mod-overlay-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(temp.join("example-mod")).expect("mod dir");
        std::fs::write(
            temp.join("example-mod").join(MOD_LAW_FILENAME),
            r#"(
                version: 0,
                laws: [
                    (
                        id: "policy_audit_trail",
                        kind: Conservation,
                        era_min: 2,
                        inputs: [],
                        outputs: [],
                        losses: [],
                        dependencies: ["mass_conservation"],
                    ),
                ],
            )"#,
        )
        .expect("write overlay");

        let db = LawDb::load_with_mod_overlays(&temp).expect("load with overlays");
        assert_eq!(db.laws.len(), 4);
        assert!(db.get("policy_audit_trail").is_some());
        let _ = std::fs::remove_dir_all(&temp);
    }
}
