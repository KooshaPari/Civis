using System;
using System.IO;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Headless AssetBundle builder for DINOForge warfare-starwars pack.
/// Invoked via: Unity.exe -batchmode -nographics -executeMethod BuildAssetBundles.BuildHeadless -quit
/// </summary>
public static class BuildAssetBundles
{
    private const string OutputDir = "AssetBundles";

    public static void BuildHeadless()
    {
        try
        {
            Debug.Log("[BuildAssetBundles] Starting headless AssetBundle build...");

            if (!Directory.Exists(OutputDir))
                Directory.CreateDirectory(OutputDir);

            // Build for Windows Standalone (matches DINO's target platform)
            var manifest = BuildPipeline.BuildAssetBundles(
                OutputDir,
                BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);

            if (manifest == null)
            {
                Debug.LogError("[BuildAssetBundles] Build failed — manifest is null.");
                EditorApplication.Exit(1);
                return;
            }

            string[] bundles = manifest.GetAllAssetBundles();
            Debug.Log($"[BuildAssetBundles] Built {bundles.Length} bundle(s):");
            foreach (string b in bundles)
                Debug.Log($"  {b}");

            Debug.Log("[BuildAssetBundles] Build complete.");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[BuildAssetBundles] Exception: {ex}");
            EditorApplication.Exit(1);
        }
    }
}
