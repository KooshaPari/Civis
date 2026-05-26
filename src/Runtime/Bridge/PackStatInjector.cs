#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Applies pack unit stat definitions to matching vanilla ECS entities via
    /// <see cref="StatModifierSystem.ApplyImmediate"/>. Called once after
    /// <c>RebuildCatalogAndApplyStats</c> when the entity count exceeds 1000.
    ///
    /// Strategy:
    ///   For each loaded pack unit definition with a non-null <c>vanilla_mapping</c>:
    ///     1. Resolve <c>vanilla_mapping</c> → ECS component type name
    ///        via <see cref="PackStatMappings.TryResolveMapping"/>.
    ///     2. For each stat returned by <see cref="PackStatMappings.EnumerateStatPaths"/>:
    ///        - Create a <see cref="StatModification"/> with <c>FilterComponentType</c> set to
    ///          the resolved component type so only entities of that class are touched.
    ///        - Call <see cref="StatModifierSystem.ApplyImmediate"/> synchronously.
    ///     3. Skip <c>aerial_fighter</c> (null component type → handled by AerialSpawnSystem).
    ///     4. Skip units where <c>vanilla_mapping</c> is null/empty.
    ///     5. Catch per-unit exceptions so one bad definition never blocks the rest.
    ///
    /// This replaces the no-op <see cref="OverrideApplicator.ApplyUnitOverrides"/> path for
    /// the vanilla-mapping use case while leaving global YAML overrides untouched.
    ///
    /// Pure-C# mapping logic lives in <see cref="PackStatMappings"/> (no Unity dependency),
    /// which is separately unit-testable in CI without the game DLLs.
    /// </summary>
    public static class PackStatInjector
    {
        /// <summary>
        /// Applies pack unit stat definitions to live vanilla ECS entities.
        /// Must be called from the main Unity thread (EntityManager is not thread-safe).
        /// </summary>
        /// <param name="em">The active world's EntityManager.</param>
        /// <param name="registry">The registry manager populated by ContentLoader.</param>
        /// <param name="log">Logging callback. Null is treated as a no-op.</param>
        /// <returns>Total number of entity-field writes performed across all units.</returns>
        /// <exception cref="ArgumentNullException">If <paramref name="registry"/> is null.</exception>
        public static int Apply(EntityManager em, RegistryManager registry, Action<string>? log)
        {
            if (registry == null) throw new ArgumentNullException(nameof(registry));

            Action<string> write = log ?? (_ => { });

            int totalWrites = 0;
            int unitsProcessed = 0;
            int unitsSkipped = 0;

            IReadOnlyDictionary<string, RegistryEntry<UnitDefinition>> allUnits =
                registry.Units.All;

            foreach (KeyValuePair<string, RegistryEntry<UnitDefinition>> kv in allUnits)
            {
                UnitDefinition unit = kv.Value.Data;

                // Skip units with no vanilla_mapping
                if (string.IsNullOrWhiteSpace(unit.VanillaMapping))
                {
                    unitsSkipped++;
                    continue;
                }

                string vanillaMapping = unit.VanillaMapping!;

                // Resolve vanilla_mapping → ECS component type
                if (!PackStatMappings.TryResolveMapping(vanillaMapping, out string? componentType))
                {
                    write($"[PackStatInjector] Unknown vanilla_mapping '{vanillaMapping}' " +
                          $"for unit '{unit.Id}' — skipping.");
                    unitsSkipped++;
                    continue;
                }

                // Null componentType = intentionally skipped (e.g. aerial_fighter)
                if (componentType == null)
                {
                    write($"[PackStatInjector] vanilla_mapping '{vanillaMapping}' for unit '{unit.Id}' " +
                          "is handled by another system — skipping.");
                    unitsSkipped++;
                    continue;
                }

                // Apply each stat to entities filtered by the resolved component type
                try
                {
                    int unitWrites = ApplyUnitStats(em, unit, componentType, write);
                    totalWrites = totalWrites + unitWrites;
                    unitsProcessed++;
                    write($"[PackStatInjector] Unit '{unit.Id}' " +
                          $"({vanillaMapping} → {componentType}): {unitWrites} write(s).");
                }
                catch (Exception ex)
                {
                    // Per-unit isolation: log and continue — one bad definition must not block others
                    write($"[PackStatInjector] ERROR applying stats for unit '{unit.Id}': {ex.Message}");
                }
            }

            write($"[PackStatInjector] Done. " +
                  $"Processed {unitsProcessed} unit(s), skipped {unitsSkipped}, " +
                  $"total writes {totalWrites}.");
            return totalWrites;
        }

        // ──────────────────────────────────────────────────────────────────────────
        //  Internal helpers
        // ──────────────────────────────────────────────────────────────────────────

        /// <summary>
        /// Applies all resolvable stats for a single unit definition to entities that
        /// carry <paramref name="filterComponentType"/>, using ApplyImmediate for each.
        /// </summary>
        private static int ApplyUnitStats(
            EntityManager em,
            UnitDefinition unit,
            string filterComponentType,
            Action<string> log)
        {
            int writes = 0;

            foreach ((string sdkPath, float value) in PackStatMappings.EnumerateStatPaths(unit.Stats))
            {
                try
                {
                    StatModification mod = new StatModification(
                        sdkPath,
                        value,
                        ModifierMode.Override,
                        filterComponentType: filterComponentType);

                    int affected = StatModifierSystem.ApplyImmediate(em, mod);
                    if (affected > 0)
                        writes = writes + affected;
                    else if (affected == -1)
                        log($"[PackStatInjector]   No ComponentMapping for '{sdkPath}' — skipped.");
                }
                catch (Exception ex)
                {
                    // Per-stat isolation: log and continue with the next stat
                    log($"[PackStatInjector]   ERROR applying '{sdkPath}' " +
                        $"for unit '{unit.Id}': {ex.Message}");
                }
            }

            return writes;
        }
    }
}
