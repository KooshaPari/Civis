using System;
using System.Linq;
using UnityEditor;
using UnityEngine;

/// <summary>
/// Loads built AssetBundles back and reports mesh vertex counts to prove
/// they contain real (non-primitive) meshes. Primitive cube/capsule = 24/... verts.
/// Run: Unity.exe -batchmode -nographics -executeMethod VerifyBundleMesh.Run -quit
/// </summary>
public static class VerifyBundleMesh
{
    private static readonly string[] Keys =
    {
        "sw-cis-spider-droid", "sw-rep-clone-commander", "sw-rep-at-te-walker",
        "sw-cis-droideka", "sw-rep-clone-pilot" // last is a known primitive control
    };

    public static void Run()
    {
        try
        {
            foreach (string key in Keys)
            {
                string path = System.IO.Path.Combine("AssetBundles", key);
                if (!System.IO.File.Exists(path)) { Debug.Log($"[VERIFY] {key}: MISSING"); continue; }
                var ab = AssetBundle.LoadFromFile(path);
                if (ab == null) { Debug.Log($"[VERIFY] {key}: load failed"); continue; }
                int maxVerts = 0; int meshCount = 0;
                foreach (var go in ab.LoadAllAssets<GameObject>())
                {
                    foreach (var mf in go.GetComponentsInChildren<MeshFilter>(true))
                    {
                        if (mf.sharedMesh != null)
                        {
                            meshCount++;
                            maxVerts = Mathf.Max(maxVerts, mf.sharedMesh.vertexCount);
                        }
                    }
                }
                Debug.Log($"[VERIFY] {key}: meshes={meshCount} maxVerts={maxVerts}");
                ab.Unload(true);
            }
            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[VerifyBundleMesh] {ex}");
            EditorApplication.Exit(1);
        }
    }
}
