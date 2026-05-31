using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Builds REAL Unity 2021.3.45f1 AssetBundles for the 12 remaining Star Wars BUILDING
/// stub bundles (task #982). Each was a 90-byte placeholder, causing the building to stay
/// native in-game and the AssetSwap log to flood "another AssetBundle with the same files
/// is already loaded".
///
/// Bundle filename == visual_asset key == Addressable key (CLAUDE.md asset pipeline).
///
/// Source-mesh decision per building:
///   * Each building gets a CLEAN-ROOM PROCEDURAL mesh generated in C# (no external Blender
///     / glb round-trip, no glTF importer dependency). These are recognizable blocky
///     SW-styled building silhouettes (factory, wall, tower, refinery, etc.), NOT 90-byte
///     stubs and NOT a single cube. Faction-tinted Standard material (CIS grey / Republic white).
///
/// Run headless (one Unity.exe at a time):
///   Unity.exe -batchmode -nographics -quit -projectPath . -executeMethod BuildSwBuildingBundles.Build -logFile build.log
/// </summary>
public static class BuildSwBuildingBundles
{
    private const string OutputDir = "AssetBundles";

    private enum Arch { Factory, Wall, Tower, Refinery, Mine, Lab, Shield, Nest, Barrier }

    // (bundleKey == visual_asset, faction, archetype)
    private static readonly (string Key, string Faction, Arch Arch)[] Defs =
    {
        ("sw-assembly-line",       "CIS",      Arch.Factory),
        ("sw-blast-wall",          "Republic", Arch.Wall),
        ("sw-durasteel-barrier",   "CIS",      Arch.Barrier),
        ("sw-guard-tower",         "Republic", Arch.Tower),
        ("sw-heavy-foundry",       "CIS",      Arch.Factory),
        ("sw-mining-facility",     "CIS",      Arch.Mine),
        ("sw-processing-plant",    "CIS",      Arch.Refinery),
        ("sw-skyshield-generator", "Republic", Arch.Shield),
        ("sw-tech-union-lab",      "CIS",      Arch.Lab),
        ("sw-tibanna-refinery",    "Republic", Arch.Refinery),
        ("sw-vulture-nest",        "CIS",      Arch.Nest),
        ("sw-weapons-factory",     "Republic", Arch.Factory),
    };

    private static readonly Color RepublicWhite = new Color(0.92f, 0.92f, 0.90f);
    private static readonly Color CisGrey = new Color(0.50f, 0.50f, 0.46f);

    public static void Build()
    {
        try
        {
            EnsureDirs();
            int ok = 0;
            foreach (var def in Defs)
            {
                string faction = def.Faction;
                string matPath = $"Assets/Materials/{faction}/{def.Key}.mat";
                string meshPath = $"Assets/Meshes/{def.Key}.asset";
                string prefabPath = $"Assets/Prefabs/{faction}/{def.Key}.prefab";

                // Material
                var mat = new Material(Shader.Find("Standard"))
                { color = faction == "CIS" ? CisGrey : RepublicWhite };
                AssetDatabase.CreateAsset(mat, matPath);

                // Mesh (procedural, baked to asset so it persists in the bundle)
                Mesh mesh = BuildMesh(def.Arch);
                mesh.name = def.Key;
                AssetDatabase.CreateAsset(mesh, meshPath);

                // Prefab: GameObject with MeshFilter + MeshRenderer
                var go = new GameObject(def.Key);
                go.AddComponent<MeshFilter>().sharedMesh = mesh;
                go.AddComponent<MeshRenderer>().sharedMaterial = mat;
                PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
                GameObject.DestroyImmediate(go);

                var pi = AssetImporter.GetAtPath(prefabPath);
                if (pi != null) pi.assetBundleName = def.Key;
                ok++;
                Debug.Log($"[SwBuildings] prepared {def.Key} ({def.Arch}, {faction})");
            }

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            if (!Directory.Exists(OutputDir)) Directory.CreateDirectory(OutputDir);
            var manifest = BuildPipeline.BuildAssetBundles(
                OutputDir, BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);
            if (manifest == null)
            {
                Debug.LogError("[SwBuildings] manifest null");
                EditorApplication.Exit(1);
                return;
            }

            Debug.Log($"[SwBuildings] Done: {ok} prefabs. Bundles built:");
            foreach (var b in manifest.GetAllAssetBundles())
                Debug.Log($"  {b}");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[SwBuildings] EXCEPTION {ex}");
            EditorApplication.Exit(1);
        }
    }

