#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using DINOForge.Tools.PackCompiler.Models;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Service for auto-updating game definitions with visual asset references.
    /// Injects visual_asset fields into unit/building YAML definitions.
    /// </summary>
    public class DefinitionUpdateService
    {
        /// <summary>
        /// Update game definitions with visual asset references.
        /// For each asset with UpdateDefinition config, writes the visual_asset path to the YAML file.
        /// </summary>
        public async Task UpdateDefinitionsAsync(
            List<(OptimizedAsset asset, AssetDefinition config)> processedAssets,
            string basePath)
        {
            await Task.Run(() =>
            {
                foreach (var (asset, config) in processedAssets)
                {
                    if (config.UpdateDefinition?.Enabled != true)
                        continue;

                    try
                    {
                        UpdateDefinition(asset, config, basePath);
                    }
                    catch (Exception ex)
                    {
                        throw new InvalidOperationException(
                            $"Failed to update definition for asset '{asset.AssetId}': {ex.Message}",
                            ex
                        );
                    }
                }
            }).ConfigureAwait(false);
        }

        /// <summary>
        /// Update a single game definition file with the visual asset reference.
        /// </summary>
        private void UpdateDefinition(OptimizedAsset asset, AssetDefinition config, string basePath)
        {
            if (config.UpdateDefinition?.File == null)
                return;

            var defFile = Path.Combine(basePath, config.UpdateDefinition.File);

            if (!File.Exists(defFile))
            {
                throw new FileNotFoundException($"Definition file not found: {defFile}");
            }

            var lines = File.ReadAllLines(defFile, Encoding.UTF8);
            var updated = false;

            for (int i = 0; i < lines.Length; i++)
            {
                // Look for the asset ID entry
                if (lines[i].TrimStart().StartsWith($"- id: {config.UpdateDefinition.Id}"))
                {
                    // Find the field to update
                    int fieldIndex = -1;
                    int indentLevel = GetIndentLevel(lines[i]);

                    for (int j = i + 1; j < lines.Length; j++)
                    {
                        int currentIndent = GetIndentLevel(lines[j]);

                        // Stop if we've moved to next definition (same or lower indent)
                        if (lines[j].Trim().Length > 0 && currentIndent <= indentLevel)
                            break;

                        // Find the field
                        if (lines[j].TrimStart().StartsWith(
                            $"{config.UpdateDefinition.Field}:"))
                        {
                            fieldIndex = j;
                            break;
                        }
                    }

                    if (fieldIndex >= 0)
                    {
                        // Update existing field
                        int indent = GetIndentLevel(lines[fieldIndex]);
                        lines[fieldIndex] = new string(' ', indent) +
                            $"{config.UpdateDefinition.Field}: {config.OutputPrefab}";
                        updated = true;
                    }
                    else
                    {
                        // Add new field after ID
                        int insertIndex = i + 1;
                        int indent = GetIndentLevel(lines[i]) + 2;
                        var newLine = new string(' ', indent) +
                            $"{config.UpdateDefinition.Field}: {config.OutputPrefab}";

                        var newLines = lines.ToList();
                        newLines.Insert(insertIndex, newLine);
                        lines = newLines.ToArray();
                        updated = true;
                    }

                    break;
                }
            }

            if (updated)
            {
                File.WriteAllLines(defFile, lines, Encoding.UTF8);
            }
        }

        private int GetIndentLevel(string line)
        {
            int count = 0;
            foreach (char c in line)
            {
                if (c == ' ')
                    count++;
                else
                    break;
            }
            return count;
        }

        /// <summary>
        /// Validate that all definition updates can be performed before execution.
        /// </summary>
        public (bool isValid, List<string> errors, List<string> warnings) ValidateDefinitionUpdates(
            List<(OptimizedAsset asset, AssetDefinition config)> processedAssets,
            string basePath)
        {
            var errors = new List<string>();
            var warnings = new List<string>();

            foreach (var (asset, config) in processedAssets)
            {
                if (config.UpdateDefinition?.Enabled != true)
                    continue;

                // Check: File path specified
                if (config.UpdateDefinition.File == null)
                {
                    errors.Add($"Asset '{asset.AssetId}': UpdateDefinition.File not specified");
                    continue;
                }

                var defFile = Path.Combine(basePath, config.UpdateDefinition.File);

                // Check: File exists
                if (!File.Exists(defFile))
                {
                    errors.Add($"Asset '{asset.AssetId}': Definition file not found: {config.UpdateDefinition.File}");
                    continue;
                }

                // Check: Asset ID specified
                if (config.UpdateDefinition.Id == null)
                {
                    errors.Add($"Asset '{asset.AssetId}': UpdateDefinition.Id not specified");
                    continue;
                }

                // Check: Field name specified
                if (config.UpdateDefinition.Field == null)
                {
                    errors.Add($"Asset '{asset.AssetId}': UpdateDefinition.Field not specified");
                    continue;
                }

                // Check: Asset ID exists in definition file
                var lines = File.ReadAllLines(defFile, Encoding.UTF8);
                bool foundAssetId = lines.Any(l => l.TrimStart().StartsWith($"- id: {config.UpdateDefinition.Id}"));

                if (!foundAssetId)
                {
                    warnings.Add($"Asset '{asset.AssetId}': ID '{config.UpdateDefinition.Id}' not found in {config.UpdateDefinition.File}");
                }
            }

            return (errors.Count == 0, errors, warnings);
        }
    }

}
