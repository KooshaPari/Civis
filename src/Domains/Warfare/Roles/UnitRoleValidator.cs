using System;
using System.Collections.Generic;
using System.Reflection;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Warfare.Roles
{
    /// <summary>
    /// Validates that a faction's roster fills all required unit role slots.
    /// Each faction should map its roster slots to real unit IDs in the unit registry.
    /// </summary>
    public sealed class UnitRoleValidator
    {
        /// <summary>
        /// The required unit roles that every faction must fill.
        /// Maps to the property names on <see cref="FactionRoster"/>.
        /// </summary>
        public static IReadOnlyList<string> RequiredRoles { get; } = new List<string>
        {
            "cheap_infantry",
            "line_infantry",
            "elite_infantry",
            "anti_armor",
            "support_weapon",
            "recon",
            "light_vehicle",
            "heavy_vehicle",
            "artillery",
            "hero_commander",
            "spike_unit"
        }.AsReadOnly();

        // Map from role key to the FactionRoster property that holds its unit ID.
        private static readonly Dictionary<string, Func<FactionRoster, string?>> RoleAccessors =
            new Dictionary<string, Func<FactionRoster, string?>>(StringComparer.OrdinalIgnoreCase)
            {
                { "cheap_infantry", r => r.CheapInfantry },
                { "line_infantry", r => r.LineInfantry },
                { "elite_infantry", r => r.EliteInfantry },
                { "anti_armor", r => r.AntiArmor },
                { "support_weapon", r => r.SupportWeapon },
                { "recon", r => r.Recon },
                { "light_vehicle", r => r.LightVehicle },
                { "heavy_vehicle", r => r.HeavyVehicle },
                { "artillery", r => r.Artillery },
                { "hero_commander", r => r.HeroCommander },
                { "spike_unit", r => r.SpikeUnit },
            };

        /// <summary>
        /// Validate that a faction's roster fills all required roles with units
        /// that exist in the unit registry.
        /// </summary>
        /// <param name="faction">The faction definition to validate.</param>
        /// <param name="units">The unit registry to verify unit references against.</param>
        /// <returns>A validation result detailing filled and missing roles.</returns>
        public RosterValidationResult ValidateRoster(FactionDefinition faction, IRegistry<UnitDefinition> units)
        {
            if (faction == null) throw new ArgumentNullException(nameof(faction));
            if (units == null) throw new ArgumentNullException(nameof(units));

            var missing = new List<string>();
            var filled = new List<string>();
            var roleMap = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

            foreach (string role in RequiredRoles)
            {
                if (!RoleAccessors.TryGetValue(role, out Func<FactionRoster, string?>? accessor))
                {
                    missing.Add(role);
                    continue;
                }

                string? unitId = accessor(faction.Roster);

                if (string.IsNullOrWhiteSpace(unitId))
                {
                    missing.Add(role);
                }
                else if (unitId != null && !units.Contains(unitId))
                {
                    missing.Add(role);
                }
                else
                {
                    filled.Add(role);
                    roleMap[role] = unitId!;
                }
            }

            return new RosterValidationResult(
                isComplete: missing.Count == 0,
                missingRoles: missing.AsReadOnly(),
                filledRoles: filled.AsReadOnly(),
                roleToUnitMap: roleMap);
        }
    }
}
