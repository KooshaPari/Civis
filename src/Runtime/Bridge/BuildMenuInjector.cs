#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Registers DINOForge pack buildings into DINO's live build menu by ALIASING each pack
    /// building onto an existing vanilla <c>BuildingType</c> buildable slot.
    ///
    /// Why aliasing (see docs/sessions/dino-build-catalog-20260530.md):
    /// DINO's build menu is data-driven by compiled config structs —
    /// <c>ScriptableObjectDefinitions.BuildingsCategory</c> (field
    /// <c>List&lt;BuildingTypeContainer&gt; types</c>) where each
    /// <c>BuildingTypeContainer</c> is keyed by the <b>closed compiled</b>
    /// <c>Utility.EnumsStorage.BuildingType</c> enum. A brand-new building type cannot be
    /// added to the native menu at runtime (no enum slot, no placement/cost wiring), so each
    /// pack building rides on an existing buildable slot, then gets reskinned (mesh-swap,
    /// #964) and re-targeted (UnitsShop production) so the player sees the pack building.
    ///
    /// This injector runs once at world-ready, on the main thread (it touches the live
    /// config ScriptableObject via reflection). It:
    ///   1. discovers the live config object holding <c>List&lt;BuildingsCategory&gt;</c>
    ///      (by field-shape scan, robust to the host type name),
    ///   2. validates each pack building's alias against the live <c>BuildingType</c> enum
    ///      and confirms the aliased slot is present in some category,
    ///   3. records the alias → pack-building mapping in <see cref="Registrations"/> for the
    ///      reskin + production-injection layers to consume.
    ///
    /// All driven from pack YAML (<c>build_alias</c>/<c>building_type</c>/<c>visual_asset</c>/
    /// <c>production</c>) — no hardcoded content IDs.
    /// </summary>
    public static class BuildMenuInjector
    {
        /// <summary>One resolved pack building → vanilla buildable-slot registration.</summary>
        public sealed class BuildRegistration
        {
            /// <summary>The pack building definition.</summary>
            public BuildingDefinition Definition { get; }

            /// <summary>The vanilla <c>BuildingType</c> enum member name this building aliases.</summary>
            public string Alias { get; }

            /// <summary>True if the aliased slot was found present in a live build-menu category.</summary>
            public bool SlotPresentInMenu { get; }

            /// <summary>Initializes a registration record.</summary>
            public BuildRegistration(BuildingDefinition definition, string alias, bool slotPresentInMenu)
            {
                Definition = definition;
                Alias = alias;
                SlotPresentInMenu = slotPresentInMenu;
            }
        }

        private static readonly List<BuildRegistration> _registrations = new List<BuildRegistration>();
        private static RegistryManager? _registry;
        private static bool _done;

        /// <summary>
        /// Snapshot of resolved registrations (alias → pack building). Consumed by the reskin
        /// and UnitsShop-production layers. Empty until <see cref="RunInjection"/> succeeds.
        /// </summary>
        public static IReadOnlyList<BuildRegistration> Registrations => _registrations;

        /// <summary>Supplies the registry. Call from ModPlatform after packs load.</summary>
        /// <param name="registry">Loaded pack registry.</param>
        public static void Initialize(RegistryManager? registry)
        {
            _registry = registry;
            _done = false;
            DebugLog.Write("BuildMenuInjector", "Initialize: registry set");
        }

        /// <summary>True once <see cref="RunInjection"/> has executed (success or not).</summary>
        public static bool HasRun => _done;

        /// <summary>
        /// Runs the build-menu injection once. Must be called on the Unity main thread at
        /// world-ready (it uses <c>Resources.FindObjectsOfTypeAll</c> via reflection). Safe to
        /// call repeatedly; only the first call does work.
        /// </summary>
        /// <returns>Number of pack buildings successfully aliased into the live menu.</returns>
        public static int RunInjection()
        {
            if (_done) return _registrations.Count;
            _done = true;
            _registrations.Clear();

            if (_registry == null)
            {
                DebugLog.Write("BuildMenuInjector", "RunInjection: registry not initialised, skipping.");
                return 0;
            }

            // Collect pack building definitions.
            List<BuildingDefinition> defs = new List<BuildingDefinition>();
            foreach (KeyValuePair<string, RegistryEntry<BuildingDefinition>> kvp in _registry.Buildings.All)
            {
                if (kvp.Value?.Data != null)
                    defs.Add(kvp.Value.Data);
            }

            if (defs.Count == 0)
            {
                DebugLog.Write("BuildMenuInjector", "RunInjection: no pack building definitions, nothing to do.");
                return 0;
            }

            // Resolve the live BuildingType enum and the set of buildable slots in the menu.
            Type? buildingTypeEnum = EntityQueries.ResolveType("Utility.EnumsStorage.BuildingType");
            HashSet<string> menuSlots = DiscoverMenuBuildingTypeNames(buildingTypeEnum);

            HashSet<string> validEnumNames = new HashSet<string>(StringComparer.Ordinal);
            if (buildingTypeEnum != null && buildingTypeEnum.IsEnum)
            {
                foreach (string n in Enum.GetNames(buildingTypeEnum))
                    validEnumNames.Add(n);
            }

            int registered = 0;
            foreach (BuildingDefinition def in defs)
            {
                string alias = BuildAliasMapper.ResolveAlias(def);

                // If we resolved the live enum, validate; if the alias is unknown, fall back.
                if (validEnumNames.Count > 0 && !validEnumNames.Contains(alias))
                {
                    DebugLog.Write("BuildMenuInjector",
                        $"Building '{def.Id}' alias '{alias}' is not a valid vanilla BuildingType; " +
                        $"falling back to '{BuildAliasMapper.FallbackAlias}'.");
                    alias = BuildAliasMapper.FallbackAlias;
                }

                bool present = menuSlots.Count == 0 /* unknown menu => assume present */
                               || menuSlots.Contains(alias);

                _registrations.Add(new BuildRegistration(def, alias, present));
                registered++;

                DebugLog.Write("BuildMenuInjector",
                    $"Registered pack building '{def.Id}' ({def.DisplayName}) -> vanilla slot '{alias}' " +
                    $"(in-menu={present}, visual='{def.VisualAsset ?? "none"}', produces={def.Production.Count}).");
            }

            DebugLog.Write("BuildMenuInjector",
                $"RunInjection: {registered} pack building(s) aliased into live build menu " +
                $"(menu slots discovered={menuSlots.Count}).");
            return registered;
        }

        /// <summary>
        /// Discovers the set of <c>BuildingType</c> enum-member names that are present as
        /// buildable slots in the live build menu, by locating the config object that holds a
        /// <c>List&lt;BuildingsCategory&gt;</c> and walking its <c>BuildingTypeContainer</c>
        /// entries. Returns an empty set if the live config cannot be found (caller then
        /// assumes the aliased slot is present, since vanilla slots always are).
        /// </summary>
        private static HashSet<string> DiscoverMenuBuildingTypeNames(Type? buildingTypeEnum)
        {
            HashSet<string> result = new HashSet<string>(StringComparer.Ordinal);
            try
            {
                Type? categoryType = EntityQueries.ResolveType("ScriptableObjectDefinitions.BuildingsCategory");
                Type? containerType = EntityQueries.ResolveType("ScriptableObjectDefinitions.BuildingTypeContainer");
                if (categoryType == null || containerType == null)
                {
                    DebugLog.Write("BuildMenuInjector", "DiscoverMenuBuildingTypeNames: category/container types not resolvable.");
                    return result;
                }

                FieldInfo? typeField = containerType.GetField("type",
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                if (typeField == null)
                {
                    DebugLog.Write("BuildMenuInjector", "DiscoverMenuBuildingTypeNames: BuildingTypeContainer.type field missing.");
                    return result;
                }

                // Find any loaded object exposing List<BuildingsCategory> and enumerate it.
                foreach (IList categoryList in EnumerateBuildingsCategoryLists(categoryType))
                {
                    FieldInfo? typesField = categoryType.GetField("types",
                        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                    if (typesField == null) continue;

                    foreach (object? category in categoryList)
                    {
                        if (category == null) continue;
                        if (typesField.GetValue(category) is not IList containers) continue;
                        foreach (object? container in containers)
                        {
                            if (container == null) continue;
                            object? bt = typeField.GetValue(container);
                            if (bt != null) result.Add(bt.ToString());
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("BuildMenuInjector", $"DiscoverMenuBuildingTypeNames failed: {ex.Message}");
            }
            return result;
        }

        /// <summary>
        /// Scans all loaded Unity objects for any whose type declares a
        /// <c>List&lt;BuildingsCategory&gt;</c> field and yields each such live list. Uses
        /// reflection over UnityEngine.Resources so this assembly stays free of a hard
        /// UnityEngine reference shape that breaks netstandard2.0 codegen-free builds.
        /// </summary>
        private static IEnumerable<IList> EnumerateBuildingsCategoryLists(Type categoryType)
        {
            Type? resourcesType = EntityQueries.ResolveType("UnityEngine.Resources");
            Type? scriptableObjectType = EntityQueries.ResolveType("UnityEngine.ScriptableObject");
            if (resourcesType == null || scriptableObjectType == null)
                yield break;

            MethodInfo? findAll = resourcesType.GetMethod(
                "FindObjectsOfTypeAll",
                BindingFlags.Public | BindingFlags.Static,
                null, new[] { typeof(Type) }, null);
            if (findAll == null) yield break;

            object? found = findAll.Invoke(null, new object[] { scriptableObjectType });
            if (found is not Array objects) yield break;

            Type listOfCategory = typeof(List<>).MakeGenericType(categoryType);

            foreach (object? obj in objects)
            {
                if (obj == null) continue;
                Type t = obj.GetType();
                FieldInfo[] fields = t.GetFields(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                foreach (FieldInfo f in fields)
                {
                    if (!listOfCategory.IsAssignableFrom(f.FieldType)) continue;
                    if (f.GetValue(obj) is IList list && list.Count > 0)
                        yield return list;
                }
            }
        }
    }
}
