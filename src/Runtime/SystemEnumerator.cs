using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using BepInEx.Logging;
using Unity.Entities;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Runtime
{
    /// <summary>
    /// Enumerates all ECS Systems registered in each World.
    /// This reveals the game's update loop structure and processing order.
    /// </summary>
    public class SystemEnumerator
    {
#pragma warning disable DF1006 // disposable field (ManualLogSource is BepInEx-owned; not owned here)
        private readonly ManualLogSource _log;
#pragma warning restore DF1006

        public SystemEnumerator(ManualLogSource log)
        {
            _log = log;
        }

        /// <summary>
        /// Enumerate all systems in all worlds and log them.
        /// </summary>
        public void EnumerateAll()
        {
            try
            {
                if (World.All.Count == 0)
                {
                    _log.LogWarning("No ECS Worlds found for system enumeration.");
                    return;
                }

                foreach (World world in World.All)
                {
                    EnumerateWorld(world);
                }
            }
            catch (Exception ex)
            {
                _log.LogError($"System enumeration failed: {ex}");
            }
        }

        private void EnumerateWorld(World world)
        {
            _log.LogInfo($"=== Systems in World: {world.Name} ===");

            try
            {
                // NoAllocReadOnlyCollection cannot be cast to IEnumerable<T> (throws NotSupportedException)
                // Use index-based access instead
                var systems = world.Systems;
                int count = 0;

                List<JObject> systemList = new List<JObject>();

                for (int i = 0; i < systems.Count; i++)
                {
                    ComponentSystemBase system = systems[i];
                    Type systemType = system.GetType();

                    JObject sysObj = new JObject
                    {
                        ["name"] = systemType.Name,
                        ["fullName"] = systemType.FullName,
                        ["namespace"] = systemType.Namespace,
                        ["assembly"] = systemType.Assembly.GetName().Name,
                        ["enabled"] = system.Enabled
                    };

                    // Check for UpdateInGroup attribute
                    UpdateInGroupAttribute? groupAttr = systemType
                        .GetCustomAttribute<UpdateInGroupAttribute>();
                    if (groupAttr != null)
                    {
                        sysObj["updateGroup"] = groupAttr.GroupType.Name;
                    }

                    // Check for UpdateBefore/After attributes
                    IEnumerable<UpdateBeforeAttribute> beforeAttrs = systemType
                        .GetCustomAttributes<UpdateBeforeAttribute>();
                    if (beforeAttrs.Any())
                    {
                        sysObj["updateBefore"] = new JArray(
                            beforeAttrs.Select(a => a.SystemType.Name));
                    }

                    IEnumerable<UpdateAfterAttribute> afterAttrs = systemType
                        .GetCustomAttributes<UpdateAfterAttribute>();
                    if (afterAttrs.Any())
                    {
                        sysObj["updateAfter"] = new JArray(
                            afterAttrs.Select(a => a.SystemType.Name));
                    }

                    systemList.Add(sysObj);
                    count++;

                    _log.LogInfo($"  [{systemType.Namespace}] {systemType.Name} " +
                        $"(enabled={system.Enabled}" +
                        $"{(groupAttr != null ? $", group={groupAttr.GroupType.Name}" : "")})");
                }

                _log.LogInfo($"  Total: {count} systems in '{world.Name}'");

                // Write systems dump
                string dumpDir = Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_dumps");
                Directory.CreateDirectory(dumpDir);

                string latestDir = Directory.GetDirectories(dumpDir)
                    .OrderByDescending(d => d)
                    .FirstOrDefault() ?? dumpDir;

                string systemsPath = Path.Combine(latestDir, $"systems_{world.Name}.json");
                JArray systemsJson = new JArray(systemList);
                File.WriteAllText(systemsPath, JsonConvert.SerializeObject(systemsJson, Formatting.Indented), Encoding.UTF8);
                _log.LogInfo($"  Wrote {systemsPath}");
            }
            catch (Exception ex)
            {
                _log.LogError($"  Failed to enumerate systems in '{world.Name}': {ex}");
            }
        }
    }
}
