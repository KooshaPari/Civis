//! PBR (physically-based rendering) policy substrate.
//!
//! Pure-Rust, engine-agnostic data layer for the four spec-only FRs flagged
//! `BUILD-NEXT` in `docs/audits/spec-only-triage-batch2.md` rows 63–70 that
//! the slice 2 PBR materials work (`clients/bevy-ref/src/materials.rs`)
//! deliberately does not own: licensing attestation, render-mode selection
//! for distant LOD, canonical-vs-primitive material seed manifests, and the
//! missing-texture policy (loud in dev, flat tint in player builds).
//!
//! The Bevy-gated `materials.rs` module is the engine adapter that *consumes*
//! these types. The substrate itself is intentionally lightweight so it can
//! be unit-tested without pulling in `bevy`, `wgpu`, or any asset loader.
//!
//! FR coverage:
//!
//! - [`LicenseAttestation`]            → `FR-CIV-PBR-001` (CC0 sourcing).
//! - [`RenderMode`] / [`LodRenderPlan`] → `FR-CIV-PBR-005` (vertex-color
//!   shading fallback for distant LOD).
//! - [`MaterialMode`] / [`MaterialSeedManifest`] → `FR-CIV-PBR-007`
//!   (canonical exemplar vs. per-matid override).
//! - [`MissingTexturePolicy`] / [`PolicyAction`] → `FR-CIV-PBR-008` (loud
//!   failure in dev, flat-tint degrade in player builds).
//!
//! ## Determinism
//!
//! Every public function is pure: no I/O, no `Instant::now`, no RNG. Asset
//! paths, policy flags, and mode decisions depend only on their explicit
//! inputs. This is the prerequisite for replay-safe test coverage and for
//! the `bevy-ref` adapter to reuse the same logic in dev and player builds.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::MaterialId;

// ---------------------------------------------------------------------------
// FR-CIV-PBR-001 — CC0 sourcing
// ---------------------------------------------------------------------------

/// Sources we accept as CC0 for shipped PBR textures.
///
/// The list is intentionally tiny and explicitly enumerated so the loader
/// rejects typos at attestation time. Adding a new source is a deliberate
/// code change, not a free-form string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cc0Source {
    /// ambientCG (https://ambientcg.com) — CC0 PBR texture sets.
    AmbientCg,
    /// Poly Haven (https://polyhaven.com) — CC0 textures and models.
    PolyHaven,
}

impl Cc0Source {
    /// Stable string slug used in asset manifests and log lines.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Cc0Source::AmbientCg => "ambient_cg",
            Cc0Source::PolyHaven => "poly_haven",
        }
    }
}

/// Errors that [`LicenseAttestation::new`] can return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationError {
    /// `license` was not `"CC0"`. The spec is intentionally strict: any other
    /// string (including lowercase `cc0` or hyphenated variants) is rejected.
    NotCc0(String),
    /// `source` was not one of the enumerated [`Cc0Source`] values.
    UnknownSource(String),
    /// `manifest_url` was empty. We refuse to attest an asset we cannot
    /// point back at the CC0 distribution.
    EmptyManifestUrl,
    /// `attested_by` was empty. We refuse unsigned attestations.
    EmptyAttestedBy,
}

/// One row of the asset-manifest CC0 attestation table.
///
/// The Bevy-gated loader reads these from a per-build manifest file and the
/// dev-build startup check refuses to start a client with any un-attested
/// asset. The substrate here defines the data shape; the loader owns the
/// filesystem read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseAttestation {
    /// Asset path inside the build (e.g. `textures/grass_field/albedo.jpg`).
    pub asset_path: String,
    /// License string. Must equal `"CC0"` exactly.
    pub license: String,
    /// Enumerated CC0 source.
    pub source: Cc0Source,
    /// URL to the original distribution manifest entry on the source site.
    pub manifest_url: String,
    /// Human or build-pipeline id that added this attestation.
    pub attested_by: String,
    /// Optional content-hash for tamper detection; `None` means "trust the
    /// manifest as-is, the build pipeline repacks it".
    pub content_sha256: Option<String>,
}

impl LicenseAttestation {
    /// Construct a [`LicenseAttestation`] after validating the four required
    /// invariants. The `content_sha256` field is optional and accepted as-is.
    pub fn new(
        asset_path: impl Into<String>,
        source: Cc0Source,
        manifest_url: impl Into<String>,
        attested_by: impl Into<String>,
    ) -> Result<Self, AttestationError> {
        let asset_path = asset_path.into();
        let manifest_url = manifest_url.into();
        let attested_by = attested_by.into();
        if manifest_url.trim().is_empty() {
            return Err(AttestationError::EmptyManifestUrl);
        }
        if attested_by.trim().is_empty() {
            return Err(AttestationError::EmptyAttestedBy);
        }
        Ok(Self {
            asset_path,
            license: "CC0".to_string(),
            source,
            manifest_url,
            attested_by,
            content_sha256: None,
        })
    }

    /// Builder helper to attach a content hash after [`Self::new`].
    #[must_use]
    pub fn with_content_sha256(mut self, hash: impl Into<String>) -> Self {
        self.content_sha256 = Some(hash.into());
        self
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-005 — distant-LOD vertex-color shading
// ---------------------------------------------------------------------------

/// Render mode selected for a given chunk distance from the camera.
///
/// The substrate does not know how a renderer draws each mode — it only
/// encodes the policy. The Bevy adapter maps each variant to a concrete
/// material binding in `materials.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// Full PBR triplanar shading (Phase 1/2 of the materials plan).
    PbrTriplanar,
    /// PBR atlas sampling (cheaper than triplanar, no surface-orientation
    /// projection).
    PbrAtlas,
    /// Flat vertex-color shading. Used at the outer LOD ring where per-pixel
    /// shading is wasted and the triplanar cost would dominate frame time.
    VertexColor,
}

/// Default distance thresholds (in chunk units) for the render-mode switch.
/// Distances are monotonically non-decreasing: `near < mid < far`. Beyond
/// `far` we always fall back to `VertexColor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LodDistanceConfig {
    /// Distance below which we keep `PbrTriplanar` (inclusive).
    pub near_chunks: u32,
    /// Distance at which we switch to `PbrAtlas` (inclusive).
    pub mid_chunks: u32,
    /// Distance at which we fall back to `VertexColor` (inclusive).
    pub far_chunks: u32,
}

impl Default for LodDistanceConfig {
    fn default() -> Self {
        // These match the streaming-window default rings (Phase-1 numbers).
        // They are pure data; the render adapter can override.
        Self {
            near_chunks: 2,
            mid_chunks: 4,
            far_chunks: 8,
        }
    }
}

