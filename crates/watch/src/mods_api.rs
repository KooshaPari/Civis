//! Mod catalog, upload, publish, fetch, and remote registry APIs.

use std::path::{Path, PathBuf};

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use base64::Engine as _;
use civ_engine::{load_manifest, ModType};
use civ_mod_host::{read_civmod_archive, read_manifest_from_civmod, CIVMOD_MANIFEST_NAME};
use sha2::{Digest, Sha256};

use crate::app::{
    AppState, ControlOk, FetchModReq, FetchModResponse, InstallModReq, ModCatalogEntry,
    PublishModReq, PublishModResponse, PublishedModEntry, ReloadModReq, RemoteModEntry,
    RemoteModMeta, RemoteModRegistry, RemoteModRegistryEntry, UnloadModReq, UploadModReq,
    UploadModResponse, REMOTE_MOD_ARCHIVE_NAME, REMOTE_MOD_MAX_BYTES,
    REMOTE_MOD_META_NAME, REMOTE_REGISTRY_NAME,
};

pub(crate) fn load_remote_mod_registry(mods_dir: &Path) -> RemoteModRegistry {
    let path = mods_dir.join(REMOTE_REGISTRY_NAME);
    let Ok(contents) = std::fs::read_to_string(path) else {
        return RemoteModRegistry::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub(crate) fn match_registry_entry<'a>(
    registry: &'a RemoteModRegistry,
    url: &str,
) -> Option<&'a RemoteModRegistryEntry> {
    let trimmed = url.trim();
    registry
        .entries
        .iter()
        .find(|entry| trimmed.starts_with(entry.url_prefix.trim()))
}

pub(crate) fn validate_remote_fetch_against_registry<'a>(
    registry: &'a RemoteModRegistry,
    url: &str,
    mod_id: Option<&str>,
) -> Result<Option<&'a RemoteModRegistryEntry>, String> {
    let matched = match_registry_entry(registry, url);
    if registry.require_registry && matched.is_none() {
        return Err(format!("url not in signed remote mod registry: {}", url.trim()));
    }
    if let (Some(entry), Some(requested_id)) = (matched, mod_id) {
        if let Some(expected) = entry.mod_id.as_deref() {
            if expected != requested_id {
                return Err(format!(
                    "mod_id {requested_id} does not match registry entry ({expected})"
                ));
            }
        }
    }
    Ok(matched)
}

pub(crate) fn validate_remote_mod_against_registry(
    entry: Option<&RemoteModRegistryEntry>,
    manifest: &civ_mod_host::ModManifest,
) -> Result<(), String> {
    let Some(entry) = entry else {
        return Ok(());
    };
    if let Some(expected) = entry.mod_id.as_deref() {
        if manifest.meta.id != expected {
            return Err(format!(
                "archive mod id `{}` does not match registry (`{expected}`)",
                manifest.meta.id
            ));
        }
    }
    if entry.require_signature && manifest.meta.author_pubkey_hex.is_none() {
        return Err("remote mod must be signed (author_pubkey_hex missing)".into());
    }
    if let Some(pk) = manifest.meta.author_pubkey_hex.as_deref() {
        if !entry.allowed_pubkeys.is_empty()
            && !entry
                .allowed_pubkeys
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(pk))
        {
            return Err("author pubkey not in registry allowlist".into());
        }
    }
    Ok(())
}


pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn mod_type_label(kind: ModType) -> &'static str {
    match kind {
        ModType::Policy => "policy",
        ModType::Economic => "economic",
        ModType::Event => "event",
        ModType::Scenario => "scenario",
    }
}

pub(crate) fn catalog_entry_from_manifest(
    source: String,
    kind: &str,
    manifest: &civ_mod_host::ModManifest,
    installed_ids: &std::collections::HashSet<String>,
    signed: bool,
    author_pubkey_hex: Option<String>,
) -> ModCatalogEntry {
    ModCatalogEntry {
        source,
        id: manifest.meta.id.clone(),
        name: manifest.meta.name.clone(),
        version: manifest.meta.version.clone(),
        mod_type: mod_type_label(manifest.meta.mod_type).to_owned(),
        kind: kind.to_owned(),
        installed: installed_ids.contains(&manifest.meta.id),
        signed,
        author_pubkey_hex,
    }
}

