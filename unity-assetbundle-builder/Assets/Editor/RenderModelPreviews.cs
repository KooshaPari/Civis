using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEditor.SceneManagement;

namespace DinoForge
{
    /// <summary>
    /// Headless preview-render gallery for the warfare-starwars shipped prefabs.
    ///
    /// Renders EXACTLY what the AssetBundle build ships: the prefabs under
    /// Assets/Prefabs/{Republic,CIS}/ — which reference the real FBX meshes from
    /// Assets/Models/ and the Standard-shader faction-tinted materials from
    /// Assets/Materials/ (same construction as GenerateStarWarsPrefabsFromModels /
    /// BuildAll). No extra post-proc is applied at bundle time (BuildAll just calls
    /// BuildPipeline.BuildAssetBundles with ChunkBasedCompression), so the preview
    /// reproduces the shipped look: real mesh + faction material under neutral lighting.
    ///
    /// Each prefab is instantiated into a temp scene, framed by an auto-fit camera,
    /// lit with a 3-point rig + ambient, on a neutral background, and rendered to a
    /// PNG via RenderTexture readback (works in -batchmode WITHOUT -nographics).
    ///
    /// Output: E:\DinoWork\model-previews\warfare-starwars\&lt;PrefabName&gt;[_angle].png
    ///
    /// Run:
    ///   Unity.exe -batchmode -projectPath . -executeMethod DinoForge.PreviewRenderer.RenderAll -quit
    /// (omit -nographics so a GPU context is available for RenderTexture readback)
    /// </summary>
    public static class PreviewRenderer
    {
        private const string OutDir = @"E:\DinoWork\model-previews\warfare-starwars";
        private const int Res = 1024;

        // Neutral studio-grey background, slightly warm.
        private static readonly Color BgColor = new Color(0.16f, 0.17f, 0.19f, 1f);

        // Camera angles to render per prefab: (name, eulerYaw, eulerPitch).
        private static readonly (string Name, float Yaw, float Pitch)[] Angles =
        {
            ("front",  0f,   12f),
            ("hero",  35f,   22f),   // 3/4 hero view
        };

        private const string ModelsOutDir = @"E:\DinoWork\model-previews\warfare-starwars-models";

        /// <summary>
        /// Renders the REAL post-processed FBX models under Assets/Models/ directly
        /// (the actual detailed geometry, not the primitive-placeholder prefabs under
        /// Assets/Prefabs/). Each imported model asset is instantiated, auto-framed,
        /// lit, and rendered to a PNG.
        /// </summary>
        public static void RenderModels()
        {
            int total = 0, ok = 0, failed = 0;
            var failures = new List<string>();
            try
            {
                Directory.CreateDirectory(ModelsOutDir);
                Debug.Log($"[PreviewRenderer] Models output dir: {ModelsOutDir}");
                AssetDatabase.Refresh();

                string[] modelGuids = AssetDatabase.FindAssets("t:Model", new[] { "Assets/Models" });
                Debug.Log($"[PreviewRenderer] Found {modelGuids.Length} models under Assets/Models");

                EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);
                Camera cam = BuildCamera();
                BuildLighting();
                RenderSettings.ambientMode = UnityEngine.Rendering.AmbientMode.Flat;
                RenderSettings.ambientLight = new Color(0.35f, 0.36f, 0.40f);

                var rt = new RenderTexture(Res, Res, 24, RenderTextureFormat.ARGB32) { antiAliasing = 8 };

                foreach (string guid in modelGuids.OrderBy(g => g))
                {
                    string path = AssetDatabase.GUIDToAssetPath(guid);
                    string modelName = Path.GetFileNameWithoutExtension(path);
                    var modelAsset = AssetDatabase.LoadAssetAtPath<GameObject>(path);
                    if (modelAsset == null) { failed++; failures.Add($"{modelName}: load null"); continue; }

                    GameObject inst = null;
                    try
                    {
                        inst = (GameObject)UnityEngine.Object.Instantiate(modelAsset);
                        inst.transform.position = Vector3.zero;
                        inst.transform.rotation = Quaternion.identity;

                        var renderers = inst.GetComponentsInChildren<Renderer>();
                        if (renderers.Length == 0) { failed++; failures.Add($"{modelName}: no renderers"); continue; }

                        Bounds b = renderers[0].bounds;
                        foreach (var r in renderers) b.Encapsulate(r.bounds);

                        bool wroteAny = false;
                        foreach (var ang in Angles)
                        {
                            FrameCamera(cam, b, ang.Yaw, ang.Pitch);
                            string outPath = Path.Combine(ModelsOutDir, $"{modelName}_{ang.Name}.png");
                            if (RenderToPng(cam, rt, outPath)) { wroteAny = true; total++; }
                        }
                        if (wroteAny) ok++; else { failed++; failures.Add($"{modelName}: no file"); }
                    }
                    catch (Exception exItem) { failed++; failures.Add($"{modelName}: {exItem.Message}"); }
                    finally { if (inst != null) UnityEngine.Object.DestroyImmediate(inst); }
                }

                rt.Release(); UnityEngine.Object.DestroyImmediate(rt);
                Debug.Log($"[PreviewRenderer] MODELS DONE. ok={ok} failed={failed} images={total}");
                if (failures.Count > 0) Debug.LogWarning("[PreviewRenderer] MODEL FAILURES:\n" + string.Join("\n", failures));
                Debug.Log($"[PreviewRenderer] MODELS SUMMARY ok={ok} failed={failed} images={total} outdir={ModelsOutDir}");
                EditorApplication.Exit(0);
            }
            catch (Exception ex)
            {
                Debug.LogError($"[PreviewRenderer] MODELS FATAL: {ex}");
                EditorApplication.Exit(1);
            }
        }

