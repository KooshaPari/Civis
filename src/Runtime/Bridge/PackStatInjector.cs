#nullable enable
using System;
using System.Collections.Generic;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Telemetry;
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
        /// <summary>
        /// Minimum live-entity count required before PackStatInjector will attempt to apply stats.
        /// In MainMenu state the ECS world holds only prefab templates (~100-300 entities); attempting
        /// to write component data into prefab archetypes that haven't been instantiated yet causes
        /// EntityManager.GetComponentData to throw NullReferenceException because the underlying
        /// chunk buffers are in a transitional state. Gameplay scenes typically report >1000 entities
        /// (3474+ in prior successful runs), so this threshold cleanly distinguishes the two states.
        /// </summary>
        public const int MinEntityCountForApply = 1000;

        public static int Apply(EntityManager em, RegistryManager registry, Action<string>? log)
        {
            if (registry == null) throw new ArgumentNullException(nameof(registry));

            var __metricsSw = System.Diagnostics.Stopwatch.StartNew();
            Action<string> write = log ?? (_ => { });

            // Gate on total entity count: in MainMenu only prefab templates exist (~100s of entities)
            // and GetComponentData on those archetypes throws NRE from EntityManager internals because
            // chunk buffers are transitional. Gameplay scenes report >1000 real entities. Skipping the
            // injection here is safe because RebuildCatalogAndApplyStats is re-invoked on scene
            // transitions — once gameplay loads, this will be called again with a populated world.
            int totalEntities = SafeEntityCount(em);
            if (totalEntities < MinEntityCountForApply)
            {
                write($"[PackStatInjector] Skipping — only {totalEntities} entities in world " +
                      $"(threshold {MinEntityCountForApply}). Likely MainMenu state; will retry " +
                      "when gameplay scene populates the world.");
                return 0;
            }

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

            // #920: Telemetry — record stat injection metrics.
            try
            {
                __metricsSw.Stop();
                MetricsCollector.Instance.RecordValue("stat_inject.writes_total", totalWrites);
                MetricsCollector.Instance.RecordValue("stat_inject.units_processed", unitsProcessed);
                MetricsCollector.Instance.RecordDuration("stat_inject.duration_ms", __metricsSw.Elapsed);
            }
            catch
            {
                // Best-effort: telemetry must never throw
            }

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
                    // Per-stat isolation: log and continue with the next stat.
                    // Unwrap TargetInvocationException so the user-facing log surfaces the real
                    // exception type (was previously masked, making NRE source unidentifiable).
                    Exception root = ex is TargetInvocationException tie ? (tie.InnerException ?? ex) : ex;
                    log($"[PackStatInjector]   ERROR applying '{sdkPath}' " +
                        $"for unit '{unit.Id}': {root.GetType().Name}: {root.Message}");
                    // Full stack trace to dedicated debug log for diagnosis; user log stays concise.
                    DebugLog.Write("PackStatInjector",
                        $"ERROR applying '{sdkPath}' for unit '{unit.Id}': {root}");
                }
            }

            return writes;
        }

        /// <summary>
        /// Defensively reads <see cref="EntityManager.UniversalQuery"/> entity count.
        /// Returns 0 if the EntityManager is in a transitional state that throws on access.
        /// </summary>
        private static int SafeEntityCount(EntityManager em)
        {
            try
            {
                return em.UniversalQuery.CalculateEntityCount();
            }
            catch (Exception ex)
            {
                DebugLog.Write("PackStatInjector",
                    $"SafeEntityCount: EntityManager not queryable yet: {ex.GetType().Name}: {ex.Message}");
                return 0;
            }
        }
    }
}