impl LodDistanceConfig {
    /// Sanity-check the three thresholds. Returns `Err` if the configuration
    /// is not monotonically non-decreasing. We refuse to start with
    /// `near > far` because that would always collapse to vertex color.
    pub fn validate(self) -> Result<(), &'static str> {
        if self.near_chunks > self.mid_chunks {
            return Err("near_chunks must be <= mid_chunks");
        }
        if self.mid_chunks > self.far_chunks {
            return Err("mid_chunks must be <= far_chunks");
        }
        Ok(())
    }
}

/// Output of [`LodRenderPlan::for_distance`]: which mode to use plus the
/// blend parameter the Bevy adapter feeds into `TriplanarParams::vertex_color_blend`.
///
/// `vertex_color_blend == 1.0` is a hard vertex-color pass; `0.0` is the
/// default "full PBR" setting. Anything in between is reserved for a future
/// seam-blend ring.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LodRenderPlan {
    /// Selected render mode for the requested distance.
    pub mode: RenderMode,
    /// `0.0`–`1.0` blend factor; the Bevy `TriplanarParams` struct copies
    /// this straight into `vertex_color_blend`.
    pub vertex_color_blend: f32,
}

impl LodRenderPlan {
    /// Decide the render mode for a chunk that is `distance_chunks` rings
    /// away from the camera. Out-of-order `LodDistanceConfig` is rejected
    /// loudly (the `validate` method should be called at startup).
    pub fn for_distance(distance_chunks: u32, config: LodDistanceConfig) -> Self {
        let mode = if distance_chunks <= config.near_chunks {
            RenderMode::PbrTriplanar
        } else if distance_chunks <= config.mid_chunks {
            RenderMode::PbrAtlas
        } else {
            RenderMode::VertexColor
        };
        let vertex_color_blend = match mode {
            RenderMode::PbrTriplanar => 0.0,
            RenderMode::PbrAtlas => 0.0,
            RenderMode::VertexColor => 1.0,
        };
        Self {
            mode,
            vertex_color_blend,
        }
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-007 — canonical exemplar vs. primitive per-matid mode
// ---------------------------------------------------------------------------

/// Selects whether the material seed manifest references a canonical
/// exemplar (full PBR set) or allows callers to override per-`MaterialId`.
///
/// The two modes are not just data; the engine adapter interprets them
/// differently:
///
/// - `Canonical` — the loader expects every requested material to resolve
///   through [`MaterialSeedManifest::exemplar_id`]. Per-matid overrides are
///   rejected at lookup time.
/// - `Primitive` — the loader may fall through to
///   [`MaterialSeedManifest::per_matid_overrides`] for any matid that is not
///   covered by the exemplar, and an explicit override wins over the
///   exemplar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialMode {
    /// Reference an exemplar material seed manifest by stable id.
    Canonical,
    /// Allow per-`MaterialId` overrides on top of the exemplar.
    Primitive,
}

/// A single per-`MaterialId` override used in `Primitive` mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MaterialOverride {
    /// Subset of the PBR maps this override wants to provide. Empty means
    /// "use the exemplar for this slot". The engine adapter never synthesises
    /// data — it just hands each present map to the loader.
    pub albedo_path: Option<String>,
    /// Optional normal-map override.
    pub normal_path: Option<String>,
    /// Optional ORM (`R=AO,G=Rough,B=Metal`) override.
    pub orm_path: Option<String>,
    /// Per-matid roughness override (0.0–1.0). `None` falls through to the
    /// exemplar.
    pub perceptual_roughness: Option<f32>,
    /// Per-matid tint multiplied with the exemplar albedo. `None` is
    /// identity. RGB is sRGB on disk but stored as `f32` here for blending.
    pub tint_srgb: Option<[f32; 3]>,
}

/// Errors that [`MaterialSeedManifest::resolve`] can return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The manifest is `Canonical` but a per-matid override was supplied.
    /// Canonical mode forbids per-matid overrides by spec; reject loudly so
    /// callers know to switch to `Primitive` instead.
    CanonicalRejectsOverrides(MaterialId),
    /// Neither an exemplar nor a per-matid override produced a result. The
    /// missing-texture policy ([`MissingTexturePolicy`]) is what callers
    /// fall back to at runtime.
    NoEntryForMaterial(MaterialId),
}

/// The full material seed manifest. The Bevy-gated loader reads this once at
/// startup and the [`Self::resolve`] function is the single source of truth
/// for "what should this matid look like in this build?".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialSeedManifest {
    /// Mode flag — see [`MaterialMode`].
    pub mode: MaterialMode,
    /// Stable id of the exemplar set (e.g. `civis_ground_v1`). Required in
    /// both modes; the canonical exemplar defines the base PBR set.
    pub exemplar_id: String,
    /// Optional per-matid overrides, sorted by `MaterialId` for replay
    /// determinism. Only consulted in `Primitive` mode.
    pub per_matid_overrides: BTreeMap<MaterialId, MaterialOverride>,
    /// Schema version of this manifest. Bump on breaking changes to
    /// [`MaterialOverride`] or [`MaterialSeedManifest`] so a future migration
    /// can detect old manifests.
    pub schema_version: String,
}

impl MaterialSeedManifest {
    /// Construct a manifest in `Canonical` mode without overrides.
    #[must_use]
    pub fn canonical(exemplar_id: impl Into<String>) -> Self {
        Self {
            mode: MaterialMode::Canonical,
            exemplar_id: exemplar_id.into(),
            per_matid_overrides: BTreeMap::new(),
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }

    /// Construct a manifest in `Primitive` mode, with optional per-matid
    /// overrides layered on top of the exemplar.
    #[must_use]
    pub fn primitive(
        exemplar_id: impl Into<String>,
        per_matid_overrides: BTreeMap<MaterialId, MaterialOverride>,
    ) -> Self {
        Self {
            mode: MaterialMode::Primitive,
            exemplar_id: exemplar_id.into(),
            per_matid_overrides,
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }

    /// Resolve a single matid to the override (if any) that should be
    /// applied on top of the exemplar. The function does not pick textures
    /// or load files — it only chooses which [`MaterialOverride`] (and
    /// therefore which PBR slots) win for the requested matid.
    ///
    /// `Canonical` mode never returns per-matid overrides; it always
    /// succeeds because the exemplar covers every matid by definition.
    /// `Primitive` mode returns an override when one exists and the
    /// [`ManifestError::NoEntryForMaterial`] error otherwise.
    pub fn resolve(&self, matid: MaterialId) -> Result<Option<&MaterialOverride>, ManifestError> {
        match self.mode {
            MaterialMode::Canonical => {
                if self.per_matid_overrides.contains_key(&matid) {
                    Err(ManifestError::CanonicalRejectsOverrides(matid))
                } else {
                    Ok(None)
                }
            }
            MaterialMode::Primitive => match self.per_matid_overrides.get(&matid) {
                Some(ov) => Ok(Some(ov)),
                None => Err(ManifestError::NoEntryForMaterial(matid)),
            },
        }
    }
}

/// Schema version of the [`MaterialSeedManifest`] data shape. Bump on
/// breaking changes to the manifest struct.
pub const SCHEMA_VERSION: &str = "0.1.0-pbr-manifest";

// ---------------------------------------------------------------------------
// FR-CIV-PBR-008 — missing-texture loud vs. flat-tint degrade
// ---------------------------------------------------------------------------

/// Build flavour the client is currently running in. Decides whether a
/// missing texture panics (dev) or degrades to a flat tint (player).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildFlavour {
    /// Developer build — missing textures MUST fail loud.
    Dev,
    /// Player/shipping build — missing textures MUST degrade to flat tint
    /// with a logged warning, never crash the game.
    Player,
}

/// Build-time selectable policy. Default is `Auto` (decide from
/// [`BuildFlavour`]). A user can pin to one or the other for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingTexturePolicy {
    /// Choose `Loud` in `Dev` and `FlatTint` in `Player`.
    Auto,
    /// Always panic with a clear error. Used in dev builds and CI.
    Loud,
    /// Always log a warning and degrade to a flat sRGB tint. Used in player
    /// builds and offline replay renders.
    FlatTint,
}

