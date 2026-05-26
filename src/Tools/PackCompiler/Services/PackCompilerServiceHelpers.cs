#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Shared helpers for PackCompiler services (output paths, Unity YAML stubs, mesh bounds).
    /// </summary>
    internal static class ServiceOutputHelper
    {
        public static void EnsureParentDirectory(string outputPath)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(outputPath) ?? ".");
        }

        public static void WriteUtf8Text(string outputPath, string content)
        {
            EnsureParentDirectory(outputPath);
            File.WriteAllText(outputPath, content, Encoding.UTF8);
        }

        public static void WriteUtf8Lines(string outputPath, IEnumerable<string> lines)
        {
            EnsureParentDirectory(outputPath);
            File.WriteAllLines(outputPath, lines, Encoding.UTF8);
        }
    }

    /// <summary>
    /// Common Unity YAML document headers used by Addressables and prefab writers.
    /// </summary>
    internal static class UnityYamlHelper
    {
        public static void AppendYaml11Header(StringBuilder sb)
        {
            sb.AppendLine("%YAML 1.1");
            sb.AppendLine("%TAG !u! tag:unity3d.com,2011:");
        }

        public static void AppendAddressableDocumentStart(StringBuilder sb, string rootTypeName)
        {
            AppendYaml11Header(sb);
            sb.AppendLine("--- !u!114 &11400000");
            sb.AppendLine($"{rootTypeName}:");
            AppendUnitySerializableBase(sb);
        }

        public static void AppendUnitySerializableBase(StringBuilder sb)
        {
            sb.AppendLine("  m_ObjectHideFlags: 0");
            sb.AppendLine("  m_CorrespondingSourceObject: {fileID: 0}");
            sb.AppendLine("  m_PrefabInstance: {fileID: 0}");
            sb.AppendLine("  m_PrefabAsset: {fileID: 0}");
        }
    }

    /// <summary>
    /// Axis-aligned bounds from interleaved XYZ vertex data.
    /// </summary>
    internal static class MeshBoundsHelper
    {
        public static (float[] Min, float[] Max) Calculate(float[] vertices)
        {
            if (vertices.Length < 3)
            {
                return (new[] { 0f, 0f, 0f }, new[] { 0f, 0f, 0f });
            }

            float minX = vertices[0], minY = vertices[1], minZ = vertices[2];
            float maxX = minX, maxY = minY, maxZ = minZ;

            for (int i = 0; i < vertices.Length; i += 3)
            {
                minX = Math.Min(minX, vertices[i]);
                minY = Math.Min(minY, vertices[i + 1]);
                minZ = Math.Min(minZ, vertices[i + 2]);

                maxX = Math.Max(maxX, vertices[i]);
                maxY = Math.Max(maxY, vertices[i + 1]);
                maxZ = Math.Max(maxZ, vertices[i + 2]);
            }

            return (new[] { minX, minY, minZ }, new[] { maxX, maxY, maxZ });
        }
    }
}
