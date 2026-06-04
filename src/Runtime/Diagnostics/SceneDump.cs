#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using DINOForge.SDK.Assets;
using System.Reflection;
using DINOForge.Runtime.Bridge;
using DINOForge.Runtime.UI;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;
using Unity.Transforms;

namespace DINOForge.Runtime.Diagnostics
{
    /// <summary>
    /// Captures a headless scene and simulation dump for DINOForge diagnostics.
    /// </summary>
    internal sealed class SceneDumper
    {
        private readonly string _outputPath;
        private readonly string _packsDirectory;
        private readonly Func<IReadOnlyList<string>?> _loadedPackIds;
        private readonly Func<IReadOnlyList<PackDisplayInfo>?> _loadedPackInfos;

        public SceneDumper(
            string outputPath,
            string packsDirectory,
            Func<IReadOnlyList<string>?> loadedPackIds,
            Func<IReadOnlyList<PackDisplayInfo>?> loadedPackInfos)
        {
            _outputPath = outputPath;
            _packsDirectory = packsDirectory;
            _loadedPackIds = loadedPackIds;
            _loadedPackInfos = loadedPackInfos;
        }

        public void Dump(World world, EntityManager entityManager)
        {
            DumpResult result = BuildDump(world, entityManager);
            Directory.CreateDirectory(Path.GetDirectoryName(_outputPath) ?? ".");
            string tempPath = _outputPath + ".tmp";
            string json = JsonSerializer.Serialize(result, SceneDumpJson.Options);
            File.WriteAllText(tempPath, json);
            if (File.Exists(_outputPath))
            {
                File.Delete(_outputPath);
            }
            File.Move(tempPath, _outputPath);
        }

        private DumpResult BuildDump(World world, EntityManager entityManager)
        {
            Dictionary<string, int> meshCounts = new Dictionary<string, int>(StringComparer.Ordinal);
            List<EntityDumpRow> entities = new List<EntityDumpRow>();
            int renderMeshCount = 0;

            using EntityQuery query = EntityQueries.GetRenderMeshEntities(entityManager) ?? entityManager.CreateEntityQuery(new EntityQueryDesc { Options = EntityQueryOptions.IncludePrefab });

            NativeArray<Entity> allEntities = query.ToEntityArray(Allocator.Temp);
            try
            {
                foreach (Entity entity in allEntities)
                {
                    List<string> componentNames = GetComponentNames(entityManager, entity);
                    if (!TryGetRenderMeshSnapshot(entityManager, entity, out Mesh? mesh, out Material? material))
                    {
                        continue;
                    }

                    renderMeshCount++;
                    string meshName = mesh?.name ?? "";
                    string materialName = material?.name ?? "";
                    if (!string.IsNullOrEmpty(meshName))
                    {
                        meshCounts.TryGetValue(meshName, out int meshCount);
                        meshCounts[meshName] = meshCount + 1;
                    }

                    entities.Add(new EntityDumpRow
                    {
                        EntityIndex = entity.Index,
                        ArchetypeComponents = componentNames,
                        MeshName = meshName,
                        MaterialName = materialName,
                        TranslationPosition = TryGetTranslation(entityManager, entity)
                    });
                }
            }
            finally
            {
                allEntities.Dispose();
            }

            IReadOnlyList<string>? loadedPackIds = _loadedPackIds();
            IReadOnlyList<PackDisplayInfo>? packInfos = _loadedPackInfos();

            return new DumpResult
            {
                WorldName = world.Name,
                Summary = new DumpSummary
                {
                    TotalEntities = query.CalculateEntityCount(),
                    EntitiesWithRenderMesh = renderMeshCount,
                    SwapAppliedCount = AssetSwapRegistry.Count - AssetSwapRegistry.GetPending().Count,
                    LoadedPackIds = loadedPackIds?.ToArray() ?? Array.Empty<string>(),
                    ActiveTotalConversion = SelectActiveTotalConversion(packInfos),
                    ActiveCanvasNames = GetActiveCanvasNames(),
                    UniqueMeshNames = meshCounts
                        .OrderByDescending(kvp => kvp.Value)
                        .ThenBy(kvp => kvp.Key, StringComparer.Ordinal)
                        .Select(kvp => new MeshCount { Name = kvp.Key, Count = kvp.Value })
                        .ToArray()
                },
                Entities = entities
            };
        }

        private static List<string> GetComponentNames(EntityManager entityManager, Entity entity)
        {
            NativeArray<ComponentType> types = entityManager.GetComponentTypes(entity, Allocator.Temp);
            try
            {
                List<string> names = new List<string>(types.Length);
                foreach (ComponentType type in types)
                {
                    Type? managedType = type.GetManagedType();
                    names.Add(managedType?.Name ?? type.ToString());
                }
                names.Sort(StringComparer.Ordinal);
                return names;
            }
            finally
            {
                types.Dispose();
            }
        }

