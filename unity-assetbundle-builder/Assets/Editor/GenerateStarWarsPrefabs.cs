using System;
using System.IO;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Generates colored primitive placeholder prefabs for the warfare-starwars pack
/// and assigns them to AssetBundles for building.
///
/// Republic units: white + blue (Clone trooper colors)
/// CIS units: light grey (Droid colors)
///
/// Run via: Unity.exe -batchmode -nographics -executeMethod GenerateStarWarsPrefabs.Generate -quit
/// Then run BuildAssetBundles.BuildHeadless to compile the bundles.
/// </summary>
public static class GenerateStarWarsPrefabs
{
    // Republic (white base + blue accent)
    private static readonly Color RepublicWhite = new Color(0.95f, 0.95f, 0.95f);
    private static readonly Color RepublicBlue  = new Color(0.18f, 0.40f, 0.78f);

    // CIS (droid grey)
    private static readonly Color CisGrey       = new Color(0.55f, 0.55f, 0.50f);
    private static readonly Color CisDark       = new Color(0.30f, 0.28f, 0.25f);

    private static readonly (string BundleKey, string Faction, Color Primary, Color Accent, PrimitiveType Shape)[] Definitions =
    {
        // ── Republic units ────────────────────────────────────────────────────
        ("sw-rep-clone-trooper",      "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-arc-trooper",        "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-clone-heavy",        "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-sniper",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-commander",    "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Capsule),
        ("sw-rep-clone-pilot",        "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-engineer",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-medic",        "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-jet",          "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-clone-mortar",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule),
        ("sw-rep-at-rt-walker",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-at-te-walker",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-laat-gunship",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-jedi-fighter",       "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Sphere),
        // ── Republic buildings ────────────────────────────────────────────────
        ("sw-rep-clone-facility",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-turbolaser-tower",   "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cylinder),
        ("sw-rep-med-center",         "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-shield-generator",   "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Sphere),
        ("sw-rep-vehicle-bay",        "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-command-center",     "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Cube),
        ("sw-rep-senate-outpost",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-ion-cannon",         "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Cylinder),
        ("sw-rep-gunship-bay",        "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-research-lab",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        ("sw-rep-supply-depot",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
        // ── CIS units ─────────────────────────────────────────────────────────
        ("sw-cis-b1-battle-droid",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-b2-super-droid",     "CIS",      CisDark,       CisGrey,       PrimitiveType.Capsule),
        ("sw-cis-droideka",           "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere),
        ("sw-cis-commando-droid",     "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-magna-guard",        "CIS",      CisDark,       CisGrey,       PrimitiveType.Capsule),
        ("sw-cis-grapple-droid",      "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-sniper-droid",       "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-rocket-droid",       "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-droid-pilot",        "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule),
        ("sw-cis-octuptarra",         "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere),
        ("sw-cis-hmp-droid-gunship",  "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-stap",               "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder),
        ("sw-cis-spider-droid",       "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-nantex-fighter",     "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere),
        // ── CIS buildings ─────────────────────────────────────────────────────
        ("sw-cis-droid-factory",      "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-control-ship",       "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-aa-tower",           "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder),
        ("sw-cis-shield-generator",   "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere),
        ("sw-cis-repair-bay",         "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
        ("sw-cis-command-center",     "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-hangar-bay",         "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
        ("sw-cis-ion-cannon",         "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder),
        ("sw-cis-trade-fed-core",     "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere),
        ("sw-cis-supply-depot",       "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
        ("sw-cis-sensor-array",       "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder),
    };

    public static void Generate()
    {
        try
        {
            Debug.Log("[GenerateStarWarsPrefabs] Starting prefab generation...");

            EnsureDirectories();
            GenerateMaterials();
            GeneratePrefabs();

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            Debug.Log("[GenerateStarWarsPrefabs] Done. All prefabs generated.");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[GenerateStarWarsPrefabs] Exception: {ex}");
            EditorApplication.Exit(1);
        }
    }

    private static void EnsureDirectories()
    {
        string[] dirs =
        {
            "Assets/Materials/Republic",
            "Assets/Materials/CIS",
            "Assets/Prefabs/Republic",
            "Assets/Prefabs/CIS",
        };

        foreach (string d in dirs)
        {
            if (!AssetDatabase.IsValidFolder(d))
            {
                string parent = Path.GetDirectoryName(d)!.Replace('\\', '/');
                string child  = Path.GetFileName(d);
                AssetDatabase.CreateFolder(parent, child);
            }
        }
    }

    private static void GenerateMaterials()
    {
        foreach (var def in Definitions)
        {
            CreateMaterial($"{def.BundleKey}-primary",  def.Faction, def.Primary,  def.BundleKey);
            CreateMaterial($"{def.BundleKey}-accent",   def.Faction, def.Accent,   def.BundleKey);
        }
    }

    private static void CreateMaterial(string name, string faction, Color color, string bundleKey)
    {
        string path = $"Assets/Materials/{faction}/{name}.mat";
        if (AssetDatabase.LoadAssetAtPath<Material>(path) != null)
            return; // already exists

        var mat = new Material(Shader.Find("Standard"))
        {
            color = color
        };
        AssetDatabase.CreateAsset(mat, path);

        // Assign to bundle
        var importer = AssetImporter.GetAtPath(path);
        if (importer != null)
            importer.assetBundleName = bundleKey;
    }

    private static void GeneratePrefabs()
    {
        foreach (var def in Definitions)
        {
            string prefabPath = $"Assets/Prefabs/{def.Faction}/{def.BundleKey}.prefab";
            if (AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null)
            {
                Debug.Log($"  [skip] {def.BundleKey} already exists");
                continue;
            }

            // Create primitive GameObject
            GameObject go = GameObject.CreatePrimitive(def.Shape);
            go.name = def.BundleKey;

            // Assign primary material
            var renderer = go.GetComponent<Renderer>();
            string matPath = $"Assets/Materials/{def.Faction}/{def.BundleKey}-primary.mat";
            var mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
            if (mat != null)
                renderer.sharedMaterial = mat;

            // Save as prefab
            PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
            GameObject.DestroyImmediate(go);

            // Assign to asset bundle
            var importer = AssetImporter.GetAtPath(prefabPath);
            if (importer != null)
                importer.assetBundleName = def.BundleKey;

            Debug.Log($"  [created] {def.BundleKey}");
        }
    }
}
