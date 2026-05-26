using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using Microsoft.Extensions.Logging;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Service for detecting pack conflicts, missing dependencies, and load order issues
    /// using the SDK's <see cref="PackDependencyResolver"/>.
    /// </summary>
    public sealed class ConflictDetectionService
    {
        private readonly ILogger<ConflictDetectionService>? _logger;
        private readonly PackDependencyResolver _resolver;

        /// <summary>
        /// Initializes a new instance of <see cref="ConflictDetectionService"/>.
        /// </summary>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public ConflictDetectionService(ILogger<ConflictDetectionService>? logger = null)
        {
            _logger = logger;
            _resolver = new PackDependencyResolver();
        }

        /// <summary>
        /// Analyzes the given packs and returns a comprehensive conflict report.
        /// </summary>
        /// <param name="packs">The packs to analyze for conflicts.</param>
        /// <returns>A detailed conflict report with errors, warnings, and load order.</returns>
        public ConflictReport AnalyzeConflicts(IEnumerable<PackViewModel> packs)
        {
            if (packs == null)
            {
                _logger?.LogWarning("Conflict analysis received null pack list");
                return new ConflictReport();
            }

            List<PackViewModel> packList = packs.ToList();
            _logger?.LogInformation("Analyzing conflicts for {Count} packs", packList.Count);

            // Convert to SDK PackManifest for dependency resolver
            List<PackManifest> manifests = packList
                .Select(p => new PackManifest
                {
                    Id = p.Id,
                    Name = p.Name,
                    Version = p.Version,
                    Author = p.Author,
                    Type = p.Type,
                    Description = p.Description,
                    DependsOn = p.DependsOn.ToList(),
                    ConflictsWith = p.ConflictsWith.ToList(),
                    LoadOrder = p.LoadOrder
                })
                .ToList();

            List<ConflictItem> conflicts = new List<ConflictItem>();
            List<DependencyIssue> missingDependencies = new List<DependencyIssue>();
            List<string> loadOrderErrors = new List<string>();
            List<string> loadOrder = new List<string>();

            // Step 1: Detect direct conflicts (pack A declares conflict with pack B in active set)
            IReadOnlyList<string> conflictMessages = _resolver.DetectConflicts(manifests);
            foreach (string msg in conflictMessages)
            {
                conflicts.Add(new ConflictItem
                {
                    Severity = ConflictSeverity.Error,
                    Message = msg,
                    PackIds = ExtractPackIds(msg)
                });
                _logger?.LogWarning("Conflict detected: {Message}", msg);
            }

            // Step 2: Check for missing dependencies
            HashSet<string> availableIds = manifests
                .Select(m => m.Id)
                .ToHashSet(StringComparer.OrdinalIgnoreCase);

            foreach (PackManifest manifest in manifests)
            {
                foreach (string dep in manifest.DependsOn)
                {
                    if (!availableIds.Contains(dep))
                    {
                        missingDependencies.Add(new DependencyIssue
                        {
                            PackId = manifest.Id,
                            PackName = manifest.Name,
                            MissingDependencyId = dep,
                            Severity = DependencySeverity.Missing
                        });
                        _logger?.LogWarning(
                            "Pack {PackId} is missing dependency: {Dependency}",
                            manifest.Id,
                            dep);
                    }
                }
            }

            // Step 3: Compute load order (will also detect cycles)
            DependencyResult result = _resolver.ComputeLoadOrder(manifests);
            if (!result.IsSuccess)
            {
                foreach (string error in result.Errors)
                {
                    loadOrderErrors.Add(error);
                    _logger?.LogError("Load order error: {Error}", error);
                }
            }
            else
            {
                loadOrder = result.LoadOrder.Select(m => m.Id).ToList();
            }

            return new ConflictReport
            {
                Conflicts = conflicts.AsReadOnly(),
                MissingDependencies = missingDependencies.AsReadOnly(),
                LoadOrderErrors = loadOrderErrors.AsReadOnly(),
                ComputedLoadOrder = loadOrder.AsReadOnly(),
                HasIssues = conflicts.Count > 0 || missingDependencies.Count > 0 || loadOrderErrors.Count > 0
            };
        }

        /// <summary>
        /// Gets dependency tree visualization data for a specific pack.
        /// </summary>
        /// <param name="packId">The pack ID to visualize dependencies for.</param>
        /// <param name="allPacks">All available packs.</param>
        /// <returns>Tree nodes representing the dependency hierarchy.</returns>
        public IReadOnlyList<DependencyTreeNode> GetDependencyTree(
            string packId,
            IEnumerable<PackViewModel> allPacks)
        {
            if (string.IsNullOrEmpty(packId) || allPacks == null)
                return Array.Empty<DependencyTreeNode>();

            List<PackViewModel> packList = allPacks.ToList();
            Dictionary<string, PackViewModel> byId = packList
                .ToDictionary(p => p.Id, StringComparer.OrdinalIgnoreCase);

            HashSet<string> visited = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            List<DependencyTreeNode> result = new List<DependencyTreeNode>();

            BuildTreeNodes(packId, byId, visited, result, 0);

            return result.AsReadOnly();
        }

        private void BuildTreeNodes(
            string packId,
            Dictionary<string, PackViewModel> byId,
            HashSet<string> visited,
            List<DependencyTreeNode> result,
            int depth)
        {
            if (!byId.TryGetValue(packId, out PackViewModel? pack))
                return;

            if (visited.Contains(packId))
            {
                result.Add(new DependencyTreeNode
                {
                    PackId = packId,
                    PackName = $"[Circular] {packId}",
                    Depth = depth,
                    IsCircular = true
                });
                return;
            }

            visited.Add(packId);

            result.Add(new DependencyTreeNode
            {
                PackId = pack.Id,
                PackName = pack.Name,
                Version = pack.Version,
                Depth = depth,
                IsCircular = false
            });

            foreach (string depId in pack.DependsOn)
            {
                BuildTreeNodes(depId, byId, visited, result, depth + 1);
            }
        }

        private static IReadOnlyList<string> ExtractPackIds(string message)
        {
            // Extract pack IDs from conflict messages like "Pack 'X' conflicts with 'Y'"
            List<string> ids = new List<string>();
            string[] parts = message.Split('\'');
            foreach (string part in parts)
            {
                if (!string.IsNullOrWhiteSpace(part) && !part.Contains(" "))
                    ids.Add(part);
            }
            return ids.AsReadOnly();
        }
    }

    /// <summary>
    /// Represents a pack conflict or issue.
    /// </summary>
    public sealed class ConflictItem
    {
        /// <summary>The severity level of the conflict.</summary>
        public ConflictSeverity Severity { get; init; }

        /// <summary>Human-readable description of the conflict.</summary>
        public string Message { get; init; } = "";

        /// <summary>Pack IDs involved in the conflict.</summary>
        public IReadOnlyList<string> PackIds { get; init; } = Array.Empty<string>();
    }

    /// <summary>
    /// Severity levels for conflicts.
    /// </summary>
    public enum ConflictSeverity
    {
        /// <summary>Error-level conflict (blocks loading).</summary>
        Error,

        /// <summary>Warning-level issue (may cause problems).</summary>
        Warning,

        /// <summary>Informational notice.</summary>
        Info
    }

    /// <summary>
    /// Represents a missing or problematic dependency.
    /// </summary>
    public sealed class DependencyIssue
    {
        /// <summary>The pack that has the dependency issue.</summary>
        public string PackId { get; init; } = "";

        /// <summary>The display name of the pack.</summary>
        public string PackName { get; init; } = "";

        /// <summary>The ID of the missing dependency.</summary>
        public string MissingDependencyId { get; init; } = "";

        /// <summary>The severity of the issue.</summary>
        public DependencySeverity Severity { get; init; }
    }

    /// <summary>
    /// Severity levels for dependency issues.
    /// </summary>
    public enum DependencySeverity
    {
        /// <summary>Required dependency is missing.</summary>
        Missing,

        /// <summary>Dependency version is incompatible.</summary>
        Incompatible,

        /// <summary>Dependency is satisfied.</summary>
        OK
    }

    /// <summary>
    /// A node in the dependency tree visualization.
    /// </summary>
    public sealed class DependencyTreeNode
    {
        /// <summary>The pack ID.</summary>
        public string PackId { get; init; } = "";

        /// <summary>The display name of the pack.</summary>
        public string PackName { get; init; } = "";

        /// <summary>The version of the pack.</summary>
        public string Version { get; init; } = "";

        /// <summary>Tree depth level (0 = root).</summary>
        public int Depth { get; init; }

        /// <summary>Whether this node represents a circular dependency.</summary>
        public bool IsCircular { get; init; }
    }

    /// <summary>
    /// Complete report of pack conflict analysis.
    /// </summary>
    public sealed class ConflictReport
    {
        /// <summary>All detected pack conflicts.</summary>
        public IReadOnlyList<ConflictItem> Conflicts { get; init; } = Array.Empty<ConflictItem>();

        /// <summary>All detected missing dependency issues.</summary>
        public IReadOnlyList<DependencyIssue> MissingDependencies { get; init; } = Array.Empty<DependencyIssue>();

        /// <summary>Errors from load order computation (e.g., cycles).</summary>
        public IReadOnlyList<string> LoadOrderErrors { get; init; } = Array.Empty<string>();

        /// <summary>The computed load order for all packs.</summary>
        public IReadOnlyList<string> ComputedLoadOrder { get; init; } = Array.Empty<string>();

        /// <summary>Total count of all issues.</summary>
        public int IssueCount => Conflicts.Count + MissingDependencies.Count + LoadOrderErrors.Count;

        /// <summary>Whether any issues were detected.</summary>
        public bool HasIssues { get; init; }
    }
}
