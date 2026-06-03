using System;
using System.IO;
using UnityEditor;
using UnityEngine;
using TMPro;
using TMPro.EditorUtilities;

/// <summary>
/// Headless TMP SDF font bake + AssetBundle build.
/// Usage: Unity.exe -batchmode -nographics -projectPath <proj> -executeMethod BakeTMPFont.BakeAndBundle -quit
/// </summary>
public static class BakeTMPFont
{
    private const string FontTtfPath    = "Assets/Fonts/sw_menu_font.ttf";
    private const string FontAssetPath  = "Assets/Fonts/sw_menu_font_asset.asset";
    private const string BundleKey      = "assets/ui/sw_menu_font_asset";
    private const string OutDir         = "AssetBundles";

    public static void BakeAndBundle()
    {
        try
        {
            Debug.Log("[BakeTMPFont] Starting TMP SDF font bake...");

            // Ensure TMP Essentials are imported
            if (!Directory.Exists("Assets/TextMesh Pro"))
            {
                Debug.Log("[BakeTMPFont] Importing TMP Essentials...");
                TMP_PackageResourceImporter.ImportResources(true, false, false);
                AssetDatabase.Refresh();
            }

            // Load source font
            Font srcFont = AssetDatabase.LoadAssetAtPath<Font>(FontTtfPath);
            if (srcFont == null)
            {
                Debug.LogError($"[BakeTMPFont] Source TTF not found at {FontTtfPath}");
                EditorApplication.Exit(1);
                return;
            }
            Debug.Log($"[BakeTMPFont] Loaded source font: {srcFont.name}");

            // Create or load existing TMP_FontAsset
            TMP_FontAsset fontAsset = AssetDatabase.LoadAssetAtPath<TMP_FontAsset>(FontAssetPath);
            if (fontAsset == null)
            {
                Debug.Log("[BakeTMPFont] Creating new TMP_FontAsset via FontEngine...");

                // Use TMP_FontAsset.CreateFontAsset API (Unity 2021+)
                fontAsset = TMP_FontAsset.CreateFontAsset(
                    srcFont,
                    samplingPointSize: 28,
                    atlasPadding: 4,
                    renderMode: GlyphRenderMode.SDFAA,
                    atlasWidth: 512,
                    atlasHeight: 512,
                    atlasPopulationMode: AtlasPopulationMode.Dynamic,
                    enableMultiAtlasSupport: false
                );

                if (fontAsset == null)
                {
                    Debug.LogError("[BakeTMPFont] TMP_FontAsset.CreateFontAsset returned null.");
                    EditorApplication.Exit(1);
                    return;
                }

                fontAsset.name = "sw_menu_font_asset";

                // Ensure folder exists
                if (!AssetDatabase.IsValidFolder("Assets/Fonts"))
                    AssetDatabase.CreateFolder("Assets", "Fonts");

                AssetDatabase.CreateAsset(fontAsset, FontAssetPath);
                AssetDatabase.SaveAssets();
                AssetDatabase.Refresh();
                Debug.Log($"[BakeTMPFont] Created asset at {FontAssetPath}");
            }
            else
            {
                Debug.Log($"[BakeTMPFont] Loaded existing asset at {FontAssetPath}");
            }

            // Assign bundle name
            var importer = AssetImporter.GetAtPath(FontAssetPath);
            if (importer != null)
            {
                importer.assetBundleName = BundleKey;
                importer.SaveAndReimport();
                Debug.Log($"[BakeTMPFont] Bundle name set: {BundleKey}");
            }
            else
            {
                Debug.LogError("[BakeTMPFont] Could not get AssetImporter for font asset.");
                EditorApplication.Exit(1);
                return;
            }

            // Build bundles
            if (!Directory.Exists(OutDir))
                Directory.CreateDirectory(OutDir);

            var manifest = BuildPipeline.BuildAssetBundles(
                OutDir,
                BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);

            if (manifest == null)
            {
                Debug.LogError("[BakeTMPFont] BuildAssetBundles returned null manifest.");
                EditorApplication.Exit(1);
                return;
            }

            string[] built = manifest.GetAllAssetBundles();
            Debug.Log($"[BakeTMPFont] Built {built.Length} bundle(s):");
            foreach (string b in built)
                Debug.Log($"  {b}");

            // Verify font bundle exists
            string fontBundlePath = Path.Combine(OutDir, BundleKey);
            if (File.Exists(fontBundlePath))
            {
                var fi = new FileInfo(fontBundlePath);
                Debug.Log($"[BakeTMPFont] SUCCESS — font bundle at {fontBundlePath} ({fi.Length} bytes)");
            }
            else
            {
                // Bundle key may be flattened to just the last segment
                string flatKey = "sw_menu_font_asset";
                string flatPath = Path.Combine(OutDir, flatKey);
                if (File.Exists(flatPath))
                {
                    var fi = new FileInfo(flatPath);
                    Debug.Log($"[BakeTMPFont] SUCCESS (flat key) — font bundle at {flatPath} ({fi.Length} bytes)");
                }
                else
                {
                    Debug.LogError($"[BakeTMPFont] Font bundle not found at expected path. Built bundles: {string.Join(", ", built)}");
                    EditorApplication.Exit(1);
                    return;
                }
            }

            Debug.Log("[BakeTMPFont] Complete.");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[BakeTMPFont] Fatal: {ex}");
            EditorApplication.Exit(1);
        }
    }
}
