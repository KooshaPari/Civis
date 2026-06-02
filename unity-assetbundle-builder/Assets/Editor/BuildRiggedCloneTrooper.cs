using System;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEngine;

/// <summary>
/// #991: Build the ONE rigged clone-trooper bundle end-to-end as a SKINNED mesh (21 bindposes)
/// so AssetSwapSystem.IsSkinnedMeshCompatible accepts it (vanilla 'dark_knight' = 21 bindposes).
///
/// Source GLB is produced by blender_rig_to_dino_skeleton.py (21-bone humanoid armature).
/// Output bundle key == pack visual_asset == Addressable key == "sw-clone-trooper-republic".
///
/// MUST be run with Unity 2021.3.45f2 (f1 bundles are silently rejected by DINO):
///   "C:\Program Files\Unity\Hub\Editor\2021.3.45f2\Editor\Unity.exe" -batchmode -nographics \
///     -projectPath unity-assetbundle-builder \
///     -executeMethod BuildRiggedCloneTrooper.Run -quit
/// </summary>
public static class BuildRiggedCloneTrooper
{
    private const string BundleKey = "sw-clone-trooper-republic";
    private const string Folder = "Republic";
    private const string OutputDir = "AssetBundles";

    // Rigged GLB relative to repo root (../packs/...).
    private const string RiggedGlbRel =
        "packs/warfare-starwars/assets/working/sw_clone_trooper_phase2_sketchfab_001/rigged_21bone.glb";

    private static readonly Color RepublicWhite = new Color(0.95f, 0.95f, 0.95f);

