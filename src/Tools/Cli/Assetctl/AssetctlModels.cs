#nullable enable
using System.Text.Json.Serialization;
using YamlDotNet.Serialization;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Represents a candidate returned by a source adapter search result.
/// </summary>
public sealed class AssetCandidate
{
    /// <summary>
    /// Source identifier that generated this candidate.
    /// </summary>
    [JsonPropertyName("source")]
    public string Source { get; init; } = string.Empty;

    /// <summary>
    /// Source-native identifier for the candidate.
    /// </summary>
    [JsonPropertyName("external_id")]
    public string ExternalId { get; init; } = string.Empty;

    /// <summary>
    /// Human-readable title for the result.
    /// </summary>
    [JsonPropertyName("title")]
    public string Title { get; init; } = string.Empty;

    /// <summary>
    /// License label as described by the source.
    /// </summary>
    [JsonPropertyName("license_label")]
    public string? LicenseLabel { get; init; }

    /// <summary>
    /// License URL if available.
    /// </summary>
    [JsonPropertyName("license_url")]
    public string? LicenseUrl { get; init; }

    /// <summary>
    /// Source URL for direct reference.
    /// </summary>
    [JsonPropertyName("source_url")]
    public string SourceUrl { get; init; } = string.Empty;

    /// <summary>
    /// Creator credited by the source.
    /// </summary>
    [JsonPropertyName("author_name")]
    public string? AuthorName { get; init; }

    /// <summary>
    /// Candidate category/faction hints from the source.
    /// </summary>
    [JsonPropertyName("category")]
    public string? Category { get; init; }

    /// <summary>
    /// Confidence score used for ordering.
    /// </summary>
    [JsonPropertyName("confidence_score")]
    public double ConfidenceScore { get; init; }

    /// <summary>
    /// Default format for discovered payload.
    /// </summary>
    [JsonPropertyName("original_format")]
    public string OriginalFormat { get; init; } = "glb";

    /// <summary>
    /// Baseline IP-risk classification.
    /// </summary>
    [JsonPropertyName("ip_status")]
    public string IpStatus { get; init; } = "fan_star_wars_private_only";

    /// <summary>
    /// Estimated provenance confidence, from 0 to 1.
    /// </summary>
    [JsonPropertyName("provenance_confidence")]
    public double ProvenanceConfidence { get; init; }

    /// <summary>
    /// Optional tags for sorting and matching.
    /// </summary>
    [JsonPropertyName("tags")]
    public string[]? Tags { get; init; }

    /// <summary>
    /// Optional estimated polycount.
    /// </summary>
    [JsonPropertyName("polycount_estimate")]
    public int? PolycountEstimate { get; init; }

    /// <summary>
    /// Optional texture count metadata.
    /// </summary>
    [JsonPropertyName("texture_sets")]
    public int? TextureSets { get; init; }

    /// <summary>
    /// If the source model is rigged.
    /// </summary>
    [JsonPropertyName("rigged")]
    public bool? Rigged { get; init; }

    /// <summary>
    /// If the source model is animated.
    /// </summary>
    [JsonPropertyName("animated")]
    public bool? Animated { get; init; }

    /// <summary>
    /// Optional subtype hint.
    /// </summary>
    [JsonPropertyName("subtype")]
    public string? Subtype { get; init; }
}

/// <summary>
/// Serializable manifest written at every stage of the pipeline.
/// </summary>
public sealed class AssetManifest
{
    /// <summary>
    /// Stable identifier.
    /// </summary>
    [JsonPropertyName("asset_id")]
    [YamlMember(Alias = "asset_id")]
    public string AssetId { get; set; } = string.Empty;

    /// <summary>
    /// Human-readable canonical name.
    /// </summary>
    [JsonPropertyName("canonical_name")]
    [YamlMember(Alias = "canonical_name")]
    public string CanonicalName { get; set; } = string.Empty;

    /// <summary>
    /// Family classification of source inspiration.
    /// </summary>
    [JsonPropertyName("franchise_tag")]
    [YamlMember(Alias = "franchise_tag")]
    public string FranchiseTag { get; set; } = "star_wars";

