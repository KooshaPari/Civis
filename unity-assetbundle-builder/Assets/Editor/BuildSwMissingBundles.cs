using System;
using System.IO;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Builds correctly-named AssetBundles for the ~19 previously-stub Star Wars units.
/// Bundle filename == visual_asset key == Addressable key (what the runtime mesh-swap reads).
/// Maps each runtime visual_asset key to its converted FBX in Assets/Models.
///
/// Run:
///   Unity.exe -batchmode -nographics -projectPath . -executeMethod BuildSwMissingBundles.Build -quit
/// </summary>
public static class BuildSwMissingBundles
{
    private const string OutputDir = "AssetBundles";

    // (bundleKey == visual_asset, fbxAssetName in Assets/Models, faction)
    private static readonly (string Key, string Fbx, string Faction)[] Defs =
    {
        ("sw-general-grievous",   "sw_general_grievous",    "CIS"),
        ("sw-jedi-knight",        "rep_jedi_knight",        "Republic"),
        ("sw-rep-at-te-walker",   "sw_at_te_walker",        "Republic"),
        ("sw-tri-fighter",        "cis_tri_fighter",        "CIS"),
        ("sw-rep-arc-trooper",    "sw_arc_trooper",         "Republic"),
        ("sw-cis-magna-guard",    "cis_magnaguard",         "CIS"),
        ("sw-clone-commando",     "rep_clone_commando",     "Republic"),
        ("sw-rep-clone-sniper",   "rep_clone_sniper",       "Republic"),
        ("sw-cis-medical-droid",  "cis_medical_droid",      "CIS"),
        ("sw-b1-squad",           "cis_b1_squad",           "CIS"),
        ("sw-clone-wall-guard",   "rep_clone_wall_guard",   "Republic"),
        ("sw-aat-walker",         "sw_aat_walker",          "CIS"),
        ("sw-cis-aat",            "sw_aat_walker",          "CIS"),
        ("sw-v19-torrent",        "rep_v19_torrent",        "Republic"),
        ("sw-rep-laat-gunship",   "cis_laat_gunship",       "Republic"),
        ("sw-cis-spider-droid",   "cis_dwarf_spider_droid", "CIS"),
        ("sw-cis-stap",           "cis_stap_speeder",       "CIS"),
        ("sw-barc-speeder",       "rep_barc_speeder",       "Republic"),
        ("sw-droideka",           "sw_droideka",            "CIS"),
    };

    private static readonly Color RepublicWhite = new Color(0.95f, 0.95f, 0.95f);
    private static readonly Color CisGrey = new Color(0.55f, 0.55f, 0.50f);

    public static void Build()
    {
        try
        {
            EnsureDirs();
            AssetDatabase.ImportAsset("Assets/Models", ImportAssetOptions.ImportRecursive);
            AssetDatabase.Refresh();

            int ok = 0, miss = 0;
            foreach (var def in Defs)
            {
                string prefabPath = $"Assets/Prefabs/{def.Faction}/{def.Key}.prefab";
                string matPath = $"Assets/Materials/{def.Faction}/{def.Key}.mat";

                if (AssetDatabase.LoadAssetAtPath<Material>(matPath) == null)
                {
                    var m = CreateUrpMaterial(def.Faction == "CIS" ? CisGrey : RepublicWhite);
                    AssetDatabase.CreateAsset(m, matPath);
                }
                else
                {
                    var existing = AssetDatabase.LoadAssetAtPath<Material>(matPath);
                    if (existing != null && (existing.shader == null || !existing.shader.name.StartsWith("Universal Render Pipeline/")))
                    {
                        CreateUrpMaterialForExisting(existing, def.Faction == "CIS" ? CisGrey : RepublicWhite);
                    }
                }

                string[] guids = AssetDatabase.FindAssets($"{def.Fbx} t:Model", new[] { "Assets/Models" });
                if (guids.Length == 0)
                {
                    Debug.LogError($"[BuildSwMissing] MISSING FBX for {def.Key}: {def.Fbx}");
                    miss++;
                    continue;
                }
                string modelPath = AssetDatabase.GUIDToAssetPath(guids[0]);
                var modelPrefab = AssetDatabase.LoadAssetAtPath<GameObject>(modelPath);
                var go = (GameObject)PrefabUtility.InstantiatePrefab(modelPrefab);
                go.name = def.Key;

                var mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
                foreach (var r in go.GetComponentsInChildren<Renderer>())
                    r.sharedMaterial = mat;

                if (AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null)
                    AssetDatabase.DeleteAsset(prefabPath);
                PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
                GameObject.DestroyImmediate(go);

                var pi = AssetImporter.GetAtPath(prefabPath);
                if (pi != null) pi.assetBundleName = def.Key;
                ok++;
                Debug.Log($"[BuildSwMissing] prefab {def.Key} <- {modelPath}");
            }

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            if (!Directory.Exists(OutputDir)) Directory.CreateDirectory(OutputDir);
            var manifest = BuildPipeline.BuildAssetBundles(
                OutputDir, BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);
            if (manifest == null) { Debug.LogError("[BuildSwMissing] manifest null"); EditorApplication.Exit(1); return; }

            Debug.Log($"[BuildSwMissing] Done: {ok} prefabs, {miss} missing. Bundles:");
            foreach (var b in manifest.GetAllAssetBundles()) Debug.Log($"  {b}");
            EditorApplication.Exit(miss > 0 ? 2 : 0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[BuildSwMissing] {ex}");
            EditorApplication.Exit(1);
        }
    }

    private static void EnsureDirs()
    {
        string[] dirs = {
            "Assets/Materials/Republic", "Assets/Materials/CIS",
            "Assets/Prefabs/Republic", "Assets/Prefabs/CIS", "Assets/Models",
        };
        foreach (var d in dirs)
            if (!AssetDatabase.IsValidFolder(d))
            {
                string parent = Path.GetDirectoryName(d)!.Replace('\\', '/');
                AssetDatabase.CreateFolder(parent, Path.GetFileName(d));
            }
    }

    private static Material CreateUrpMaterial(Color tint)
    {
        var shader = Shader.Find("Universal Render Pipeline/Lit")
            ?? Shader.Find("Universal Render Pipeline/Simple Lit");
        var mat = new Material(shader);
        mat.SetColor("_BaseColor", tint);
        return mat;
    }

    private static void CreateUrpMaterialForExisting(Material material, Color tint)
    {
        var shader = Shader.Find("Universal Render Pipeline/Lit")
            ?? Shader.Find("Universal Render Pipeline/Simple Lit");
        material.shader = shader;
        material.SetColor("_BaseColor", tint);
        EditorUtility.SetDirty(material);
    }
}