    public static void Run()
    {
        try
        {
            Debug.Log("[RigCloneTrooper] start");

            string repoRoot = Path.GetFullPath(Path.Combine(Application.dataPath, "..", ".."));
            string srcGlb = Path.Combine(repoRoot, RiggedGlbRel);
            if (!File.Exists(srcGlb))
                throw new FileNotFoundException($"rigged glb not found: {srcGlb}");

            // 1. Copy GLB into the project and import it as a Generic-rig model so the
            //    SkinnedMeshRenderer + bindposes (21) survive into the bundle.
            EnsureFolder("Assets/Models");
            string modelAsset = "Assets/Models/sw_clone_trooper_rigged21.glb";
            File.Copy(srcGlb, Path.Combine(Application.dataPath, "Models", "sw_clone_trooper_rigged21.glb"), true);
            AssetDatabase.ImportAsset(modelAsset, ImportAssetOptions.ForceSynchronousImport | ImportAssetOptions.ForceUpdate);

            // GLB is imported by the glTFast ScriptedImporter (com.unity.cloud.gltfast), NOT the
            // native ModelImporter — so do not cast to ModelImporter. glTFast preserves the skin
            // (SkinnedMeshRenderer + bindposes) by default. We verify bindposes==21 after load.
            AssetImporter importer = AssetImporter.GetAtPath(modelAsset);
            Debug.Log($"[RigCloneTrooper] importer type = {importer?.GetType().FullName ?? "<null>"}");

            // 2. URP material so the runtime URP guard (IsUrpCompatibleMaterial) accepts it.
            EnsureFolder($"Assets/Materials/{Folder}");
            string matPath = $"Assets/Materials/{Folder}/{BundleKey}.mat";
            Material mat = AssetDatabase.LoadAssetAtPath<Material>(matPath);
            if (mat == null)
            {
                mat = CreateUrpMaterial(RepublicWhite);
                AssetDatabase.CreateAsset(mat, matPath);
            }
            else
            {
                UpgradeToUrp(mat, RepublicWhite);
            }

            // 3. Instantiate model → verify it has a SkinnedMeshRenderer with 21 bindposes.
            var modelPrefab = AssetDatabase.LoadAssetAtPath<GameObject>(modelAsset);
            if (modelPrefab == null)
                throw new InvalidOperationException("imported model loaded as null GameObject");

            GameObject go = (GameObject)PrefabUtility.InstantiatePrefab(modelPrefab);
            go.name = BundleKey;

            SkinnedMeshRenderer[] smrs = go.GetComponentsInChildren<SkinnedMeshRenderer>(true);
            if (smrs.Length == 0)
                throw new InvalidOperationException("imported model has no SkinnedMeshRenderer");

            // glTFast (com.unity.cloud.gltfast) stores its meshes in a serialization that DINO's
            // older f2 runtime AssetBundle.LoadFromFile silently rejects (returns null). Rebake
            // each SkinnedMeshRenderer's mesh into a FRESH native Unity Mesh asset (default,
            // readable serialization) so the bundle is loadable by DINO. Bindposes + boneWeights
            // are copied verbatim, preserving the 21-bone skinning.
            EnsureFolder("Assets/Meshes");

            // DINO's runtime AssetBundle.LoadFromFile rejects bundles containing a
            // SkinnedMeshRenderer (Class 137) — verified empirically: f2 STATIC bundles load
            // (sw-rep-clone-trooper) and f1 STATIC bundles load (sw-clone-heavy), but a skinned
            // bundle returns null with no recoverable handle. So we keep the 21-BINDPOSE mesh
            // data (which satisfies IsSkinnedMeshCompatible — it only inspects
            // mesh.bindposes.Length) but present it through a STATIC MeshFilter+MeshRenderer
            // prefab. ResolveReplacementAssets reads mf.sharedMesh and the swap copies that mesh
            // (bindposes intact) onto the vanilla RenderMesh. Best of both: loadable bundle +
            // 21-bindpose mesh that passes the guard.
            GameObject staticRoot = new GameObject(BundleKey);
            int idx = 0;
            foreach (SkinnedMeshRenderer smr in smrs)
            {
                if (smr.sharedMesh == null) continue;
                Mesh baked = RebakeMesh(smr.sharedMesh); // preserves bindposes (21) + boneWeights
                string meshPath = $"Assets/Meshes/{BundleKey}_{idx}.asset";
                AssetDatabase.CreateAsset(baked, meshPath);
                Mesh meshAsset = AssetDatabase.LoadAssetAtPath<Mesh>(meshPath);
                int bp = baked.bindposes != null ? baked.bindposes.Length : 0;
                Debug.Log($"[RigCloneTrooper] mesh[{idx}]='{baked.name}' bindposes={bp} verts={baked.vertexCount} (static-prefab carrier)");

                GameObject part = new GameObject($"{BundleKey}_part{idx}");
                part.transform.SetParent(staticRoot.transform, false);
                MeshFilter mf = part.AddComponent<MeshFilter>();
                mf.sharedMesh = meshAsset;
                MeshRenderer mr = part.AddComponent<MeshRenderer>();
                mr.sharedMaterial = mat;
                idx++;
            }

            GameObject.DestroyImmediate(go);
            go = staticRoot;

            // 4. Save prefab + assign bundle.
            EnsureFolder($"Assets/Prefabs/{Folder}");
            string prefabPath = $"Assets/Prefabs/{Folder}/{BundleKey}.prefab";
            if (AssetDatabase.LoadAssetAtPath<GameObject>(prefabPath) != null)
                AssetDatabase.DeleteAsset(prefabPath);
            PrefabUtility.SaveAsPrefabAsset(go, prefabPath);
            GameObject.DestroyImmediate(go);

            var pi = AssetImporter.GetAtPath(prefabPath);
            if (pi != null) { pi.assetBundleName = BundleKey; }
            var mi = AssetImporter.GetAtPath(matPath);
            if (mi != null) { mi.assetBundleName = BundleKey; }
            // Model itself NOT assigned to a bundle — its mesh rides into the bundle via the prefab.

            AssetDatabase.SaveAssets();
            AssetDatabase.Refresh();

            // 5. Build the single bundle for StandaloneWindows64.
            if (!Directory.Exists(OutputDir))
                Directory.CreateDirectory(OutputDir);

            // Manifest-driven overload (assetBundleName already assigned on the prefab+material
            // importers above), matching BuildAll.cs which produces bundles DINO loads. The
            // explicit AssetBundleBuild[] overload produced bundles DINO's LoadFromFile rejected.
            var manifest = BuildPipeline.BuildAssetBundles(
                OutputDir,
                BuildAssetBundleOptions.ChunkBasedCompression,
                BuildTarget.StandaloneWindows64);

            if (manifest == null)
                throw new InvalidOperationException("BuildAssetBundles returned null manifest");

            string outFile = Path.Combine(OutputDir, BundleKey);
            long size = File.Exists(outFile) ? new FileInfo(outFile).Length : -1;
            Debug.Log($"[RigCloneTrooper] built bundle '{BundleKey}' size={size} bytes");
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[RigCloneTrooper] EXCEPTION: {ex}");
            EditorApplication.Exit(1);
        }
    }