    /// <summary>
    /// Source platform name.
    /// </summary>
    [JsonPropertyName("source_platform")]
    [YamlMember(Alias = "source_platform")]
    public string SourcePlatform { get; set; } = string.Empty;

    /// <summary>
    /// Primary source URL.
    /// </summary>
    [JsonPropertyName("source_url")]
    [YamlMember(Alias = "source_url")]
    public string SourceUrl { get; set; } = string.Empty;

    /// <summary>
    /// Source-native asset identifier.
    /// </summary>
    [JsonPropertyName("external_id")]
    [YamlMember(Alias = "external_id")]
    public string ExternalId { get; set; } = string.Empty;

    /// <summary>
    /// Author or modeler name.
    /// </summary>
    [JsonPropertyName("author_name")]
    [YamlMember(Alias = "author_name")]
    public string AuthorName { get; set; } = "unknown";

    /// <summary>
    /// License short label.
    /// </summary>
    [JsonPropertyName("license_label")]
    [YamlMember(Alias = "license_label")]
    public string LicenseLabel { get; set; } = "CC-BY-4.0";

    /// <summary>
    /// License URL.
    /// </summary>
    [JsonPropertyName("license_url")]
    [YamlMember(Alias = "license_url")]
    public string LicenseUrl { get; set; } = string.Empty;

    /// <summary>
    /// How the asset was acquired.
    /// </summary>
    [JsonPropertyName("acquisition_mode")]
    [YamlMember(Alias = "acquisition_mode")]
    public string AcquisitionMode { get; set; } = string.Empty;

    /// <summary>
    /// UTC acquisition timestamp in ISO-8601 format.
    /// </summary>
    [JsonPropertyName("acquired_at_utc")]
    [YamlMember(Alias = "acquired_at_utc")]
    public string AcquiredAtUtc { get; set; } = string.Empty;

    /// <summary>
    /// Original format on intake.
    /// </summary>
    [JsonPropertyName("original_format")]
    [YamlMember(Alias = "original_format")]
    public string? OriginalFormat { get; set; }

    /// <summary>
    /// URL used to download the payload.
    /// </summary>
    [JsonPropertyName("download_url")]
    [YamlMember(Alias = "download_url")]
    public string? DownloadUrl { get; set; }

    /// <summary>
    /// Relative local path where the current source artifact lives.
    /// </summary>
    [JsonPropertyName("local_path")]
    [YamlMember(Alias = "local_path")]
    public string? LocalPath { get; set; }

    /// <summary>
    /// Content hash if computed.
    /// </summary>
    [JsonPropertyName("sha256")]
    [YamlMember(Alias = "sha256")]
    public string? Sha256 { get; set; }

    /// <summary>
    /// Optional estimated polycount.
    /// </summary>
    [JsonPropertyName("polycount_estimate")]
    [YamlMember(Alias = "polycount_estimate")]
    public int? PolycountEstimate { get; set; }

    /// <summary>
    /// Optional texture count metadata.
    /// </summary>
    [JsonPropertyName("texture_sets")]
    [YamlMember(Alias = "texture_sets")]
    public int? TextureSets { get; set; }

    /// <summary>
    /// If the source model is rigged.
    /// </summary>
    [JsonPropertyName("rigged")]
    [YamlMember(Alias = "rigged")]
    public bool? Rigged { get; set; }

    /// <summary>
    /// If the source model is animated.
    /// </summary>
    [JsonPropertyName("animated")]
    [YamlMember(Alias = "animated")]
    public bool? Animated { get; set; }

    /// <summary>
    /// Optional category hint.
    /// </summary>
    [JsonPropertyName("category")]
    [YamlMember(Alias = "category")]
    public string? Category { get; set; }

    /// <summary>
    /// Optional subtype hint.
    /// </summary>
    [JsonPropertyName("subtype")]
    [YamlMember(Alias = "subtype")]
    public string? Subtype { get; set; }

    /// <summary>
    /// Technical lifecycle state.
    /// </summary>
    [JsonPropertyName("technical_status")]
    [YamlMember(Alias = "technical_status")]
    public string TechnicalStatus { get; set; } = string.Empty;

