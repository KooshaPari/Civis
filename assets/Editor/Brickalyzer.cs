#if UNITY_EDITOR
#nullable enable
using System;
using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

namespace DINOForge.EditorTools
{
    /// <summary>
    /// Prototype mesh-to-brick converter for the Brickalyzer / Legolizer art mode.
    ///
    /// The tool voxelizes the selected mesh against its bounds, emits a cube for each filled voxel,
    /// merges the cubes into a single mesh, and applies a URP/Lit material so the result stays
    /// compatible with the game's URP/HRV2 render path.
    /// </summary>
    public static class Brickalyzer
    {
        private const float DefaultBrickSize = 0.25f;
        private const string OutputSuffix = "_Brickalyzed";
        private const string UrpLitShaderName = "Universal Render Pipeline/Lit";
        private static Mesh? s_unitCubeMesh;

        [MenuItem("Tools/DINOForge/Brickalyzer/Brickalyze Selected Mesh")]
        private static void BrickalyzeSelectedMesh()
        {
            GameObject? selection = Selection.activeGameObject;
            if (selection == null)
            {
                EditorUtility.DisplayDialog("Brickalyzer", "Select a GameObject with a MeshFilter first.", "OK");
                return;
            }

            MeshFilter? meshFilter = selection.GetComponent<MeshFilter>();
            MeshRenderer? meshRenderer = selection.GetComponent<MeshRenderer>();
            if (meshFilter == null || meshFilter.sharedMesh == null || meshRenderer == null)
            {
                EditorUtility.DisplayDialog("Brickalyzer", "Selected object must have both a MeshFilter and MeshRenderer.", "OK");
                return;
            }

            Mesh sourceMesh = meshFilter.sharedMesh;
            float brickSize = Mathf.Max(0.05f, DefaultBrickSize);

            Mesh? brickMesh = BuildBrickMesh(sourceMesh, brickSize);
            if (brickMesh == null)
            {
                EditorUtility.DisplayDialog("Brickalyzer", "No filled voxels were detected for the selected mesh.", "OK");
                return;
            }

            Material brickMaterial = CreateBrickMaterial(meshRenderer.sharedMaterial);
            GameObject output = new GameObject(selection.name + OutputSuffix);
            output.transform.SetPositionAndRotation(selection.transform.position, selection.transform.rotation);
            output.transform.localScale = selection.transform.localScale;

            MeshFilter outputFilter = output.AddComponent<MeshFilter>();
            outputFilter.sharedMesh = brickMesh;

            MeshRenderer outputRenderer = output.AddComponent<MeshRenderer>();
            outputRenderer.sharedMaterial = brickMaterial;

            Undo.RegisterCreatedObjectUndo(output, "Brickalyze Mesh");
            Selection.activeGameObject = output;
        }

        private static Mesh? BuildBrickMesh(Mesh sourceMesh, float brickSize)
        {
            Bounds bounds = sourceMesh.bounds;
            Vector3 min = bounds.min;
            Vector3 max = bounds.max;
            Vector3[] vertices = sourceMesh.vertices;
            int[] triangles = sourceMesh.triangles;

            List<CombineInstance> combines = new List<CombineInstance>();
            Mesh cubeMesh = GetUnitCubeMesh();
            int countX = Mathf.Max(1, Mathf.CeilToInt((max.x - min.x) / brickSize));
            int countY = Mathf.Max(1, Mathf.CeilToInt((max.y - min.y) / brickSize));
            int countZ = Mathf.Max(1, Mathf.CeilToInt((max.z - min.z) / brickSize));

            for (int x = 0; x < countX; x++)
            {
                for (int y = 0; y < countY; y++)
                {
                    for (int z = 0; z < countZ; z++)
                    {
                        Vector3 center = new Vector3(
                            min.x + (x + 0.5f) * brickSize,
                            min.y + (y + 0.5f) * brickSize,
                            min.z + (z + 0.5f) * brickSize);

                        if (!IsFilled(bounds, vertices, triangles, center))
                        {
                            continue;
                        }

                        CombineInstance combine = new CombineInstance
                        {
                            mesh = cubeMesh,
                            transform = Matrix4x4.TRS(center, Quaternion.identity, Vector3.one * brickSize)
                        };
                        combines.Add(combine);
                    }
                }
            }

            if (combines.Count == 0)
            {
                return null;
            }

            Mesh output = new Mesh
            {
                name = sourceMesh.name + "_BrickMesh"
            };
            output.indexFormat = UnityEngine.Rendering.IndexFormat.UInt32;
            output.CombineMeshes(combines.ToArray(), true, true, false);
            output.RecalculateBounds();
            output.RecalculateNormals();
            return output;
        }

        private static bool IsFilled(Bounds bounds, Vector3[] vertices, int[] triangles, Vector3 center)
        {
            if (!bounds.Contains(center))
            {
                return false;
            }

            Vector3 origin = center + Vector3.left * 10000f;
            Vector3 direction = Vector3.right;
            int hitCount = 0;

            for (int i = 0; i < triangles.Length; i += 3)
            {
                Vector3 a = vertices[triangles[i]];
                Vector3 b = vertices[triangles[i + 1]];
                Vector3 c = vertices[triangles[i + 2]];
                if (RayIntersectsTriangle(origin, direction, a, b, c, out float distance) && distance > 0f)
                {
                    hitCount++;
                }
            }

            return (hitCount & 1) == 1;
        }

        private static bool RayIntersectsTriangle(
            Vector3 origin,
            Vector3 direction,
            Vector3 v0,
            Vector3 v1,
            Vector3 v2,
            out float distance)
        {
            const float epsilon = 0.000001f;
            distance = 0f;

            Vector3 edge1 = v1 - v0;
            Vector3 edge2 = v2 - v0;
            Vector3 h = Vector3.Cross(direction, edge2);
            float a = Vector3.Dot(edge1, h);
            if (Mathf.Abs(a) < epsilon)
            {
                return false;
            }

            float f = 1f / a;
            Vector3 s = origin - v0;
            float u = f * Vector3.Dot(s, h);
            if (u < 0f || u > 1f)
            {
                return false;
            }

            Vector3 q = Vector3.Cross(s, edge1);
            float v = f * Vector3.Dot(direction, q);
            if (v < 0f || u + v > 1f)
            {
                return false;
            }

            distance = f * Vector3.Dot(edge2, q);
            return distance > epsilon;
        }

        private static Material CreateBrickMaterial(Material? sourceMaterial)
        {
            Shader? shader = Shader.Find(UrpLitShaderName);
            if (shader == null)
            {
                shader = Shader.Find("Hidden/InternalErrorShader");
            }

            Material material = new Material(shader ?? Shader.Find("Sprites/Default"))
            {
                name = "Brickalyzer_URP_Lit"
            };

            Color tint = sourceMaterial != null && sourceMaterial.HasProperty("_Color")
                ? sourceMaterial.color
                : Color.white;
            if (material.HasProperty("_BaseColor"))
            {
                material.SetColor("_BaseColor", tint);
            }
            else if (material.HasProperty("_Color"))
            {
                material.SetColor("_Color", tint);
            }

            return material;
        }

        private static Mesh GetUnitCubeMesh()
        {
            if (s_unitCubeMesh != null)
            {
                return s_unitCubeMesh;
            }

            GameObject temp = GameObject.CreatePrimitive(PrimitiveType.Cube);
            try
            {
                s_unitCubeMesh = temp.GetComponent<MeshFilter>()!.sharedMesh;
                return s_unitCubeMesh;
            }
            finally
            {
                UnityEngine.Object.DestroyImmediate(temp);
            }
        }
    }
}
#endif
