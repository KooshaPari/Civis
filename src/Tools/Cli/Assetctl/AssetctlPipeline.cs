#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Pre-implementation asset intake pipeline used by <c>assetctl</c> commands.
/// </summary>
internal sealed class AssetctlPipeline
{
    private const string DefaultSourceRulesPath = "manifests/asset-intake/source-rules.yaml";
    private const string DefaultSchemaPath = "schemas/asset-manifest.schema.json";
    private static readonly string[] SupportedBundles = ["unit", "vehicle", "prop"];

    /// <summary>
    /// Lazy-initialized YAML deserializer to avoid YamlDotNet 16.x static type initialization
    /// deadlock on Windows .NET 8.0 with System.CommandLine. The DeserializerBuilder.Build()
    /// call is deferred until first use, breaking the static initialization chain that causes
    /// the hang in CLI context.
    /// </summary>
    private static readonly Lazy<IDeserializer> DeserializerLazy = new(() =>
        new DeserializerBuilder().Build(), LazyThreadSafetyMode.ExecutionAndPublication);

    private readonly SourceRulesDocument _rules;
    private readonly NJsonSchemaValidator _schemaValidator;
    private readonly JsonSerializerOptions _jsonOptions = new() { WriteIndented = true };
    private readonly AssetCatalogStore? _catalogStore;
    private readonly LocalSourceAdapter? _localSourceAdapter;
    private readonly string _repositoryRoot;

    /// <summary>
    /// Initializes a new instance of the <see cref="AssetctlPipeline"/> class with a catalog store.
    /// </summary>
    /// <param name="catalogStore">The asset catalog store to use for queries. If null, falls back to hardcoded candidates.</param>
    /// <param name="repositoryRoot">The repository root path for local source adapter.</param>
    public AssetctlPipeline(AssetCatalogStore? catalogStore = null, string? repositoryRoot = null)
    {
        _rules = LoadRules();
        _schemaValidator = new NJsonSchemaValidator(new Dictionary<string, string>
        {
            ["asset-manifest"] = File.ReadAllText(DefaultSchemaPath)
        });
        _catalogStore = catalogStore;
        _repositoryRoot = repositoryRoot ?? Directory.GetCurrentDirectory();
        _localSourceAdapter = catalogStore is not null ? new LocalSourceAdapter(_repositoryRoot) : null;
    }

    /// <summary>
    /// Search catalog candidates and rank them using policy weights.
    /// </summary>
    public IReadOnlyList<RankedCandidate> Search(string query, string sourceFilter, int limit, AssetctlSearchFilters filters)
    {
        if (limit <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(limit), "Limit must be greater than zero.");
        }

        List<AssetCandidate> catalog = CandidateCatalog();
        HashSet<string> allowedSources = ParseSourceFilter(sourceFilter);
        HashSet<string> allowedLicenses = ParseCsv(filters.License);

        IEnumerable<AssetCandidate> filtered = catalog
            .Where(candidate => allowedSources.Contains(candidate.Source))
            .Where(candidate =>
                string.IsNullOrWhiteSpace(filters.License) ||
                string.Equals(filters.License, "all", StringComparison.OrdinalIgnoreCase) ||
                string.IsNullOrWhiteSpace(candidate.LicenseLabel) ||
                allowedLicenses.Contains(NormalizeLicenseLabel(candidate.LicenseLabel)))
            .Where(candidate =>
                !filters.MaxPoly.HasValue ||
                !candidate.PolycountEstimate.HasValue ||
                candidate.PolycountEstimate.Value <= filters.MaxPoly.Value)
            .Where(candidate =>
                !filters.MinPoly.HasValue ||
                !candidate.PolycountEstimate.HasValue ||
                candidate.PolycountEstimate.Value >= filters.MinPoly.Value)
            .Where(candidate => MatchesQuery(candidate, query));

        List<RankedCandidate> ranked = new();
        foreach (AssetCandidate candidate in filtered)
        {
            Dictionary<string, double> scores = CalculateScores(candidate, query);
            double finalScore = ComputeScore(candidate, scores);
            ranked.Add(new RankedCandidate
            {
                Candidate = candidate,
                Score = finalScore,
                ScoreBreakdown = scores
            });
        }