    /// <summary>
    /// IP risk classification.
    /// </summary>
    [JsonPropertyName("ip_status")]
    [YamlMember(Alias = "ip_status")]
    public string IpStatus { get; set; } = "fan_star_wars_private_only";

    /// <summary>
    /// Release-safe computed gate.
    /// </summary>
    [JsonPropertyName("release_allowed")]
    [YamlMember(Alias = "release_allowed")]
    public bool? ReleaseAllowed { get; set; }

    /// <summary>
    /// Provenance confidence score.
    /// </summary>
    [JsonPropertyName("provenance_confidence")]
    [YamlMember(Alias = "provenance_confidence")]
    public double? ProvenanceConfidence { get; set; }

    /// <summary>
    /// Tags used for filtering and matching.
    /// </summary>
    [JsonPropertyName("tags")]
    [YamlMember(Alias = "tags")]
    public string[]? Tags { get; set; }

    /// <summary>
    /// Supplemental notes from pipeline operations.
    /// </summary>
    [JsonPropertyName("notes")]
    [YamlMember(Alias = "notes")]
    public string[]? Notes { get; set; }

    /// <summary>
    /// Supplemental references collected during intake.
    /// </summary>
    [JsonPropertyName("references")]
    [YamlMember(Alias = "references")]
    public string[]? References { get; set; }

    /// <summary>
    /// Validation report generated by command stages.
    /// </summary>
    [JsonPropertyName("validation_report")]
    [YamlMember(Alias = "validation_report")]
    public AssetValidationReport? ValidationReport { get; set; }
}

/// <summary>
/// Per-manifest validation report.
/// </summary>
public sealed class AssetValidationReport
{
    /// <summary>
    /// Whether validation passed.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Validation errors.
    /// </summary>
    [JsonPropertyName("errors")]
    public string[] Errors { get; set; } = Array.Empty<string>();

    /// <summary>
    /// Validation warnings.
    /// </summary>
    [JsonPropertyName("warnings")]
    public string[] Warnings { get; set; } = Array.Empty<string>();

    /// <summary>
    /// UTC timestamp when validation ran.
    /// </summary>
    [JsonPropertyName("checked_at_utc")]
    public string? CheckedAtUtc { get; set; }
}

/// <summary>
/// Asset registry entry for <c>assetctl register</c>.
/// </summary>
public sealed class AssetRegistryRecord
{
    /// <summary>
    /// Stable asset identifier.
    /// </summary>
    [JsonPropertyName("asset_id")]
    public string AssetId { get; set; } = string.Empty;

    /// <summary>
    /// Canonical name.
    /// </summary>
    [JsonPropertyName("canonical_name")]
    public string CanonicalName { get; set; } = string.Empty;

    /// <summary>
    /// Source platform.
    /// </summary>
    [JsonPropertyName("source_platform")]
    public string SourcePlatform { get; set; } = string.Empty;

    /// <summary>
    /// Technical status at registration time.
    /// </summary>
    [JsonPropertyName("technical_status")]
    public string TechnicalStatus { get; set; } = string.Empty;

    /// <summary>
    /// IP risk status.
    /// </summary>
    [JsonPropertyName("ip_status")]
    public string IpStatus { get; set; } = string.Empty;

    /// <summary>
    /// Release gate.
    /// </summary>
    [JsonPropertyName("release_allowed")]
    public bool ReleaseAllowed { get; set; }
}

/// <summary>
/// Reusable response container for command-style result reporting.
/// </summary>
public sealed class AssetctlCommandResponse
{
    /// <summary>
    /// Whether command logic completed successfully.
    /// </summary>
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    /// <summary>
    /// Command name, e.g., <c>search</c> or <c>validate</c>.
    /// </summary>
    [JsonPropertyName("command")]
    public string Command { get; set; } = string.Empty;

    /// <summary>
    /// Human-readable message.
    /// </summary>
    [JsonPropertyName("message")]
    public string? Message { get; set; }

    /// <summary>
    /// Additional payload object.
    /// </summary>
    [JsonPropertyName("data")]
    public object? Data { get; set; }
}

