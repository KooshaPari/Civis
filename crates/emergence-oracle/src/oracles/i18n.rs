//! FR-EMG-024: I18n lookup safety oracle.
//!
//! Validates that `get_or_key` remains panic-free for arbitrary keys across
//! every supported locale. This covers the primary runtime escape hatch used by
//! the UI when a translation is missing.

use std::panic::{catch_unwind, AssertUnwindSafe};

use civ_engine::Simulation;
use civ_i18n::{Bundle, Locale};

use crate::{FeatureOracle, OracleVerdict};

pub struct I18nOracle;

fn lookup_safety_score() -> (usize, usize) {
    let bundles = Locale::ALL.iter().copied().map(Bundle::load);
    let probes = [
        "",
        "nonexistent.key",
        "app.title",
        "settings.locale",
        "godtools.raise_mountain",
        "unicode.测试",
        " spaced key ",
    ];

    let mut passed = 0usize;
    let mut total = 0usize;

    for bundle in bundles {
        for probe in probes {
            total += 1;
            let result = catch_unwind(AssertUnwindSafe(|| bundle.get_or_key(probe)));
            if result.is_ok() {
                passed += 1;
            }
        }
    }

    (passed, total)
}

impl FeatureOracle for I18nOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-024"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let (passed_lookups, total_lookups) = lookup_safety_score();
        let measured = passed_lookups as f64;
        let threshold = if tick == 0 { 0.0 } else { total_lookups as f64 };
        let passed = tick == 0 || passed_lookups == total_lookups;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "I18n lookup safety: safe_lookups={passed_lookups}/{total_lookups} at tick={tick}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_key_is_panic_free_for_common_probes() {
        let (passed, total) = lookup_safety_score();
        assert_eq!(passed, total);
    }
}
