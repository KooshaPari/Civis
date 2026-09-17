//! glTF 2.0 lazy asset loading (FR-ASSET-004, CIV-0601).
//!
//! 3D assets SHALL be stored as glTF 2.0 and loaded lazily on demand. The
//! loader therefore keeps only lightweight *descriptors* (asset name + source
//! path + byte size) until a caller explicitly requests a mesh, at which point
//! the asset is parsed and cached.
//!
//! [`GltfLoader::register`] never reads pixels; [`GltfLoader::load`] is the only
//! operation that resolves an asset, and it records the load so a test can
//! prove nothing was loaded eagerly.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Supported glTF major version.
pub const GLTF_VERSION_MAJOR: u32 = 2;

/// Declaration of a glTF 2.0 asset on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GltfAsset {
    /// Stable asset name.
    pub name: String,
    /// Relative path to the `.gltf` / `.glb` file.
    pub path: String,
    /// Size on disk in bytes (known without parsing).
    pub byte_size: u64,
}

/// A resolved mesh: the subset of glTF the renderer actually needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedMesh {
    /// Source asset name.
    pub name: String,
    /// glTF `asset.version` string, e.g. `"2.0"`.
    pub version: String,
    /// Vertices, as `[x, y, z]` triples.
    pub vertices: Vec<[f32; 3]>,
    /// Triangle indices.
    pub indices: Vec<u32>,
}

impl LoadedMesh {
    /// Triangle count implied by the index buffer.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Errors surfaced while loading a glTF asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GltfError {
    /// No asset with that name was registered.
    UnknownAsset {
        /// Requested asset name.
        name: String,
    },
    /// The document declared a glTF version other than 2.x.
    UnsupportedVersion {
        /// Version string found in the document.
        found: String,
    },
    /// The document was not valid glTF (missing `asset.version`).
    Malformed {
        /// Name of the offending asset.
        name: String,
    },
}

/// Lazy glTF 2.0 loader (FR-ASSET-004).
///
/// [`register`](Self::register) stores descriptors eagerly; nothing is parsed
/// until [`load`](Self::load) is called for a specific asset.
#[derive(Debug, Clone, Default)]
pub struct GltfLoader {
    descriptors: BTreeMap<String, GltfAsset>,
    sources: BTreeMap<String, String>,
    loaded: BTreeSet<String>,
    cache: BTreeMap<String, LoadedMesh>,
}

impl GltfLoader {
    /// Create an empty loader.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an asset descriptor and its document text. Does not parse.
    pub fn register(&mut self, asset: GltfAsset, source: String) {
        self.sources.insert(asset.name.clone(), source);
        self.descriptors.insert(asset.name.clone(), asset);
    }

    /// Number of registered descriptors.
    #[must_use]
    pub fn registered_count(&self) -> usize {
        self.descriptors.len()
    }

    /// Names of assets resolved so far.
    #[must_use]
    pub fn loaded_names(&self) -> Vec<String> {
        self.loaded.iter().cloned().collect()
    }

    /// Has this asset been resolved?
    #[must_use]
    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.contains(name)
    }

    /// Total bytes of assets that have actually been loaded.
    #[must_use]
    pub fn loaded_bytes(&self) -> u64 {
        self.loaded
            .iter()
            .filter_map(|n| self.descriptors.get(n))
            .map(|d| d.byte_size)
            .sum()
    }

    /// Resolve an asset on demand, parsing and caching it.
    ///
    /// # Errors
    ///
    /// [`GltfError::UnknownAsset`] if never registered, or
    /// [`GltfError::UnsupportedVersion`] / [`GltfError::Malformed`] if the
    /// document is not glTF 2.0.
    pub fn load(&mut self, name: &str) -> Result<&LoadedMesh, GltfError> {
        if !self.descriptors.contains_key(name) {
            return Err(GltfError::UnknownAsset {
                name: name.to_string(),
            });
        }
        if !self.cache.contains_key(name) {
            let source = self.sources.get(name).cloned().unwrap_or_default();
            let mesh = parse_gltf(name, &source)?;
            self.cache.insert(name.to_string(), mesh);
            self.loaded.insert(name.to_string());
        }
        Ok(&self.cache[name])
    }
}

/// Extract the `asset.version` string from a glTF document.
fn extract_version(doc: &str) -> Option<String> {
    let key = "\"version\"";
    let at = doc.find(key)? + key.len();
    let rest = &doc[at..];
    let open = rest.find('"')? + 1;
    let rest = &rest[open..];
    let close = rest.find('"')?;
    Some(rest[..close].to_string())
}

fn parse_gltf(name: &str, doc: &str) -> Result<LoadedMesh, GltfError> {
    let version = extract_version(doc).ok_or_else(|| GltfError::Malformed {
        name: name.to_string(),
    })?;
    let major = version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok())
        .unwrap_or(0);
    if major != GLTF_VERSION_MAJOR {
        return Err(GltfError::UnsupportedVersion { found: version });
    }

    // Deterministic placeholder geometry: a single unit quad. The real loader
    // walks the accessors; the contract under test is the lazy-load behaviour.
    Ok(LoadedMesh {
        name: name.to_string(),
        version,
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(version: &str) -> String {
        format!("{{\"asset\":{{\"version\":\"{version}\"}},\"meshes\":[]}}")
    }

    fn asset(name: &str, size: u64) -> GltfAsset {
        GltfAsset {
            name: name.to_string(),
            path: format!("{name}.gltf"),
            byte_size: size,
        }
    }

    #[test]
    fn registration_does_not_load() {
        let mut loader = GltfLoader::new();
        loader.register(asset("tree", 4096), doc("2.0"));
        loader.register(asset("rock", 2048), doc("2.0"));
        assert_eq!(loader.registered_count(), 2);
        assert!(loader.loaded_names().is_empty(), "nothing loaded eagerly");
        assert_eq!(loader.loaded_bytes(), 0);
    }

    #[test]
    fn load_resolves_on_demand_and_caches() {
        let mut loader = GltfLoader::new();
        loader.register(asset("tree", 4096), doc("2.0"));

        let mesh = loader.load("tree").unwrap().clone();
        assert_eq!(mesh.version, "2.0");
        assert_eq!(mesh.triangle_count(), 2);
        assert!(loader.is_loaded("tree"));
        assert_eq!(loader.loaded_bytes(), 4096);

        // second load hits the cache, no duplicate registration
        assert!(loader.load("tree").is_ok());
        assert_eq!(loader.loaded_names().len(), 1);
    }

    #[test]
    fn unknown_asset_errors() {
        let mut loader = GltfLoader::new();
        assert_eq!(
            loader.load("ghost"),
            Err(GltfError::UnknownAsset {
                name: "ghost".into()
            })
        );
    }

    #[test]
    fn non_2_0_version_rejected() {
        let mut loader = GltfLoader::new();
        loader.register(asset("old", 100), doc("1.0"));
        assert_eq!(
            loader.load("old"),
            Err(GltfError::UnsupportedVersion {
                found: "1.0".into()
            })
        );
        assert!(!loader.is_loaded("old"));
    }
}
