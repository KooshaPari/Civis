using System;
using System.IO;
using UnityEditor;
using UnityEngine;

/// <summary>
/// One-shot entry point: generate prefabs + build AssetBundles in a single headless run.
/// Bundle keys match visual_asset values in warfare-starwars YAML definitions exactly.
/// Usage: Unity.exe -batchmode -nographics -executeMethod BuildAll.Run -quit
/// </summary>
public static class BuildAll
{
    // Republic colors (Clone Trooper white + blue accent)
    private static readonly Color RepublicWhite = new Color(0.95f, 0.95f, 0.95f);
    private static readonly Color RepublicBlue  = new Color(0.18f, 0.40f, 0.78f);
    private static readonly Color RepublicGold  = new Color(0.85f, 0.70f, 0.10f);

    // CIS colors (droid grey/dark)
    private static readonly Color CisGrey  = new Color(0.55f, 0.55f, 0.50f);
    private static readonly Color CisDark  = new Color(0.30f, 0.28f, 0.25f);
    private static readonly Color CisRed   = new Color(0.70f, 0.10f, 0.10f);

    // Neutral/special
    private static readonly Color JediBlue   = new Color(0.10f, 0.45f, 0.90f);
    private static readonly Color JediGreen  = new Color(0.10f, 0.75f, 0.30f);
    private static readonly Color NeutralGrey = new Color(0.60f, 0.58f, 0.55f);

    // (key, faction-folder, color, shape)
    private static readonly (string Key, string Folder, Color Color, PrimitiveType Shape)[] Defs =
    {
        // ── Republic units ────────────────────────────────────────────────────────
        ("sw-rep-clone-trooper",  "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-clone-heavy",    "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-clone-sniper",   "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-at-te-walker",   "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-rep-clone-medic",    "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-arc-trooper",    "Republic", RepublicBlue,  PrimitiveType.Capsule),
        ("sw-clone-militia",      "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-barc-speeder",       "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-arf-trooper",        "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-jedi-knight",        "Republic", JediBlue,      PrimitiveType.Capsule),
        ("sw-clone-wall-guard",   "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-clone-commando",     "Republic", RepublicBlue,  PrimitiveType.Capsule),
        ("sw-v19-torrent-unit",   "Republic", RepublicWhite, PrimitiveType.Sphere),

        // ── Republic buildings ────────────────────────────────────────────────────
        ("sw-rep-command-center",  "Republic", RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-clone-facility",  "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-weapons-factory",     "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-rep-vehicle-bay",     "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-guard-tower",         "Republic", RepublicWhite, PrimitiveType.Cylinder),
        ("sw-rep-shield-generator","Republic", RepublicBlue,  PrimitiveType.Sphere),
        ("sw-rep-supply-depot",    "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-tibanna-refinery",    "Republic", NeutralGrey,   PrimitiveType.Cube),
        ("sw-rep-research-lab",    "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-blast-wall",          "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-skyshield-generator", "Republic", RepublicBlue,  PrimitiveType.Sphere),

        // ── CIS units ─────────────────────────────────────────────────────────────
        ("sw-cis-b1-battle-droid", "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-b1-squad",            "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-cis-b2-super-droid",  "CIS",      CisDark,      PrimitiveType.Capsule),
        ("sw-cis-sniper-droid",    "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-cis-stap",            "CIS",      CisGrey,      PrimitiveType.Cylinder),
        ("sw-aat-walker",          "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-medical-droid",       "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-probe-droid",         "CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-cis-commando-droid",  "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-general-grievous",    "CIS",      CisDark,      PrimitiveType.Capsule),
        ("sw-cis-droideka",        "CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-cis-spider-droid",    "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-cis-magna-guard",     "CIS",      CisDark,      PrimitiveType.Capsule),
        ("sw-tri-fighter",         "CIS",      CisRed,       PrimitiveType.Sphere),

        // ── CIS buildings ─────────────────────────────────────────────────────────
        ("sw-cis-command-center",  "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-cis-droid-factory",   "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-assembly-line",       "CIS",      CisGrey,      PrimitiveType.Cube),
        ("sw-heavy-foundry",       "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-cis-aa-tower",        "CIS",      CisGrey,      PrimitiveType.Cylinder),
        ("sw-cis-shield-generator","CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-mining-facility",     "CIS",      NeutralGrey,  PrimitiveType.Cube),
        ("sw-processing-plant",    "CIS",      CisGrey,      PrimitiveType.Cube),
        ("sw-tech-union-lab",      "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-durasteel-barrier",   "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-vulture-nest",        "CIS",      CisDark,      PrimitiveType.Cube),
    };

    public static void Run()
    {
        try
        {
            Debug.Log("[BuildAll] Generating prefabs...");
            EnsureFolders();
            int created = 0;
            foreach (var def in Defs)
            {
                if (CreatePrefab(def.Key, def.Folder, def.Color, def.Shape))
                    created++;
            }
            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();
            Debug.Log($"[BuildAll] Generated {created}/{Defs.Length} prefabs (skipped {Defs.Length - created} existing).");

            Debug.Log("[BuildAll] Building AssetBundles...");
            string outDir = "AssetBundles";
            if (!Directory.Exists(outDir))
                Directory.CreateDirectory(outDir);

            var manifest = BuildPipeline.BuildAssetBundles(
                outDir,
                BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);

            if (manifest == null)
            {
                Debug.LogError("[BuildAll] Build failed — manifest null.");
                EditorApplication.Exit(1);
                return;
            }

            string[] built = manifest.GetAllAssetBundles();
            Debug.Log($"[BuildAll] Built {built.Length} bundle(s):");
            foreach (string b in built)
                Debug.Log($"  {b}");

            Debug.Log("[BuildAll] Complete.");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[BuildAll] Fatal: {ex}");
            EditorApplication.Exit(1);
        }
    }

    private static void EnsureFolders()
    {
        foreach (string f in new[] {
            "Assets/Materials", "Assets/Materials/Republic", "Assets/Materials/CIS",
            "Assets/Prefabs",   "Assets/Prefabs/Republic",   "Assets/Prefabs/CIS" })
        {
            if (!AssetDatabase.IsValidFolder(f))
            {
                string parent = System.IO.Path.GetDirectoryName(f)!.Replace('\\', '/');
                string child  = System.IO.Path.GetFileName(f);
                AssetDatabase.CreateFolder(parent, child);
            }
        }
    }

    /// <returns>true if created, false if skipped.</returns>
    private static bool CreatePrefab(string key, string folder, Color color, PrimitiveType shape)
    {
        string prefabPath = $"Assets/Prefabs/{folder}/{key}.prefab";
        if (AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null)
        {
            Debug.Log($"  [skip] {key}");
            return false;
        }

        string matPath = $"Assets/Materials/{folder}/{key}.mat";
        var mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
        if (mat == null)
        {
            mat = new Material(Shader.Find("Standard")) { color = color };
            AssetDatabase.CreateAsset(mat, matPath);
            var mi = AssetImporter.GetAtPath(matPath);
            if (mi != null) mi.assetBundleName = key;
        }

        GameObject go = GameObject.CreatePrimitive(shape);
        go.name = key;
        go.GetComponent<Renderer>().sharedMaterial = mat;
        PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
        GameObject.DestroyImmediate(go);

        var pi = AssetImporter.GetAtPath(prefabPath);
        if (pi != null) pi.assetBundleName = key;

        Debug.Log($"  [ok] {key}");
        return true;
    }
}
