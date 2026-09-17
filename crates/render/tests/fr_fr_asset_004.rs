//! FR-ASSET-004 — 3D assets SHALL be stored as glTF 2.0 and loaded lazily on
//! demand.
//!
//! Matrix check: `asset::gltf_lazy_loaded`.

use civ_render::gltf::{GltfAsset, GltfError, GltfLoader, GLTF_VERSION_MAJOR};

fn asset(name: &str, size: u64) -> GltfAsset {
    GltfAsset {
        name: name.to_string(),
        path: format!("assets/models/{name}.gltf"),
        byte_size: size,
    }
}

fn gltf_2_0() -> String {
    "{\"asset\":{\"version\":\"2.0\"},\"meshes\":[]}".to_string()
}

/// Registering an asset costs no work; only `load` resolves it, and it must be
/// glTF 2.0.
#[test]
fn gltf_lazy_loaded() {
    assert_eq!(GLTF_VERSION_MAJOR, 2);

    let mut loader = GltfLoader::new();
    loader.register(asset("tree", 4096), gltf_2_0());
    loader.register(asset("rock", 2048), gltf_2_0());
    loader.register(asset("house", 8192), gltf_2_0());

    // Nothing is resolved at registration time.
    assert_eq!(loader.registered_count(), 3);
    assert!(loader.loaded_names().is_empty(), "no eager loading");
    assert_eq!(loader.loaded_bytes(), 0, "no bytes read eagerly");
    assert!(!loader.is_loaded("tree"));

    // Loading one asset resolves only that asset.
    let mesh = loader.load("tree").expect("tree loads").clone();
    assert_eq!(mesh.version, "2.0");
    assert_eq!(mesh.name, "tree");
    assert!(mesh.triangle_count() > 0);
    assert!(loader.is_loaded("tree"));
    assert!(!loader.is_loaded("rock"), "siblings stay unloaded");
    assert_eq!(loader.loaded_names(), vec!["tree".to_string()]);
    assert_eq!(loader.loaded_bytes(), 4096);

    // Loading again is cached, not re-read.
    let _ = loader.load("tree").expect("cached load");
    assert_eq!(loader.loaded_names().len(), 1);

    // Now resolve the rest on demand.
    loader.load("rock").expect("rock loads");
    loader.load("house").expect("house loads");
    assert_eq!(loader.loaded_bytes(), 4096 + 2048 + 8192);

    // Unknown and non-2.0 documents are rejected.
    assert_eq!(
        loader.load("ghost"),
        Err(GltfError::UnknownAsset { name: "ghost".into() })
    );

    let mut legacy = GltfLoader::new();
    legacy.register(
        asset("old", 100),
        "{\"asset\":{\"version\":\"1.0\"},\"meshes\":[]}".to_string(),
    );
    assert_eq!(
        legacy.load("old"),
        Err(GltfError::UnsupportedVersion { found: "1.0".into() })
    );
    assert!(!legacy.is_loaded("old"), "rejected asset stays unloaded");
}
