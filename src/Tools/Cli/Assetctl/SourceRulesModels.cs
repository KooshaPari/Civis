#nullable enable
using System;
using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Top-level source policy document loaded from <c>manifests/asset-intake/source-rules.yaml</c>.
/// </summary>
public sealed class SourceRulesDocument
{
    /// <summary>
    /// Schema version.
    /// </summary>
    [YamlMember(Alias = "version")]
    public string Version { get; set; } = "1.0";

    /// <summary>
    /// Generation timestamp stored in source policy file.
    /// </summary>
    [YamlMember(Alias = "generated_at_utc")]
    public string? GeneratedAtUtc { get; set; }

    /// <summary>
    /// Policy-level decisions for intake behavior.
    /// </summary>
    [YamlMember(Alias = "policy")]
    public PolicySection Policy { get; set; } = new PolicySection();

    /// <summary>
    /// Ranked source tier definitions.
    /// </summary>
    [YamlMember(Alias = "source_tiers")]
    public List<SourceTier> SourceTiers { get; set; } = new List<SourceTier>();

    /// <summary>
    /// Risk rules by <c>ip_status</c> key.
    /// </summary>
    [YamlMember(Alias = "risk_rules")]
    public Dictionary<string, RiskRule> RiskRules { get; set; } = new Dictionary<string, RiskRule>(StringComparer.Ordinal);

    [YamlMember(Alias = "scoring")]
    public ScoringSection Scoring { get; set; } = new ScoringSection();

    [YamlMember(Alias = "status_penalty")]
    public Dictionary<string, double> StatusPenalty { get; set; } = new Dictionary<string, double>(StringComparer.Ordinal);
}

/// <summary>
/// Scoring weights and multipliers for candidate selection.
/// </summary>
public sealed class ScoringSection
{
    [YamlMember(Alias = "weights")]
    public Dictionary<string, double> Weights { get; set; } = new Dictionary<string, double>(StringComparer.Ordinal);

    [YamlMember(Alias = "status_penalty")]
    public Dictionary<string, double> StatusPenalty { get; set; } = new Dictionary<string, double>(StringComparer.Ordinal);
}

/// <summary>
/// Decision section from source policy.
/// </summary>
public sealed class PolicySection
{
    /// <summary>
    /// Primary goal for policy decisions.
    /// </summary>
    [YamlMember(Alias = "decision_goal")]
    public string DecisionGoal { get; set; } = "private_prototype_only_default";

    /// <summary>
    /// Sources explicitly allowed for automated intake.
    /// </summary>
    [YamlMember(Alias = "allow_release_safe_mark")]
    public List<string> AllowReleaseSafeMark { get; set; } = new List<string>();

    /// <summary>
    /// IP gates that block release.
    /// </summary>
    [YamlMember(Alias = "forbid_release_if_ip_status_not")]
    public List<string> ForbidReleaseIfIpStatusNot { get; set; } = new List<string>();

    /// <summary>
    /// Preferred acquisition order.
    /// </summary>
    [YamlMember(Alias = "prefer_acquisition_order")]
    public List<string> PreferAcquisitionOrder { get; set; } = new List<string>();
}

/// <summary>
/// Source tier definition from policy file.
/// </summary>
public sealed class SourceTier
{
    /// <summary>
    /// Source name.
    /// </summary>
    [YamlMember(Alias = "name")]
    public string Name { get; set; } = string.Empty;

    /// <summary>
    /// Relative source rank from policy.
    /// </summary>
    [YamlMember(Alias = "rank")]
    public int Rank { get; set; }

    /// <summary>
    /// Source role in the pipeline.
    /// </summary>
    [YamlMember(Alias = "role")]
    public string Role { get; set; } = "secondary";

    /// <summary>
    /// Capabilities for automation.
    /// </summary>
    [YamlMember(Alias = "capabilities")]
    public SourceCapabilities? Capabilities { get; set; }

    /// <summary>
    /// Default IP status.
    /// </summary>
    [YamlMember(Alias = "risk_default")]
    public string RiskDefault { get; set; } = "fan_star_wars_private_only";

    /// <summary>
    /// Default acquisition mode.
    /// </summary>
    [YamlMember(Alias = "acquisition_mode")]
    public string AcquisitionMode { get; set; } = "unknown";
}

/// <summary>
/// Capabilities for a source tier.
/// </summary>
public sealed class SourceCapabilities
{
    [YamlMember(Alias = "search")]
    public bool Search { get; set; }

    [YamlMember(Alias = "metadata")]
    public bool Metadata { get; set; }

    [YamlMember(Alias = "api_download")]
    public bool ApiDownload { get; set; }

    [YamlMember(Alias = "formats")]
    public List<string> Formats { get; set; } = new List<string>();
}

/// <summary>
/// Risk rule details for an IP status.
/// </summary>
public sealed class RiskRule
{
    [YamlMember(Alias = "legal_profile")]
    public string LegalProfile { get; set; } = string.Empty;

    [YamlMember(Alias = "release_allowed")]
    public bool ReleaseAllowed { get; set; }

    [YamlMember(Alias = "note")]
    public string Note { get; set; } = string.Empty;
}