impl MissingTexturePolicy {
    /// Resolve `Auto` against a [`BuildFlavour`]. `Loud` and `FlatTint`
    /// pass through unchanged.
    #[must_use]
    pub fn resolve(self, flavour: BuildFlavour) -> PolicyAction {
        match self {
            MissingTexturePolicy::Auto => match flavour {
                BuildFlavour::Dev => PolicyAction::Panic,
                BuildFlavour::Player => PolicyAction::FlatTintWithWarning,
            },
            MissingTexturePolicy::Loud => PolicyAction::Panic,
            MissingTexturePolicy::FlatTint => PolicyAction::FlatTintWithWarning,
        }
    }
}

/// The runtime action the engine adapter must take on a missing texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    /// Panic with a descriptive error (dev build default).
    Panic,
    /// Log a warning, draw the chunk with a flat sRGB tint, and continue.
    FlatTintWithWarning,
}

/// Outcome of a single texture load attempt. The Bevy adapter builds one of
/// these per `load_with_settings` call and feeds it to
/// [`policy_action_for_report`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingTextureReport {
    /// Asset path that was requested.
    pub path: String,
    /// `true` if the loader returned a non-empty handle, `false` if the
    /// asset was missing or failed to decode.
    pub loaded: bool,
    /// Build-time policy selected by the user or the auto-default.
    pub policy: MissingTexturePolicy,
    /// Build flavour the policy resolves against in `Auto` mode.
    pub flavour: BuildFlavour,
}

impl MissingTextureReport {
    /// Compute the runtime action the engine adapter should take for this
    /// report. `loaded == true` is always `Keep`; `loaded == false` is
    /// decided by the resolved policy.
    #[must_use]
    pub fn action(&self) -> RuntimeAction {
        if self.loaded {
            return RuntimeAction::Keep;
        }
        match self.policy.resolve(self.flavour) {
            PolicyAction::Panic => RuntimeAction::Panic,
            PolicyAction::FlatTintWithWarning => RuntimeAction::FlatTintWithWarning,
        }
    }
}

/// Runtime action the engine adapter takes for a single chunk's texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAction {
    /// Texture loaded; no change to the rendered material.
    Keep,
    /// Missing texture in dev: panic with a clear error referencing `path`.
    Panic,
    /// Missing texture in player: log a warning, draw the chunk with the
    /// flat tint from the report, and continue.
    FlatTintWithWarning,
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-002 — glTF channel wiring (separate MR + AO maps; ORM fan-out)
// ---------------------------------------------------------------------------

/// Which PBR data slot a single texture channel feeds.
///
/// `glTF 2.0` packs metallic-roughness into a single texture by convention
/// (G=roughness, B=metallic) and leaves AO as either a dedicated map or the R
/// channel of an "ORM" packing. The Civis engine adapter accepts BOTH layouts
/// — a single dedicated MR map, a single dedicated AO map, or one combined
/// ORM map — and routes channels accordingly. This enum is the routing key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PbrChannel {
    /// sRGB albedo / base color.
    Albedo,
    /// Linear tangent-space normal map.
    Normal,
    /// Metallic factor (0.0 dielectric .. 1.0 metal).
    Metallic,
    /// Perceptual roughness (0.0 mirror .. 1.0 matte).
    Roughness,
    /// Ambient occlusion (multiplicative; 1.0 = un-occluded).
    AmbientOcclusion,
}

/// The PBR channel-to-source-map binding for one logical material slot.
///
/// A `TextureChannelMap` is pure data: it tells the loader which texture file
/// supplies which channel(s). The loader never re-packs channels; if the user
/// provided an `OrmCombined` file, we still read R/G/B into the three separate
/// slots and the engine binds them individually, satisfying
/// `FR-CIV-PBR-002`'s "ORM sources MAY fan into both without repack" clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TextureChannelMap {
    /// Asset path for the albedo texture (sRGB on disk). `None` means the
    /// material has no albedo (rare; used for emissive-only decals).
    pub albedo_path: Option<String>,
    /// Asset path for the tangent-space normal map. Always linear.
    pub normal_path: Option<String>,
    /// Asset path for the standalone metallic-roughness map (G=rough,
    /// B=metal). When `None` the engine falls back to `orm_path` if present.
    pub mr_path: Option<String>,
    /// Asset path for the standalone AO map (R=AO). When `None` the engine
    /// falls back to `orm_path` if present.
    pub ao_path: Option<String>,
    /// Asset path for the combined ORM map (R=AO, G=Rough, B=Metal). May be
    /// set in addition to `mr_path`/`ao_path` as a redundant source; the
    /// adapter picks the most-specific one available.
    pub orm_path: Option<String>,
}

impl TextureChannelMap {
    /// Construct a minimal channel map with only an albedo + normal. Used
    /// for cheap materials that don't need PBR data maps.
    #[must_use]
    pub fn minimal(albedo: impl Into<String>, normal: impl Into<String>) -> Self {
        Self {
            albedo_path: Some(albedo.into()),
            normal_path: Some(normal.into()),
            mr_path: None,
            ao_path: None,
            orm_path: None,
        }
    }

    /// Construct a full standalone map: albedo + normal + dedicated MR + AO.
    #[must_use]
    pub fn standalone(
        albedo: impl Into<String>,
        normal: impl Into<String>,
        mr: impl Into<String>,
        ao: impl Into<String>,
    ) -> Self {
        Self {
            albedo_path: Some(albedo.into()),
            normal_path: Some(normal.into()),
            mr_path: Some(mr.into()),
            ao_path: Some(ao.into()),
            orm_path: None,
        }
    }

