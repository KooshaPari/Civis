using System;
using System.IO;
using TMPro;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Offline TMP SDF font-asset baker (Option A — Unity 2021.3.45f1 batchmode).
///
/// Why: TMP_FontAsset.CreateFontAsset() returns null at RUNTIME inside DINO for
/// OS-dynamic fonts (the Mono TMP atlas-generator path is unavailable in the
/// shipped player). Baking the SDF atlas + glyph table offline in the Editor —
/// where CreateFontAsset() works — sidesteps the runtime failure entirely.
///
/// Pipeline:
///   1. Import menu_font.ttf as a Unity Font (TrueType).
///   2. TMP_FontAsset.CreateFontAsset(font) -> generates SDF atlas + glyph table.
///   3. Pre-populate the ASCII printable range so the menu glyphs are baked into
///      a STATIC atlas (no runtime atlas growth needed).
///   4. Persist the .asset (+ material + atlas texture) and tag it for the
///      'sw_menu_font' AssetBundle. BuildAssetBundles then emits the bundle,
///      version-locked to 2021.3.45f1 so DINO can load it.
///
/// Invoke (batchmode):
///   "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" \
///     -batchmode -nographics -noUpm -quit \
///     -projectPath "<repo>\unity-assetbundle-builder" \
///     -executeMethod BakeTmpFontAsset.BakeHeadless \
///     -logFile "<repo>\docs\sessions\tmp-font-bake.log"
///
/// The source ttf is expected at Assets/Fonts/menu_font.ttf (copy it in before
/// running — see scripts/game/bake-sw-menu-font.ps1).
/// </summary>
public static class BakeTmpFontAsset
{
    private const string FontTtfPath = "Assets/Fonts/menu_font.ttf";
    private const string OutputAssetPath = "Assets/Fonts/SW_MenuFont SDF.asset";
    private const string BundleName = "sw_menu_font";

    // Sampling point size for the SDF atlas. 90px gives crisp menu-scale glyphs.
    private const int SamplingPointSize = 90;
    private const int AtlasPadding = 9;
    private const int AtlasWidth = 1024;
    private const int AtlasHeight = 1024;

    public static void BakeHeadless()
    {
        try
        {
            Debug.Log("[BakeTmpFontAsset] Starting TMP SDF font bake...");

            if (!File.Exists(FontTtfPath))
            {
                Debug.LogError($"[BakeTmpFontAsset] Missing source font at {FontTtfPath}. " +
                               "Copy packs/warfare-starwars/assets/ui/menu_font.ttf there first.");
                EditorApplication.Exit(2);
                return;
            }

            // Ensure the ttf is imported as a usable Font.
            AssetDatabase.ImportAsset(FontTtfPath, ImportAssetOptions.ForceUpdate);
            Font sourceFont = AssetDatabase.LoadAssetAtPath<Font>(FontTtfPath);
            if (sourceFont == null)
            {
                Debug.LogError("[BakeTmpFontAsset] Failed to load source ttf as Font.");
                EditorApplication.Exit(3);
                return;
            }

            // Editor-side CreateFontAsset DOES work (unlike the DINO runtime).
            TMP_FontAsset fontAsset = TMP_FontAsset.CreateFontAsset(
                sourceFont,
                SamplingPointSize,
                AtlasPadding,
                UnityEngine.TextCore.LowLevel.GlyphRenderMode.SDFAA,
                AtlasWidth,
                AtlasHeight,
                AtlasPopulationMode.Static,
                enableMultiAtlasSupport: true);

            if (fontAsset == null)
            {
                Debug.LogError("[BakeTmpFontAsset] CreateFontAsset returned null in editor — unexpected.");
                EditorApplication.Exit(4);
                return;
            }

            fontAsset.name = "SW_MenuFont SDF";

            // Pre-bake the printable ASCII range into the static atlas so no
            // runtime glyph generation is required inside DINO.
            const string chars =
                " !\"#$%&'()*+,-./0123456789:;<=>?@" +
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`" +
                "abcdefghijklmnopqrstuvwxyz{|}~";
            fontAsset.TryAddCharacters(chars, out string missing);
            if (!string.IsNullOrEmpty(missing))
                Debug.LogWarning($"[BakeTmpFontAsset] Glyphs not present in font: '{missing}'");

            // Persist the font asset, its atlas texture and material as sub-assets.
            string dir = Path.GetDirectoryName(OutputAssetPath);
            if (!Directory.Exists(dir)) Directory.CreateDirectory(dir);
            AssetDatabase.CreateAsset(fontAsset, OutputAssetPath);

            // Atlas texture + material must be embedded so the bundle is self-contained.
            if (fontAsset.atlasTextures != null)
            {
                foreach (Texture2D tex in fontAsset.atlasTextures)
                {
                    if (tex != null && !AssetDatabase.Contains(tex))
                        AssetDatabase.AddObjectToAsset(tex, fontAsset);
                }
            }
            if (fontAsset.material != null && !AssetDatabase.Contains(fontAsset.material))
                AssetDatabase.AddObjectToAsset(fontAsset.material, fontAsset);

            EditorUtility.SetDirty(fontAsset);
            AssetDatabase.SaveAssets();
            AssetDatabase.ImportAsset(OutputAssetPath, ImportAssetOptions.ForceUpdate);

            // Tag the .asset for the AssetBundle so BuildAssetBundles emits it.
            AssetImporter importer = AssetImporter.GetAtPath(OutputAssetPath);
            if (importer != null)
            {
                importer.assetBundleName = BundleName;
                importer.SaveAndReimport();
            }
            else
            {
                Debug.LogError("[BakeTmpFontAsset] Could not get AssetImporter to tag bundle.");
                EditorApplication.Exit(5);
                return;
            }

            Debug.Log($"[BakeTmpFontAsset] Baked '{OutputAssetPath}', tagged bundle '{BundleName}'. " +
                      "Now run BuildAssetBundles.BuildHeadless to emit the bundle.");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[BakeTmpFontAsset] Exception: {ex}");
            EditorApplication.Exit(1);
        }
    }
}
