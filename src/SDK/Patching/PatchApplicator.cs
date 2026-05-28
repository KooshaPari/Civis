using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;

namespace DINOForge.SDK.Patching
{
    /// <summary>
    /// Applies a sequence of <see cref="PatchSet"/>s to a dictionary of loaded pack content.
    /// The content dictionary is keyed by section name (e.g. "units", "factions") and maps to
    /// a nested <c>Dictionary&lt;string, Dictionary&lt;string, object&gt;&gt;</c> where the inner key
    /// is the content item ID.
    ///
    /// Call order: load all packs → call <see cref="Apply"/> once → register content to registries.
    /// If an operation fails (path not found, type mismatch) the error is logged and remaining
    /// operations in that <see cref="PatchSet"/> are skipped; the load is NOT aborted.
    /// </summary>
    public sealed class PatchApplicator
    {
        private readonly Action<string> _log;

        /// <summary>
        /// Initialises a new <see cref="PatchApplicator"/>.
        /// </summary>
        /// <param name="log">Optional log callback (receives human-readable status strings).</param>
        public PatchApplicator(Action<string>? log = null)
        {
            _log = log ?? (_ => { });
        }

        /// <summary>
        /// Applies all patch sets from all loaded packs to the shared content dictionary.
        /// </summary>
        /// <param name="packContents">
        /// Mutable content dictionary keyed by pack ID, then section name ("units", etc.),
        /// then content-item ID, value is a free-form property bag.
        /// Shape: <c>packId → sectionName → itemId → propertyName → value</c>.
        /// </param>
        /// <param name="allPatchSets">
        /// All patch sets declared across all loaded packs, paired with the ID of the pack
        /// that declared them (used for diagnostic logging only).
        /// </param>
        public void Apply(
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> packContents,
            IReadOnlyList<(string SourcePackId, PatchSet PatchSet)> allPatchSets)
        {
            if (packContents == null) throw new ArgumentNullException(nameof(packContents));
            if (allPatchSets == null) throw new ArgumentNullException(nameof(allPatchSets));

            foreach ((string sourcePackId, PatchSet patchSet) in allPatchSets)
            {
                ApplyPatchSet(packContents, patchSet, sourcePackId);
            }
        }

        private void ApplyPatchSet(
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> packContents,
            PatchSet patchSet,
            string sourcePackId)
        {
            if (!packContents.TryGetValue(patchSet.TargetPack, out Dictionary<string, Dictionary<string, object>>? targetSections))
            {
                _log($"[Patching] Skipping patch set from '{sourcePackId}' targeting '{patchSet.TargetPack}': target pack not loaded.");
                return;
            }

            _log($"[Patching] Applying {patchSet.Operations.Count} operation(s) from '{sourcePackId}' → '{patchSet.TargetPack}'");

            foreach (PatchOperation op in patchSet.Operations)
            {
                try
                {
                    ApplyOperation(targetSections, op);
                }
                catch (PatchException ex)
                {
                    _log($"[Patching] FAILED op '{op.Op}' path='{op.Path}' in patch from '{sourcePackId}': {ex.Message}. Skipping remaining ops in this set.");
                    return; // atomic per set: skip rest on first failure
                }
            }
        }

        // ----------------------------------------------------------------------------------
        // Path resolution
        // ----------------------------------------------------------------------------------

        /// <summary>
        /// Parses a patch path like /units/rep_clone_trooper/stats/hp into segments.
        /// Segment[0] = section (e.g. "units"), segment[1] = item id or "*", rest = nested keys.
        /// </summary>
        private static string[] ParsePath(string path)
        {
            if (string.IsNullOrEmpty(path) || path[0] != '/')
                throw new PatchException($"Path must start with '/'. Got: '{path}'");

            string[] segments = path.TrimStart('/').Split('/');
            if (segments.Length < 2)
                throw new PatchException($"Path must have at least two segments (section/item). Got: '{path}'");

            return segments;
        }

        /// <summary>
        /// Resolves the target items described by a path.
        /// When segment[1] is "*", all items in the section are returned.
        /// </summary>
        private static List<Dictionary<string, object>> ResolveTargetItems(
            Dictionary<string, Dictionary<string, object>> sections,
            string[] segments)
        {
            string section = segments[0];
            string itemId = segments[1];

            if (!sections.TryGetValue(section, out Dictionary<string, object>? sectionDict))
                throw new PatchException($"Section '{section}' not found in target pack.");

#pragma warning disable DF0094 // wildcard sentinel, not a version constraint
            if (itemId == "*")
#pragma warning restore DF0094
                return sectionDict.Values.Cast<Dictionary<string, object>>().ToList();

            if (!sectionDict.TryGetValue(itemId, out object? itemObj))
                throw new PatchException($"Item '{itemId}' not found in section '{section}'.");

            if (itemObj is not Dictionary<string, object> itemDict)
                throw new PatchException($"Item '{itemId}' in section '{section}' is not a dictionary (got {itemObj?.GetType().Name ?? "null"}).");

            return new List<Dictionary<string, object>> { itemDict };
        }

        // ----------------------------------------------------------------------------------
        // Operation dispatch
        // ----------------------------------------------------------------------------------