        private static bool TryGetRenderMeshSnapshot(EntityManager entityManager, Entity entity, out Mesh? mesh, out Material? material)
        {
            mesh = null;
            material = null;
            Type? renderMeshType = ResolveRenderMeshType();
            if (renderMeshType == null)
            {
                return false;
            }

            try
            {
                MethodInfo method = typeof(EntityManager).GetMethods(BindingFlags.Public | BindingFlags.Instance)
                    .First(m => m.Name == "GetSharedComponentData" && m.IsGenericMethodDefinition && m.GetParameters().Length == 1);
                MethodInfo generic = method.MakeGenericMethod(renderMeshType);
                object? renderMesh = generic.Invoke(entityManager, new object[] { entity });
                if (renderMesh == null)
                {
                    return false;
                }

                FieldInfo? meshField = renderMeshType.GetField("mesh");
                FieldInfo? materialField = renderMeshType.GetField("material");
                mesh = meshField?.GetValue(renderMesh) as Mesh;
                material = materialField?.GetValue(renderMesh) as Material;
                return true;
            }
            catch
            {
                return false;
            }
        }

        private static float[] TryGetTranslation(EntityManager entityManager, Entity entity)
        {
            try
            {
                if (entityManager.HasComponent<Translation>(entity))
                {
                    Translation translation = entityManager.GetComponentData<Translation>(entity);
                    return new[] { translation.Value.x, translation.Value.y, translation.Value.z };
                }
            }
            catch
            {
            }

            return Array.Empty<float>();
        }

        private string SelectActiveTotalConversion(IReadOnlyList<PackDisplayInfo>? packs)
        {
            if (packs == null || packs.Count == 0)
            {
                return "";
            }

            PackDisplayInfo? best = null;
            PackDisplayInfo? fallback = null;
            foreach (PackDisplayInfo pack in packs.OrderBy(p => p.Id, StringComparer.Ordinal))
            {
                if (!pack.IsEnabled) continue;
                if (!string.Equals(pack.Type, "total_conversion", StringComparison.OrdinalIgnoreCase)) continue;
                if (fallback == null) fallback = pack;
                string yamlPath = Path.Combine(_packsDirectory, pack.Id, "pack.yaml");
                if (File.Exists(yamlPath))
                {
                    string content = File.ReadAllText(yamlPath);
                    if (content.IndexOf("ui_theme:", StringComparison.Ordinal) >= 0)
                    {
                        best = pack;
                        break;
                    }
                }
            }

            return (best ?? fallback)?.Id ?? "";
        }

        private static Type? ResolveRenderMeshType()
        {
            string[] typeNames = { "Unity.Rendering.RenderMesh", "Unity.Rendering.RenderMeshUnmanaged", "Unity.Rendering.MaterialMeshInfo" };
            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                Type? type = typeNames.Select(assembly.GetType).FirstOrDefault(t => t != null);
                if (type != null)
                {
                    return type;
                }
            }

            return null;
        }

        private static string[] GetActiveCanvasNames()
        {
            Canvas[] canvases = Resources.FindObjectsOfTypeAll<Canvas>();
            List<string> names = new List<string>();
            foreach (Canvas canvas in canvases)
            {
                if (canvas == null || canvas.gameObject == null) continue;
                if (!canvas.gameObject.activeInHierarchy || !canvas.enabled) continue;
                names.Add(canvas.name);
            }

            names.Sort(StringComparer.Ordinal);
            return names.ToArray();
        }

        private sealed class DumpResult
        {
            public string WorldName { get; set; } = "";
            public DumpSummary Summary { get; set; } = new DumpSummary();
            public List<EntityDumpRow> Entities { get; set; } = new List<EntityDumpRow>();
        }

        private sealed class DumpSummary
        {
            public int TotalEntities { get; set; }
            public int EntitiesWithRenderMesh { get; set; }
            public int SwapAppliedCount { get; set; }
            public string[] LoadedPackIds { get; set; } = Array.Empty<string>();
            public string ActiveTotalConversion { get; set; } = "";
            public string[] ActiveCanvasNames { get; set; } = Array.Empty<string>();
            public MeshCount[] UniqueMeshNames { get; set; } = Array.Empty<MeshCount>();
        }

        private sealed class EntityDumpRow
        {
            public int EntityIndex { get; set; }
            public List<string> ArchetypeComponents { get; set; } = new List<string>();
            public string MeshName { get; set; } = "";
            public string MaterialName { get; set; } = "";
            public float[] TranslationPosition { get; set; } = Array.Empty<float>();
        }

        private sealed class MeshCount
        {
            public string Name { get; set; } = "";
            public int Count { get; set; }
        }

        private static class SceneDumpJson
        {
            public static readonly JsonSerializerOptions Options = new JsonSerializerOptions
            {
                WriteIndented = true,
                DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
            };
        }
    }
}
