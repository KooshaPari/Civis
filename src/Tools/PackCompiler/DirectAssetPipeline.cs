using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tools.PackCompiler
{
    /// <summary>
    /// Direct Asset Pipeline Runner - bypasses CLI and Spectre Console for reliable output
    /// </summary>
    public static class DirectAssetPipeline
    {
        /// <summary>
        /// Execute Phase 3A of the asset pipeline (import optimization for Clone Infantry assets).
        /// Loads asset_pipeline.yaml, imports GLB/FBX models, applies optimization, and logs results.
        /// </summary>
        /// <param name="packPath">Path to the pack directory containing asset_pipeline.yaml</param>
        /// <returns>Exit code: 0 on success, non-zero on failure</returns>
        public static async Task<int> RunPhase3A(string packPath)
        {
            var logPath = Path.Combine(packPath, "phase3a_execution.log");
            using var logWriter = new StreamWriter(logPath, append: false);

            try
            {
                logWriter.WriteLine("=== Phase 3A: Clone Infantry Asset Pipeline ===");
                logWriter.WriteLine($"Started: {DateTime.UtcNow:yyyy-MM-dd HH:mm:ss}");
                logWriter.WriteLine($"Pack Path: {packPath}");
                logWriter.WriteLine();

                // Step 1: Load config
                logWriter.WriteLine("Step 1: Loading asset_pipeline.yaml...");
                logWriter.Flush();

                var configPath = Path.Combine(packPath, "asset_pipeline.yaml");
                if (!File.Exists(configPath))
                {
                    logWriter.WriteLine($"ERROR: Config not found at {configPath}");
                    logWriter.Flush();
                    return 1;
                }

                var deserializer = YamlLoader.Deserializer;

                var configYaml = File.ReadAllText(configPath);
                var config = deserializer.Deserialize<AssetPipelineConfig>(configYaml);

                if (config == null)
                {
                    logWriter.WriteLine("ERROR: Failed to parse configuration");
                    logWriter.Flush();
                    return 1;
                }

                logWriter.WriteLine($"OK: Loaded config for pack {config.PackId} v{config.Version}");
                logWriter.WriteLine($"    Phases: {config.Phases.Count}");
                logWriter.WriteLine();

                // Step 2: Import
                logWriter.WriteLine("Step 2: Importing Phase v0_8_1_infantry...");
                logWriter.Flush();

                var importService = new AssetImportService();
                int importSuccess = 0, importFail = 0;
                var importedDir = Path.Combine(packPath, "assets", "imported");
                Directory.CreateDirectory(importedDir);

                if (config.Phases.ContainsKey("v0_8_1_infantry"))
                {
                    var phase = config.Phases["v0_8_1_infantry"];
                    foreach (var asset in phase.Models)
                    {
                        var assetPath = Path.Combine(packPath, config.AssetSettings.BasePath, asset.File);

                        try
                        {
                            if (!File.Exists(assetPath))
                            {
                                logWriter.WriteLine($"  SKIP: {asset.Id} - file not found ({asset.File})");
                                importFail++;
                                continue;
                            }

                            var imported = await importService.ImportAsync(asset.Id, assetPath);
                            var outputPath = Path.Combine(importedDir, $"{asset.Id}.json");
                            var json = System.Text.Json.JsonSerializer.Serialize(imported, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
                            File.WriteAllText(outputPath, json);

                            logWriter.WriteLine($"  OK: {asset.Id} → {Path.GetFileName(outputPath)}");
                            importSuccess++;
                        }
                        catch (Exception ex)
                        {
                            logWriter.WriteLine($"  FAIL: {asset.Id} - {ex.Message}");
                            importFail++;
                        }

                        logWriter.Flush();
                    }
                }

                logWriter.WriteLine($"Import Results: {importSuccess} success, {importFail} failed");
                logWriter.WriteLine();

                // Step 3: Optimize
                logWriter.WriteLine("Step 3: Optimizing assets (LOD generation)...");
                logWriter.Flush();

                var optimizeService = new AssetOptimizationService();
                int optimizeSuccess = 0, optimizeFail = 0;
                var optimizedDir = Path.Combine(packPath, "assets", "optimized");
                Directory.CreateDirectory(optimizedDir);

                var importedFiles = Directory.GetFiles(importedDir, "*.json");
                foreach (var importedFile in importedFiles)
                {
                    try
                    {
                        var importedJson = File.ReadAllText(importedFile);
                        var imported = System.Text.Json.JsonSerializer.Deserialize<ImportedAsset>(importedJson);

                        if (imported == null)
                        {
                            logWriter.WriteLine($"  SKIP: {Path.GetFileNameWithoutExtension(importedFile)} - failed to deserialize");
                            optimizeFail++;
                            continue;
                        }

                        // Build a default AssetDefinition for legacy optimize path
                        var defaultDef = new AssetDefinition
                        {
                            Id = imported.AssetId,
                            File = imported.SourcePath,
                            Type = "unit",
                            Faction = "unknown",
                            PolyCountTarget = imported.Mesh.TriangleCount,
                            Scale = 1.0f,
                            LOD = new LODDefinition
                            {
                                Enabled = true,
                                Levels = new List<int> { 100, 60, 30 },
                                ScreenSizes = new List<int> { 100, 50, 20 }
                            },
                            Material = "default",
                            AddressableKey = imported.AssetId,
                            OutputPrefab = $"prefabs/{imported.AssetId}.prefab"
                        };
                        var optimized = await optimizeService.OptimizeAsync(imported, defaultDef);
                        var outputPath = Path.Combine(optimizedDir, $"{imported.AssetId}_optimized.json");
                        var json = System.Text.Json.JsonSerializer.Serialize(optimized, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
                        File.WriteAllText(outputPath, json);

                        logWriter.WriteLine($"  OK: {imported.AssetId} → {Path.GetFileName(outputPath)}");
                        optimizeSuccess++;
                    }
                    catch (Exception ex)
                    {
                        logWriter.WriteLine($"  FAIL: {Path.GetFileNameWithoutExtension(importedFile)} - {ex.Message}");
                        optimizeFail++;
                    }

                    logWriter.Flush();
                }

                logWriter.WriteLine($"Optimize Results: {optimizeSuccess} success, {optimizeFail} failed");
                logWriter.WriteLine();

                // Step 4: Generate Prefabs
                logWriter.WriteLine("Step 4: Generating prefabs...");
                logWriter.Flush();

                var prefabService = new PrefabGenerationService();
                int prefabSuccess = 0, prefabFail = 0;
                var prefabDir = Path.Combine(packPath, "assets", "prefabs");
                Directory.CreateDirectory(prefabDir);

                var optimizedFiles = Directory.GetFiles(optimizedDir, "*.json");
                foreach (var optimizedFile in optimizedFiles)
                {
                    try
                    {
                        var optimizedJson = File.ReadAllText(optimizedFile);
                        var optimized = System.Text.Json.JsonSerializer.Deserialize<OptimizedAsset>(optimizedJson);

                        if (optimized == null)
                        {
                            logWriter.WriteLine($"  SKIP: {Path.GetFileNameWithoutExtension(optimizedFile)} - failed to deserialize");
                            prefabFail++;
                            continue;
                        }

                        var prefabPath = Path.Combine(prefabDir, $"{optimized.AssetId}.prefab");
                        // Build a minimal AssetDefinition for prefab generation
                        var prefabDef = new AssetDefinition
                        {
                            Id = optimized.AssetId,
                            File = string.Empty,
                            Type = "unit",
                            Faction = "unknown",
                            PolyCountTarget = optimized.LOD0.TriangleCount,
                            Scale = 1.0f,
                            LOD = new LODDefinition
                            {
                                Enabled = true,
                                Levels = new List<int> { 100, 60, 30 },
                                ScreenSizes = new List<int> { 100, 50, 20 }
                            },
                            Material = "default",
                            AddressableKey = optimized.AssetId,
                            OutputPrefab = $"prefabs/{optimized.AssetId}.prefab"
                        };
                        await prefabService.GeneratePrefabAsync(optimized, prefabDef, prefabPath);

                        logWriter.WriteLine($"  OK: {optimized.AssetId} → {Path.GetFileName(prefabPath)}");
                        prefabSuccess++;
                    }
                    catch (Exception ex)
                    {
                        logWriter.WriteLine($"  FAIL: {Path.GetFileNameWithoutExtension(optimizedFile)} - {ex.Message}");
                        prefabFail++;
                    }

                    logWriter.Flush();
                }

                logWriter.WriteLine($"Prefab Results: {prefabSuccess} success, {prefabFail} failed");
                logWriter.WriteLine();

                // Summary
                logWriter.WriteLine("=== Phase 3A Complete ===");
                logWriter.WriteLine($"Import:   {importSuccess}/{importSuccess + importFail} assets");
                logWriter.WriteLine($"Optimize: {optimizeSuccess}/{optimizeSuccess + optimizeFail} assets");
                logWriter.WriteLine($"Prefabs:  {prefabSuccess}/{prefabSuccess + prefabFail} assets");
                logWriter.WriteLine($"Completed: {DateTime.UtcNow:yyyy-MM-dd HH:mm:ss}");

                logWriter.Flush();

                return (importFail + optimizeFail + prefabFail) == 0 ? 0 : 1;
            }
            catch (Exception ex)
            {
                logWriter.WriteLine($"FATAL ERROR: {ex.Message}");
                logWriter.WriteLine(ex.StackTrace);
                logWriter.Flush();
                return 1;
            }
        }
    }
}