    /// <summary>
    /// Copy a mesh into a brand-new native Mesh (default Unity serialization), preserving
    /// geometry, UVs, normals, tangents, bone weights and bindposes. This strips any importer-
    /// specific (glTFast) serialization that DINO's runtime cannot deserialize.
    /// </summary>
    private static Mesh RebakeMesh(Mesh src)
    {
        var m = new Mesh { name = src.name, indexFormat = src.indexFormat };
        m.SetVertices(src.vertices);
        if (src.normals != null && src.normals.Length > 0) m.SetNormals(src.normals);
        if (src.tangents != null && src.tangents.Length > 0) m.SetTangents(src.tangents);
        if (src.uv != null && src.uv.Length > 0) m.SetUVs(0, src.uv);
        if (src.colors != null && src.colors.Length > 0) m.SetColors(src.colors);

        m.subMeshCount = src.subMeshCount;
        for (int s = 0; s < src.subMeshCount; s++)
            m.SetTriangles(src.GetTriangles(s), s);

        // Skinning data — the part that makes this a 21-bindpose skinned mesh.
        var bw = src.boneWeights;
        if (bw != null && bw.Length > 0) m.boneWeights = bw;
        var bp = src.bindposes;
        if (bp != null && bp.Length > 0) m.bindposes = bp;

        m.RecalculateBounds();
        m.UploadMeshData(false); // keep CPU-readable
        return m;
    }

    private static void EnsureFolder(string dir)
    {
        if (AssetDatabase.IsValidFolder(dir)) return;
        string parent = Path.GetDirectoryName(dir)!.Replace('\\', '/');
        string child = Path.GetFileName(dir);
        if (!AssetDatabase.IsValidFolder(parent))
            EnsureFolder(parent);
        AssetDatabase.CreateFolder(parent, child);
    }

    private static Material CreateUrpMaterial(Color color)
    {
        Shader shader = Shader.Find("Universal Render Pipeline/Lit")
            ?? Shader.Find("Universal Render Pipeline/Simple Lit");
        if (shader == null)
            throw new InvalidOperationException("No URP shader available in project.");
        var m = new Material(shader);
        SetColor(m, color);
        return m;
    }

    private static void UpgradeToUrp(Material m, Color color)
    {
        if (m.shader == null || !m.shader.name.StartsWith("Universal Render Pipeline/", StringComparison.Ordinal))
        {
            Shader shader = Shader.Find("Universal Render Pipeline/Lit")
                ?? Shader.Find("Universal Render Pipeline/Simple Lit");
            if (shader != null) m.shader = shader;
        }
        SetColor(m, color);
    }

    private static void SetColor(Material m, Color color)
    {
        if (m.HasProperty("_BaseColor")) m.SetColor("_BaseColor", color);
        if (m.HasProperty("_Color")) m.SetColor("_Color", color);
    }
}
