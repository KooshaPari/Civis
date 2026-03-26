using UnityEditor;
using UnityEngine;
using System.IO;
using System.Collections.Generic;
using System.Linq;
using YamlDotNet.Serialization;

public class DINOForgeBundleBuilder
{
    private static readonly string RepoRoot = @"C:\Users\koosh\Dino";

    [MenuItem("DINOForge/Build All Bundles")]
    public static void BuildAllBundles()
    {
        Debug.Log("[DINOForge] Starting asset bundle build...");

        string[] packNames = {
            "warfare-starwars",
            "warfare-modern"
        };

        foreach (string pack in packNames)
        {
            BuildPack(pack);
        }

        Debug.Log("[DINOForge] All asset bundles built successfully.");
        EditorApplication.Exit(0);
    }

    private static void BuildPack(string packName)
    {
        Debug.Log($"[DINOForge] Building pack: {packName}");

        string packRoot = Path.Combine(RepoRoot, "packs", packName);
        string bundleOutDir = Path.Combine(packRoot, "assets", "bundles");

        // Create output directory
        if (!Directory.Exists(bundleOutDir))
        {
            Directory.CreateDirectory(bundleOutDir);
        }

        // Collect all visual_asset IDs from YAML files
        var visualAssets = CollectVisualAssets(packRoot);
        Debug.Log($"[DINOForge] Found {visualAssets.Count} visual assets in {packName}");

        // Create stub prefabs for missing assets
        CreateStubPrefabs(packName, visualAssets);

        // Build asset bundles
        var manifest = BuildPipeline.BuildAssetBundles(
            bundleOutDir,
            BuildAssetBundleOptions.ChunkBasedCompression,
            BuildTarget.StandaloneWindows64
        );

        if (manifest != null)
        {
            Debug.Log($"[DINOForge] Successfully built bundles for {packName}");
            Debug.Log($"[DINOForge] Bundles saved to: {bundleOutDir}");
        }
        else
        {
            Debug.LogError($"[DINOForge] Failed to build bundles for {packName}");
        }
    }

    private static HashSet<string> CollectVisualAssets(string packRoot)
    {
        var assets = new HashSet<string>();

        // Search for visual_asset references in YAML files
        string[] yamlDirs = { "units", "buildings", "factions" };

        foreach (string dir in yamlDirs)
        {
            string searchDir = Path.Combine(packRoot, dir);
            if (!Directory.Exists(searchDir))
                continue;

            var yamlFiles = Directory.GetFiles(searchDir, "*.yaml", SearchOption.AllDirectories)
                .Concat(Directory.GetFiles(searchDir, "*.yml", SearchOption.AllDirectories));

            foreach (var file in yamlFiles)
            {
                try
                {
                    string content = File.ReadAllText(file);
                    var lines = content.Split('\n');
                    foreach (var line in lines)
                    {
                        if (line.Contains("visual_asset:"))
                        {
                            var parts = line.Split(':');
                            if (parts.Length > 1)
                            {
                                string asset = parts[1].Trim();
                                if (!string.IsNullOrEmpty(asset) && !asset.StartsWith("#"))
                                {
                                    assets.Add(asset);
                                }
                            }
                        }
                    }
                }
                catch (System.Exception ex)
                {
                    Debug.LogWarning($"[DINOForge] Error reading {file}: {ex.Message}");
                }
            }
        }

        return assets;
    }

    private static void CreateStubPrefabs(string packName, HashSet<string> visualAssets)
    {
        string prefabDir = Path.Combine("Assets", "DINOForge", packName);
        if (!Directory.Exists(prefabDir))
        {
            Directory.CreateDirectory(prefabDir);
        }

        foreach (var assetId in visualAssets)
        {
            string prefabPath = Path.Combine(prefabDir, $"{assetId}.prefab");

            // Skip if prefab already exists
            if (File.Exists(prefabPath))
            {
                Debug.Log($"[DINOForge] Prefab already exists: {assetId}");
                continue;
            }

            // Create a stub prefab (simple capsule)
            GameObject go = GameObject.CreatePrimitive(PrimitiveType.Capsule);
            go.name = assetId;

            // Clean up unnecessary components
            var collider = go.GetComponent<CapsuleCollider>();
            if (collider != null)
                Object.DestroyImmediate(collider);

            // Set up material with faction-based color
            var mat = new Material(Shader.Find("Standard"));
            mat.color = GetFactionColor(packName);
            var renderer = go.GetComponent<Renderer>();
            if (renderer != null)
                renderer.material = mat;

            // Save as prefab
            PrefabUtility.SaveAsPrefabAsset(go, prefabPath);

            // Set asset bundle name
            var importer = AssetImporter.GetAtPath(prefabPath) as AssetImporter;
            if (importer != null)
            {
                importer.assetBundleName = assetId;
                importer.SaveAndReimport();
            }

            Object.DestroyImmediate(go);
            Debug.Log($"[DINOForge] Created prefab: {assetId}");
        }

        AssetDatabase.Refresh();
        AssetDatabase.SaveAssets();
    }

    private static Color GetFactionColor(string packName)
    {
        // Return faction-specific colors for visual distinction
        if (packName.Contains("starwars"))
        {
            // Republic blue/white
            return new Color(0.96f, 0.96f, 0.96f);
        }
        else if (packName.Contains("modern"))
        {
            // Modern warfare gray/blue
            return new Color(0.4f, 0.4f, 0.5f);
        }
        else
        {
            // Default gray
            return new Color(0.8f, 0.8f, 0.8f);
        }
    }
}