    /// Construct an ORM-packed map: albedo + normal + a single ORM file that
    /// fans out to the metallic/roughness/AO slots at load time.
    #[must_use]
    pub fn orm_packed(
        albedo: impl Into<String>,
        normal: impl Into<String>,
        orm: impl Into<String>,
    ) -> Self {
        Self {
            albedo_path: Some(albedo.into()),
            normal_path: Some(normal.into()),
            mr_path: None,
            ao_path: None,
            orm_path: Some(orm.into()),
        }
    }

    /// Return every channel-source path the loader will need to read, in
    /// deterministic order. The engine adapter uses this to warm the
    /// asset-cache before the first draw.
    pub fn required_paths(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::with_capacity(5);
        if let Some(p) = self.albedo_path.as_deref() {
            out.push(p);
        }
        if let Some(p) = self.normal_path.as_deref() {
            out.push(p);
        }
        if let Some(p) = self.mr_path.as_deref() {
            out.push(p);
        }
        if let Some(p) = self.ao_path.as_deref() {
            out.push(p);
        }
        if let Some(p) = self.orm_path.as_deref() {
            out.push(p);
        }
        out
    }

    /// `true` when the configuration can drive every standard PBR channel.
    /// The engine refuses to bind an incomplete map (loud in dev, flat tint
    /// in player — see [`MissingTexturePolicy`]).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.albedo_path.is_some()
            && self.normal_path.is_some()
            && (self.mr_path.is_some() || self.orm_path.is_some())
            && (self.ao_path.is_some() || self.orm_path.is_some())
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-003 — triplanar splatting driven by mesher matid
// ---------------------------------------------------------------------------

/// One layer of a triplanar splat plan. Each layer is keyed by a single
/// `MaterialId` from the voxel mesher; the Bevy adapter drives the
/// `bevy_triplanar_splatting` shader with the layer stack ordered by
/// `splat_weight` descending.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriplanarLayer {
    /// Material id (from the mesher) this layer binds to.
    pub matid: MaterialId,
    /// Per-channel map for this layer.
    pub channels: TextureChannelMap,
    /// Relative blend weight (0.0..1.0) when this matid shares a voxel with
    /// another. Higher wins at the blend step.
    pub splat_weight: f32,
    /// World-space tile size in fixed-point voxel units. 1.0 means one
    /// tile per voxel. Larger values produce coarser surface detail.
    pub world_tile: f32,
}

impl TriplanarLayer {
    /// Construct a triplanar layer with a sane default tile size and
    /// moderate weight. Used as the building block for [`TriplanarSplatPlan`].
    #[must_use]
    pub fn new(matid: MaterialId, channels: TextureChannelMap, splat_weight: f32) -> Self {
        Self {
            matid,
            channels,
            splat_weight,
            world_tile: 1.0,
        }
    }

    /// Override the world-space tile size. Returns `self` for chaining.
    #[must_use]
    pub fn with_world_tile(mut self, world_tile: f32) -> Self {
        self.world_tile = world_tile;
        self
    }
}

/// A pure-data triplanar splat plan. The Bevy adapter wraps
/// `bevy_triplanar_splatting` and binds the layers in `splat_weight` order
/// (deterministic — `BTreeMap<MaterialId, _>` underneath). The plan does not
/// know how to draw, only what to draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriplanarSplatPlan {
    /// Per-`MaterialId` layer, stored in a `BTreeMap` for deterministic
    /// iteration order. The Bevy adapter iterates in ascending `matid` to
    /// build the layer stack.
    pub layers: BTreeMap<MaterialId, TriplanarLayer>,
    /// Schema version of this plan shape; bump on breaking changes so a
    /// future migration can detect old plans.
    pub schema_version: String,
}

impl TriplanarSplatPlan {
    /// Construct an empty plan bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            layers: BTreeMap::new(),
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert or replace a layer. Returns `&mut self` for chaining.
    pub fn insert(&mut self, layer: TriplanarLayer) -> &mut Self {
        self.layers.insert(layer.matid, layer);
        self
    }

    /// Iterate layers in deterministic `matid`-ascending order.
    pub fn iter_ordered(&self) -> impl Iterator<Item = (&MaterialId, &TriplanarLayer)> {
        self.layers.iter()
    }

    /// Normalize the `splat_weight` of every layer so the maximum equals
    /// `1.0`. Used at the end of authoring so the Bevy adapter can pass the
    /// weight straight into the shader's blend slot without re-scaling.
    pub fn normalize_weights(&mut self) {
        let max = self
            .layers
            .values()
            .map(|l| l.splat_weight)
            .fold(0.0_f32, f32::max);
        if max > 0.0 {
            for layer in self.layers.values_mut() {
                layer.splat_weight /= max;
            }
        }
    }

    /// `true` if every layer has a complete `TextureChannelMap` and at
    /// least one layer exists. The Bevy adapter refuses to start with an
    /// incomplete plan; this is the substrate-side check.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.layers.is_empty() && self.layers.values().all(TriplanarLayer::is_complete_layer)
    }
}

impl TriplanarLayer {
    #[must_use]
    fn is_complete_layer(&self) -> bool {
        self.channels.is_complete() && self.splat_weight.is_finite() && self.splat_weight >= 0.0
    }
}

impl Default for TriplanarSplatPlan {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-009 — pure triplanar PBR blend helper
// ---------------------------------------------------------------------------

/// A single axis sample for a triplanar PBR blend.
///
/// The helper is intentionally engine-agnostic and deterministic: callers
/// provide the three axis samples already fetched from their texture sources,
/// and this module blends them without I/O, RNG, or hidden state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriplanarAxisSample {
    /// sRGB albedo converted to linear before blending by the caller.
    pub albedo_linear: [f32; 3],
    /// Perceptual roughness in the inclusive `0.0..=1.0` range.
    pub perceptual_roughness: f32,
    /// World-space or tangent-space normal, expected to be unit length.
    pub normal: [f32; 3],
}

/// Output of [`blend_triplanar_pbr`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriplanarPbrBlend {
    /// Blended linear albedo.
    pub albedo_linear: [f32; 3],
    /// Blended perceptual roughness.
    pub perceptual_roughness: f32,
    /// Renormalized blended normal.
    pub normal: [f32; 3],
}

/// Compute deterministic triplanar axis weights from a world-space normal.
///
/// The weights are the absolute normal components normalized to sum to `1.0`.
/// Degenerate zero vectors fall back to equal axis weights to keep the helper
/// total and replay-safe.
#[must_use]
pub fn triplanar_axis_weights(world_normal: [f32; 3]) -> [f32; 3] {
    let wx = world_normal[0].abs();
    let wy = world_normal[1].abs();
    let wz = world_normal[2].abs();
    let sum = wx + wy + wz;
    if sum > 0.0 {
        [wx / sum, wy / sum, wz / sum]
    } else {
        [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]
    }
}

