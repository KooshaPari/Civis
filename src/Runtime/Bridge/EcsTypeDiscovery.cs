#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Discovers and logs all available ECS component types from loaded game assemblies.
    /// Used for diagnostic purposes when DINO component type names don't match expectations.
    /// 
    /// Usage:
    ///   1. Check dinoforge_debug.log after startup for discovered types
    ///   2. Compare against ComponentMap.cs expected types
    ///   3. Update mappings if game version changed component names
    /// </summary>
    public static class EcsTypeDiscovery
    {
        private static bool _discoveryLogged;
        private static List<string>? _discoveredTypes;
        private static List<string>? _discoveredAssemblies;

        /// <summary>
        /// Discovers all ECS component types and logs to debug file.
        /// Safe to call multiple times - only logs once per session.
        /// </summary>
        public static void DiscoverAndLog()
        {
            if (_discoveryLogged) return;
            _discoveryLogged = true;

            var sb = new StringBuilder();
            sb.AppendLine("=== DINOForge ECS Type Discovery ===");
            sb.AppendLine($"Timestamp: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
            sb.AppendLine();

            // Discover assemblies
            var assemblies = new List<Assembly>();
            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    if (IsGameAssembly(asm.GetName().Name ?? ""))
                    {
                        assemblies.Add(asm);
                    }
                }
                catch { }
            }

            sb.AppendLine($"Game assemblies found: {assemblies.Count}");
            _discoveredAssemblies = assemblies.Select(a => a.GetName().Name ?? "unknown").ToList();

            // Scan for ECS component types
            var componentTypes = new List<(string FullName, string Assembly)>();
            var allTypes = new List<string>();

            foreach (var asm in assemblies)
            {
                try
                {
                    foreach (Type type in asm.GetTypes())
                    {
                        if (type == null) continue;

                        // Check if it's a struct (ECS components are structs)
                        if (!type.IsValueType || type.IsPrimitive) continue;

                        // Skip generic types (except Nullable<T>)
                        if (type.IsGenericType && !type.IsGenericTypeDefinition) continue;

                        string fullName = type.FullName ?? type.Name;
                        allTypes.Add(fullName);

                        // Categorize by namespace
                        if (fullName.StartsWith("Components.") ||
                            fullName.StartsWith("Utility.") ||
                            fullName.Contains("Unit") ||
                            fullName.Contains("Building") ||
                            fullName.Contains("Resource") ||
                            fullName.Contains("Projectile") ||
                            fullName.Contains("Health") ||
                            fullName.Contains("Food") ||
                            fullName.Contains("Wood") ||
                            fullName.Contains("Stone") ||
                            fullName.Contains("Iron") ||
                            fullName.Contains("Money") ||
                            fullName.Contains("Attack") ||
                            fullName.Contains("Move") ||
                            fullName.Contains("Speed"))
                        {
                            componentTypes.Add((fullName, asm.GetName().Name ?? "unknown"));
                        }
                    }
                }
                catch
                {
                    // Some assemblies don't allow type enumeration
                }
            }

            sb.AppendLine($"Total types scanned: {allTypes.Count}");
            sb.AppendLine($"Potentially relevant types: {componentTypes.Count}");
            sb.AppendLine();

            // Create lookup set for ComponentMap verification
            var discoveredNames = new HashSet<string>(componentTypes.Select(t => t.FullName), StringComparer.OrdinalIgnoreCase);

            // Sort and group by namespace
            var grouped = componentTypes
                .OrderBy(t => t.FullName)
                .GroupBy(t => GetNamespace(t.FullName))
                .OrderBy(g => g.Key);

            foreach (var group in grouped)
            {
                sb.AppendLine($"--- {group.Key} ---");
                foreach (var (fullName, assembly) in group.OrderBy(t => t.FullName))
                {
                    sb.AppendLine($"  {fullName}");
                }
                sb.AppendLine();
            }

            // Compare against expected ComponentMap types
            sb.AppendLine("=== ComponentMap Verification ===");
            VerifyComponentMapTypes(sb, componentTypes, discoveredNames);

            _discoveredTypes = allTypes;

            WriteDebug(sb.ToString());
        }

        /// <summary>
        /// Get all discovered type names.
        /// </summary>
        public static IReadOnlyList<string>? GetDiscoveredTypes() => _discoveredTypes;

        /// <summary>
        /// Get all discovered game assembly names.
        /// </summary>
        public static IReadOnlyList<string>? GetDiscoveredAssemblies() => _discoveredAssemblies;

        /// <summary>
        /// Check if a specific type exists in loaded assemblies.
        /// </summary>
        public static bool TypeExists(string typeName)
        {
            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    if (IsGameAssembly(asm.GetName().Name ?? ""))
                    {
                        var type = asm.GetType(typeName, throwOnError: false);
                        if (type != null) return true;
                    }
                }
                catch { }
            }
            return false;
        }

        /// <summary>
        /// Find types matching a search pattern.
        /// </summary>
        public static IEnumerable<string> FindTypes(string pattern)
        {
            if (_discoveredTypes == null) DiscoverAndLog();

            pattern = pattern.ToLowerInvariant();
            return _discoveredTypes?
                .Where(t => t.ToLowerInvariant().Contains(pattern))
                .OrderBy(t => t) ?? Enumerable.Empty<string>();
        }

        /// <summary>
        /// Dump all types to debug log for analysis.
        /// </summary>
        public static void DumpAllTypes()
        {
            if (_discoveredTypes == null) DiscoverAndLog();

            var sb = new StringBuilder();
            sb.AppendLine("=== All Discovered Types ===");
            foreach (var type in _discoveredTypes?.OrderBy(t => t) ?? Enumerable.Empty<string>())
            {
                sb.AppendLine(type);
            }
            WriteDebug(sb.ToString());
        }

        private static void VerifyComponentMapTypes(StringBuilder sb, List<(string FullName, string Assembly)> discovered, HashSet<string> discoveredNames)
        {
            // discoveredNames is already passed in

            // Critical types that must exist for core functionality
            string[] criticalTypes = new[]
            {
                "Components.Unit",
                "Components.Enemy",
                "Components.BuildingBase",
                "Components.Health",
                "Components.MeleeUnit",
                "Components.RangeUnit",
                "Components.CavalryUnit",
                "Components.SiegeUnit",
                "Components.Archer",
            };

            sb.AppendLine("Critical types:");
            foreach (var type in criticalTypes)
            {
                bool exists = discoveredNames.Contains(type) || TypeExists(type);
                sb.AppendLine($"  {(exists ? "✓" : "✗")} {type}");
            }

            sb.AppendLine();
            sb.AppendLine("Resource types:");
            var resourcePatterns = new[] { "Food", "Wood", "Stone", "Iron", "Money", "Soul", "Bone", "Spirit" };
            foreach (var pattern in resourcePatterns)
            {
                var matches = discovered.Where(t => t.FullName.Contains(pattern)).ToList();
                if (matches.Any())
                {
                    sb.AppendLine($"  {pattern}:");
                    foreach (var match in matches.Take(5))
                    {
                        sb.AppendLine($"    - {match.FullName}");
                    }
                }
                else
                {
                    sb.AppendLine($"  {pattern}: NOT FOUND");
                }
            }
        }

        private static bool IsGameAssembly(string assemblyName)
        {
            if (string.IsNullOrEmpty(assemblyName)) return false;

            // DINO game assemblies typically start with these patterns
            string[] gameAssemblyPrefixes = new[]
            {
                "DNO.",
                "Unity.",
                "Unity.Entities",
                "Unity.Rendering",
                "Main",
                "Game",
            };

            foreach (var prefix in gameAssemblyPrefixes)
            {
                if (assemblyName.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
                    return true;
            }

            return false;
        }

        private static string GetNamespace(string fullTypeName)
        {
            int lastDot = fullTypeName.LastIndexOf('.');
            return lastDot > 0 ? fullTypeName.Substring(0, lastDot) : "(global)";
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.Now:u}] {msg}\n");
                Debug.Log($"[EcsTypeDiscovery] {msg.Split('\n')[0]}");
            }
            catch { }
        }
    }
}