        return ranked
            .OrderByDescending(item => item.Score)
            .ThenBy(item => item.Candidate.Title)
            .Take(limit)
            .ToList();
    }

    /// <summary>
    /// Creates raw intake artifacts and returns the created manifest path.
    /// </summary>
    public AssetctlIntakeResult Intake(string source, string externalId, string pipelineRoot)
    {
        AssetCandidate? candidate = CandidateCatalog().FirstOrDefault(item =>
            item.Source.Equals(source, StringComparison.OrdinalIgnoreCase) &&
            item.ExternalId.Equals(externalId, StringComparison.OrdinalIgnoreCase));
        if (candidate is null)
        {
            return new AssetctlIntakeResult
            {
                Success = false,
                Message = $"Unknown candidate: {source}:{externalId}"
            };
        }

        SourceTier sourceTier = GetSourceTier(source);
        string assetId = BuildAssetId(source, externalId);
        string assetDir = Path.Combine(pipelineRoot, "raw", assetId);
        Directory.CreateDirectory(assetDir);

        string sourceDownloadPath = Path.Combine(assetDir, $"source_download.{candidate.OriginalFormat}");
        if (!File.Exists(sourceDownloadPath))
        {
            File.WriteAllText(sourceDownloadPath, BuildSourceFileContents(candidate));
        }

        AssetManifest manifest = BuildManifest(candidate, assetId, sourceDownloadPath, sourceTier);
        if (!IsManifestStructurallyValid(manifest, out List<string> parseErrors))
        {
            return new AssetctlIntakeResult
            {
                Success = false,
                Message = string.Join(" | ", parseErrors),
                Candidate = candidate
            };
        }

        string manifestPath = Path.Combine(assetDir, "asset_manifest.json");
        File.WriteAllText(manifestPath, JsonSerializer.Serialize(manifest, _jsonOptions));
        File.WriteAllText(Path.Combine(assetDir, "metadata.json"), JsonSerializer.Serialize(new
        {
            source,
            external_id = externalId,
            source_rules = _rules,
            policy_version = _rules.Version,
            acquired_at_utc = manifest.AcquiredAtUtc,
            raw_source_file = sourceDownloadPath
        }, _jsonOptions));

        return new AssetctlIntakeResult
        {
            Success = true,
            AssetId = assetId,
            ManifestPath = manifestPath,
            Candidate = candidate,
            RawDir = assetDir
        };
    }

    /// <summary>
    /// Normalizes asset via Blender headless CLI with LOD generation.
    /// </summary>
    public AssetctlNormalizeResult Normalize(string assetId, string pipelineRoot, string? blenderPath = null, int targetPolycount = 3000)
    {
        string? manifestPath = FindManifestPath(assetId, pipelineRoot);
        if (manifestPath is null)
        {
            return new AssetctlNormalizeResult
            {
                Success = false,
                Message = $"Manifest not found for {assetId}"
            };
        }

        AssetManifest manifest = ReadManifest(manifestPath);

        // Find source GLB file
        string rawDir = Path.GetDirectoryName(manifestPath) ?? pipelineRoot;
        string sourceGlb = Path.Combine(rawDir, "source_download.glb");
        if (!File.Exists(sourceGlb))
        {
            UpdateManifestError(manifestPath, manifest, "source_download.glb not found - run download first");
            return new AssetctlNormalizeResult
            {
                Success = false,
                Message = "Source GLB not found - download asset first"
            };
        }

        string workingDir = Path.Combine(pipelineRoot, "working", assetId);
        Directory.CreateDirectory(workingDir);

        try
        {
            // Resolve Blender executable
            string blenderExe = ResolveBlenderPath(blenderPath);
            string scriptPath = ResolveNormalizeScript();

            // Run Blender with script
            var psi = new System.Diagnostics.ProcessStartInfo(blenderExe)
            {
                ArgumentList = { "--background", "--python", scriptPath, "--", sourceGlb, workingDir, targetPolycount.ToString() },
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false
            };

            using var proc = System.Diagnostics.Process.Start(psi) ?? throw new InvalidOperationException("Failed to start Blender");
            string stdout = proc.StandardOutput.ReadToEnd();
            string stderr = proc.StandardError.ReadToEnd();
            proc.WaitForExit();

            if (proc.ExitCode != 0)
            {
                string errMsg = $"Blender exited with code {proc.ExitCode}: {stderr.Trim()}";
                UpdateManifestError(manifestPath, manifest, errMsg);
                return new AssetctlNormalizeResult { Success = false, Message = errMsg };
            }

            // Parse normalization_report.json
            string reportPath = Path.Combine(workingDir, "normalization_report.json");
            if (!File.Exists(reportPath))
            {
                UpdateManifestError(manifestPath, manifest, "Blender did not produce normalization_report.json");
                return new AssetctlNormalizeResult { Success = false, Message = "Normalization report not found" };
            }

            NormalizationReport report = JsonSerializer.Deserialize<NormalizationReport>(File.ReadAllText(reportPath))
                ?? throw new InvalidOperationException("Failed to parse normalization report");

            if (!report.Success)
            {
                UpdateManifestError(manifestPath, manifest, "Blender normalization failed");
                return new AssetctlNormalizeResult { Success = false, Message = "Blender script reported failure" };
            }

            // Compute SHA256 of normalized.glb
            string normalizedGlb = Path.Combine(workingDir, "normalized.glb");
            string sha256 = ComputeSha256(normalizedGlb);

            // Update manifest
            manifest.TechnicalStatus = "normalized";
            manifest.LocalPath = Path.Combine("working", assetId, "normalized.glb");
            manifest.PolycountEstimate = report.Lod0Polycount;
            manifest.Sha256 = sha256;
            var notes = manifest.Notes?.ToList() ?? new List<string>();
            notes.Add($"Normalized via Blender. LOD0={report.Lod0Polycount} LOD1={report.Lod1Polycount} LOD2={report.Lod2Polycount} triangles");
            manifest.Notes = notes.ToArray();

            // Write manifest to both locations
            File.WriteAllText(Path.Combine(workingDir, "asset_manifest.json"), JsonSerializer.Serialize(manifest, _jsonOptions));
            File.WriteAllText(manifestPath, JsonSerializer.Serialize(manifest, _jsonOptions));

            return new AssetctlNormalizeResult
            {
                Success = true,
                AssetId = assetId,
                WorkingDir = workingDir,
                WorkingManifestPath = Path.Combine(workingDir, "asset_manifest.json")
            };
        }
        catch (Exception ex)
        {
            UpdateManifestError(manifestPath, manifest, $"Normalization failed: {ex.Message}");
            return new AssetctlNormalizeResult { Success = false, Message = ex.Message };
        }
    }

    /// <summary>
    /// Validates manifest fields and schema compatibility.
    /// </summary>
    public AssetctlValidationResult Validate(string assetId, string pipelineRoot)
    {
        string? manifestPath = FindManifestPath(assetId, pipelineRoot);
        if (manifestPath is null)
        {
            return new AssetctlValidationResult
            {
                Success = false,
                Message = $"Manifest not found for {assetId}"
            };
        }

        AssetManifest manifest = ReadManifest(manifestPath);
        List<string> errors = new();
        errors.AddRange(ValidateManifestBasic(manifest));
        bool schemaValid = IsSchemaValid(manifest, out List<string> schemaErrors);
        if (!schemaValid)
        {
            errors.AddRange(schemaErrors);
        }

        bool isValid = errors.Count == 0;
        manifest.TechnicalStatus = isValid ? "validated" : manifest.TechnicalStatus;
        manifest.ValidationReport = new AssetValidationReport
        {
            Success = isValid,
            Errors = errors.ToArray(),
            Warnings = Array.Empty<string>(),
            CheckedAtUtc = DateTime.UtcNow.ToString("O", CultureInfo.InvariantCulture)
        };
        File.WriteAllText(manifestPath, JsonSerializer.Serialize(manifest, _jsonOptions));
        File.WriteAllText(Path.Combine(Path.GetDirectoryName(manifestPath) ?? pipelineRoot, "validation_report.json"), JsonSerializer.Serialize(manifest.ValidationReport, _jsonOptions));

        return new AssetctlValidationResult
        {
            Success = isValid,
            AssetId = assetId,
            TechnicalStatus = manifest.TechnicalStatus,
            Errors = errors,
            ValidationPath = Path.Combine(Path.GetDirectoryName(manifestPath) ?? pipelineRoot, "validation_report.json")
        };
    }

    /// <summary>
    /// Stylizes asset via Blender with faction-specific palettes.
    /// </summary>
    public AssetctlStylizeResult Stylize(string assetId, string profile, string pipelineRoot,
        string? factionOverride = null, string? blenderPath = null, bool dryRun = false)
    {
        string? manifestPath = FindManifestPath(assetId, pipelineRoot);
        if (manifestPath is null)
        {
            return new AssetctlStylizeResult
            {
                Success = false,
                Message = $"Manifest not found for {assetId}"
            };
        }

        AssetManifest manifest = ReadManifest(manifestPath);

        // Verify prerequisite: must be normalized or validated
        if (manifest.TechnicalStatus != "normalized" && manifest.TechnicalStatus != "validated")
        {
            return new AssetctlStylizeResult
            {
                Success = false,
                Message = $"Asset must be normalized first (current: {manifest.TechnicalStatus})"
            };
        }

        string workingDir = Path.Combine(pipelineRoot, "working", assetId);
        string normalizedGlb = Path.Combine(workingDir, "normalized.glb");
        if (!File.Exists(normalizedGlb))
        {
            return new AssetctlStylizeResult
            {
                Success = false,
                Message = "normalized.glb not found in working directory"
            };
        }

        // Resolve faction from override, manifest tags, or default to neutral
        string faction = factionOverride
            ?? (manifest.Tags?.FirstOrDefault(t => t.Equals("republic", StringComparison.OrdinalIgnoreCase)
                || t.Equals("cis", StringComparison.OrdinalIgnoreCase)
                || t.Equals("neutral", StringComparison.OrdinalIgnoreCase)) ?? "neutral");

        FactionPalette palette = BuildFactionPalette(faction, assetId);

        // If dry run, return palette preview without running Blender
        if (dryRun)
        {
            return new AssetctlStylizeResult
            {
                Success = true,
                AssetId = assetId,
                Profile = profile,
                DryRunPalette = JsonSerializer.Serialize(palette, _jsonOptions)
            };
        }

        // Write palette to temp JSON file for Blender
        string paletteTempPath = Path.Combine(Path.GetTempPath(), $"palette_{assetId}_{Guid.NewGuid():N}.json");
        try
        {
            File.WriteAllText(paletteTempPath, JsonSerializer.Serialize(palette, _jsonOptions));

            // Resolve Blender executable
            string blenderExe = ResolveBlenderPath(blenderPath);
            string scriptPath = ResolveStylizeScript();

            // Run Blender with stylize script
            var psi = new System.Diagnostics.ProcessStartInfo(blenderExe)
            {
                ArgumentList = { "--background", "--python", scriptPath, "--", normalizedGlb, workingDir, paletteTempPath },
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false
            };

            using var proc = System.Diagnostics.Process.Start(psi) ?? throw new InvalidOperationException("Failed to start Blender");
            string stdout = proc.StandardOutput.ReadToEnd();
            string stderr = proc.StandardError.ReadToEnd();
            proc.WaitForExit();

            if (proc.ExitCode != 0)
            {
                string errMsg = $"Blender stylize failed (exit {proc.ExitCode}): {stderr.Trim()}";
                UpdateManifestError(manifestPath, manifest, errMsg);
                return new AssetctlStylizeResult { Success = false, Message = errMsg };
            }

            // Parse stylization_report.json
            string reportPath = Path.Combine(workingDir, "stylization_report.json");
            if (!File.Exists(reportPath))
            {
                UpdateManifestError(manifestPath, manifest, "Blender did not produce stylization_report.json");
                return new AssetctlStylizeResult { Success = false, Message = "Stylization report not found" };
            }

            StylizationReport report = JsonSerializer.Deserialize<StylizationReport>(File.ReadAllText(reportPath))
                ?? throw new InvalidOperationException("Failed to parse stylization report");

            // Update manifest
            manifest.TechnicalStatus = "ready_for_prototype";
            var notes = manifest.Notes?.ToList() ?? new List<string>();
            notes.Add($"Stylized with faction '{faction}' palette (profile: {profile})");
            manifest.Notes = notes.ToArray();

            File.WriteAllText(manifestPath, JsonSerializer.Serialize(manifest, _jsonOptions));
            File.WriteAllText(Path.Combine(workingDir, "asset_manifest.json"), JsonSerializer.Serialize(manifest, _jsonOptions));

            return new AssetctlStylizeResult
            {
                Success = true,
                AssetId = assetId,
                Profile = profile,
                StylizeReportPath = reportPath
            };
        }
        finally
        {
            // Clean up temp palette file
            if (File.Exists(paletteTempPath))
            {
                try { File.Delete(paletteTempPath); }
                catch { /* ignore cleanup errors */ }
            }
        }
    }

    /// <summary>
    /// Adds or updates an entry in <c>asset_index.json</c>.
    /// </summary>
    public AssetctlRegisterResult Register(string assetId, string pipelineRoot)
    {
        string? manifestPath = FindManifestPath(assetId, pipelineRoot);
        if (manifestPath is null)
        {
            return new AssetctlRegisterResult
            {
                Success = false,
                Message = $"Manifest not found for {assetId}"
            };
        }

        AssetManifest manifest = ReadManifest(manifestPath);
        string registryDir = Path.Combine(pipelineRoot, "registry");
        Directory.CreateDirectory(registryDir);
        string registryPath = Path.Combine(registryDir, "asset_index.json");
        List<AssetRegistryRecord> registry = File.Exists(registryPath)
            ? JsonSerializer.Deserialize<List<AssetRegistryRecord>>(File.ReadAllText(registryPath)) ?? new List<AssetRegistryRecord>()
            : new List<AssetRegistryRecord>();

        if (!registry.Any(item => string.Equals(item.AssetId, manifest.AssetId, StringComparison.OrdinalIgnoreCase)))
        {
            registry.Add(new AssetRegistryRecord
            {
                AssetId = manifest.AssetId,
                CanonicalName = manifest.CanonicalName,
                SourcePlatform = manifest.SourcePlatform,
                TechnicalStatus = manifest.TechnicalStatus,
                IpStatus = manifest.IpStatus,
                ReleaseAllowed = manifest.ReleaseAllowed ?? false
            });
            File.WriteAllText(registryPath, JsonSerializer.Serialize(registry, _jsonOptions));
        }

        return new AssetctlRegisterResult
        {
            Success = true,
            AssetId = assetId,
            RegistryPath = registryPath,
            TotalRegistered = registry.Count
        };
    }

    /// <summary>
    /// Exports to <c>assets-pipeline/export/unity/{bundle}</c>.
    /// </summary>
    public AssetctlExportResult ExportUnity(string assetId, string pipelineRoot, string bundle)
    {
        if (!SupportedBundles.Contains(bundle, StringComparer.OrdinalIgnoreCase))
        {
            return new AssetctlExportResult
            {
                Success = false,
                Message = $"Unsupported bundle '{bundle}'"
            };
        }

        string? manifestPath = FindManifestPath(assetId, pipelineRoot);
        if (manifestPath is null)
        {
            return new AssetctlExportResult
            {
                Success = false,
                Message = $"Manifest not found for {assetId}"
            };
        }

        string exportDir = Path.Combine(pipelineRoot, "export", "unity", bundle.ToLowerInvariant(), assetId);
        Directory.CreateDirectory(exportDir);
        string manifestDir = Path.GetDirectoryName(manifestPath) ?? pipelineRoot;

        foreach (string file in new[] { "normalized.blend", "normalized.glb", "preview.png", "asset_manifest.json" })
        {
            string src = Path.Combine(manifestDir, file);
            string dst = Path.Combine(exportDir, file);
            if (File.Exists(src))
            {
                File.Copy(src, dst, true);
            }
        }

        File.WriteAllText(Path.Combine(exportDir, "export_manifest.json"), JsonSerializer.Serialize(new
        {
            asset_id = assetId,
            bundle,
            exported_at_utc = DateTime.UtcNow.ToString("O", CultureInfo.InvariantCulture)
        }, _jsonOptions));

        return new AssetctlExportResult
        {
            Success = true,
            AssetId = assetId,
            Bundle = bundle,
            ExportDir = exportDir
        };
    }

    /// <summary>
    /// Returns loaded source rules.
    /// </summary>
    public SourceRulesDocument ReadRules()
    {
        return _rules;
    }

    private static bool MatchesQuery(AssetCandidate candidate, string query)
    {
        if (string.IsNullOrWhiteSpace(query))
        {
            return true;
        }

        string haystack = string.Join(" ", candidate.Title, candidate.ExternalId, candidate.AuthorName, string.Join(" ", candidate.Tags ?? Array.Empty<string>()));
        string loweredHaystack = haystack.ToLowerInvariant();
        string[] tokens = query
            .Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Select(token => token.ToLowerInvariant())
            .ToArray();

        return tokens.All(token => loweredHaystack.Contains(token, StringComparison.Ordinal));
    }

    private static HashSet<string> ParseCsv(string? text)
    {
        if (string.IsNullOrWhiteSpace(text))
        {
            return new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        }

        return new HashSet<string>(
            text.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Select(NormalizeLicenseLabel),
            StringComparer.OrdinalIgnoreCase);
    }

    private static string NormalizeLicenseLabel(string? license)
    {
        if (string.IsNullOrWhiteSpace(license))
        {
            return string.Empty;
        }

        string normalized = license.ToLowerInvariant().Replace(" ", string.Empty, StringComparison.OrdinalIgnoreCase);
        return normalized switch
        {
            var value when value.StartsWith("cc0", StringComparison.OrdinalIgnoreCase) => "cc0",
            var value when value.StartsWith("cc-by-sa", StringComparison.OrdinalIgnoreCase) || value.StartsWith("ccbysa", StringComparison.OrdinalIgnoreCase) => "cc-by-sa",
            var value when value.StartsWith("cc-by", StringComparison.OrdinalIgnoreCase) || value.StartsWith("ccby", StringComparison.OrdinalIgnoreCase) => "cc-by",
            _ => normalized
        };
    }

    private HashSet<string> ParseSourceFilter(string sourceFilter)
    {
        if (string.IsNullOrWhiteSpace(sourceFilter))
        {
            sourceFilter = "all";
        }

        if (string.Equals("all", sourceFilter, StringComparison.OrdinalIgnoreCase))
        {
            return _rules.SourceTiers
                .Where(item => item.Capabilities == null || item.Capabilities.Search)
                .Select(item => item.Name)
                .ToHashSet(StringComparer.OrdinalIgnoreCase);
        }

        return new HashSet<string>([sourceFilter], StringComparer.OrdinalIgnoreCase);
    }

    private Dictionary<string, double> CalculateScores(AssetCandidate candidate, string query)
    {
        string lowered = query.ToLowerInvariant();

        double styleFit = lowered.Contains("low poly") || lowered.Contains("low-poly")
            ? 0.95
            : 0.74;

        double factionFit = candidate.Tags is null
            ? 0.38
            : (candidate.Tags.Any(tag => lowered.Contains(tag, StringComparison.OrdinalIgnoreCase)) ? 0.90 : 0.56);

        double automationEase = candidate.Source switch
        {
            "sketchfab" => 1.0,
            "blendswap" => 0.75,
            "moddb" => 0.5,
            _ => 0.25
        };

        double topologyQuality = candidate.PolycountEstimate.HasValue
            ? Math.Clamp(1.0 - Math.Min(candidate.PolycountEstimate.Value, 12000) / 16000.0, 0.0, 1.0)
            : 0.60;

        double lowPolyFit = candidate.PolycountEstimate.HasValue
            ? Math.Clamp(1.0 - Math.Min(candidate.PolycountEstimate.Value, 12000) / 10000.0 + 0.2, 0.0, 1.0)
            : 0.60;

        double provenanceConfidence = candidate.ProvenanceConfidence > 0 ? candidate.ProvenanceConfidence : 0.5;

        double ipRisk = _rules.StatusPenalty.TryGetValue(candidate.IpStatus, out double statusPenalty)
            ? statusPenalty
            : 0.85;

        double cleanupCost = candidate.OriginalFormat.Equals("blend", StringComparison.OrdinalIgnoreCase) ? 0.65 : 0.45;

        return new Dictionary<string, double>
        {
            ["style_fit"] = styleFit,
            ["faction_fit"] = factionFit,
            ["automation_ease"] = automationEase,
            ["topology_quality"] = topologyQuality,
            ["low_poly_fit"] = lowPolyFit,
            ["provenance_confidence"] = provenanceConfidence,
            ["ip_risk_penalty"] = ipRisk,
            ["cleanup_cost"] = cleanupCost
        };
    }

    private double ComputeScore(AssetCandidate candidate, Dictionary<string, double> dimensions)
    {
        double styleWeight = _rules.Scoring.Weights.GetValueOrDefault("style_fit", 0.30);
        double factionWeight = _rules.Scoring.Weights.GetValueOrDefault("faction_fit", 0.20);
        double automationWeight = _rules.Scoring.Weights.GetValueOrDefault("automation_ease", 0.15);
        double topologyWeight = _rules.Scoring.Weights.GetValueOrDefault("topology_quality", 0.10);
        double lowPolyWeight = _rules.Scoring.Weights.GetValueOrDefault("low_poly_fit", 0.10);
        double provenanceWeight = _rules.Scoring.Weights.GetValueOrDefault("provenance_confidence", 0.10);
        double ipPenaltyWeight = _rules.Scoring.Weights.GetValueOrDefault("ip_risk_penalty", 0.15);
        double cleanupWeight = _rules.Scoring.Weights.GetValueOrDefault("cleanup_cost", 0.10);

        double score = (dimensions["style_fit"] * styleWeight) +
            (dimensions["faction_fit"] * factionWeight) +
            (dimensions["automation_ease"] * automationWeight) +
            (dimensions["topology_quality"] * topologyWeight) +
            (dimensions["low_poly_fit"] * lowPolyWeight) +
            (dimensions["provenance_confidence"] * provenanceWeight) -
            (dimensions["ip_risk_penalty"] * ipPenaltyWeight) -
            (dimensions["cleanup_cost"] * cleanupWeight);

        if (candidate.ConfidenceScore > 0)
        {
            score += 0.03 * candidate.ConfidenceScore;
        }

        return Math.Clamp(score, 0.0, 1.0);
    }

    private static string BuildAssetId(string source, string externalId)
    {
        string safeExternal = externalId
            .Replace("-", "_", StringComparison.Ordinal)
            .Replace(" ", "_", StringComparison.Ordinal)
            .Replace("/", "_", StringComparison.Ordinal);
        return $"sw_{source}_{safeExternal}".ToLowerInvariant();
    }

    public static string BuildSourceAssetId(string source, string externalId) => BuildAssetId(source, externalId);

    private static string BuildSourceFileContents(AssetCandidate candidate)
    {
        using SHA256 sha = SHA256.Create();
        string hash = Convert.ToHexString(sha.ComputeHash(Encoding.UTF8.GetBytes(candidate.Source + candidate.ExternalId)));
        return $"source={candidate.Source}{Environment.NewLine}id={candidate.ExternalId}{Environment.NewLine}sha={hash.ToLowerInvariant()}{Environment.NewLine}";
    }

    private static string BuildLicenseUrl(string license)
    {
        if (string.IsNullOrWhiteSpace(license))
        {
            return "https://creativecommons.org/";
        }

        return license.ToLowerInvariant() switch
        {
            "cc0" => "https://creativecommons.org/publicdomain/zero/1.0/",
            "cc-by" => "https://creativecommons.org/licenses/by/4.0/",
            "cc-by-sa" => "https://creativecommons.org/licenses/by-sa/4.0/",
            _ => "https://creativecommons.org/"
        };
    }

    private AssetManifest BuildManifest(AssetCandidate candidate, string assetId, string sourcePath, SourceTier sourceTier)
    {
        bool releaseAllowed = _rules.RiskRules.TryGetValue(candidate.IpStatus, out RiskRule? riskRule) && riskRule is not null && riskRule.ReleaseAllowed;
        string normalizedLicense = string.IsNullOrWhiteSpace(candidate.LicenseLabel) ? "CC-BY-4.0" : candidate.LicenseLabel!;
        return new AssetManifest
        {
            AssetId = assetId,
            CanonicalName = candidate.Title,
            FranchiseTag = "star_wars",
            SourcePlatform = candidate.Source,
            SourceUrl = candidate.SourceUrl,
            ExternalId = candidate.ExternalId,
            AuthorName = string.IsNullOrWhiteSpace(candidate.AuthorName) ? "unknown" : candidate.AuthorName,
            LicenseLabel = normalizedLicense,
            LicenseUrl = string.IsNullOrWhiteSpace(candidate.LicenseUrl)
                ? BuildLicenseUrl(normalizedLicense)
                : candidate.LicenseUrl,
            AcquisitionMode = sourceTier.AcquisitionMode,
            AcquiredAtUtc = DateTime.UtcNow.ToString("O", CultureInfo.InvariantCulture),
            OriginalFormat = candidate.OriginalFormat,
            DownloadUrl = candidate.SourceUrl,
            LocalPath = sourcePath,
            PolycountEstimate = candidate.PolycountEstimate,
            TextureSets = candidate.TextureSets,
            Rigged = candidate.Rigged,
            Animated = candidate.Animated,
            Category = candidate.Category,
            Subtype = candidate.Subtype,
            TechnicalStatus = "downloaded",
            IpStatus = candidate.IpStatus,
            ProvenanceConfidence = candidate.ProvenanceConfidence,
            Tags = candidate.Tags,
            Notes = new[]
            {
                "Pre-implementation intake placeholder created.",
                "No distribution policy decision is implied."
            },
            References = new[] { candidate.SourceUrl },
            ReleaseAllowed = releaseAllowed
        };
    }

    private static SourceTier GetSourceTier(SourceRulesDocument rules, string source)
    {
        return rules.SourceTiers.FirstOrDefault(item => item.Name.Equals(source, StringComparison.OrdinalIgnoreCase))
            ?? new SourceTier
            {
                Name = source,
                Role = "reference",
                RiskDefault = "unknown_provenance",
                AcquisitionMode = "manual_copy"
            };
    }

    private SourceTier GetSourceTier(string source) => GetSourceTier(_rules, source);

    private static string? FindManifestPath(string assetId, string pipelineRoot)
    {
        string[] dirs =
        {
            Path.Combine(pipelineRoot, "raw", assetId),
            Path.Combine(pipelineRoot, "working", assetId),
            Path.Combine(pipelineRoot, "export", "unity", "unit", assetId),
            Path.Combine(pipelineRoot, "export", "unity", "vehicle", assetId),
            Path.Combine(pipelineRoot, "export", "unity", "prop", assetId)
        };

        foreach (string dir in dirs)
        {
            string manifestPath = Path.Combine(dir, "asset_manifest.json");
            if (File.Exists(manifestPath))
            {
                return manifestPath;
            }
        }

        return null;
    }

    private static List<string> ValidateManifestBasic(AssetManifest manifest)
    {
        List<string> errors = new();
        if (string.IsNullOrWhiteSpace(manifest.AssetId))
        {
            errors.Add("asset_id is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.CanonicalName))
        {
            errors.Add("canonical_name is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.SourcePlatform))
        {
            errors.Add("source_platform is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.SourceUrl))
        {
            errors.Add("source_url is required");
        }
        else if (!Uri.TryCreate(manifest.SourceUrl, UriKind.Absolute, out _))
        {
            errors.Add("source_url is invalid");
        }

        if (string.IsNullOrWhiteSpace(manifest.LicenseLabel))
        {
            errors.Add("license_label is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.AcquisitionMode))
        {
            errors.Add("acquisition_mode is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.TechnicalStatus))
        {
            errors.Add("technical_status is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.IpStatus))
        {
            errors.Add("ip_status is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.AcquiredAtUtc))
        {
            errors.Add("acquired_at_utc is required");
        }

        return errors;
    }

    private static bool IsManifestStructurallyValid(AssetManifest manifest, out List<string> errors)
    {
        errors = ValidateManifestBasic(manifest);
        return errors.Count == 0;
    }

    private bool IsSchemaValid(AssetManifest manifest, out List<string> errors)
    {
        errors = new List<string>();
        try
        {
            string manifestYaml = new SerializerBuilder().Build().Serialize(manifest);
            ValidationResult result = _schemaValidator.Validate("asset-manifest", manifestYaml);
            if (result.IsValid)
            {
                return true;
            }

            foreach (ValidationError error in result.Errors)
            {
                errors.Add($"{error.Path}: {error.Message}");
            }
        }
        catch (Exception ex)
        {
            errors.Add($"schema_exception:{ex.Message}");
        }

        return errors.Count == 0;
    }

    private static AssetManifest ReadManifest(string manifestPath)
    {
        string text = File.ReadAllText(manifestPath);
        return JsonSerializer.Deserialize<AssetManifest>(text) ?? new AssetManifest();
    }

    private static SourceRulesDocument LoadRules()
    {
        if (!File.Exists(DefaultSourceRulesPath))
        {
            return CreateDefaultRules();
        }

        string yaml = File.ReadAllText(DefaultSourceRulesPath);
        IDeserializer deserializer = DeserializerLazy.Value;
        SourceRulesDocument? rules = deserializer.Deserialize<SourceRulesDocument>(yaml);
        if (rules == null || rules.SourceTiers.Count == 0)
        {
            return CreateDefaultRules();
        }

        if (rules.Scoring == null)
        {
            rules.Scoring = new ScoringSection();
        }

        if (rules.Policy == null)
        {
            rules.Policy = new PolicySection();
        }

        if (rules.RiskRules == null)
        {
            rules.RiskRules = new Dictionary<string, RiskRule>();
        }

        if (rules.StatusPenalty == null)
        {
            rules.StatusPenalty = new Dictionary<string, double>();
        }

        return rules;
    }

    private static SourceRulesDocument CreateDefaultRules()
    {
        return new SourceRulesDocument
        {
            Version = "1.0",
            Policy = new PolicySection
            {
                DecisionGoal = "private_prototype_only_default",
                AllowReleaseSafeMark = new List<string> { "generic_safe" },
                ForbidReleaseIfIpStatusNot = new List<string> { "generic_safe" },
                PreferAcquisitionOrder = new List<string> { "api", "direct_download", "scrape", "browser_automation", "manual_copy" }
            },
            SourceTiers = new List<SourceTier>
            {
                new SourceTier
                {
                    Name = "sketchfab",
                    Rank = 1,
                    Role = "primary",
                    AcquisitionMode = "api",
                    RiskDefault = "fan_star_wars_private_only",
                    Capabilities = new SourceCapabilities
                    {
                        Search = true,
                        Metadata = true,
                        ApiDownload = true,
                        Formats = new List<string> { "glb", "gltf", "usdz" }
                    }
                },
                new SourceTier
                {
                    Name = "blendswap",
                    Rank = 2,
                    Role = "secondary",
                    AcquisitionMode = "scrape",
                    RiskDefault = "fan_star_wars_private_only",
                    Capabilities = new SourceCapabilities
                    {
                        Search = true,
                        Metadata = true,
                        ApiDownload = false,
                        Formats = new List<string> { "blend", "fbx", "obj" }
                    }
                }
            },
            RiskRules = new Dictionary<string, RiskRule>
            {
                ["generic_safe"] = new RiskRule { LegalProfile = "generic_safe", ReleaseAllowed = true, Note = "Low legal risk profile." },
                ["fan_star_wars_private_only"] = new RiskRule { LegalProfile = "fan_work", ReleaseAllowed = false, Note = "Private prototype risk only." },
                ["official_game_mod_tool_reference_only"] = new RiskRule { LegalProfile = "reference_only", ReleaseAllowed = false, Note = "Reference-only." },
                ["unknown_provenance"] = new RiskRule { LegalProfile = "uncertain", ReleaseAllowed = false, Note = "Provenance not complete." },
                ["high_risk_do_not_ship"] = new RiskRule { LegalProfile = "blocked", ReleaseAllowed = false, Note = "Blocked by policy." }
            },
            Scoring = new ScoringSection
            {
                Weights = new Dictionary<string, double>
                {
                    ["style_fit"] = 0.30,
                    ["faction_fit"] = 0.20,
                    ["automation_ease"] = 0.15,
                    ["topology_quality"] = 0.10,
                    ["low_poly_fit"] = 0.10,
                    ["provenance_confidence"] = 0.10,
                    ["ip_risk_penalty"] = 0.15,
                    ["cleanup_cost"] = 0.10
                },
                StatusPenalty = new Dictionary<string, double>
                {
                    ["fan_star_wars_private_only"] = 0.40,
                    ["official_game_mod_tool_reference_only"] = 0.70,
                    ["unknown_provenance"] = 0.95,
                    ["high_risk_do_not_ship"] = 1.20
                }
            },
            StatusPenalty = new Dictionary<string, double>
            {
                ["fan_star_wars_private_only"] = 0.40,
                ["official_game_mod_tool_reference_only"] = 0.70,
                ["unknown_provenance"] = 0.95,
                ["high_risk_do_not_ship"] = 1.20
            }
        };
    }

    /// <summary>
    /// Gets the candidate catalog, using the asset catalog store if available,
    /// otherwise falling back to the hardcoded placeholder list.
    /// </summary>
    private List<AssetCandidate> CandidateCatalog()
    {
        // If we have a local source adapter with a catalog store, query it
        if (_localSourceAdapter is not null)
        {
            try
            {
                IEnumerable<AssetCandidate> localCandidates = _localSourceAdapter.SearchAsync("").GetAwaiter().GetResult();
                List<AssetCandidate> result = localCandidates.ToList();
                if (result.Count > 0)
                {
                    return result;
                }
            }
            catch
            {
                // Fall back to hardcoded list if local adapter fails
            }
        }

        // Fall back to hardcoded placeholder candidates
        return GetHardcodedCandidates();
    }

    /// <summary>
    /// Returns the hardcoded placeholder candidate list.
    /// </summary>
    private static List<AssetCandidate> GetHardcodedCandidates()
    {
        return new List<AssetCandidate>
        {
            new()
            {
                Source = "sketchfab",
                ExternalId = "star-wars-x-wing-a93f607a94d747568371b8910a81fb12",
                Title = "Star Wars X-Wing",
                SourceUrl = "https://sketchfab.com/3d-models/star-wars-x-wing-a93f607a94d747568371b8910a81fb12",
                AuthorName = "unknown",
                LicenseLabel = "CC-BY-4.0",
                LicenseUrl = "https://creativecommons.org/licenses/by/4.0/",
                OriginalFormat = "glb",
                Category = "vehicle",
                Subtype = "fighter",
                Tags = new[] { "republic", "aircraft", "low_poly" },
                PolycountEstimate = 3200,
                TextureSets = 2,
                Rigged = false,
                Animated = false,
                ConfidenceScore = 0.91,
                ProvenanceConfidence = 0.79,
                IpStatus = "fan_star_wars_private_only"
            },
            new()
            {
                Source = "sketchfab",
                ExternalId = "low-poly-tie-fighter-star-wars-4cf6cb507ba54533a50e608f2aadb47c",
                Title = "Low Poly TIE Fighter",
                SourceUrl = "https://sketchfab.com/3d-models/low-poly-tie-fighter-star-wars-4cf6cb507ba54533a50e608f2aadb47c",
                AuthorName = "unknown",
                LicenseLabel = "CC-BY-4.0",
                LicenseUrl = "https://creativecommons.org/licenses/by/4.0/",
                OriginalFormat = "glb",
                Category = "vehicle",
                Subtype = "fighter",
                Tags = new[] { "empire", "aircraft", "low_poly" },
                PolycountEstimate = 2900,
                TextureSets = 2,
                Rigged = false,
                Animated = false,
                ConfidenceScore = 0.89,
                ProvenanceConfidence = 0.74,
                IpStatus = "fan_star_wars_private_only"
            },
            new()
            {
                Source = "sketchfab",
                ExternalId = "star-wars-clone-trooper-phase1-shiny-updated-aae018747b054186bd48e460b0ad1192",
                Title = "Clone Trooper Phase 1",
                SourceUrl = "https://sketchfab.com/3d-models/star-wars-clone-trooper-phase1-shiny-updated-aae018747b054186bd48e460b0ad1192",
                AuthorName = "unknown",
                LicenseLabel = "CC-BY-4.0",
                LicenseUrl = "https://creativecommons.org/licenses/by/4.0/",
                OriginalFormat = "glb",
                Category = "unit_character",
                Subtype = "trooper",
                Tags = new[] { "republic", "infantry", "low_poly" },
                PolycountEstimate = 1600,
                TextureSets = 2,
                Rigged = true,
                Animated = true,
                ConfidenceScore = 0.88,
                ProvenanceConfidence = 0.77,
                IpStatus = "fan_star_wars_private_only"
            },
            new()
            {
                Source = "sketchfab",
                ExternalId = "star-wars-b1-battle-droid-d35313348ba246e892c3eef18dd0aee7",
                Title = "B1 Battle Droid",
                SourceUrl = "https://sketchfab.com/3d-models/star-wars-b1-battle-droid-d35313348ba246e892c3eef18dd0aee7",
                AuthorName = "unknown",
                LicenseLabel = "CC-BY-4.0",
                LicenseUrl = "https://creativecommons.org/licenses/by/4.0/",
                OriginalFormat = "glb",
                Category = "unit_character",
                Subtype = "droid",
                Tags = new[] { "cis", "infantry", "low_poly" },
                PolycountEstimate = 1400,
                TextureSets = 3,
                Rigged = true,
                Animated = false,
                ConfidenceScore = 0.86,
                ProvenanceConfidence = 0.75,
                IpStatus = "fan_star_wars_private_only"
            },
            new()
            {
                Source = "blendswap",
                ExternalId = "sw-x-wing-ccby",
                Title = "Star Wars X-Wing (Blend)",
                SourceUrl = "https://www.blendswap.com/blend/17032",
                AuthorName = "unknown",
                LicenseLabel = "CC-BY-4.0",
                LicenseUrl = "https://creativecommons.org/licenses/by/4.0/",
                OriginalFormat = "blend",
                Category = "vehicle",
                Subtype = "fighter",
                Tags = new[] { "republic", "aircraft" },
                PolycountEstimate = 7200,
                TextureSets = 3,
                Rigged = true,
                Animated = false,
                ConfidenceScore = 0.68,
                ProvenanceConfidence = 0.56,
                IpStatus = "fan_star_wars_private_only"
            },
            new()
            {
                Source = "moddb",
                ExternalId = "swbf2-mod-tools",
                Title = "Battlefront II Mod Tools",
                SourceUrl = "https://www.moddb.com/games/star-wars-battlefront-ii/downloads/star-wars-battlefront-ii-mod-tools-pc",
                AuthorName = "LucasArts",
                LicenseLabel = "Mod Kit Terms",
                LicenseUrl = "https://www.moddb.com",
                OriginalFormat = "zip",
                Category = "misc",
                Subtype = "reference",
                Tags = new[] { "reference" },
                PolycountEstimate = 200,
                TextureSets = 0,
                Rigged = false,
                Animated = false,
                ConfidenceScore = 0.12,
                ProvenanceConfidence = 0.34,
                IpStatus = "official_game_mod_tool_reference_only"
            },
            new()
            {
                Source = "playwright_fallback",
                ExternalId = "forum-unclassified",
                Title = "Archived Candidate",
                SourceUrl = "https://example.com/star-wars-fallback",
                AuthorName = "community",
                LicenseLabel = "Unknown",
                LicenseUrl = "https://creativecommons.org/",
                OriginalFormat = "unknown",
                Category = "misc",
                Subtype = "reference",
                Tags = new[] { "unknown" },
                PolycountEstimate = 5000,
                TextureSets = 1,
                Rigged = false,
                Animated = false,
                ConfidenceScore = 0.35,
                ProvenanceConfidence = 0.42,
                IpStatus = "unknown_provenance"
            }
        };
    }


    // ===== Helper Methods for Normalization and Stylization =====

    private static string ResolveBlenderPath(string? overridePath)
    {
        if (!string.IsNullOrWhiteSpace(overridePath) && File.Exists(overridePath))
            return overridePath;

        string? envPath = Environment.GetEnvironmentVariable("BLENDER_PATH");
        if (!string.IsNullOrWhiteSpace(envPath) && File.Exists(envPath))
            return envPath;

        // Check common Blender install paths
        string[] candidates = new[]
        {
            @"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe",
            @"C:\Program Files\Blender Foundation\Blender 4.2\blender.exe",
            @"C:\Program Files\Blender Foundation\Blender 4.1\blender.exe",
            @"C:\Program Files\Blender Foundation\Blender 4.0\blender.exe",
            @"C:\Program Files\Blender Foundation\Blender 3.6\blender.exe",
            "/Applications/Blender.app/Contents/MacOS/Blender",
            "/usr/bin/blender",
            "/usr/local/bin/blender",
            "blender"
        };

        foreach (string path in candidates)
        {
            if (File.Exists(path))
                return path;
        }

        // Fallback: assume it's on PATH
        return "blender";
    }

    private static string ResolveNormalizeScript()
    {
        // Try relative to CWD
        string relative = Path.Combine("scripts", "blender", "normalize_asset.py");
        if (File.Exists(relative))
            return relative;

        // Try relative to assembly location, walking up
        string? dir = Path.GetDirectoryName(System.Reflection.Assembly.GetExecutingAssembly().Location);
        while (dir is not null)
        {
            string candidate = Path.Combine(dir, "scripts", "blender", "normalize_asset.py");
            if (File.Exists(candidate))
                return candidate;
            dir = Path.GetDirectoryName(dir);
        }

        throw new FileNotFoundException("normalize_asset.py not found in scripts/blender/");
    }

    private static string ResolveStylizeScript()
    {
        // Try relative to CWD
        string relative = Path.Combine("scripts", "blender", "stylize_asset.py");
        if (File.Exists(relative))
            return relative;

        // Try relative to assembly location, walking up
        string? dir = Path.GetDirectoryName(System.Reflection.Assembly.GetExecutingAssembly().Location);
        while (dir is not null)
        {
            string candidate = Path.Combine(dir, "scripts", "blender", "stylize_asset.py");
            if (File.Exists(candidate))
                return candidate;
            dir = Path.GetDirectoryName(dir);
        }

        throw new FileNotFoundException("stylize_asset.py not found in scripts/blender/");
    }

    private static string ComputeSha256(string filePath)
    {
        using SHA256 sha = SHA256.Create();
        using FileStream fs = File.OpenRead(filePath);
        return Convert.ToHexString(sha.ComputeHash(fs)).ToLowerInvariant();
    }

    private void UpdateManifestError(string manifestPath, AssetManifest manifest, string error)
    {
        manifest.TechnicalStatus = "rejected_technical";
        var notes = manifest.Notes?.ToList() ?? new List<string>();
        notes.Add($"ERROR: {error}");
        manifest.Notes = notes.ToArray();
        File.WriteAllText(manifestPath, JsonSerializer.Serialize(manifest, _jsonOptions));
    }

    private static FactionPalette BuildFactionPalette(string faction, string assetId)
    {
        return faction.ToLowerInvariant() switch
        {
            "republic" => new FactionPalette
            {
                Faction = "republic",
                AssetName = assetId,
                Primary = "#F5F5F5",      // White armor
                Secondary = "#1A3A6B",    // Navy stripe
                Accent = "#FFD700",       // Gold trim
                Visor = "#00DDFF",        // Cyan visor
                Roughness = 0.3,
                Metallic = 0.1
            },
            "cis" => new FactionPalette
            {
                Faction = "cis",
                AssetName = assetId,
                Primary = "#C8A87A",      // Tan base
                Secondary = "#5C3D1E",    // Dark brown joints
                Accent = "#CC2222",       // Red eye
                Steel = "#3A4A5A",        // Steel
                Roughness = 0.7,
                Metallic = 0.2
            },
            _ => new FactionPalette
            {
                Faction = "neutral",
                AssetName = assetId,
                Primary = "#888888",
                Roughness = 0.5,
                Metallic = 0.0
            }
        };
    }
}
