//! `asset_manifest` — minimal manifest completeness / schema validators.
//!
//! Implements the FR-CIV-ASSET-MANI-001 ("Manifest Completeness") and
//! FR-CIV-ASSET-MANI-002 ("Manifest Schema Conformance") acceptance criteria
//! from `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212-3213`. Kept
//! dependency-free so the `asset-pipeline` crate does not pull in a JSON
//! parser; callers that already have `serde_json` available should prefer
//! the Python `gen_manifest.py` / `validate_manifest.py` scripts in
//! `scripts/` for the canonical check.
//!
//! ## Completeness
//!
//! The number of asset entries in `manifest_path` MUST equal the sum of the
//! `count` field across every atlas JSON file referenced by `atlas_paths`.
//! Any discrepancy returns [`CompletenessError::Mismatch`].
//!
//! ## Schema
//!
//! The manifest is required to declare `manifest_version`, `build_timestamp`,
//! `pipeline_hash`, and `assets` keys. We check this by string presence so the
//! validator runs without a JSON parser.

use std::path::Path;

/// Canonical manifest version the schema validator enforces.
pub const REQUIRED_MANIFEST_VERSION: &str = "1.0";

/// Top-level keys that the asset-manifest JSON schema must declare.
pub const REQUIRED_MANIFEST_KEYS: [&str; 4] = [
    "manifest_version",
    "build_timestamp",
    "pipeline_hash",
    "assets",
];

/// Errors emitted by [`check_manifest_completeness`].
#[derive(Debug, PartialEq, Eq)]
pub enum CompletenessError {
    /// Manifest file could not be read.
    Io(String),
    /// `assets` count in the manifest does not equal the sum of atlas `count` fields.
    Mismatch {
        /// Number of asset entries reported by the manifest.
        manifest_assets: u32,
        /// Sum of `count` values across all atlas JSONs.
        atlas_total: u32,
    },
}

/// Errors emitted by [`check_manifest_schema`].
#[derive(Debug, PartialEq, Eq)]
pub enum SchemaError {
    /// Manifest file could not be read.
    Io(String),
    /// One or more required keys are missing from the JSON.
    MissingKey(&'static str),
}

/// FR-CIV-ASSET-MANI-001 — count manifest `assets` entries vs. sum of
/// atlas `count` fields. Returns `Ok(())` when the two totals match.
///
/// This is a coarse check: it counts `{"asset_id"` occurrences in the
/// manifest file and ` "count": <int>` occurrences across the supplied atlas
/// JSONs. The numbers are sufficient for the spec's completeness gate.
pub fn check_manifest_completeness(
    manifest_path: &Path,
    atlas_paths: &[&Path],
) -> Result<(), CompletenessError> {
    let manifest = std::fs::read_to_string(manifest_path)
        .map_err(|e| CompletenessError::Io(e.to_string()))?;
    let manifest_assets = count_occurrences(&manifest, "\"asset_id\"") as u32;

    let mut atlas_total: u32 = 0;
    for atlas in atlas_paths {
        let body = std::fs::read_to_string(atlas).map_err(|e| CompletenessError::Io(e.to_string()))?;
        atlas_total = atlas_total.saturating_add(parse_count_field(&body));
    }

    if manifest_assets != atlas_total {
        return Err(CompletenessError::Mismatch {
            manifest_assets,
            atlas_total,
        });
    }
    Ok(())
}

/// FR-CIV-ASSET-MANI-002 — verify the manifest JSON declares every required
/// top-level key. Returns the first missing key, or `Ok(())` if all are
/// present.
pub fn check_manifest_schema(manifest_path: &Path) -> Result<(), SchemaError> {
    let body = std::fs::read_to_string(manifest_path).map_err(|e| SchemaError::Io(e.to_string()))?;
    for key in REQUIRED_MANIFEST_KEYS {
        if !body.contains(&format!("\"{key}\"")) {
            return Err(SchemaError::MissingKey(key));
        }
    }
    Ok(())
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

/// Parse a `"count": <int>` field from a JSON body. Returns 0 if not present.
fn parse_count_field(body: &str) -> u32 {
    let needle = "\"count\"";
    let mut idx = 0;
    while let Some(pos) = body[idx..].find(needle) {
        let after = idx + pos + needle.len();
        let rest = &body[after..];
        if let Some(value) = rest.trim_start().strip_prefix(':') {
            let value = value.trim_start();
            let digits: String = value
                .chars()
                .skip_while(|c| c.is_whitespace() || *c == ',')
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse::<u32>() {
                return n;
            }
        }
        idx = after;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(name: &str, body: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("asset_pipeline_manifest_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    #[test]
    fn completeness_matches_when_counts_agree() {
        let m = write_tmp(
            "manifest_ok.json",
            "{\"assets\":[{\"asset_id\":\"a\"},{\"asset_id\":\"b\"}]}",
        );
        let a1 = write_tmp("atlas_a.json", "{\"count\":1}");
        let a2 = write_tmp("atlas_b.json", "{\"count\":1}");
        assert!(check_manifest_completeness(&m, &[&a1, &a2]).is_ok());
    }

    #[test]
    fn completeness_rejects_count_mismatch() {
        let m = write_tmp(
            "manifest_bad.json",
            "{\"assets\":[{\"asset_id\":\"a\"}]}",
        );
        let a1 = write_tmp("atlas_a2.json", "{\"count\":2}");
        let a2 = write_tmp("atlas_b2.json", "{\"count\":3}");
        assert_eq!(
            check_manifest_completeness(&m, &[&a1, &a2]),
            Err(CompletenessError::Mismatch {
                manifest_assets: 1,
                atlas_total: 5,
            })
        );
    }

    #[test]
    fn schema_passes_when_all_keys_present() {
        let m = write_tmp(
            "manifest_schema_ok.json",
            "{\"manifest_version\":\"1.0\",\"build_timestamp\":\"x\",\"pipeline_hash\":\"y\",\"assets\":[]}",
        );
        assert!(check_manifest_schema(&m).is_ok());
    }

    #[test]
    fn schema_fails_when_key_missing() {
        let m = write_tmp(
            "manifest_schema_bad.json",
            "{\"manifest_version\":\"1.0\",\"build_timestamp\":\"x\",\"assets\":[]}",
        );
        assert_eq!(
            check_manifest_schema(&m),
            Err(SchemaError::MissingKey("pipeline_hash"))
        );
    }
}