    // ---------------- procedural building meshes (CombineInstance of box primitives) -----

    private static Mesh BuildMesh(Arch a)
    {
        var parts = new List<CombineInstance>();
        switch (a)
        {
            case Arch.Factory:   Factory(parts); break;
            case Arch.Wall:      Wall(parts); break;
            case Arch.Tower:     Tower(parts); break;
            case Arch.Refinery:  Refinery(parts); break;
            case Arch.Mine:      Mine(parts); break;
            case Arch.Lab:       Lab(parts); break;
            case Arch.Shield:    Shield(parts); break;
            case Arch.Nest:      Nest(parts); break;
            case Arch.Barrier:   Barrier(parts); break;
        }
        var mesh = new Mesh { indexFormat = UnityEngine.Rendering.IndexFormat.UInt32 };
        mesh.CombineMeshes(parts.ToArray(), true, true);
        mesh.RecalculateNormals();
        mesh.RecalculateBounds();
        return mesh;
    }

    private static void Box(List<CombineInstance> parts, Vector3 center, Vector3 size, Quaternion rot)
    {
        parts.Add(new CombineInstance
        {
            mesh = UnitCube(),
            transform = Matrix4x4.TRS(center, rot, size),
        });
    }
    private static void Box(List<CombineInstance> parts, Vector3 center, Vector3 size)
        => Box(parts, center, size, Quaternion.identity);

    private static Mesh _cube;
    private static Mesh UnitCube()
    {
        if (_cube != null) return _cube;
        // Build a unit cube (1x1x1 centered) once.
        var go = GameObject.CreatePrimitive(PrimitiveType.Cube);
        _cube = UnityEngine.Object.Instantiate(go.GetComponent<MeshFilter>().sharedMesh);
        GameObject.DestroyImmediate(go);
        return _cube;
    }

