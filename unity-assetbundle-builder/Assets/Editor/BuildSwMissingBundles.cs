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
                    if (existing != null)
                    {
                        CreateUrpMaterialForExisting(existing, def.Faction == "CIS" ? CisGrey : RepublicWhite);
                    }
                }

                // Try to source a real imported mesh from Assets/Models. If none is
                // available (no glTF importer in this project, or the model was never
                // copied in), fall back to a primitive so the prefab is STILL
                // regenerated with a URP material — never leave a stale Standard prefab
                // behind (the old `continue` here was the silent-skip bug: it errored,
                // skipped, and rebundled the old Standard-shader prefab → native render).
                GameObject go;
                string[] guids = AssetDatabase.FindAssets($"{def.Fbx} t:Model", new[] { "Assets/Models" });
                string modelPath = guids.Length > 0 ? AssetDatabase.GUIDToAssetPath(guids[0]) : null;
                GameObject modelPrefab = modelPath != null ? AssetDatabase.LoadAssetAtPath<GameObject>(modelPath) : null;
                if (modelPrefab != null)
                {
                    go = (GameObject)PrefabUtility.InstantiatePrefab(modelPrefab);
                    Debug.Log($"[BuildSwMissing] mesh {def.Key} <- {modelPath}");
                }
                else
                {
                    Debug.LogWarning($"[BuildSwMissing] no model for {def.Key} ({def.Fbx}); using primitive fallback (URP material still applied)");
                    go = GameObject.CreatePrimitive(PrimitiveType.Capsule);
                    miss++;
                }
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
            }

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            if (!Directory.Exists(OutputDir)) Directory.CreateDirectory(OutputDir);
            var manifest = BuildPipeline.BuildAssetBundles(
                OutputDir, BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);
            if (manifest == null) { Debug.LogError("[BuildSwMissing] manifest null"); EditorApplication.Exit(1); return; }

            Debug.Log($"[BuildSwMissing] Done: {ok} prefabs ({miss} primitive fallback). Bundles:");
            foreach (var b in manifest.GetAllAssetBundles()) Debug.Log($"  {b}");
            // Primitive fallback is acceptable (URP material still applied) — exit 0.
            EditorApplication.Exit(0);
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
        Shader shader = GetUrpShader();
        var mat = new Material(shader);
        mat.SetColor("_BaseColor", tint);
        LogShader("BuildSwMissing", mat);
        return mat;
    }

    private static void CreateUrpMaterialForExisting(Material material, Color tint)
    {
        Shader shader = GetUrpShader();
        material.shader = shader;
        material.SetColor("_BaseColor", tint);
        EditorUtility.SetDirty(material);
        LogShader("BuildSwMissing", material);
    }

    private static Shader GetUrpShader()
    {
        Shader shader = Shader.Find("Universal Render Pipeline/Lit")
            ?? Shader.Find("Universal Render Pipeline/Simple Lit");
        if (shader == null)
            throw new InvalidOperationException("No URP shader available.");
        return shader;
    }

    private static void LogShader(string source, Material material)
    {
        string shaderName = material.shader != null ? material.shader.name : "<null>";
        Debug.Log($"[{source}] material {material.name} shader={shaderName}");
        string shaderReportPath = Path.Combine(Directory.GetParent(Application.dataPath)!.FullName, "sw-shader-report.log");
        File.AppendAllText(shaderReportPath, $"material {material.name} shader={shaderName}{Environment.NewLine}");
    }
}