pub(crate) fn civmod_catalog_entries(
    repo: &Path,
    dir: &Path,
    installed_ids: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) -> Vec<ModCatalogEntry> {
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("civmod") {
            continue;
        }
        let source = path
            .strip_prefix(repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = read_manifest_from_civmod(&path) {
            entries.push(catalog_entry_from_manifest(
                source,
                "civmod",
                &manifest,
                installed_ids,
                false,
                None,
            ));
        }
    }
    entries
}

pub(crate) fn scan_mod_catalog(mods_dir: &Path, installed_ids: &std::collections::HashSet<String>) -> Vec<ModCatalogEntry> {
    let repo = repo_root();
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    entries.extend(civmod_catalog_entries(
        &repo,
        mods_dir,
        installed_ids,
        &mut seen,
    ));
    entries.extend(civmod_catalog_entries(
        &repo,
        &mods_dir.join("uploads"),
        installed_ids,
        &mut seen,
    ));
    entries.extend(civmod_catalog_entries(
        &repo,
        &mods_dir.join("publish"),
        installed_ids,
        &mut seen,
    ));
    entries.extend(remote_civmod_catalog_entries(
        &repo,
        mods_dir,
        installed_ids,
        &mut seen,
    ));

    for name in ["example-policy", "example-economic"] {
        let dir = mods_dir.join(name);
        let manifest_path = dir.join(CIVMOD_MANIFEST_NAME);
        if !manifest_path.is_file() {
            continue;
        }
        let source = format!("mods/{name}");
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = load_manifest(&manifest_path) {
            entries.push(catalog_entry_from_manifest(
                source,
                "dir",
                &manifest,
                installed_ids,
                false,
                None,
            ));
        }
    }

    entries.sort_by(|a, b| a.source.cmp(&b.source));
    entries
}

pub(crate) fn resolve_install_source(source: &str, mods_dir: &Path) -> Result<String, String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err("source cannot be empty".into());
    }
    if trimmed.contains("..") {
        return Err("source must not contain '..'".into());
    }

    let normalized = trimmed.replace('\\', "/");
    if normalized.starts_with("mods/") {
        let path = repo_root().join(&normalized);
        if path.is_file() || path.is_dir() {
            return Ok(normalized);
        }
        return Err(format!("mod source not found: {normalized}"));
    }

    if normalized.ends_with(".civmod") {
        let path = mods_dir.join(
            normalized
                .trim_start_matches("mods/")
                .trim_start_matches('/'),
        );
        if path.is_file() {
            return Ok(format!("mods/{}", path.file_name().unwrap().to_string_lossy()));
        }
        return Err(format!("mod archive not found: {normalized}"));
    }

    let dir_name = normalized.trim_start_matches("mods/").trim_start_matches('/');
    let dir = mods_dir.join(dir_name);
    if dir.is_dir() {
        return Ok(format!("mods/{dir_name}"));
    }

    Err(format!("unknown mod source: {trimmed}"))
}

pub(crate) fn sanitize_mod_filename(filename: &str) -> Result<String, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("filename cannot be empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("filename must be a simple name".into());
    }
    let base = trimmed.trim_end_matches(".civmod");
    if base.is_empty() {
        return Err("filename must have a name".into());
    }
    Ok(base.to_string())
}

pub(crate) fn mod_source_relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string())
}

pub(crate) fn resolve_repo_mod_path(source: &str) -> Result<PathBuf, String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err("source cannot be empty".into());
    }
    if trimmed.contains("..") {
        return Err("source must not contain '..'".into());
    }
    let normalized = trimmed.replace('\\', "/");
    if !normalized.starts_with("mods/") {
        return Err("source must be under mods/".into());
    }
    let path = repo_root().join(&normalized);
    if !path.is_file() {
        return Err(format!("mod source not found: {normalized}"));
    }
    Ok(path)
}

