using System;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Generates prefabs for warfare-starwars pack using imported FBX models from Assets/Models/.
/// Falls back to primitive placeholders when a real mesh is not yet available.
///
/// Run via:
///   Unity.exe -batchmode -nographics -projectPath . -executeMethod GenerateStarWarsPrefabsFromModels.Generate -quit
/// </summary>
public static class GenerateStarWarsPrefabsFromModels
{
    private static readonly Color RepublicWhite = new Color(0.95f, 0.95f, 0.95f);
    private static readonly Color RepublicBlue  = new Color(0.18f, 0.40f, 0.78f);
    private static readonly Color CisGrey       = new Color(0.55f, 0.55f, 0.50f);
    private static readonly Color CisDark       = new Color(0.30f, 0.28f, 0.25f);

    // (bundleKey, faction, primaryColor, accentColor, fallbackShape, modelAssetName)
    // modelAssetName = filename in Assets/Models/ without extension (null = primitive only)
    private static readonly (string Key, string Faction, Color Primary, Color Accent,
                              PrimitiveType Shape, string? ModelName)[] Defs =
    {
        // ── Republic units ──────────────────────────────────────────────────────
        ("sw-rep-clone-trooper",   "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, "sw_clone_trooper_phase2"),
        ("sw-rep-arc-trooper",     "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Capsule, "rep_arf_trooper"),
        ("sw-rep-clone-heavy",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, "rep_clone_heavy"),
        ("sw-rep-clone-sniper",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, "rep_clone_sharpshooter"),
        ("sw-rep-clone-commander", "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Capsule, null),
        ("sw-rep-clone-pilot",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, null),
        ("sw-rep-clone-engineer",  "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, "rep_clone_engineer"),
        ("sw-rep-clone-medic",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, "rep_clone_medic"),
        ("sw-rep-clone-jet",       "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, null),
        ("sw-rep-clone-mortar",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Capsule, null),
        ("sw-rep-at-rt-walker",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    null),
        ("sw-rep-at-te-walker",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "sw_at_te_walker"),
        ("sw-rep-laat-gunship",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    null),
        ("sw-rep-jedi-fighter",    "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Sphere,  null),
        // ── Republic buildings ──────────────────────────────────────────────────
        ("sw-rep-clone-facility",  "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "rep_clone_barracks"),
        ("sw-rep-turbolaser-tower","Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cylinder,null),
        ("sw-rep-med-center",      "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    null),
        ("sw-rep-shield-generator","Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Sphere,  "rep_shield_generator"),
        ("sw-rep-vehicle-bay",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "rep_vehicle_bay"),
        ("sw-rep-command-center",  "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Cube,    "rep_command_center"),
        ("sw-rep-senate-outpost",  "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    null),
        ("sw-rep-ion-cannon",      "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Cylinder,null),
        ("sw-rep-gunship-bay",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "rep_gunship_bay"),
        ("sw-rep-research-lab",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "rep_research_lab"),
        ("sw-rep-supply-depot",    "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube,    "rep_supply_station"),
        // ── CIS units ──────────────────────────────────────────────────────────
        ("sw-cis-b1-battle-droid", "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, "cis_b1_battle_droid"),
        ("sw-cis-b2-super-droid",  "CIS",      CisDark,       CisGrey,       PrimitiveType.Capsule, "sw_b2_super_droid"),
        ("sw-cis-droideka",        "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere,  "cis_droideka"),
        ("sw-cis-commando-droid",  "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, "cis_bx_commando_droid"),
        ("sw-cis-magna-guard",     "CIS",      CisDark,       CisGrey,       PrimitiveType.Capsule, "cis_magnaguard"),
        ("sw-cis-grapple-droid",   "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, null),
        ("sw-cis-sniper-droid",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, "cis_sniper_droid"),
        ("sw-cis-rocket-droid",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, null),
        ("sw-cis-droid-pilot",     "CIS",      CisGrey,       CisDark,       PrimitiveType.Capsule, null),
        ("sw-cis-octuptarra",      "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere,  null),
        ("sw-cis-hmp-droid-gunship","CIS",     CisDark,       CisGrey,       PrimitiveType.Cube,    null),
        ("sw-cis-stap",            "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder,"cis_stap_speeder"),
        ("sw-cis-spider-droid",    "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube,    "cis_dwarf_spider_droid"),
        ("sw-cis-nantex-fighter",  "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere,  null),
        // ── CIS buildings ──────────────────────────────────────────────────────
        ("sw-cis-droid-factory",   "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube,    "cis_droid_factory"),
        ("sw-cis-control-ship",    "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube,    null),
        ("sw-cis-aa-tower",        "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder,"cis_sentry_turret"),
        ("sw-cis-shield-generator","CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere,  "cis_ray_shield"),
        ("sw-cis-repair-bay",      "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube,    null),
        ("sw-cis-command-center",  "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube,    "cis_tactical_center"),
        ("sw-cis-hangar-bay",      "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube,    "cis_assembly_line"),
        ("sw-cis-ion-cannon",      "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder,null),
        ("sw-cis-trade-fed-core",  "CIS",      CisDark,       CisGrey,       PrimitiveType.Sphere,  null),
        ("sw-cis-supply-depot",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube,    "cis_mining_facility"),
        ("sw-cis-sensor-array",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Cylinder,"cis_tech_union_lab"),
    };

    public static void Generate()
    {
        try
        {
            Debug.Log("[GenerateStarWarsPrefabsFromModels] Starting...");

            EnsureDirs();

            // Force reimport of Models folder
            AssetDatabase.ImportAsset("Assets/Models", ImportAssetOptions.ImportRecursive);
            AssetDatabase.Refresh();

            int real = 0, fallback = 0;

            foreach (var def in Defs)
            {
                string matPath    = $"Assets/Materials/{def.Faction}/{def.Key}-primary.mat";
                string prefabPath = $"Assets/Prefabs/{def.Faction}/{def.Key}.prefab";

                // Create material if missing
                if (AssetDatabase.LoadAssetAtPath<Material>(matPath) == null)
                {
                    var mat = new Material(Shader.Find("Standard")) { color = def.Primary };
                    AssetDatabase.CreateAsset(mat, matPath);
                    var mi = AssetImporter.GetAtPath(matPath);
                    if (mi != null) mi.assetBundleName = def.Key;
                }

                // Delete existing prefab to force regeneration (clears stale primitives)
                if (AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null)
                {
                    AssetDatabase.DeleteAsset(prefabPath);
                }

                // Try loading real mesh from Assets/Models/
                GameObject go = null;

                if (def.ModelName != null)
                {
                    string[] guids = AssetDatabase.FindAssets($"{def.ModelName} t:Model", new[] { "Assets/Models" });
                    if (guids.Length > 0)
                    {
                        string modelPath = AssetDatabase.GUIDToAssetPath(guids[0]);
                        var modelPrefab = AssetDatabase.LoadAssetAtPath<GameObject>(modelPath);
                        if (modelPrefab != null)
                        {
                            go = (GameObject)PrefabUtility.InstantiatePrefab(modelPrefab);
                            go.name = def.Key;
                            real++;
                            Debug.Log($"  [mesh]  {def.Key} ← {modelPath}");
                        }
                    }
                }

                if (go == null)
                {
                    go = GameObject.CreatePrimitive(def.Shape);
                    go.name = def.Key;
                    fallback++;
                    Debug.Log($"  [prim]  {def.Key} (no mesh)");
                }

                // Apply material
                var mat2 = AssetDatabase.LoadAssetAtPath<Material>(matPath);
                if (mat2 != null)
                {
                    foreach (var r in go.GetComponentsInChildren<Renderer>())
                        r.sharedMaterial = mat2;
                }

                // Save as prefab
                PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
                GameObject.DestroyImmediate(go);

                // Assign bundle
                var pi = AssetImporter.GetAtPath(prefabPath);
                if (pi != null) pi.assetBundleName = def.Key;
            }

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            Debug.Log($"[GenerateStarWarsPrefabsFromModels] Done: {real} real meshes, {fallback} primitives");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[GenerateStarWarsPrefabsFromModels] {ex}");
            EditorApplication.Exit(1);
        }
    }

    private static void EnsureDirs()
    {
        string[] dirs = {
            "Assets/Materials/Republic", "Assets/Materials/CIS",
            "Assets/Prefabs/Republic",   "Assets/Prefabs/CIS",
            "Assets/Models",
        };
        foreach (string d in dirs)
        {
            if (!AssetDatabase.IsValidFolder(d))
            {
                string parent = System.IO.Path.GetDirectoryName(d)!.Replace('\\', '/');
                string child  = System.IO.Path.GetFileName(d);
                AssetDatabase.CreateFolder(parent, child);
            }
        }
    }
}