/// Pure deterministic blend for triplanar PBR samples.
///
/// Albedo and roughness are blended linearly by the axis weights. Normals are
/// blended linearly and then renormalized so the result remains suitable for a
/// shading pipeline.
#[must_use]
pub fn blend_triplanar_pbr(
    axis_weights: [f32; 3],
    x_axis: TriplanarAxisSample,
    y_axis: TriplanarAxisSample,
    z_axis: TriplanarAxisSample,
) -> TriplanarPbrBlend {
    let albedo_linear = [
        x_axis.albedo_linear[0] * axis_weights[0]
            + y_axis.albedo_linear[0] * axis_weights[1]
            + z_axis.albedo_linear[0] * axis_weights[2],
        x_axis.albedo_linear[1] * axis_weights[0]
            + y_axis.albedo_linear[1] * axis_weights[1]
            + z_axis.albedo_linear[1] * axis_weights[2],
        x_axis.albedo_linear[2] * axis_weights[0]
            + y_axis.albedo_linear[2] * axis_weights[1]
            + z_axis.albedo_linear[2] * axis_weights[2],
    ];

    let perceptual_roughness = x_axis.perceptual_roughness * axis_weights[0]
        + y_axis.perceptual_roughness * axis_weights[1]
        + z_axis.perceptual_roughness * axis_weights[2];

    let blended_normal = [
        x_axis.normal[0] * axis_weights[0]
            + y_axis.normal[0] * axis_weights[1]
            + z_axis.normal[0] * axis_weights[2],
        x_axis.normal[1] * axis_weights[0]
            + y_axis.normal[1] * axis_weights[1]
            + z_axis.normal[1] * axis_weights[2],
        x_axis.normal[2] * axis_weights[0]
            + y_axis.normal[2] * axis_weights[1]
            + z_axis.normal[2] * axis_weights[2],
    ];

    let normal_len_sq = blended_normal[0] * blended_normal[0]
        + blended_normal[1] * blended_normal[1]
        + blended_normal[2] * blended_normal[2];
    let normal = if normal_len_sq > 0.0 {
        let inv_len = normal_len_sq.sqrt().recip();
        [
            blended_normal[0] * inv_len,
            blended_normal[1] * inv_len,
            blended_normal[2] * inv_len,
        ]
    } else {
        [0.0, 0.0, 1.0]
    };

    TriplanarPbrBlend {
        albedo_linear,
        perceptual_roughness,
        normal,
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-004 — greedy mesh + texture-array atlas for built voxels
// ---------------------------------------------------------------------------

/// One slice of the texture-array atlas used by the greedy mesher for
/// built (non-triplanar) voxels. The Bevy adapter uploads the atlas as a
/// single 2D-array texture and binds `array_layer` to the matching face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AtlasSlice {
    /// Stable id of the slice; equals the `MaterialId` it represents when
    /// one matid is bound to one slice. Multiple matids MAY share a slice
    /// when they are visually identical (palette aliasing).
    pub matid: MaterialId,
    /// Zero-based layer index in the 2D array texture.
    pub array_layer: u16,
}

/// Greedy-mesh + texture-array atlas plan. The Bevy adapter uses this in
/// place of triplanar splatting when the material is unsuitable for
/// world-space projection (e.g. high-frequency tile-able decals).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GreedyAtlasPlan {
    /// Sorted by `matid` for replay determinism.
    pub slices: BTreeMap<MaterialId, AtlasSlice>,
    /// Total number of array layers actually used. Lets the adapter
    /// pre-allocate the 2D-array texture at the right depth.
    pub array_depth: u16,
    /// Schema version of this plan shape.
    pub schema_version: String,
}

impl GreedyAtlasPlan {
    /// Construct an empty plan bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slices: BTreeMap::new(),
            array_depth: 0,
            schema_version: SCHEMA_VERSION.to_string(),
        }
    }

    /// Assign the next free array layer to `matid`. Returns the new slice.
    pub fn allocate(&mut self, matid: MaterialId) -> &AtlasSlice {
        let layer = self.array_depth;
        self.array_depth = self.array_depth.saturating_add(1);
        self.slices.insert(
            matid,
            AtlasSlice {
                matid,
                array_layer: layer,
            },
        );
        self.slices.get(&matid).expect("slice was just inserted")
    }

    /// Look up the array layer for `matid`, or `None` if the plan does not
    /// cover it.
    #[must_use]
    pub fn layer_for(&self, matid: MaterialId) -> Option<u16> {
        self.slices.get(&matid).map(|s| s.array_layer)
    }
}

impl Default for GreedyAtlasPlan {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-PBR-006 — albedo sRGB; data maps load linear (`is_srgb=false`)
// ---------------------------------------------------------------------------

/// Per-slot color-space flag. The Bevy adapter reads this when calling
/// `load_with_settings` to decide whether to apply sRGB→linear conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorSpace {
    /// sRGB transfer function on disk; engine stores linear after decode.
    /// Used for albedo, emissive, and any color the player perceives.
    Srgb,
    /// Linear data; engine stores as-is. Used for normal/metallic/roughness/
    /// AO/height/curvature. Passing an sRGB-encoded texture through a
    /// linear slot would silently double-gamma the values.
    Linear,
}

/// Color-space policy. The Bevy adapter reads this once at startup and
/// consults it for every `load_with_settings` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorSpacePolicy {
    /// Color space for every albedo / emissive / decal slot.
    pub color: ColorSpace,
    /// Color space for every data slot (normal, metallic, roughness, AO,
    /// height, curvature).
    pub data: ColorSpace,
}

impl Default for ColorSpacePolicy {
    fn default() -> Self {
        // Spec-mandated defaults. Changing this is a breaking change.
        Self {
            color: ColorSpace::Srgb,
            data: ColorSpace::Linear,
        }
    }
}