pub(crate) fn scan_published_mods(mods_dir: &Path) -> Vec<PublishedModEntry> {
    let publish_dir = mods_dir.join("publish");
    let repo = repo_root();
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(&publish_dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("civmod") {
            continue;
        }
        let source = path
            .strip_prefix(&repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        if let Ok(manifest) = read_manifest_from_civmod(&path) {
            entries.push(PublishedModEntry {
                id: manifest.meta.id.clone(),
                name: manifest.meta.name.clone(),
                version: manifest.meta.version.clone(),
                source,
            });
        }
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

pub(crate) fn validate_remote_fetch_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("url cannot be empty".into());
    }
    let parsed = reqwest::Url::parse(trimmed).map_err(|err| format!("invalid url: {err}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(format!("unsupported url scheme: {scheme}")),
    }
}

pub(crate) fn sanitize_remote_mod_id(mod_id: &str) -> Result<String, String> {
    let trimmed = mod_id.trim();
    if trimmed.is_empty() {
        return Err("mod_id cannot be empty".into());
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("mod_id must not contain path separators or '..'".into());
    }
    let valid_id = trimmed.as_bytes().first().is_some_and(|b| b.is_ascii_lowercase())
        && trimmed.len() <= 64
        && trimmed
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !valid_id {
        return Err(format!(
            "mod_id `{trimmed}` must match [a-z][a-z0-9-]{{0,63}}"
        ));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn url_hash_cache_id(url: &str) -> String {
    let digest = Sha256::digest(url.trim().as_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn resolve_remote_cache_id(url: &str, mod_id: Option<&str>) -> Result<String, String> {
    validate_remote_fetch_url(url)?;
    if let Some(id) = mod_id {
        return sanitize_remote_mod_id(id);
    }
    Ok(format!("url-{}", url_hash_cache_id(url)))
}

pub(crate) fn remote_mod_cache_dir(mods_dir: &Path, cache_id: &str) -> PathBuf {
    mods_dir.join("remote").join(cache_id)
}

pub(crate) fn remote_mod_source_path(repo: &Path, cache_dir: &Path) -> String {
    cache_dir
        .join(REMOTE_MOD_ARCHIVE_NAME)
        .strip_prefix(repo)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            format!(
                "mods/remote/{}/{}",
                cache_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
                REMOTE_MOD_ARCHIVE_NAME
            )
        })
}

pub(crate) fn is_zip_payload(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes.starts_with(b"PK\x03\x04")
}

pub(crate) fn format_remote_mod_validation_error(err: civ_mod_host::ManifestError) -> String {
    let msg = err.to_string();
    if msg.contains("signature") || msg.contains("mod.wasm.sig") || msg.contains("author_pubkey_hex") {
        format!("civmod signature verification failed: {msg}")
    } else {
        format!("invalid civmod archive: {msg}")
    }
}

pub(crate) fn validate_remote_mod_bytes(
    bytes: &[u8],
    scratch_path: &Path,
    registry_entry: Option<&RemoteModRegistryEntry>,
) -> Result<(civ_mod_host::ModManifest, bool), String> {
    if bytes.is_empty() {
        return Err("downloaded payload is empty".into());
    }
    if bytes.len() > REMOTE_MOD_MAX_BYTES {
        return Err(format!(
            "downloaded payload exceeds {} byte limit",
            REMOTE_MOD_MAX_BYTES
        ));
    }
    if !is_zip_payload(bytes) {
        return Err("downloaded payload is not a zip/civmod archive".into());
    }
    if let Some(parent) = scratch_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(scratch_path, bytes).map_err(|err| err.to_string())?;
    match read_civmod_archive(scratch_path) {
        Ok((manifest, wasm)) => {
            let signed =
                manifest.meta.author_pubkey_hex.is_some() && wasm.is_some();
            match validate_remote_mod_against_registry(registry_entry, &manifest) {
                Ok(()) => Ok((manifest, signed)),
                Err(err) => {
                    let _ = std::fs::remove_file(scratch_path);
                    Err(err)
                }
            }
        }
        Err(err) => {
            let _ = std::fs::remove_file(scratch_path);
            Err(format_remote_mod_validation_error(err))
        }
    }
}

pub(crate) fn write_remote_mod_meta(cache_dir: &Path, meta: &RemoteModMeta) -> Result<(), String> {
    std::fs::create_dir_all(cache_dir).map_err(|err| err.to_string())?;
    let json = serde_json::to_string_pretty(meta).map_err(|err| err.to_string())?;
    std::fs::write(cache_dir.join(REMOTE_MOD_META_NAME), json).map_err(|err| err.to_string())
}

pub(crate) fn read_remote_mod_meta(cache_dir: &Path) -> Option<RemoteModMeta> {
    let path = cache_dir.join(REMOTE_MOD_META_NAME);
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

pub(crate) fn unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn persist_remote_mod_cache(
    mods_dir: &Path,
    cache_id: &str,
    url: &str,
    bytes: &[u8],
    registry_entry: Option<&RemoteModRegistryEntry>,
) -> Result<(PathBuf, String), String> {
    let cache_dir = remote_mod_cache_dir(mods_dir, cache_id);
    let archive_path = cache_dir.join(REMOTE_MOD_ARCHIVE_NAME);
    let (manifest, signed) = validate_remote_mod_bytes(bytes, &archive_path, registry_entry)?;
    let author_pubkey_hex = manifest.meta.author_pubkey_hex.clone();
    let meta = RemoteModMeta {
        id: cache_id.to_owned(),
        url: url.trim().to_owned(),
        fetched_at: unix_timestamp_secs(),
        signed,
        author_pubkey_hex,
    };
    write_remote_mod_meta(&cache_dir, &meta)?;
    let source = remote_mod_source_path(&repo_root(), &cache_dir);
    Ok((archive_path, source))
}

pub(crate) async fn download_remote_mod_payload(
    http: &reqwest::Client,
    url: &str,
) -> Result<Vec<u8>, String> {
    validate_remote_fetch_url(url)?;
    let response = http
        .get(url.trim())
        .send()
        .await
        .map_err(|err| format!("fetch request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!("fetch returned HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("fetch read failed: {err}"))?;
    if bytes.len() > REMOTE_MOD_MAX_BYTES {
        return Err(format!(
            "downloaded payload exceeds {} byte limit",
            REMOTE_MOD_MAX_BYTES
        ));
    }
    Ok(bytes.to_vec())
}

pub(crate) fn remote_civmod_catalog_entries(
    repo: &Path,
    mods_dir: &Path,
    installed_ids: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) -> Vec<ModCatalogEntry> {
    let remote_root = mods_dir.join("remote");
    let Ok(read) = std::fs::read_dir(&remote_root) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for entry in read.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let archive = dir.join(REMOTE_MOD_ARCHIVE_NAME);
        if !archive.is_file() {
            continue;
        }
        let source = archive
            .strip_prefix(repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| archive.display().to_string());
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = read_manifest_from_civmod(&archive) {
            let cache_meta = read_remote_mod_meta(&dir);
            let signed = cache_meta.as_ref().map(|m| m.signed).unwrap_or(false);
            let author_pubkey_hex = cache_meta.and_then(|m| m.author_pubkey_hex);
            entries.push(catalog_entry_from_manifest(
                source,
                "civmod",
                &manifest,
                installed_ids,
                signed,
                author_pubkey_hex,
            ));
        }
    }
    entries
}

pub(crate) fn scan_remote_mod_cache(mods_dir: &Path) -> Vec<RemoteModEntry> {
    let remote_root = mods_dir.join("remote");
    let repo = repo_root();
    let Ok(read) = std::fs::read_dir(&remote_root) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for entry in read.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let archive = dir.join(REMOTE_MOD_ARCHIVE_NAME);
        if !archive.is_file() {
            continue;
        }
        let meta = read_remote_mod_meta(&dir).unwrap_or_else(|| RemoteModMeta {
            id: dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_owned(),
            url: String::new(),
            fetched_at: archive
                .metadata()
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            signed: false,
            author_pubkey_hex: None,
        });
        let path = archive
            .strip_prefix(&repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| archive.display().to_string());
        entries.push(RemoteModEntry {
            id: meta.id,
            path,
            fetched_at: meta.fetched_at,
            url: meta.url,
            signed: meta.signed,
            author_pubkey_hex: meta.author_pubkey_hex,
        });
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

pub(crate) async fn upload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<UploadModReq>,
) -> Result<Json<UploadModResponse>, (StatusCode, Json<ControlOk>)> {
    let name = sanitize_mod_filename(&req.filename).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.trim())
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(ControlOk {
                    ok: false,
                    message: Some(format!("invalid base64: {err}")),
                }),
            )
        })?;
    if bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some("upload data cannot be empty".into()),
            }),
        ));
    }

    let uploads_dir = state.mods_dir.join("uploads");
    std::fs::create_dir_all(&uploads_dir).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let path = uploads_dir.join(format!("{name}.civmod"));
    std::fs::write(&path, &bytes).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;

    read_manifest_from_civmod(&path).map_err(|err| {
        let _ = std::fs::remove_file(&path);
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(format!("invalid civmod archive: {err}")),
            }),
        )
    })?;

    Ok(Json(UploadModResponse {
        ok: true,
        source: mod_source_relative(&path),
    }))
}