    // Wide industrial hall with roof vents + chimney stacks.
    private static void Factory(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 1.0f, 0), new Vector3(4.0f, 2.0f, 3.0f));
        Box(p, new Vector3(0, 2.3f, 0), new Vector3(3.6f, 0.6f, 2.6f));       // raised roof
        for (int i = -1; i <= 1; i++)
            Box(p, new Vector3(i * 1.0f, 2.9f, 0.8f), new Vector3(0.4f, 1.4f, 0.4f)); // chimneys
        Box(p, new Vector3(1.6f, 0.9f, 1.6f), new Vector3(0.6f, 1.8f, 0.6f)); // corner pylon
        Box(p, new Vector3(-1.6f, 0.9f, 1.6f), new Vector3(0.6f, 1.8f, 0.6f));
    }

    // Long low defensive wall with crenellations.
    private static void Wall(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.9f, 0), new Vector3(5.0f, 1.8f, 0.7f));
        for (int i = -2; i <= 2; i++)
            Box(p, new Vector3(i * 1.1f, 1.9f, 0), new Vector3(0.7f, 0.5f, 0.9f)); // merlons
        Box(p, new Vector3(-2.6f, 1.2f, 0), new Vector3(0.9f, 2.4f, 1.1f));        // end towers
        Box(p, new Vector3(2.6f, 1.2f, 0), new Vector3(0.9f, 2.4f, 1.1f));
    }

    // Tall guard tower: tapered shaft + flared head + antenna.
    private static void Tower(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 1.4f, 0), new Vector3(1.2f, 2.8f, 1.2f));
        Box(p, new Vector3(0, 3.1f, 0), new Vector3(1.8f, 0.8f, 1.8f));   // observation head
        Box(p, new Vector3(0, 3.7f, 0), new Vector3(1.2f, 0.5f, 1.2f));
        Box(p, new Vector3(0, 4.4f, 0), new Vector3(0.12f, 1.0f, 0.12f)); // antenna
    }

    // Refinery / processing: tanks (boxy octagon approx) + piping + flare stack.
    private static void Refinery(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.5f, 0), new Vector3(4.0f, 1.0f, 3.0f));   // base pad
        Box(p, new Vector3(-1.2f, 1.6f, 0.6f), new Vector3(1.4f, 2.2f, 1.4f), Quaternion.Euler(0, 45, 0));  // tank
        Box(p, new Vector3(1.2f, 1.4f, -0.4f), new Vector3(1.2f, 1.8f, 1.2f), Quaternion.Euler(0, 45, 0));
        Box(p, new Vector3(0, 2.6f, 1.0f), new Vector3(0.3f, 2.8f, 0.3f)); // flare stack
        Box(p, new Vector3(0, 1.0f, -1.2f), new Vector3(3.0f, 0.25f, 0.25f)); // pipe run
    }

    // Mining facility: ramped pit head + conveyor + headframe.
    private static void Mine(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.6f, 0), new Vector3(3.2f, 1.2f, 3.2f));
        Box(p, new Vector3(0, 2.0f, 0), new Vector3(0.9f, 2.6f, 0.9f));       // headframe shaft
        Box(p, new Vector3(0, 3.2f, 0), new Vector3(1.6f, 0.5f, 1.6f));       // wheelhouse
        Box(p, new Vector3(1.8f, 1.0f, 0), new Vector3(2.4f, 0.3f, 0.7f), Quaternion.Euler(0, 0, -20)); // conveyor
    }

    // Tech union lab: domed (stepped box) research hall + side annexes.
    private static void Lab(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.9f, 0), new Vector3(3.2f, 1.8f, 3.2f));
        Box(p, new Vector3(0, 2.1f, 0), new Vector3(2.2f, 0.9f, 2.2f));
        Box(p, new Vector3(0, 2.8f, 0), new Vector3(1.2f, 0.7f, 1.2f));    // stepped dome
        Box(p, new Vector3(2.0f, 0.7f, 0), new Vector3(1.0f, 1.4f, 1.6f)); // annex
        Box(p, new Vector3(-2.0f, 0.7f, 0), new Vector3(1.0f, 1.4f, 1.6f));
    }

    // Skyshield / shield generator: wide drum base + emitter dish on mast.
    private static void Shield(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.7f, 0), new Vector3(3.0f, 1.4f, 3.0f), Quaternion.Euler(0, 45, 0));
        Box(p, new Vector3(0, 1.9f, 0), new Vector3(0.8f, 1.4f, 0.8f));    // mast
        Box(p, new Vector3(0, 2.8f, 0), new Vector3(2.4f, 0.5f, 2.4f), Quaternion.Euler(0, 45, 0)); // emitter dish
        Box(p, new Vector3(0, 3.2f, 0), new Vector3(0.5f, 0.6f, 0.5f));    // emitter tip
    }

    // Vulture droid nest: hangar + launch rails + folded-wing pods.
    private static void Nest(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 0.8f, 0), new Vector3(4.2f, 1.6f, 2.6f));
        Box(p, new Vector3(0, 1.8f, 0), new Vector3(3.4f, 0.6f, 2.0f), Quaternion.Euler(15, 0, 0)); // sloped roof
        for (int i = -1; i <= 1; i++)
            Box(p, new Vector3(i * 1.3f, 2.2f, -0.9f), new Vector3(0.5f, 0.5f, 1.6f)); // launch pods
    }

    // Durasteel barrier: angled blast segments + buttresses.
    private static void Barrier(List<CombineInstance> p)
    {
        Box(p, new Vector3(0, 1.0f, 0), new Vector3(4.5f, 2.0f, 0.5f));
        for (int i = -2; i <= 2; i++)
            Box(p, new Vector3(i * 1.0f, 0.6f, 0.5f), new Vector3(0.5f, 1.2f, 0.9f), Quaternion.Euler(-25, 0, 0)); // angled buttress
        Box(p, new Vector3(0, 2.1f, 0), new Vector3(4.5f, 0.3f, 0.8f)); // cap rail
    }

    private static void EnsureDirs()
    {
        string[] dirs =
        {
            "Assets/Materials", "Assets/Materials/Republic", "Assets/Materials/CIS",
            "Assets/Meshes",
            "Assets/Prefabs", "Assets/Prefabs/Republic", "Assets/Prefabs/CIS",
        };
        foreach (var d in dirs)
            if (!AssetDatabase.IsValidFolder(d))
            {
                string parent = Path.GetDirectoryName(d).Replace('\\', '/');
                AssetDatabase.CreateFolder(parent, Path.GetFileName(d));
            }
    }
}
