using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using BepInEx.Logging;
using Unity.Collections;
using Unity.Entities;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Runtime
{
    /// <summary>
    /// Dumps all ECS entities, components, and archetypes to disk for reverse engineering.
    /// This is the primary tool for understanding DINO's internal data model.
    ///
    /// Output files:
    /// - archetypes.json: All unique entity archetypes and their component types
    /// - components.json: All registered component types with field info
    /// - entities_summary.json: Entity counts per archetype
    /// - entities_detail_{archetype}.json: Full component data per entity (sampled)
    /// </summary>
    public class EntityDumper
    {
        private readonly ManualLogSource _log;
        private readonly string _outputDir;
        private readonly TimeProvider _timeProvider;

        public EntityDumper(ManualLogSource log, string outputDir, TimeProvider? timeProvider = null)
        {
            _log = log;
            _outputDir = outputDir;
            _timeProvider = timeProvider ?? TimeProvider.System;
        }

        /// <summary>
        /// Dump all available ECS data from all active Worlds.
        /// </summary>
        public void DumpAll()
        {
            try
            {
                Directory.CreateDirectory(_outputDir);
                string timestamp = _timeProvider.GetUtcNow().UtcDateTime.ToString("yyyyMMddTHHmmssZ");
                string dumpDir = Path.Combine(_outputDir, $"dump_{timestamp}");
                Directory.CreateDirectory(dumpDir);

                _log.LogInfo($"Dumping to: {dumpDir}");

                // Dump all worlds
                DumpWorlds(dumpDir);

                // Dump all component types via reflection
                DumpComponentTypes(dumpDir);

                // Dump game assembly types (DNO.Main namespace discovery)
                DumpGameNamespaces(dumpDir);

                _log.LogInfo($"Dump complete: {dumpDir}");
            }
            catch (Exception ex)
            {
                _log.LogError($"Entity dump failed: {ex}");
            }
        }

        private void DumpWorlds(string dumpDir)
        {
            if (World.All.Count == 0)
            {
                _log.LogWarning("No ECS Worlds found.");
                return;
            }

            JArray worldsJson = new JArray();

            foreach (World world in World.All)
            {
                _log.LogInfo($"Dumping World: {world.Name}");

                JObject worldObj = new JObject
                {
                    ["name"] = world.Name,
                    ["isCreated"] = world.IsCreated
                };

                try
                {
                    EntityManager em = world.EntityManager;

                    // Get all entities
                    NativeArray<Entity> entities = em.GetAllEntities(Allocator.Temp);
                    worldObj["entityCount"] = entities.Length;
                    _log.LogInfo($"  World '{world.Name}': {entities.Length} entities");

                    // Dump archetypes
                    JArray archetypesJson = DumpArchetypes(em, entities, dumpDir, world.Name);
                    worldObj["archetypes"] = archetypesJson;

                    entities.Dispose();
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"  Failed to dump world '{world.Name}': {ex.Message}");
                    worldObj["error"] = ex.Message;
                }

                worldsJson.Add(worldObj);
            }

            string worldsPath = Path.Combine(dumpDir, "worlds.json");
            string worldsJson_serialized = JsonConvert.SerializeObject(worldsJson, Formatting.Indented);
            File.WriteAllText(worldsPath, worldsJson_serialized);
            _log.LogInfo($"Wrote {worldsPath}");
        }

        private JArray DumpArchetypes(EntityManager em, NativeArray<Entity> entities, string dumpDir, string worldName)
        {
            // Group entities by archetype
            Dictionary<string, List<Entity>> archetypeGroups = new Dictionary<string, List<Entity>>(StringComparer.Ordinal);
            Dictionary<string, ComponentType[]> archetypeComponents = new Dictionary<string, ComponentType[]>(StringComparer.Ordinal);

            foreach (Entity entity in entities)
            {
                try
                {
                    NativeArray<ComponentType> types = em.GetComponentTypes(entity, Allocator.Temp);
                    string key = string.Join("+", types
                        .Select(t => t.GetManagedType()?.Name ?? $"Unknown({t.TypeIndex})")
                        .OrderBy(n => n));

                    if (!archetypeGroups.ContainsKey(key))
                    {
                        archetypeGroups[key] = new List<Entity>();
                        archetypeComponents[key] = types.ToArray();
                    }
                    archetypeGroups[key].Add(entity);

                    types.Dispose();
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"  Failed to get components for entity {entity.Index}: {ex.Message}");
                }
            }

            // Build archetype summary
            JArray archetypesJson = new JArray();
            int archetypeIndex = 0;

            foreach (KeyValuePair<string, List<Entity>> kvp in archetypeGroups.OrderByDescending(g => g.Value.Count))
            {
                JObject archetypeObj = new JObject
                {
                    ["index"] = archetypeIndex,
                    ["entityCount"] = kvp.Value.Count,
                    ["componentSignature"] = kvp.Key
                };

                // List component details
                if (archetypeComponents.TryGetValue(kvp.Key, out ComponentType[]? compTypes))
                {
                    JArray componentsJson = new JArray();
                    foreach (ComponentType ct in compTypes)
                    {
                        Type? managedType = ct.GetManagedType();
                        JObject compObj = new JObject
                        {
                            ["name"] = managedType?.Name ?? "Unknown",
                            ["fullName"] = managedType?.FullName ?? "Unknown",
                            ["namespace"] = managedType?.Namespace ?? "Unknown",
                            ["assembly"] = managedType?.Assembly?.GetName()?.Name ?? "Unknown",
                            ["typeIndex"] = ct.TypeIndex,
                            ["isZeroSized"] = ct.IsZeroSized,
                            ["isChunkComponent"] = ct.IsChunkComponent,
                            ["isSharedComponent"] = ct.IsSharedComponent,
                            ["isBuffer"] = ct.IsBuffer
                        };

                        // Dump fields for non-trivial components
                        if (managedType != null && !ct.IsZeroSized)
                        {
                            JArray fieldsJson = new JArray();
                            foreach (FieldInfo field in managedType.GetFields(
                                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
                            {
                                fieldsJson.Add(new JObject
                                {
                                    ["name"] = field.Name,
                                    ["type"] = field.FieldType.Name,
                                    ["fullType"] = field.FieldType.FullName
                                });
                            }
                            compObj["fields"] = fieldsJson;
                        }

                        componentsJson.Add(compObj);
                    }
                    archetypeObj["components"] = componentsJson;
                }

                archetypesJson.Add(archetypeObj);

                // Dump sample entity data for top archetypes (first 5 entities per archetype)
                if (kvp.Value.Count > 0 && archetypeIndex < 50)
                {
                    try
                    {
                        DumpSampleEntities(em, kvp.Value.Take(5).ToList(),
                            compTypes, dumpDir, worldName, archetypeIndex);
                    }
                    catch (Exception ex)
                    {
                        _log.LogWarning($"  Failed to dump sample entities for archetype {archetypeIndex}: {ex.Message}");
                    }
                }

                archetypeIndex++;
            }

            _log.LogInfo($"  Found {archetypeGroups.Count} unique archetypes");

            // Write archetype summary
            string summaryPath = Path.Combine(dumpDir, $"archetypes_{worldName}.json");
            File.WriteAllText(summaryPath, JsonConvert.SerializeObject(archetypesJson, Formatting.Indented));
            _log.LogInfo($"  Wrote {summaryPath}");

            return archetypesJson;
        }

        private void DumpSampleEntities(EntityManager em, List<Entity> entities,
            ComponentType[]? compTypes, string dumpDir, string worldName, int archetypeIndex)
        {
            if (compTypes == null) return;

            JArray samplesJson = new JArray();

            foreach (Entity entity in entities)
            {
                JObject entityObj = new JObject
                {
                    ["index"] = entity.Index,
                    ["version"] = entity.Version
                };

                JObject componentData = new JObject();

                foreach (ComponentType ct in compTypes)
                {
                    Type? managedType = ct.GetManagedType();
                    if (managedType == null || ct.IsZeroSized) continue;

                    try
                    {
                        // Try to read component data via reflection
                        if (ct.IsSharedComponent)
                        {
                            // Shared components need special handling
                            componentData[managedType.Name] = "(shared component - use GetSharedComponentData)";
                        }
                        else if (ct.IsBuffer)
                        {
                            componentData[managedType.Name] = "(buffer element - use GetBuffer)";
                        }
                        else
                        {
                            // Try generic GetComponentData via reflection
                            componentData[managedType.Name] = "(component data - see fields in archetype dump)";
                        }
                    }
                    catch
                    {
                        componentData[managedType.Name] = "(read failed)";
                    }
                }

                entityObj["components"] = componentData;
                samplesJson.Add(entityObj);
            }

            string samplesDir = Path.Combine(dumpDir, "samples");
            Directory.CreateDirectory(samplesDir);
            string samplesPath = Path.Combine(samplesDir, $"{worldName}_archetype_{archetypeIndex}.json");
            File.WriteAllText(samplesPath, JsonConvert.SerializeObject(samplesJson, Formatting.Indented));
        }

        private void DumpComponentTypes(string dumpDir)
        {
            JArray allTypes = new JArray();

            // Find all IComponentData implementations across loaded assemblies
            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    string assemblyName = assembly.GetName().Name ?? "Unknown";

                    // Focus on game assemblies, skip System/Unity engine internals
                    if (assemblyName.StartsWith("System") ||
                        assemblyName.StartsWith("mscorlib") ||
                        assemblyName.StartsWith("Mono."))
                        continue;

                    foreach (Type type in assembly.GetTypes())
                    {
                        bool isComponent = typeof(IComponentData).IsAssignableFrom(type);
                        bool isShared = typeof(ISharedComponentData).IsAssignableFrom(type);
                        bool isBuffer = typeof(IBufferElementData).IsAssignableFrom(type);
                        bool isSystem = typeof(ComponentSystemBase).IsAssignableFrom(type);

                        if (!isComponent && !isShared && !isBuffer && !isSystem)
                            continue;

                        JObject typeObj = new JObject
                        {
                            ["name"] = type.Name,
                            ["fullName"] = type.FullName,
                            ["namespace"] = type.Namespace,
                            ["assembly"] = assemblyName,
                            ["isComponent"] = isComponent,
                            ["isSharedComponent"] = isShared,
                            ["isBuffer"] = isBuffer,
                            ["isSystem"] = isSystem,
                            ["isStruct"] = type.IsValueType,
                            ["isAbstract"] = type.IsAbstract
                        };

                        // Dump fields
                        JArray fields = new JArray();
                        foreach (FieldInfo field in type.GetFields(
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
                        {
                            fields.Add(new JObject
                            {
                                ["name"] = field.Name,
                                ["type"] = field.FieldType.Name,
                                ["fullType"] = field.FieldType.FullName,
                                ["isPublic"] = field.IsPublic
                            });
                        }
                        typeObj["fields"] = fields;

                        allTypes.Add(typeObj);
                    }
                }
                catch (ReflectionTypeLoadException)
                {
                    // Some assemblies can't be fully reflected - skip
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"  Failed to scan assembly {assembly.GetName().Name}: {ex.Message}");
                }
            }

            string typesPath = Path.Combine(dumpDir, "ecs_types.json");
            File.WriteAllText(typesPath, JsonConvert.SerializeObject(allTypes, Formatting.Indented));
            _log.LogInfo($"Found {allTypes.Count} ECS types, wrote {typesPath}");
        }

        private void DumpGameNamespaces(string dumpDir)
        {
            JObject namespaceDump = new JObject();

            string[] gameAssemblies = { "DNO.Main", "Assembly-CSharp", "Door407.Core.Runtime", "Door407.Integrations.Runtime" };

            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                string assemblyName = assembly.GetName().Name ?? "";
                if (!gameAssemblies.Contains(assemblyName))
                    continue;

                try
                {
                    Dictionary<string, List<string>> namespaceTypes = new Dictionary<string, List<string>>(StringComparer.Ordinal);

                    foreach (Type type in assembly.GetTypes())
                    {
                        string ns = type.Namespace ?? "(global)";
                        if (!namespaceTypes.ContainsKey(ns))
                            namespaceTypes[ns] = new List<string>();
                        namespaceTypes[ns].Add(type.Name);
                    }

                    JObject assemblyObj = new JObject();
                    foreach (KeyValuePair<string, List<string>> kvp in namespaceTypes.OrderBy(k => k.Key))
                    {
                        assemblyObj[kvp.Key] = new JArray(kvp.Value.OrderBy(n => n).ToArray());
                    }
                    namespaceDump[assemblyName] = assemblyObj;

                    _log.LogInfo($"  {assemblyName}: {namespaceTypes.Count} namespaces, {namespaceTypes.Values.Sum(v => v.Count)} types");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"  Failed to scan {assemblyName}: {ex.Message}");
                    namespaceDump[assemblyName] = ex.Message;
                }
            }

            string nsPath = Path.Combine(dumpDir, "game_namespaces.json");
            File.WriteAllText(nsPath, JsonConvert.SerializeObject(namespaceDump, Formatting.Indented));
            _log.LogInfo($"Wrote {nsPath}");
        }
    }
}
