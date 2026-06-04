using System;
using System.IO;
using UnityEditor;
using UnityEngine;
using UnityEngine.TextCore.LowLevel;
using TMPro;

/// <summary>
/// Headless TMP SDF font asset creation + AssetBundle build.
/// Bypasses TMP_FontAsset.CreateFontAsset (which needs GPU for shader init in -nographics).
/// Creates a Dynamic-population TMP_FontAsset that will bake its atlas at runtime on first use.
/// Usage: Unity.exe -batchmode -nographics -projectPath <proj> -executeMethod BakeTMPFont.BakeAndBundle -quit
/// </summary>
public static class BakeTMPFont
{
    private const string FontTtfPath   = "Assets/Fonts/sw_menu_font.ttf";
    private const string FontAssetPath = "Assets/Fonts/sw_menu_font_asset.asset";
    private const string BundleKey     = "assets/ui/sw_menu_font_asset";
    private const string OutDir        = "AssetBundles";

    public static void BakeAndBundle()
    {
        try
        {
            Debug.Log("[BakeTMPFont] Starting TMP font asset creation (headless/no-GPU mode)...");
            AssetDatabase.Refresh();

            // Load source TTF
            Font srcFont = AssetDatabase.LoadAssetAtPath<Font>(FontTtfPath);
            if (srcFont == null)
            {
                Debug.LogError($"[BakeTMPFont] Source TTF not found at {FontTtfPath}");
                EditorApplication.Exit(1);
                return;
            }
            Debug.Log($"[BakeTMPFont] Loaded source font: {srcFont.name}");

            // Ensure folder
            if (!AssetDatabase.IsValidFolder("Assets/Fonts"))
                AssetDatabase.CreateFolder("Assets", "Fonts");

            // Create or load TMP_FontAsset
            TMP_FontAsset fontAsset = AssetDatabase.LoadAssetAtPath<TMP_FontAsset>(FontAssetPath);
            if (fontAsset == null)
            {
                Debug.Log("[BakeTMPFont] Creating TMP_FontAsset ScriptableObject (bypassing shader-dependent path)...");

                // Create instance directly — avoids CreateFontAsset which crashes on null ShaderRef_MobileSDF
                fontAsset = ScriptableObject.CreateInstance<TMP_FontAsset>();
                fontAsset.name = "sw_menu_font_asset";

                // Wire up FontEngine to get face info (no GPU needed for this step)
                var initErr = FontEngine.InitializeFontEngine();
                Debug.Log($"[BakeTMPFont] FontEngine.Init = {initErr}");

                var loadErr = FontEngine.LoadFontFace(srcFont, 28);
                Debug.Log($"[BakeTMPFont] FontEngine.LoadFontFace = {loadErr}");

                if (loadErr == FontEngineError.Success)
                {
                    fontAsset.faceInfo = FontEngine.GetFaceInfo();
                    Debug.Log($"[BakeTMPFont] FaceInfo loaded: family={fontAsset.faceInfo.familyName}");
                }
                else
                {
                    Debug.LogWarning($"[BakeTMPFont] FontEngine.LoadFontFace returned {loadErr} — faceInfo will be default");
                }

                // Set Dynamic population mode so runtime builds atlas on demand
                fontAsset.atlasPopulationMode = AtlasPopulationMode.Dynamic;

                // Set source font file reference (required for Dynamic mode)
                // sourceFontFile.set is internal — use SerializedObject to set backing field
                var so = new SerializedObject(fontAsset);
                var srcProp = so.FindProperty("m_SourceFontFile");
                if (srcProp != null)
                {
                    srcProp.objectReferenceValue = srcFont;
                    so.ApplyModifiedPropertiesWithoutUndo();
                    Debug.Log("[BakeTMPFont] Set m_SourceFontFile via SerializedObject");
                }
                else
                {
                    Debug.LogWarning("[BakeTMPFont] m_SourceFontFile property not found in SerializedObject");
                }

                AssetDatabase.CreateAsset(fontAsset, FontAssetPath);
                AssetDatabase.SaveAssets();
                AssetDatabase.Refresh();
                Debug.Log($"[BakeTMPFont] Asset saved at {FontAssetPath}");
            }
            else
            {
                Debug.Log($"[BakeTMPFont] Reusing existing asset at {FontAssetPath}");
            }

            // Assign AssetBundle name
            var importer = AssetImporter.GetAtPath(FontAssetPath);
            if (importer == null)
            {
                Debug.LogError($"[BakeTMPFont] No importer for {FontAssetPath}");
                EditorApplication.Exit(1);
                return;
            }
            importer.assetBundleName = BundleKey;
            importer.SaveAndReimport();
            Debug.Log($"[BakeTMPFont] AssetBundle name = {BundleKey}");

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
            Debug.Log($"[BakeTMPFont] Built {built.Length} bundle(s): {string.Join(", ", built)}");

            foreach (string b in built)
            {
                string bundlePath = Path.Combine(OutDir, b);
                if (File.Exists(bundlePath))
                    Debug.Log($"[BakeTMPFont]   '{b}' = {new FileInfo(bundlePath).Length} bytes");
            }

            // Final check — does our font bundle exist?
            string expectedBundle = Path.Combine(OutDir, BundleKey);
            if (File.Exists(expectedBundle))
                Debug.Log($"[BakeTMPFont] SUCCESS — font bundle at {expectedBundle} ({new FileInfo(expectedBundle).Length} bytes)");
            else
                Debug.LogWarning($"[BakeTMPFont] Font bundle not at {expectedBundle} — see bundle list above for actual filename");

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
