//! TODO(FR): PBR material system module stub.

/// Atlas slice for material texturing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasSlice;

/// Attestation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttestationError;

/// Build flavour variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFlavour {
    /// TODO
    Default,
}

/// CC0 license source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cc0Source;

/// Color space identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// TODO
    Linear,
}

/// Color space policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorSpacePolicy;

/// Greedy atlas packing plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreedyAtlasPlan;

/// License attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LicenseAttestation;

/// LOD distance configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodDistanceConfig;

/// LOD render plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodRenderPlan;

/// Manifest error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestError;

/// Material mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialMode {
    /// TODO
    Opaque,
}

/// Material override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialOverride;

/// Material seed manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialSeedManifest;

/// Missing texture policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingTexturePolicy {
    /// TODO
    Default,
}

/// Missing texture report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingTextureReport;

/// PBR channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbrChannel {
    /// TODO
    Albedo,
}

/// Policy action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyAction {
    /// TODO
    Accept,
}

/// Render mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// TODO
    Forward,
}

/// Runtime action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAction {
    /// TODO
    Load,
}

/// Texture channel map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureChannelMap;

/// Triplanar layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriplanarLayer;

/// Triplanar splat plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriplanarSplatPlan;

/// PBR manifest schema version.
pub const SCHEMA_VERSION: u32 = 1;