        public static void RenderAll()
        {
            int total = 0, ok = 0, failed = 0;
            var failures = new List<string>();

            try
            {
                Directory.CreateDirectory(OutDir);
                Debug.Log($"[PreviewRenderer] Output dir: {OutDir}");

                // Make sure imported meshes/materials are up to date.
                AssetDatabase.Refresh();

                string[] prefabGuids = AssetDatabase.FindAssets("t:Prefab", new[] { "Assets/Prefabs" });
                Debug.Log($"[PreviewRenderer] Found {prefabGuids.Length} prefabs under Assets/Prefabs");

                // Build an isolated scene with lighting + camera once.
                var scene = EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);

                Camera cam = BuildCamera();
                BuildLighting();
                RenderSettings.ambientMode = UnityEngine.Rendering.AmbientMode.Flat;
                RenderSettings.ambientLight = new Color(0.35f, 0.36f, 0.40f);

                var rt = new RenderTexture(Res, Res, 24, RenderTextureFormat.ARGB32)
                {
                    antiAliasing = 8
                };

                foreach (string guid in prefabGuids.OrderBy(g => g))
                {
                    string path = AssetDatabase.GUIDToAssetPath(guid);
                    string prefabName = Path.GetFileNameWithoutExtension(path);
                    var prefabAsset = AssetDatabase.LoadAssetAtPath<GameObject>(path);
                    if (prefabAsset == null)
                    {
                        failed++; failures.Add($"{prefabName}: load null");
                        continue;
                    }

                    GameObject inst = null;
                    try
                    {
                        inst = (GameObject)PrefabUtility.InstantiatePrefab(prefabAsset);
                        inst.transform.position = Vector3.zero;
                        inst.transform.rotation = Quaternion.identity;

                        // Compute combined renderer bounds.
                        var renderers = inst.GetComponentsInChildren<Renderer>();
                        if (renderers.Length == 0)
                        {
                            failed++; failures.Add($"{prefabName}: no renderers");
                            continue;
                        }

                        Bounds b = renderers[0].bounds;
                        foreach (var r in renderers) b.Encapsulate(r.bounds);

                        bool wroteAny = false;
                        foreach (var ang in Angles)
                        {
                            FrameCamera(cam, b, ang.Yaw, ang.Pitch);
                            string outPath = Path.Combine(OutDir, $"{prefabName}_{ang.Name}.png");
                            if (RenderToPng(cam, rt, outPath))
                            {
                                wroteAny = true;
                                total++;
                            }
                        }

                        if (wroteAny) { ok++; }
                        else { failed++; failures.Add($"{prefabName}: render produced no file"); }
                    }
                    catch (Exception exItem)
                    {
                        failed++; failures.Add($"{prefabName}: {exItem.Message}");
                    }
                    finally
                    {
                        if (inst != null) UnityEngine.Object.DestroyImmediate(inst);
                    }
                }

                rt.Release();
                UnityEngine.Object.DestroyImmediate(rt);

                Debug.Log($"[PreviewRenderer] DONE. prefabs_ok={ok} prefabs_failed={failed} images_written={total}");
                if (failures.Count > 0)
                    Debug.LogWarning("[PreviewRenderer] FAILURES:\n" + string.Join("\n", failures));

                // Machine-readable summary line for the harness to grep.
                Debug.Log($"[PreviewRenderer] SUMMARY ok={ok} failed={failed} images={total} outdir={OutDir}");

                EditorApplication.Exit(0);
            }
            catch (Exception ex)
            {
                Debug.LogError($"[PreviewRenderer] FATAL: {ex}");
                EditorApplication.Exit(1);
            }
        }

        private static Camera BuildCamera()
        {
            var camGo = new GameObject("PreviewCamera");
            var cam = camGo.AddComponent<Camera>();
            cam.clearFlags = CameraClearFlags.SolidColor;
            cam.backgroundColor = BgColor;
            cam.fieldOfView = 35f;
            cam.nearClipPlane = 0.01f;
            cam.farClipPlane = 5000f;
            cam.allowHDR = true;
            cam.allowMSAA = true;
            return cam;
        }

        private static void BuildLighting()
        {
            // Key light.
            MakeLight("Key",  new Vector3(40f, 35f, 0f),  1.35f, new Color(1.0f, 0.97f, 0.92f));
            // Fill light (softer, opposite side).
            MakeLight("Fill", new Vector3(15f, -55f, 0f), 0.55f, new Color(0.85f, 0.90f, 1.0f));
            // Rim / back light.
            MakeLight("Rim",  new Vector3(-10f, 200f, 0f), 0.9f, new Color(0.9f, 0.95f, 1.0f));
        }

        private static void MakeLight(string name, Vector3 euler, float intensity, Color color)
        {
            var go = new GameObject($"Light_{name}");
            var l = go.AddComponent<Light>();
            l.type = LightType.Directional;
            l.intensity = intensity;
            l.color = color;
            l.shadows = LightShadows.Soft;
            go.transform.rotation = Quaternion.Euler(euler);
        }

        /// <summary>Auto-fit the camera to the object's bounds from a given yaw/pitch.</summary>
        private static void FrameCamera(Camera cam, Bounds b, float yaw, float pitch)
        {
            float radius = b.extents.magnitude;
            if (radius < 0.0001f) radius = 1f;

            float fovRad = cam.fieldOfView * Mathf.Deg2Rad;
            float dist = radius / Mathf.Sin(fovRad * 0.5f);
            dist *= 1.25f; // margin

            Quaternion rot = Quaternion.Euler(pitch, yaw, 0f);
            Vector3 dir = rot * Vector3.forward;
            Vector3 camPos = b.center - dir * dist;

            cam.transform.position = camPos;
            cam.transform.LookAt(b.center);
            cam.nearClipPlane = Mathf.Max(0.01f, dist - radius * 2f);
            cam.farClipPlane = dist + radius * 4f;
        }

        private static bool RenderToPng(Camera cam, RenderTexture rt, string outPath)
        {
            RenderTexture prevTarget = cam.targetTexture;
            RenderTexture prevActive = RenderTexture.active;
            try
            {
                cam.targetTexture = rt;
                cam.Render();

                RenderTexture.active = rt;
                var tex = new Texture2D(rt.width, rt.height, TextureFormat.RGBA32, false);
                tex.ReadPixels(new Rect(0, 0, rt.width, rt.height), 0, 0);
                tex.Apply();

                byte[] png = tex.EncodeToPNG();
                File.WriteAllBytes(outPath, png);
                UnityEngine.Object.DestroyImmediate(tex);

                return png != null && png.Length > 0;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[PreviewRenderer] render fail {Path.GetFileName(outPath)}: {ex.Message}");
                return false;
            }
            finally
            {
                cam.targetTexture = prevTarget;
                RenderTexture.active = prevActive;
            }
        }
    }
}
