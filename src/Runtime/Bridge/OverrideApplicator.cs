#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Translates loaded pack content (units, buildings, etc.) into
    /// <see cref="StatModification"/> objects and enqueues them into
    /// <see cref="StatModifierSystem"/> for runtime application.
    /// </summary>
    public static class OverrideApplicator
    {
        /// <summary>
        /// Per-unit stat overrides via StatModifierSystem are intentionally skipped.
        ///
        /// The StatModifierSystem applies modifications to ALL entities matching a component type
        /// (e.g. all entities with Components.Health), with no per-unit filtering. Applying
        /// per-unit HP/Speed/etc. values from pack YAML would overwrite all game entities
        /// with the last value processed, breaking game balance entirely.
        ///
        /// Per-unit stat values in pack YAML are used by PackUnitSpawner when spawning new
        /// custom units — they are NOT applied as ECS overrides to vanilla entities.
        ///
        /// Use ApplyStatOverrides (YAML overrides section) for intentional global balance
        /// changes (e.g. multiply all HP by 2.0 as a difficulty modifier).
        /// </summary>
        /// <param name="registryManager">The registry manager containing loaded content.</param>
        /// <param name="log">Logging callback for diagnostics.</param>
        /// <returns>Always 0 — no modifications enqueued.</returns>
        public static int ApplyUnitOverrides(RegistryManager registryManager, Action<string> log)
        {
            if (registryManager == null) throw new ArgumentNullException(nameof(registryManager));
            if (log == null) throw new ArgumentNullException(nameof(log));

            log("[OverrideApplicator] Per-unit stat overrides skipped (StatModifierSystem applies globally, not per-unit-type).");
            return 0;
        }

        /// <summary>
        /// Applies YAML-loaded stat overrides to entities via StatModifierSystem.
        /// Converts each override entry to a StatModification and enqueues for application.
        /// </summary>
        /// <param name="overrides">The stat override definitions loaded from YAML.</param>
        /// <param name="log">Logging callback for diagnostics.</param>
        /// <returns>The number of stat modifications enqueued.</returns>
        public static int ApplyStatOverrides(IReadOnlyList<SDK.Models.StatOverrideDefinition> overrides, Action<string> log)
        {
            if (overrides == null) throw new ArgumentNullException(nameof(overrides));
            if (log == null) throw new ArgumentNullException(nameof(log));

            List<StatModification> modifications = new List<StatModification>();

            foreach (SDK.Models.StatOverrideDefinition definition in overrides)
            {
                if (definition.Overrides == null || definition.Overrides.Count == 0)
                    continue;

                try
                {
                    foreach (SDK.Models.StatOverrideEntry entry in definition.Overrides)
                    {
                        // entry.Mode is a StatOverrideMode enum from YAML deserialization.
                        // Map to runtime ModifierMode (the runtime enum is internal to
                        // the Bridge layer; we keep the mapping explicit so the layers can
                        // diverge in the future without breaking pack authors).
                        ModifierMode mode = entry.Mode switch
                        {
                            StatOverrideMode.Add => ModifierMode.Add,
                            StatOverrideMode.Multiply => ModifierMode.Multiply,
                            StatOverrideMode.Override => ModifierMode.Override,
                            _ => ModifierMode.Override,  // Default fallback
                        };

                        // Create stat modification
                        StatModification mod = new StatModification(
                            entry.Target,
                            entry.Value,
                            mode,
                            entry.Filter);

                        modifications.Add(mod);
                        log($"[OverrideApplicator] YAML override: {entry.Target} = {entry.Value} ({mode})");
                    }
                }
                catch (Exception ex)
                {
                    log($"[OverrideApplicator] Error processing stat override definition: {ex.Message}");
                }
            }

            if (modifications.Count > 0)
            {
                StatModifierSystem.EnqueueRange(modifications);
                log($"[OverrideApplicator] Enqueued {modifications.Count} YAML stat override(s).");
            }

            return modifications.Count;
        }

    }
}