pub(crate) async fn publish_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<PublishModReq>,
) -> Result<Json<PublishModResponse>, (StatusCode, Json<ControlOk>)> {
    let source_path = resolve_repo_mod_path(&req.source).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let manifest = read_manifest_from_civmod(&source_path).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(format!("invalid civmod archive: {err}")),
            }),
        )
    })?;
    let id = manifest.meta.id.trim();
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some("manifest mod id must be a simple name".into()),
            }),
        ));
    }

    let publish_dir = state.mods_dir.join("publish");
    std::fs::create_dir_all(&publish_dir).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let dest = publish_dir.join(format!("{id}.civmod"));
    std::fs::copy(&source_path, &dest).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;

    Ok(Json(PublishModResponse {
        ok: true,
        published_source: mod_source_relative(&dest),
    }))
}

pub(crate) async fn list_published_mods_handler(
    State(state): State<AppState>,
) -> Json<Vec<PublishedModEntry>> {
    Json(scan_published_mods(state.mods_dir.as_ref()))
}

pub(crate) async fn list_mod_catalog_handler(
    State(state): State<AppState>,
) -> Json<Vec<ModCatalogEntry>> {
    let sim = state.sim.lock().await;
    let installed: std::collections::HashSet<String> = sim
        .mod_browser_entries()
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    Json(scan_mod_catalog(state.mods_dir.as_ref(), &installed))
}

