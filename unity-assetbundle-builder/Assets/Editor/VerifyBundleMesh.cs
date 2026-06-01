using System;
using System.IO;
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
    public static void Run()
    {
        try
        {
            const string bundleDir = "AssetBundles";
            if (!Directory.Exists(bundleDir))
            {
                Debug.Log($"[VERIFY] missing output directory: {bundleDir}");
                EditorApplication.Exit(1);
                return;
            }

            string reportPath = Path.Combine(Directory.GetParent(Application.dataPath)!.FullName, "sw-bundle-verify-report.log");
            if (File.Exists(reportPath))
                File.Delete(reportPath);
            AppendLine(reportPath, $"Bundle verification (AssetBundles/*) on {DateTime.UtcNow:O}");

            foreach (string path in Directory.GetFiles(bundleDir))
            {
                if (Path.HasExtension(path))
                    continue;

                string key = Path.GetFileName(path);
                var ab = AssetBundle.LoadFromFile(path);
                if (ab == null)
                {
                    AppendLine(reportPath, $"[VERIFY] {key}: load failed");
                    Debug.Log($"[VERIFY] {key}: load failed");
                    continue;
                }

                int maxVerts = 0; int meshCount = 0;
                var shaderSet = new System.Collections.Generic.HashSet<string>();
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

                    foreach (var r in go.GetComponentsInChildren<Renderer>(true))
                    {
                        if (r.sharedMaterial != null && r.sharedMaterial.shader != null)
                            shaderSet.Add(r.sharedMaterial.shader.name);
                    }
                }

                string shaderReport = string.Join(",", shaderSet.OrderBy(x => x));
                string line = $"[VERIFY] {key}: meshes={meshCount} maxVerts={maxVerts} shaders={shaderReport}";
                AppendLine(reportPath, line);
                Debug.Log(line);
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

    private static void AppendLine(string path, string line)
    {
        File.AppendAllText(path, line + Environment.NewLine);
    }
}