impl ColorSpacePolicy {
    /// Spec-mandated policy: color slots are sRGB, data slots are linear.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            color: ColorSpace::Srgb,
            data: ColorSpace::Linear,
        }
    }

    /// Look up the color space for a given channel. Returns `color` for
    /// color channels and `data` for everything else.
    #[must_use]
    pub const fn for_channel(&self, channel: PbrChannel) -> ColorSpace {
        match channel {
            PbrChannel::Albedo => self.color,
            PbrChannel::Normal
            | PbrChannel::Metallic
            | PbrChannel::Roughness
            | PbrChannel::AmbientOcclusion => self.data,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- FR-CIV-PBR-001 -----------------------------------------------

    /// FR-CIV-PBR-001 — `LicenseAttestation::new` enforces CC0 source and
    /// non-empty manifest / attester fields.
    #[test]
    fn fr_pbr_001_cc0_attestation_round_trip() {
        let att = LicenseAttestation::new(
            "textures/grass_field/albedo.jpg",
            Cc0Source::AmbientCg,
            "https://ambientcg.com/view?id=Grass005",
            "ci-fetch-pbr-textures@v1.4.0",
        )
        .expect("ambientCG row is valid");
        assert_eq!(att.license, "CC0");
        assert_eq!(att.source, Cc0Source::AmbientCg);
        assert_eq!(att.source.slug(), "ambient_cg");
        assert!(att.content_sha256.is_none());

        let poly = LicenseAttestation::new(
            "textures/rock_cliff/albedo.jpg",
            Cc0Source::PolyHaven,
            "https://polyhaven.com/a/rock_cliff",
            "ci-fetch-pbr-textures@v1.4.0",
        )
        .expect("polyhaven row is valid");
        assert_eq!(poly.source.slug(), "poly_haven");
    }

    /// FR-CIV-PBR-001 — attestation rejects empty manifest URL and empty
    /// attester regardless of source.
    #[test]
    fn fr_pbr_001_cc0_attestation_rejects_empty_fields() {
        let missing_url = LicenseAttestation::new(
            "textures/grass_field/albedo.jpg",
            Cc0Source::AmbientCg,
            "   ",
            "ci",
        );
        assert_eq!(missing_url, Err(AttestationError::EmptyManifestUrl));

        let missing_attester = LicenseAttestation::new(
            "textures/grass_field/albedo.jpg",
            Cc0Source::AmbientCg,
            "https://ambientcg.com/view?id=Grass005",
            "",
        );
        assert_eq!(missing_attester, Err(AttestationError::EmptyAttestedBy));
    }

    /// FR-CIV-PBR-001 — `with_content_sha256` attaches a content hash for
    /// tamper detection; the manifest then round-trips through serde.
    #[test]
    fn fr_pbr_001_attestation_content_hash_round_trips() {
        let att = LicenseAttestation::new(
            "textures/sand_beach/albedo.jpg",
            Cc0Source::PolyHaven,
            "https://polyhaven.com/a/sand_beach",
            "ci-fetch-pbr-textures@v1.4.0",
        )
        .unwrap()
        .with_content_sha256("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
        let json = serde_json::to_string(&att).expect("serialize");
        let back: LicenseAttestation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, att);
    }

    // -- FR-CIV-PBR-005 -----------------------------------------------

    /// FR-CIV-PBR-005 — `LodRenderPlan::for_distance` steps from
    /// `PbrTriplanar` (near) → `PbrAtlas` (mid) → `VertexColor` (far) and
    /// the `vertex_color_blend` parameter is exactly `1.0` for vertex color
    /// and `0.0` otherwise.
    #[test]
    fn fr_pbr_005_lod_render_plan_steps_pbr_to_vertex_color() {
        let cfg = LodDistanceConfig::default();
        assert!(cfg.validate().is_ok(), "default config must validate");

        let near = LodRenderPlan::for_distance(0, cfg);
        assert_eq!(near.mode, RenderMode::PbrTriplanar);
        assert_eq!(near.vertex_color_blend, 0.0);

        let mid = LodRenderPlan::for_distance(3, cfg);
        assert_eq!(mid.mode, RenderMode::PbrAtlas);
        assert_eq!(mid.vertex_color_blend, 0.0);

        let far = LodRenderPlan::for_distance(10, cfg);
        assert_eq!(far.mode, RenderMode::VertexColor);
        assert_eq!(far.vertex_color_blend, 1.0);
    }

    /// FR-CIV-PBR-005 — `LodDistanceConfig::validate` rejects out-of-order
    /// thresholds so the adapter cannot be mis-configured silently.
    #[test]
    fn fr_pbr_005_lod_distance_config_validates_thresholds() {
        let bad = LodDistanceConfig {
            near_chunks: 5,
            mid_chunks: 3,
            far_chunks: 8,
        };
        assert!(bad.validate().is_err());

        let ok = LodDistanceConfig {
            near_chunks: 2,
            mid_chunks: 4,
            far_chunks: 4, // mid == far is allowed; they alias
        };
        assert!(ok.validate().is_ok());
    }

    // -- FR-CIV-PBR-007 -----------------------------------------------

    /// FR-CIV-PBR-007 — `Canonical` mode resolves every matid through the
    /// exemplar and refuses to honour per-matid overrides.
    #[test]
    fn fr_pbr_007_canonical_mode_rejects_per_matid_overrides() {
        let mut overrides: BTreeMap<MaterialId, MaterialOverride> = BTreeMap::new();
        overrides.insert(
            crate::material::SAND,
            MaterialOverride {
                albedo_path: Some("textures/sand_beach/albedo.jpg".to_string()),
                ..Default::default()
            },
        );
        let manifest = MaterialSeedManifest {
            mode: MaterialMode::Canonical,
            exemplar_id: "civis_ground_v1".to_string(),
            per_matid_overrides: overrides,
            schema_version: SCHEMA_VERSION.to_string(),
        };
        let resolved = manifest.resolve(crate::material::SAND);
        assert_eq!(
            resolved,
            Err(ManifestError::CanonicalRejectsOverrides(
                crate::material::SAND
            ))
        );
    }

    /// FR-CIV-PBR-007 — `Primitive` mode returns the per-matid override
    /// when one exists and reports a missing entry when it does not, so the
    /// caller can fall through to the missing-texture policy.
    #[test]
    fn fr_pbr_007_primitive_mode_returns_override_or_missing_entry() {
        let mut overrides: BTreeMap<MaterialId, MaterialOverride> = BTreeMap::new();
        overrides.insert(
            crate::material::SAND,
            MaterialOverride {
                perceptual_roughness: Some(0.42),
                tint_srgb: Some([0.7, 0.6, 0.5]),
                ..Default::default()
            },
        );
        let manifest = MaterialSeedManifest::primitive("civis_ground_v1", overrides);

        let hit = manifest.resolve(crate::material::SAND).expect("hit");
        let hit = hit.expect("primitive manifest has an override for SAND");
        assert_eq!(hit.perceptual_roughness, Some(0.42));
        assert_eq!(hit.tint_srgb, Some([0.7, 0.6, 0.5]));

        let miss = manifest.resolve(crate::material::STONE);
        assert_eq!(
            miss,
            Err(ManifestError::NoEntryForMaterial(crate::material::STONE))
        );
    }

    /// FR-CIV-PBR-007 — `MaterialMode` round-trips through serde so a
    /// runtime config can be reloaded from disk without surprises.
    #[test]
    fn fr_pbr_007_material_mode_round_trips() {
        let canonical = serde_json::to_string(&MaterialMode::Canonical).unwrap();
        assert_eq!(canonical, "\"canonical\"");
        let primitive = serde_json::to_string(&MaterialMode::Primitive).unwrap();
        assert_eq!(primitive, "\"primitive\"");
        let back: MaterialMode = serde_json::from_str(&canonical).unwrap();
        assert_eq!(back, MaterialMode::Canonical);
    }

    // -- FR-CIV-PBR-008 -----------------------------------------------

    /// FR-CIV-PBR-008 — `MissingTexturePolicy::Auto` resolves to `Panic`
    /// in dev builds and `FlatTintWithWarning` in player builds.
    #[test]
    fn fr_pbr_008_auto_policy_resolves_by_build_flavour() {
        assert_eq!(
            MissingTexturePolicy::Auto.resolve(BuildFlavour::Dev),
            PolicyAction::Panic
        );
        assert_eq!(
            MissingTexturePolicy::Auto.resolve(BuildFlavour::Player),
            PolicyAction::FlatTintWithWarning
        );
    }

    /// FR-CIV-PBR-008 — Explicit `Loud` and `FlatTint` policies are
    /// honoured regardless of build flavour, so tests can pin the action.
    #[test]
    fn fr_pbr_008_explicit_policies_override_flavour() {
        assert_eq!(
            MissingTexturePolicy::Loud.resolve(BuildFlavour::Player),
            PolicyAction::Panic
        );
        assert_eq!(
            MissingTexturePolicy::FlatTint.resolve(BuildFlavour::Dev),
            PolicyAction::FlatTintWithWarning
        );
    }

    /// FR-CIV-PBR-008 — A loaded texture is `Keep` regardless of policy;
    /// a missing texture in dev/Auto is `Panic`; a missing texture in
    /// player/Auto is `FlatTintWithWarning`.
    #[test]
    fn fr_pbr_008_missing_texture_report_drives_runtime_action() {
        let loaded = MissingTextureReport {
            path: "textures/grass_field/albedo.jpg".to_string(),
            loaded: true,
            policy: MissingTexturePolicy::Auto,
            flavour: BuildFlavour::Dev,
        };
        assert_eq!(loaded.action(), RuntimeAction::Keep);

        let missing_dev = MissingTextureReport {
            path: "textures/grass_field/normal.jpg".to_string(),
            loaded: false,
            policy: MissingTexturePolicy::Auto,
            flavour: BuildFlavour::Dev,
        };
        assert_eq!(missing_dev.action(), RuntimeAction::Panic);

        let missing_player = MissingTextureReport {
            path: "textures/grass_field/normal.jpg".to_string(),
            loaded: false,
            policy: MissingTexturePolicy::Auto,
            flavour: BuildFlavour::Player,
        };
        assert_eq!(missing_player.action(), RuntimeAction::FlatTintWithWarning);
    }

    // -- FR-CIV-PBR-002 -----------------------------------------------

    /// FR-CIV-PBR-002 — `TextureChannelMap::is_complete` requires albedo +
    /// normal AND at least one of {`mr_path`, `orm_path`} AND at least one
    /// of {`ao_path`, `orm_path`}. A standalone map, an ORM-packed map, and
    /// a redundant (both) map are all complete; a minimal or default map is
    /// not.
    #[test]
    fn fr_pbr_002_texture_channel_map_completeness() {
        assert!(!TextureChannelMap::default().is_complete());
        assert!(!TextureChannelMap::minimal("a.png", "n.png").is_complete());

        let standalone = TextureChannelMap::standalone("a.png", "n.png", "mr.png", "ao.png");
        assert!(standalone.is_complete());

        let orm = TextureChannelMap::orm_packed("a.png", "n.png", "orm.png");
        assert!(orm.is_complete());

        // Redundant: both standalone and ORM present — adapter picks
        // most-specific, so completeness stays true.
        let redundant = TextureChannelMap {
            albedo_path: Some("a.png".into()),
            normal_path: Some("n.png".into()),
            mr_path: Some("mr.png".into()),
            ao_path: Some("ao.png".into()),
            orm_path: Some("orm.png".into()),
        };
        assert!(redundant.is_complete());
    }

    /// FR-CIV-PBR-002 — `required_paths` lists every set path in
    /// deterministic order: albedo → normal → mr → ao → orm.
    #[test]
    fn fr_pbr_002_required_paths_deterministic_order() {
        let m = TextureChannelMap {
            albedo_path: Some("a.png".into()),
            normal_path: Some("n.png".into()),
            mr_path: Some("mr.png".into()),
            ao_path: Some("ao.png".into()),
            orm_path: Some("orm.png".into()),
        };
        assert_eq!(
            m.required_paths(),
            vec!["a.png", "n.png", "mr.png", "ao.png", "orm.png"]
        );

        let minimal = TextureChannelMap::minimal("a.png", "n.png");
        assert_eq!(minimal.required_paths(), vec!["a.png", "n.png"]);
    }

    /// FR-CIV-PBR-002 — `TextureChannelMap` round-trips through serde
    /// `ron` so the manifest loader can persist plans to disk.
    #[test]
    fn fr_pbr_002_texture_channel_map_ron_round_trip() {
        let m = TextureChannelMap::orm_packed("a.png", "n.png", "orm.png");
        let encoded = ron::to_string(&m).expect("serialize");
        let decoded: TextureChannelMap = ron::from_str(&encoded).expect("deserialize");
        assert_eq!(m, decoded);
    }

    // -- FR-CIV-PBR-003 -----------------------------------------------

    /// FR-CIV-PBR-003 — `TriplanarSplatPlan` is keyed by `MaterialId` and
    /// iterates in deterministic `matid`-ascending order regardless of the
    /// order in which layers were `insert`-ed.
    #[test]
    fn fr_pbr_003_triplanar_plan_iterates_in_matid_order() {
        let mut plan = TriplanarSplatPlan::new();
        let ch = TextureChannelMap::standalone("a", "n", "mr", "ao");
        plan.insert(TriplanarLayer::new(MaterialId(7), ch.clone(), 0.5));
        plan.insert(TriplanarLayer::new(MaterialId(2), ch.clone(), 0.5));
        plan.insert(TriplanarLayer::new(MaterialId(5), ch, 0.5));
        let ids: Vec<u16> = plan.iter_ordered().map(|(m, _)| m.0).collect();
        assert_eq!(ids, vec![2, 5, 7]);
    }

    /// FR-CIV-PBR-003 — `normalize_weights` rescales every layer so the
    /// maximum weight is exactly `1.0`. A degenerate all-zero plan is left
    /// alone (we never divide by zero).
    #[test]
    fn fr_pbr_003_normalize_weights_rescales_to_unit_max() {
        let mut plan = TriplanarSplatPlan::new();
        let ch = TextureChannelMap::standalone("a", "n", "mr", "ao");
        plan.insert(TriplanarLayer::new(MaterialId(1), ch.clone(), 0.25));
        plan.insert(TriplanarLayer::new(MaterialId(2), ch.clone(), 1.0));
        plan.insert(TriplanarLayer::new(MaterialId(3), ch, 0.5));
        plan.normalize_weights();
        let max = plan
            .iter_ordered()
            .map(|(_, l)| l.splat_weight)
            .fold(0.0_f32, f32::max);
        assert!(
            (max - 1.0).abs() < 1e-6,
            "max weight must normalize to 1.0, got {max}"
        );
        assert_eq!(plan.layers.get(&MaterialId(2)).unwrap().splat_weight, 1.0);

        // All-zero plan stays all-zero (no NaN / no panic).
        let mut zero = TriplanarSplatPlan::new();
        let ch = TextureChannelMap::minimal("a", "n");
        zero.insert(TriplanarLayer::new(MaterialId(1), ch, 0.0));
        zero.normalize_weights();
        assert_eq!(zero.iter_ordered().count(), 1);
    }

    #[test]
    fn triplanar_layer_with_world_tile_overrides_default() {
        let ch = TextureChannelMap::standalone("a", "n", "mr", "ao");
        let layer = TriplanarLayer::new(MaterialId(1), ch, 0.5);
        assert_eq!(layer.world_tile, 1.0);
        let layer2 = layer.with_world_tile(4.0);
        assert_eq!(layer2.world_tile, 4.0);
    }

    /// FR-CIV-PBR-003 — `is_complete` is `false` for an empty plan and
    /// `true` for a plan whose layers all have a complete channel map.
    #[test]
    fn fr_pbr_003_is_complete_requires_non_empty_and_full_layers() {
        // Empty plan is not complete.
        let plan = TriplanarSplatPlan::new();
        assert!(!plan.is_complete(), "empty plan is not complete");

        // A plan with one minimal (incomplete) channel map is not complete.
        let mut plan = TriplanarSplatPlan::new();
        let ch_min = TextureChannelMap::minimal("a", "n");
        plan.insert(TriplanarLayer::new(MaterialId(1), ch_min, 1.0));
        assert!(!plan.is_complete(), "minimal channel map is not complete");

        // Replace the layer with a complete one; plan is now complete.
        let ch_full = TextureChannelMap::standalone("a", "n", "mr", "ao");
        plan.insert(TriplanarLayer::new(MaterialId(1), ch_full, 1.0));
        assert!(plan.is_complete(), "all layers complete -> plan complete");
    }

    /// FR-CIV-PBR-009 — `triplanar_axis_weights` and `blend_triplanar_pbr`
    /// deterministically blend albedo / roughness / normal across the three
    /// axes and renormalize the normal.
    #[test]
    fn fr_pbr_009_triplanar_pbr_blend_is_deterministic_and_normalized() {
        let world_normal = [2.0, 1.0, 1.0];
        let weights = triplanar_axis_weights(world_normal);
        assert_eq!(weights, [0.5, 0.25, 0.25]);

        let x = TriplanarAxisSample {
            albedo_linear: [1.0, 0.0, 0.0],
            perceptual_roughness: 0.2,
            normal: [1.0, 0.0, 0.0],
        };
        let y = TriplanarAxisSample {
            albedo_linear: [0.0, 1.0, 0.0],
            perceptual_roughness: 0.6,
            normal: [0.0, 1.0, 0.0],
        };
        let z = TriplanarAxisSample {
            albedo_linear: [0.0, 0.0, 1.0],
            perceptual_roughness: 1.0,
            normal: [0.0, 0.0, 1.0],
        };

        let blended = blend_triplanar_pbr(weights, x, y, z);
        assert_eq!(blended.albedo_linear, [0.5, 0.25, 0.25]);
        assert!((blended.perceptual_roughness - 0.5).abs() < 1e-6);
        assert!(
            (blended.normal[0] * blended.normal[0]
                + blended.normal[1] * blended.normal[1]
                + blended.normal[2] * blended.normal[2]
                - 1.0)
                .abs()
                < 1e-6
        );
        assert!(blended.normal[0] > blended.normal[1]);
        assert!(blended.normal[1] >= blended.normal[2]);
    }

    // -- FR-CIV-PBR-004 -----------------------------------------------

    /// FR-CIV-PBR-004 — `GreedyAtlasPlan::allocate` hands out strictly
    /// increasing array layers per call, and `layer_for` returns the same
    /// layer on subsequent lookups (idempotent).
    #[test]
    fn fr_pbr_004_greedy_atlas_allocates_strictly_increasing_layers() {
        let mut plan = GreedyAtlasPlan::new();
        let a = *plan.allocate(MaterialId(10));
        let b = *plan.allocate(MaterialId(20));
        let c = *plan.allocate(MaterialId(30));
        assert_eq!(a.array_layer, 0);
        assert_eq!(b.array_layer, 1);
        assert_eq!(c.array_layer, 2);
        assert_eq!(plan.array_depth, 3);
        assert_eq!(plan.layer_for(MaterialId(20)), Some(1));
        assert_eq!(plan.layer_for(MaterialId(99)), None);
    }

    /// FR-CIV-PBR-004 — `GreedyAtlasPlan` round-trips through `ron` so
    /// the asset-build pipeline can cache it next to the glTF manifests.
    #[test]
    fn fr_pbr_004_greedy_atlas_ron_round_trip() {
        let mut plan = GreedyAtlasPlan::new();
        plan.allocate(MaterialId(1));
        plan.allocate(MaterialId(2));
        let encoded = ron::to_string(&plan).expect("serialize");
        let decoded: GreedyAtlasPlan = ron::from_str(&encoded).expect("deserialize");
        assert_eq!(plan, decoded);
    }

    // -- FR-CIV-PBR-006 -----------------------------------------------

    /// FR-CIV-PBR-006 — `ColorSpacePolicy::strict` is the spec-mandated
    /// policy: color = sRGB, data = linear, and `for_channel` mirrors that
    /// for every `PbrChannel` variant.
    #[test]
    fn fr_pbr_006_color_space_policy_strict_per_channel() {
        let p = ColorSpacePolicy::strict();
        assert_eq!(p.color, ColorSpace::Srgb);
        assert_eq!(p.data, ColorSpace::Linear);
        assert_eq!(p.for_channel(PbrChannel::Albedo), ColorSpace::Srgb);
        assert_eq!(p.for_channel(PbrChannel::Normal), ColorSpace::Linear);
        assert_eq!(p.for_channel(PbrChannel::Metallic), ColorSpace::Linear);
        assert_eq!(p.for_channel(PbrChannel::Roughness), ColorSpace::Linear);
        assert_eq!(
            p.for_channel(PbrChannel::AmbientOcclusion),
            ColorSpace::Linear
        );
    }

    /// FR-CIV-PBR-006 — `Default::default()` equals `strict()` so the
    /// Bevy adapter can construct the policy with `..Default::default()`
    /// and inherit the spec-mandated mapping.
    #[test]
    fn fr_pbr_006_color_space_policy_default_matches_strict() {
        assert_eq!(ColorSpacePolicy::default(), ColorSpacePolicy::strict());
    }
}
