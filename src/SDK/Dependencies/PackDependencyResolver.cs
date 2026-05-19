using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.SDK.Dependencies
{
    using DINOForge.SDK;

    /// <summary>
    /// Resolves pack dependency graphs, detects conflicts, computes topological load order,
    /// and checks framework version compatibility.
    /// </summary>
    public sealed class PackDependencyResolver
    {
        /// <summary>
        /// Resolves dependencies for a target pack against a set of available packs.
        /// Returns a load order that satisfies all transitive dependencies.
        /// </summary>
        /// <param name="available">All packs available for dependency resolution.</param>
        /// <param name="target">The pack whose dependencies to resolve.</param>
        /// <returns>A result containing either the resolved load order or errors.</returns>
        public DependencyResult ResolveDependencies(IEnumerable<PackManifest> available, PackManifest target)
        {
            List<PackManifest> availableList = available.ToList();
            HashSet<string> availableIds = new HashSet<string>(availableList.Select(p => p.Id), StringComparer.OrdinalIgnoreCase);

            List<string> missing = target.DependsOn
                .Where(dep => !availableIds.Contains(dep))
                .Select(dep => $"Pack '{target.Id}' requires missing dependency: '{dep}'")
                .ToList();

            if (missing.Count > 0)
                return DependencyResult.Failure(missing);

            // Include the target and all available packs so ComputeLoadOrder can resolve
            // transitive dependencies across the full set.
            IEnumerable<PackManifest> scope = availableList
                .Where(p => !string.Equals(p.Id, target.Id, StringComparison.OrdinalIgnoreCase))
                .Append(target);

            return ComputeLoadOrder(scope);
        }

        /// <summary>
        /// Detects packs that declare conflicts with each other within the active set.
        /// </summary>
        /// <param name="active">The set of currently active packs.</param>
        /// <returns>A list of conflict error messages; empty if no conflicts.</returns>
        public IReadOnlyList<string> DetectConflicts(IEnumerable<PackManifest> active)
        {
            List<PackManifest> activeList = active.ToList();
            HashSet<string> activeIds = new HashSet<string>(activeList.Select(p => p.Id), StringComparer.OrdinalIgnoreCase);
            List<string> errors = new List<string>();

            foreach (PackManifest pack in activeList)
            {
                foreach (string conflictId in pack.ConflictsWith)
                {
                    if (activeIds.Contains(conflictId))
                        errors.Add($"Pack '{pack.Id}' conflicts with active pack '{conflictId}'.");
                }
            }

            return errors.AsReadOnly();
        }

        /// <summary>
        /// Computes a topological load order for the given packs using Kahn's algorithm.
        /// Uses each pack's LoadOrder value as a tiebreaker within the same dependency level.
        /// </summary>
        /// <param name="packs">The packs to order.</param>
        /// <returns>A result containing either the sorted load order or cycle/missing dependency errors.</returns>
        public DependencyResult ComputeLoadOrder(IEnumerable<PackManifest> packs)
        {
            List<PackManifest> packList = packs.ToList();
            Dictionary<string, PackManifest> packById = packList.ToDictionary(p => p.Id, StringComparer.OrdinalIgnoreCase);

            // Build in-degree map and adjacency list (dep -> dependents).
            Dictionary<string, int> inDegree = packList.ToDictionary(p => p.Id, _ => 0, StringComparer.OrdinalIgnoreCase);
            Dictionary<string, List<string>> dependents = packList.ToDictionary(
                p => p.Id,
                _ => new List<string>(),
                StringComparer.OrdinalIgnoreCase);

            List<string> errors = new List<string>();

            foreach (PackManifest pack in packList)
            {
                foreach (string dep in pack.DependsOn)
                {
                    if (!packById.ContainsKey(dep))
                    {
                        errors.Add($"Pack '{pack.Id}' depends on unknown pack '{dep}'.");
                        continue;
                    }
                    dependents[dep].Add(pack.Id);
                    inDegree[pack.Id]++;
                }
            }

            if (errors.Count > 0)
                return DependencyResult.Failure(errors);

            // Kahn's algorithm - use LoadOrder as tiebreaker via sorted queue.
            SortedSet<(int LoadOrder, string Id)> ready = new SortedSet<(int LoadOrder, string Id)>(
                packList.Where(p => inDegree[p.Id] == 0)
                        .Select(p => (p.LoadOrder, p.Id)));

            List<PackManifest> sorted = new List<PackManifest>();

            while (ready.Count > 0)
            {
                (int _, string nextId) = ready.Min;
                ready.Remove(ready.Min);

                sorted.Add(packById[nextId]);

                foreach (string dependentId in dependents[nextId])
                {
                    inDegree[dependentId]--;
                    if (inDegree[dependentId] == 0)
                        ready.Add((packById[dependentId].LoadOrder, dependentId));
                }
            }

            if (sorted.Count != packList.Count)
                return DependencyResult.Failure(new List<string> { "Circular dependency detected among packs." });

            return DependencyResult.Success(sorted);
        }

        /// <summary>
        /// Checks whether a pack's declared framework version requirement is compatible
        /// with the running framework version.
        /// </summary>
        /// <param name="pack">The pack to check.</param>
        /// <param name="frameworkVersion">The current framework version string.</param>
        /// <returns>True if compatible or no version requirement is specified.</returns>
        public bool CheckFrameworkCompatibility(PackManifest pack, string frameworkVersion)
        {
            // Basic string equality check. Semver range parsing will be added later.
            if (string.IsNullOrWhiteSpace(pack.FrameworkVersion))
                return true;

            string required = pack.FrameworkVersion.TrimStart('>', '<', '=', '~', '^', ' ');
            string runningNormalized = frameworkVersion.TrimStart('>', '<', '=', '~', '^', ' ');

            if (string.IsNullOrWhiteSpace(required) || string.IsNullOrWhiteSpace(runningNormalized))
            {
                // Bare operator (e.g. "~") with no version: treat as universal match (compatible with anything)
                return true;
            }
            return string.Equals(required, runningNormalized, StringComparison.OrdinalIgnoreCase);
        }
    }
}
