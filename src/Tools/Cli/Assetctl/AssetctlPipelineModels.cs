#nullable enable
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Filter inputs for <see cref="AssetctlPipeline.Search"/>.
/// </summary>
public sealed class AssetctlSearchFilters
{
    /// <summary>
    /// Optional license filter value, e.g. <c>cc-by</c>.
    /// </summary>
    [JsonPropertyName("license")]
    public string? License { get; set; }

    /// <summary>
    /// Minimum accepted polycount estimate.
    /// </summary>
    [JsonPropertyName("min_poly")]
    public int? MinPoly { get; set; }

    /// <summary>
    /// Maximum accepted polycount estimate.
    /// </summary>
    [JsonPropertyName("max_poly")]
    public int? MaxPoly { get; set; }
}

/// <summary>
/// Scored candidate result for search output.
/// </summary>
public sealed class RankedCandidate
{
    /// <summary>
    /// Candidate referenced by the search.
    /// </summary>
    [JsonPropertyName("candidate")]
    public AssetCandidate Candidate { get; set; } = new();

    /// <summary>
    /// Final combined score.
    /// </summary>
    [JsonPropertyName("score")]
    public double Score { get; set; }

    /// <summary>
    /// Per-dimension score breakdown used for ranking.
    /// </summary>
    [JsonPropertyName("score_breakdown")]
    public Dictionary<string, double> ScoreBreakdown { get; set; } = new();
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.Intake"/>.
/// </summary>
public sealed class AssetctlIntakeResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Assigned pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Path to the generated manifest.
    /// </summary>
    [JsonPropertyName("manifest_path")]
    public string? ManifestPath { get; set; }

    /// <summary>
    /// Candidate that was consumed during intake.
    /// </summary>
    [JsonPropertyName("candidate")]
    public AssetCandidate? Candidate { get; set; }

    /// <summary>
    /// Directory containing raw intake artifacts.
    /// </summary>
    [JsonPropertyName("raw_dir")]
    public string? RawDir { get; set; }
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.Normalize"/>.
/// </summary>
public sealed class AssetctlNormalizeResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Path to working directory.
    /// </summary>
    [JsonPropertyName("working_dir")]
    public string? WorkingDir { get; set; }

    /// <summary>
    /// Manifest path in the working directory.
    /// </summary>
    [JsonPropertyName("working_manifest_path")]
    public string? WorkingManifestPath { get; set; }
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.Validate"/>.
/// </summary>
public sealed class AssetctlValidationResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Technical status after validation.
    /// </summary>
    [JsonPropertyName("technical_status")]
    public string? TechnicalStatus { get; set; }

    /// <summary>
    /// Validation errors.
    /// </summary>
    [JsonPropertyName("errors")]
    public List<string> Errors { get; set; } = new();

    /// <summary>
    /// Path to validation report.
    /// </summary>
    [JsonPropertyName("validation_path")]
    public string? ValidationPath { get; set; }
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.Stylize"/>.
/// </summary>
public sealed class AssetctlStylizeResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Stylization profile applied.
    /// </summary>
    [JsonPropertyName("profile")]
    public string? Profile { get; set; }

    /// <summary>
    /// Stylization metadata report path.
    /// </summary>
    [JsonPropertyName("stylize_report_path")]
    public string? StylizeReportPath { get; set; }

    /// <summary>
    /// JSON palette (for --dry-run mode preview).
    /// </summary>
    [JsonPropertyName("dry_run_palette")]
    public string? DryRunPalette { get; set; }
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.Register"/>.
/// </summary>
public sealed class AssetctlRegisterResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Registry file path.
    /// </summary>
    [JsonPropertyName("registry_path")]
    public string? RegistryPath { get; set; }

    /// <summary>
    /// Total entries in registry after this operation.
    /// </summary>
    [JsonPropertyName("total_registered")]
    public int TotalRegistered { get; set; }
}

/// <summary>
/// Result payload for <see cref="AssetctlPipeline.ExportUnity"/>.
/// </summary>
public sealed class AssetctlExportResult
{
    /// <summary>
    /// Indicates the command outcome.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Optional error or status message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Pipeline asset ID.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string? AssetId { get; set; }

    /// <summary>
    /// Target bundle.
    /// </summary>
    [JsonPropertyName("bundle")]
    public string? Bundle { get; set; }

    /// <summary>
    /// Export directory path.
    /// </summary>
    [JsonPropertyName("export_dir")]
    public string? ExportDir { get; set; }
}

/// <summary>
/// Normalization report from Blender script output.
/// </summary>
internal sealed class NormalizationReport
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("original_polycount")]
    public int OriginalPolycount { get; set; }

    [JsonPropertyName("lod0_polycount")]
    public int Lod0Polycount { get; set; }

    [JsonPropertyName("lod1_polycount")]
    public int Lod1Polycount { get; set; }

    [JsonPropertyName("lod2_polycount")]
    public int Lod2Polycount { get; set; }

    [JsonPropertyName("material_count")]
    public int MaterialCount { get; set; }

    [JsonPropertyName("output_files")]
    public string[] OutputFiles { get; set; } = [];
}

/// <summary>
/// Faction palette for stylization.
/// </summary>
internal sealed class FactionPalette
{
    [JsonPropertyName("faction")]
    public string Faction { get; set; } = "neutral";

    [JsonPropertyName("asset_name")]
    public string AssetName { get; set; } = string.Empty;

    [JsonPropertyName("primary")]
    public string Primary { get; set; } = "#888888";

    [JsonPropertyName("secondary")]
    public string? Secondary { get; set; }

    [JsonPropertyName("accent")]
    public string? Accent { get; set; }

    [JsonPropertyName("visor")]
    public string? Visor { get; set; }

    [JsonPropertyName("steel")]
    public string? Steel { get; set; }

    [JsonPropertyName("roughness")]
    public double Roughness { get; set; } = 0.5;

    [JsonPropertyName("metallic")]
    public double Metallic { get; set; } = 0.0;
}

/// <summary>
/// Stylization report from Blender script output.
/// </summary>
internal sealed class StylizationReport
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("faction")]
    public string? Faction { get; set; }

    [JsonPropertyName("material_count")]
    public int MaterialCount { get; set; }

    [JsonPropertyName("output_files")]
    public string[] OutputFiles { get; set; } = [];
}