pub(crate) async fn install_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<InstallModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let rel = resolve_install_source(&req.source, state.mods_dir.as_ref()).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let mut sim = state.sim.lock().await;
    let record = sim.install_mod_path(&rel).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("installed {} ({})", record.mod_name, record.mod_id)),
    }))
}

pub(crate) async fn unload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<UnloadModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let mut sim = state.sim.lock().await;
    let record = sim.unload_mod_by_id(&req.mod_id, "user_request").map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("unloaded {} ({})", record.mod_name, record.mod_id)),
    }))
}

pub(crate) async fn reload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<ReloadModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let mut sim = state.sim.lock().await;
    let record = sim.reload_mod_by_id(&req.mod_id).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("reloaded {} ({})", record.mod_name, record.mod_id)),
    }))
}

pub(crate) async fn fetch_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<FetchModReq>,
) -> Result<Json<FetchModResponse>, (StatusCode, Json<ControlOk>)> {
    let registry = load_remote_mod_registry(state.mods_dir.as_ref());
    let registry_entry = validate_remote_fetch_against_registry(
        &registry,
        &req.url,
        req.mod_id.as_deref(),
    )
    .map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let cache_id = resolve_remote_cache_id(&req.url, req.mod_id.as_deref()).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let bytes = download_remote_mod_payload(&state.http, &req.url)
        .await
        .map_err(|message| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ControlOk {
                    ok: false,
                    message: Some(message),
                }),
            )
        })?;
    let (path, source) = persist_remote_mod_cache(
        state.mods_dir.as_ref(),
        &cache_id,
        &req.url,
        &bytes,
        registry_entry,
    )
    .map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(FetchModResponse {
        ok: true,
        id: cache_id,
        source,
        path: path.display().to_string(),
    }))
}

pub(crate) async fn list_remote_mods_handler(
    State(state): State<AppState>,
) -> Json<Vec<RemoteModEntry>> {
    Json(scan_remote_mod_cache(state.mods_dir.as_ref()))
}
