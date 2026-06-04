using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
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

        // ── CIS units (legacy keys without cis- prefix) ──────────────────────────
        ("sw-b1-battle-droid",     "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-b2-super-droid",      "CIS",      CisDark,      PrimitiveType.Capsule),
        ("sw-bx-commando-droid",   "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-cis-aat",             "CIS",      CisDark,      PrimitiveType.Cube),
        ("sw-cis-medical-droid",   "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-commando-droid",      "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-droideka",            "CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-hmp-droid-gunship",   "CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-nantex-fighter",      "CIS",      CisRed,       PrimitiveType.Sphere),
        ("sw-octuptarra",          "CIS",      CisDark,      PrimitiveType.Sphere),
        ("sw-sniper-droid",        "CIS",      CisGrey,      PrimitiveType.Capsule),
        ("sw-trade-fed-core",      "CIS",      CisDark,      PrimitiveType.Cube),

        // ── Republic units/buildings (legacy keys) ───────────────────────────────
        ("sw-clone-heavy",         "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-clone-medic",         "Republic", RepublicWhite, PrimitiveType.Capsule),
        ("sw-v19-torrent",         "Republic", RepublicWhite, PrimitiveType.Sphere),

        // ── Neutral/shared buildings ──────────────────────────────────────────────
        ("sw-clone-barracks",      "Republic", RepublicWhite, PrimitiveType.Cube),
        ("sw-hangar-bay",          "Republic", RepublicWhite, PrimitiveType.Cube),

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

    // bundle-key → glb basename in Assets/Models (or resolvable from packs/.../raw/*/model.glb).
    // Only keys that HAVE a real glb source are listed; everything else keeps the
    // procedural primitive fallback. This upgrades BuildAll-exclusive keys (the ones
    // GenerateStarWarsPrefabsFromModels does NOT define) from capsule → real mesh.
    private static readonly Dictionary<string, string> ModelMap = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
    {
        // ── Republic units ──
        { "sw-rep-clone-trooper",  "sw_clone_trooper_phase2" },
        { "sw-rep-clone-heavy",    "rep_clone_heavy" },
        { "sw-rep-clone-sniper",   "rep_clone_sniper" },
        { "sw-rep-at-te-walker",   "sw_at_te_walker" },
        { "sw-rep-clone-medic",    "rep_clone_medic" },
        { "sw-rep-arc-trooper",    "sw_arc_trooper" },
        { "sw-clone-militia",      "rep_clone_militia" },
        { "sw-barc-speeder",       "rep_barc_speeder" },
        { "sw-arf-trooper",        "rep_arf_trooper" },
        { "sw-jedi-knight",        "rep_jedi_knight" },
        { "sw-clone-wall-guard",   "rep_clone_wall_guard" },
        { "sw-clone-commando",     "rep_clone_commando" },
        { "sw-v19-torrent-unit",   "rep_v19_torrent" },
        // ── Republic buildings ──
        { "sw-rep-clone-facility", "rep_clone_barracks" },
        { "sw-weapons-factory",    "rep_weapons_factory" },
        { "sw-rep-vehicle-bay",    "rep_vehicle_bay" },
        { "sw-guard-tower",        "rep_guard_tower" },
        { "sw-rep-shield-generator","rep_shield_generator" },
        { "sw-rep-research-lab",   "rep_research_lab" },
        { "sw-rep-command-center", "rep_command_center" },
        { "sw-rep-supply-depot",   "rep_supply_station" },
        // ── CIS units ──
        { "sw-cis-b1-battle-droid","cis_b1_battle_droid" },
        { "sw-b1-squad",           "cis_b1_squad" },
        { "sw-cis-b2-super-droid", "sw_b2_super_droid" },
        { "sw-cis-sniper-droid",   "cis_sniper_droid" },
        { "sw-cis-stap",           "cis_stap_speeder" },
        { "sw-aat-walker",         "sw_aat_walker" },
        { "sw-medical-droid",      "cis_medical_droid" },
        { "sw-probe-droid",        "cis_probe_droid" },
        { "sw-cis-commando-droid", "cis_bx_commando_droid" },
        { "sw-general-grievous",   "sw_general_grievous" },
        { "sw-cis-droideka",       "sw_droideka" },
        { "sw-cis-spider-droid",   "cis_dwarf_spider_droid" },
        { "sw-cis-magna-guard",    "cis_magnaguard" },
        { "sw-tri-fighter",        "cis_tri_fighter" },
        // ── CIS buildings ──
        { "sw-cis-droid-factory",  "cis_droid_factory" },
        { "sw-assembly-line",      "cis_assembly_line" },
        { "sw-heavy-foundry",      "cis_heavy_foundry" },
        { "sw-cis-aa-tower",       "cis_sentry_turret" },
        { "sw-cis-shield-generator","cis_ray_shield" },
        { "sw-mining-facility",    "cis_mining_facility" },
        { "sw-tech-union-lab",     "cis_tech_union_lab" },
    };

    public static void Run()
    {
        try
        {
            Debug.Log("[BuildAll] Generating prefabs...");
            EnsureFolders();
            AssetDatabase.ImportAsset("Assets/Models", ImportAssetOptions.ImportRecursive);
            AssetDatabase.Refresh();
            int created = 0;
            foreach (var def in Defs)
            {
                // ALWAYS upgrade the material to URP — even when the prefab already
                // exists. The old code only touched the material inside CreatePrefab,
                // so pre-existing prefabs kept their built-in 'Standard' shader
                // material (HRV2 rejects it → 0 unit swaps, native render). The
                // prefab references the material by GUID, so upgrading the .mat
                // in-place is sufficient to fix the bundled shader.
                EnsureUrpMaterial(def.Key, def.Folder, def.Color);

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

    /// <returns>true if created/upgraded, false if skipped.</returns>
    private static bool CreatePrefab(string key, string folder, Color color, PrimitiveType shape)
    {
        string prefabPath = $"Assets/Prefabs/{folder}/{key}.prefab";

        // Resolve a real glb mesh source for this key, if one is mapped + available.
        string modelPath = null;
        if (ModelMap.TryGetValue(key, out string modelName))
            modelPath = EnsureUsableModelAsset(modelName);

        bool exists = AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null;
        if (exists && modelPath == null)
        {
            // No real mesh to upgrade to — leave the (possibly real-mesh) prefab
            // produced by an earlier builder untouched.
            Debug.Log($"  [skip] {key}");
            return false;
        }

        string matPath = $"Assets/Materials/{folder}/{key}.mat";
        var mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
        if (mat == null)
        {
            mat = CreateUrpMaterial(color);
            AssetDatabase.CreateAsset(mat, matPath);
            var mi = AssetImporter.GetAtPath(matPath);
            if (mi != null) mi.assetBundleName = key;
        }
        else
        {
            UpgradeToUrp(mat, color);
        }

        GameObject go = null;
        if (modelPath != null)
        {
            var modelPrefab = AssetDatabase.LoadAssetAtPath<GameObject>(modelPath);
            if (modelPrefab != null)
            {
                go = (GameObject)PrefabUtility.InstantiatePrefab(modelPrefab);
                go.name = key;
                int verts = go.GetComponentsInChildren<MeshFilter>()
                    .Where(mf => mf.sharedMesh != null)
                    .Sum(mf => mf.sharedMesh.vertexCount);
                Debug.Log($"  [mesh] {key} <- {modelPath} (vertexCount={verts})");
            }
            else
            {
                Debug.LogWarning($"  [warn] {key}: glb {modelPath} loaded null GameObject; falling back to primitive");
            }
        }

        if (go == null)
        {
            go = GameObject.CreatePrimitive(shape);
            go.name = key;
            Debug.Log($"  [prim] {key} (no mesh source)");
        }

        foreach (var r in go.GetComponentsInChildren<Renderer>())
            r.sharedMaterial = mat;

        if (exists)
            AssetDatabase.DeleteAsset(prefabPath);
        PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
        GameObject.DestroyImmediate(go);

        var pi = AssetImporter.GetAtPath(prefabPath);
        if (pi != null) pi.assetBundleName = key;

        return true;
    }

    private static string EnsureUsableModelAsset(string modelName)
    {
        if (string.IsNullOrWhiteSpace(modelName))
            return null;

        foreach (string guid in AssetDatabase.FindAssets($"{modelName} t:Model", new[] { "Assets/Models" }))
        {
            string path = AssetDatabase.GUIDToAssetPath(guid);
            // Exact basename match only — avoid "rep_clone_sniper" matching "..._sketchfab_001".
            if (!string.Equals(Path.GetFileNameWithoutExtension(path), modelName, StringComparison.OrdinalIgnoreCase))
                continue;
            string ext = Path.GetExtension(path).ToLowerInvariant();
            if (ext == ".fbx" || ext == ".glb" || ext == ".gltf")
                return path;
        }

        string source = ResolveRawModelSource(modelName);
        if (source == null)
            return null;

        string glbTarget = $"Assets/Models/{modelName}.glb";
        string fullTarget = Path.Combine(Application.dataPath, "Models", $"{modelName}.glb");
        Directory.CreateDirectory(Path.GetDirectoryName(fullTarget)!);
        File.Copy(source, fullTarget, true);
        AssetDatabase.ImportAsset(glbTarget, ImportAssetOptions.ForceSynchronousImport | ImportAssetOptions.ForceUpdate);
        return glbTarget;
    }

    private static string ResolveRawModelSource(string modelName)
    {
        string rawRoot = Path.GetFullPath(Path.Combine(Application.dataPath, "..", "..", "packs", "warfare-starwars", "assets", "raw"));
        if (!Directory.Exists(rawRoot))
            return null;

        string normalized = modelName.ToLowerInvariant();
        string bestMatch = null;
        foreach (var dir in Directory.GetDirectories(rawRoot))
        {
            string dirName = Path.GetFileName(dir).ToLowerInvariant();
            if (!dirName.StartsWith(normalized))
                continue;
            string candidate = Path.Combine(dir, "model.glb");
            if (!File.Exists(candidate))
                continue;
            bool bestIsLego = bestMatch != null && bestMatch.Contains("_lego");
            bool candidateIsLego = dirName.Contains("_lego");
            if (bestMatch == null || (bestIsLego && !candidateIsLego))
                bestMatch = candidate;
        }
        return bestMatch;
    }

    /// <summary>
    /// Guarantees the material for <paramref name="key"/> exists and uses a URP shader,
    /// regardless of whether the prefab already exists. Creates it if missing, upgrades
    /// it in-place (preserving GUID, so existing prefab references stay valid) otherwise.
    /// </summary>
    private static void EnsureUrpMaterial(string key, string folder, Color color)
    {
        string matPath = $"Assets/Materials/{folder}/{key}.mat";
        var mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
        if (mat == null)
        {
            mat = CreateUrpMaterial(color);
            AssetDatabase.CreateAsset(mat, matPath);
            var mi = AssetImporter.GetAtPath(matPath);
            if (mi != null) mi.assetBundleName = key;
        }
        else
        {
            UpgradeToUrp(mat, color);
        }
    }

    private static Material CreateUrpMaterial(Color tint)
    {
        Shader shader = GetUrpShader();
        Material mat = new Material(shader);
        mat.SetColor("_BaseColor", tint);
        LogShader("BuildAll", mat);
        return mat;
    }

    private static void UpgradeToUrp(Material material, Color tint)
    {
        Shader shader = GetUrpShader();
        material.shader = shader;
        material.SetColor("_BaseColor", tint);
        EditorUtility.SetDirty(material);
        LogShader("BuildAll", material);
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