        private void ApplyOperation(Dictionary<string, Dictionary<string, object>> sections, PatchOperation op)
        {
            string normalizedOp = op.Op?.ToLowerInvariant() ?? string.Empty;
            string[] segments = ParsePath(op.Path);

            // Handle remove on a section item directly (e.g. /units/rep_clone_militia)
            if (normalizedOp == "remove" && segments.Length == 2)
            {
                string section = segments[0];
                string itemId = segments[1];

                if (!sections.TryGetValue(section, out Dictionary<string, object>? sectionDict))
                    throw new PatchException($"Section '{section}' not found.");

#pragma warning disable DF0094 // wildcard sentinel, not a version constraint
                if (itemId == "*")
#pragma warning restore DF0094
                {
                    sectionDict.Clear();
                    _log($"[Patching]   remove *  in section '{section}' — cleared all items");
                    return;
                }

                if (!sectionDict.Remove(itemId))
                    throw new PatchException($"Item '{itemId}' not found in section '{section}' for removal.");

                _log($"[Patching]   remove item '{itemId}' from section '{section}'");
                return;
            }

            List<Dictionary<string, object>> targets = ResolveTargetItems(sections, segments);
            string[] nestedKeys = segments.Skip(2).ToArray();

            if (nestedKeys.Length == 0 && normalizedOp != "remove")
                throw new PatchException($"Path must address a field within an item (path has only section/item). Use 'remove' with a 2-segment path to delete an entire item.");

            foreach (Dictionary<string, object> item in targets)
            {
                ApplyToItem(item, nestedKeys, op, normalizedOp);
            }
        }

        private void ApplyToItem(Dictionary<string, object> item, string[] nestedKeys, PatchOperation op, string normalizedOp)
        {
            // Navigate to the parent container of the leaf key.
            Dictionary<string, object> current = item;
            for (int i = 0; i < nestedKeys.Length - 1; i++)
            {
                string key = nestedKeys[i];
                if (!current.TryGetValue(key, out object? next))
                    throw new PatchException($"Intermediate path segment '{key}' not found.");

                if (next is Dictionary<string, object> nextDict)
                {
                    current = nextDict;
                }
                else
                {
                    throw new PatchException($"Intermediate path segment '{key}' is not a dictionary (got {next?.GetType().Name ?? "null"}).");
                }
            }

            string leafKey = nestedKeys[nestedKeys.Length - 1];

            switch (normalizedOp)
            {
                case "replace":
                case "set":
                    ApplyReplace(current, leafKey, op.Value);
                    break;

                case "add":
                    ApplyAdd(current, leafKey, op.Value);
                    break;

                case "remove":
                    ApplyRemoveField(current, leafKey);
                    break;

                case "multiply":
                    ApplyMultiply(current, leafKey, op.Factor);
                    break;

                default:
                    throw new PatchException($"Unknown operation '{op.Op}'. Valid ops: replace, set, add, remove, multiply.");
            }
        }

        // ----------------------------------------------------------------------------------
        // Individual operation implementations
        // ----------------------------------------------------------------------------------

        private void ApplyReplace(Dictionary<string, object> container, string key, object? value)
        {
            if (!container.ContainsKey(key))
                throw new PatchException($"Key '{key}' not found for replace. Use 'add' to create new keys.");

            object? old = container[key];
            container[key] = value ?? string.Empty;
            _log($"[Patching]   replace '{key}': {FormatValue(old)} → {FormatValue(value)}");
        }

        private void ApplyAdd(Dictionary<string, object> container, string key, object? value)
        {
            if (container.TryGetValue(key, out object? existing) && existing is IList existingList)
            {
                // Append to existing list
                existingList.Add(value);
                _log($"[Patching]   add to list '{key}': appended {FormatValue(value)}");
            }
            else
            {
                // Set new key or replace scalar
                container[key] = value ?? string.Empty;
                _log($"[Patching]   add/set '{key}' = {FormatValue(value)}");
            }
        }

        private void ApplyRemoveField(Dictionary<string, object> container, string key)
        {
            if (!container.Remove(key))
                throw new PatchException($"Key '{key}' not found for removal.");

            _log($"[Patching]   remove field '{key}'");
        }

        private void ApplyMultiply(Dictionary<string, object> container, string key, double? factor)
        {
            if (factor == null)
                throw new PatchException("'multiply' operation requires a 'factor' value.");

            if (!container.TryGetValue(key, out object? existing))
                throw new PatchException($"Key '{key}' not found for multiply.");

            double numericValue = ToDouble(existing, key);
            double result = numericValue * factor.Value;

            // Preserve integer representation when the original was integer-typed.
            container[key] = existing is int || existing is long
                ? (object)(int)Math.Round(result)
                : result;

            _log($"[Patching]   multiply '{key}': {FormatValue(existing)} × {factor.Value} = {FormatValue(container[key])}");
        }

        // ----------------------------------------------------------------------------------
        // Helpers
        // ----------------------------------------------------------------------------------

        private static double ToDouble(object? value, string key)
        {
            return value switch
            {
                int i => i,
                long l => l,
                float f => f,
                double d => d,
                string s when double.TryParse(s, NumberStyles.Any, CultureInfo.InvariantCulture, out double parsed) => parsed,
                _ => throw new PatchException($"Key '{key}' is not numeric (got {value?.GetType().Name ?? "null"}, value '{value}').")
            };
        }

        private static string FormatValue(object? v) =>
            v == null ? "(null)" : v.ToString() ?? "(null)";
    }

    /// <summary>
    /// Thrown by <see cref="PatchApplicator"/> when a patch operation cannot be applied.
    /// Signals that the remaining operations in the current <see cref="PatchSet"/> should be
    /// skipped (atomic-per-set semantics). Does not abort the overall content load.
    /// </summary>
    public sealed class PatchException : Exception
    {
        /// <inheritdoc />
        public PatchException(string message) : base(message) { }
    }
}
