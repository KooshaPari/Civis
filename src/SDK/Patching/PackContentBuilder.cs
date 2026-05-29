using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK.Patching
{
    /// <summary>
    /// Builds the nested content dictionary consumed by <see cref="PatchApplicator"/>
    /// from the raw YAML files on disk for a set of packs.
    ///
    /// Shape produced: <c>packId → sectionName → itemId → propertyName → value</c>.
    /// Only YAML files that can be deserialized as a list or single item dictionary are
    /// included; files that fail to parse are skipped and logged.
    /// </summary>
    internal static class PackContentBuilder
    {
        private static readonly IDeserializer GenericDeserializer = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .IgnoreUnmatchedProperties()
            .Build();

        private static readonly ISerializer GenericSerializer = new SerializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .Build();

        /// <summary>
        /// Reads all discovered YAML files for a pack and returns them as a nested
        /// mutable dictionary.  Each YAML file may contain a list of items or a single
        /// item; in both cases every top-level item is keyed by its <c>id</c> field
        /// (or by a sequential index when no <c>id</c> field is present).
        /// </summary>
        /// <param name="packId">The pack identifier (used as the outer key).</param>
        /// <param name="contentFiles">
        /// All content files to include, grouped by section name.
        /// Key = section name (e.g. "units"), value = list of absolute file paths.
        /// </param>
        /// <param name="log">Optional log callback.</param>
        /// <returns>
        /// Nested mutable dictionary:
        /// <c>packId → sectionName → itemId → { property → value }</c>.
        /// </returns>
        public static Dictionary<string, Dictionary<string, Dictionary<string, object>>> Build(
            string packId,
            IReadOnlyDictionary<string, IReadOnlyList<string>> contentFiles,
            Action<string>? log = null)
        {
            Action<string> logger = log ?? (_ => { });

            // section → itemId → propertyBag
            Dictionary<string, Dictionary<string, object>> sections =
                new Dictionary<string, Dictionary<string, object>>(StringComparer.OrdinalIgnoreCase);

            foreach (KeyValuePair<string, IReadOnlyList<string>> kvp in contentFiles)
            {
                string section = kvp.Key;
                IReadOnlyList<string> files = kvp.Value;

                if (!sections.TryGetValue(section, out Dictionary<string, object>? sectionDict))
                {
                    sectionDict = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase);
                    sections[section] = sectionDict;
                }

                foreach (string filePath in files)
                {
                    LoadFileIntoSection(packId, section, filePath, sectionDict, logger);
                }
            }

            return new Dictionary<string, Dictionary<string, Dictionary<string, object>>>(StringComparer.OrdinalIgnoreCase)
            {
                [packId] = sections
            };
        }

        /// <summary>
        /// Merges multiple per-pack content dictionaries into one unified dictionary.
        /// </summary>
        public static Dictionary<string, Dictionary<string, Dictionary<string, object>>> Merge(
            IEnumerable<Dictionary<string, Dictionary<string, Dictionary<string, object>>>> perPackDicts)
        {
            Dictionary<string, Dictionary<string, Dictionary<string, object>>> merged =
                new Dictionary<string, Dictionary<string, Dictionary<string, object>>>(StringComparer.OrdinalIgnoreCase);

            foreach (Dictionary<string, Dictionary<string, Dictionary<string, object>>> packDict in perPackDicts)
            {
                foreach (KeyValuePair<string, Dictionary<string, Dictionary<string, object>>> packEntry in packDict)
                {
                    if (!merged.ContainsKey(packEntry.Key))
                    {
                        merged[packEntry.Key] = packEntry.Value;
                    }
                    else
                    {
                        // Merge section-by-section
                        foreach (KeyValuePair<string, Dictionary<string, object>> sectionEntry in packEntry.Value)
                        {
                            if (!merged[packEntry.Key].ContainsKey(sectionEntry.Key))
                            {
                                merged[packEntry.Key][sectionEntry.Key] = sectionEntry.Value;
                            }
                            else
                            {
                                foreach (KeyValuePair<string, object> itemEntry in sectionEntry.Value)
                                {
                                    merged[packEntry.Key][sectionEntry.Key][itemEntry.Key] = itemEntry.Value;
                                }
                            }
                        }
                    }
                }
            }

            return merged;
        }

        /// <summary>
        /// Serializes a section's item dictionary back to a YAML string suitable for
        /// re-deserializing as a <c>List&lt;T&gt;</c>.
        /// </summary>
        public static string SerializeSectionToYaml(Dictionary<string, object> sectionItems)
        {
            // Collect all item property-bags as a list so RegistryImportService can
            // deserialize via its existing List<T> path.
            List<object> itemList = new List<object>(sectionItems.Values);
            return GenericSerializer.Serialize(itemList);
        }

        // ----------------------------------------------------------------------------------
        // Private helpers
        // ----------------------------------------------------------------------------------

        private static void LoadFileIntoSection(
            string packId,
            string section,
            string filePath,
            Dictionary<string, object> sectionDict,
            Action<string> log)
        {
            if (!File.Exists(filePath))
            {
                log($"[PackContentBuilder] File not found: {filePath} — skipped.");
                return;
            }

            string yaml;
            try
            {
                yaml = SafeFileIO.ReadText(filePath);
            }
            catch (Exception ex)
            {
                log($"[PackContentBuilder] Failed to read '{filePath}': {ex.Message} — skipped.");
                return;
            }

            if (string.IsNullOrWhiteSpace(yaml))
                return;

            try
            {
                // Try list first
                List<Dictionary<string, object>>? items = null;
                try
                {
                    items = GenericDeserializer.Deserialize<List<Dictionary<string, object>>>(yaml);
                }
                catch (Exception ex) // safe-swallow: list parse failure is expected for single-item YAML; log and fall through
                {
                    log($"[PackContentBuilder] List parse attempt failed for '{filePath}': {ex.Message} — retrying as single item.");
                }

                if (items != null && items.Count > 0)
                {
                    foreach (Dictionary<string, object> item in items)
                    {
                        AddItem(packId, section, filePath, sectionDict, item, log);
                    }

                    return;
                }

                // Single-item fallback
                Dictionary<string, object>? single = GenericDeserializer.Deserialize<Dictionary<string, object>>(yaml);
                if (single != null)
                {
                    AddItem(packId, section, filePath, sectionDict, single, log);
                }
            }
            catch (Exception ex)
            {
                log($"[PackContentBuilder] Failed to parse '{filePath}' for pack '{packId}': {ex.Message} — skipped.");
            }
        }

        private static void AddItem(
            string packId,
            string section,
            string filePath,
            Dictionary<string, object> sectionDict,
            Dictionary<string, object> item,
            Action<string> log)
        {
            if (item == null)
                return;

            // Derive an item ID: prefer "id" field, fall back to sequential key.
            string itemId;
            if (item.TryGetValue("id", out object? idObj) && idObj != null)
            {
                itemId = idObj.ToString() ?? string.Empty;
            }
            else
            {
                // No id field — use a positional key so the item is still reachable via wildcards.
                itemId = $"__item_{sectionDict.Count}";
            }

            if (string.IsNullOrEmpty(itemId))
            {
                log($"[PackContentBuilder] Item in '{filePath}' (pack '{packId}', section '{section}') has empty id — skipping.");
                return;
            }

            // Wrap item properties so nested structures are consistently Dictionary<string,object>.
            Dictionary<string, object> normalizedItem = NormalizeItem(item);
            sectionDict[itemId] = normalizedItem;
        }

        /// <summary>
        /// Recursively normalizes a parsed YAML object to ensure all nested mappings are
        /// <c>Dictionary&lt;string, object&gt;</c> with <see cref="StringComparer.OrdinalIgnoreCase"/>.
        /// YamlDotNet may produce <c>Dictionary&lt;object, object&gt;</c> for untyped YAML maps.
        /// </summary>
        private static Dictionary<string, object> NormalizeItem(Dictionary<string, object> raw)
        {
            Dictionary<string, object> result = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase);
            foreach (KeyValuePair<string, object> kvp in raw)
            {
                result[kvp.Key] = NormalizeValue(kvp.Value);
            }

            return result;
        }

        private static object NormalizeValue(object? value)
        {
            if (value is Dictionary<object, object> objDict)
            {
                Dictionary<string, object> strDict = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase);
                foreach (KeyValuePair<object, object> kvp in objDict)
                {
                    strDict[kvp.Key?.ToString() ?? string.Empty] = NormalizeValue(kvp.Value);
                }

                return strDict;
            }

            if (value is Dictionary<string, object> strDictAlready)
            {
                return NormalizeItem(strDictAlready);
            }

            if (value is List<object> list)
            {
                List<object> normalized = new List<object>(list.Count);
                foreach (object item in list)
                {
                    normalized.Add(NormalizeValue(item));
                }

                return normalized;
            }

            return value ?? string.Empty;
        }
    }
}
