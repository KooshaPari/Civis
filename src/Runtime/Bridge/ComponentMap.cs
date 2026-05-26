using System;
using System.Collections.Generic;
using System.Reflection;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Maps DINOForge SDK model fields to DINO's ECS component types.
    /// This is the Rosetta Stone between our declarative data models and the game's internals.
    ///
    /// Naming convention for SdkModelPath:
    ///   "{domain}.{section}.{field}" e.g. "unit.stats.hp", "building.cost.wood"
    ///
    /// Manual testing:
    ///   1. Load game with DINOForge Runtime plugin
    ///   2. Check BepInEx/dinoforge_debug.log for resolved type counts
    ///   3. Compare against entity dump (BepInEx/dinoforge_dumps/) to verify mappings
    /// </summary>
    public static class ComponentMap
    {
        // ──────────────────────────────────────────────
        //  Unit Component Mappings
        // ──────────────────────────────────────────────

        /// <summary>HP tracking component. Field: currentHealth (float).</summary>
        public static readonly ComponentMapping UnitHealth =
            new ComponentMapping("Components.Health", "unit.stats.hp",
                "Current HP for units and buildings", "currentHealth");

        /// <summary>Max health multiplier on HealthBase. Field: _maxHealthMultiplier (float).</summary>
        public static readonly ComponentMapping UnitHealthBase =
            new ComponentMapping("Components.HealthBase", "unit.stats.hp_base",
                "Max health multiplier (blob ref has immutable base, this float is mutable)",
                "_maxHealthMultiplier");

        /// <summary>Core unit marker — present on all units.</summary>
        public static readonly ComponentMapping UnitTag =
            new ComponentMapping("Components.Unit", "unit",
                "Zero-sized tag marking an entity as a unit");

        /// <summary>Unit base data component.</summary>
        public static readonly ComponentMapping UnitBase =
            new ComponentMapping("Components.UnitBase", "unit.base",
                "Core unit data (type identifiers, base stats)");

        /// <summary>Defense values (armor). Field: type (ArmorType enum).</summary>
        public static readonly ComponentMapping UnitArmor =
            new ComponentMapping("Components.ArmorData", "unit.stats.armor",
                "Armor type enum from Utility.EnumsStorage.ArmorType", "type");

        /// <summary>Attack cooldown timer. Field: value (float).</summary>
        public static readonly ComponentMapping UnitAttackCooldown =
            new ComponentMapping("Components.AttackCooldown", "unit.stats.attack_cooldown",
                "Cooldown timer between attacks", "value");

        /// <summary>Attack speed animation modifier. Field: Value (float).</summary>
        public static readonly ComponentMapping UnitAttackSpeedMod =
            new ComponentMapping("Components.MmAnimationPropertyAttackSpeedModifier",
                "unit.stats.attack_speed",
                "Animation speed multiplier affecting attack rate", "Value");

        /// <summary>Projectile speed on attack cooldown. Field: fixedProjectileSpeed (float).</summary>
        public static readonly ComponentMapping UnitProjectileSpeed =
            new ComponentMapping("Components.AttackCooldown", "unit.stats.projectile_speed",
                "Projectile speed set during attack targeting", "fixedProjectileSpeed");

        /// <summary>Attack range (ground units).</summary>
        public static readonly ComponentMapping UnitAttackArea =
            new ComponentMapping("Components.GroundAttackArea", "unit.stats.range",
                "Ground attack range radius");

        /// <summary>Movement speed. Field: speed (float). Also has delayModify, currentModify.</summary>
        public static readonly ComponentMapping UnitMoveHeading =
            new ComponentMapping("Components.RawComponents.MoveHeading", "unit.stats.speed",
                "Movement speed and heading vector", "speed");

        /// <summary>Squad membership marker.</summary>
        public static readonly ComponentMapping UnitSquadMarker =
            new ComponentMapping("Components.RawComponents.SquadMarker", "unit.squad",
                "Squad assignment identifier");

        /// <summary>Regeneration timer. Field: regenerationStartTime (float).</summary>
        public static readonly ComponentMapping UnitRegeneration =
            new ComponentMapping("Components.Regeneration", "unit.stats.regen",
                "Regeneration timer start", "regenerationStartTime");

        /// <summary>Object identity. Field: value (int).</summary>
        public static readonly ComponentMapping ObjectId =
            new ComponentMapping("Components.RawComponents.ObjectId", "entity.object_id",
                "Unique object identifier per entity", "value");

        /// <summary>Price/cost data (blob asset reference, immutable).</summary>
        public static readonly ComponentMapping PriceBase =
            new ComponentMapping("Components.PriceBase", "entity.cost",
                "Unit/building cost definition (BlobAssetReference<PriceBaseData>)");

        // ── Unit Class Tags ──

        /// <summary>Melee unit type tag.</summary>
        public static readonly ComponentMapping UnitClassMelee =
            new ComponentMapping("Components.MeleeUnit", "unit.class.melee",
                "Zero-sized tag: melee unit");

        /// <summary>Ranged unit type tag.</summary>
        public static readonly ComponentMapping UnitClassRanged =
            new ComponentMapping("Components.RangeUnit", "unit.class.ranged",
                "Zero-sized tag: ranged unit");

        /// <summary>Cavalry unit type tag.</summary>
        public static readonly ComponentMapping UnitClassCavalry =
            new ComponentMapping("Components.CavalryUnit", "unit.class.cavalry",
                "Zero-sized tag: cavalry unit");

        /// <summary>Siege unit type tag.</summary>
        public static readonly ComponentMapping UnitClassSiege =
            new ComponentMapping("Components.SiegeUnit", "unit.class.siege",
                "Zero-sized tag: siege unit");

        /// <summary>Archer unit type tag.</summary>
        public static readonly ComponentMapping UnitClassArcher =
            new ComponentMapping("Components.Archer", "unit.class.archer",
                "Zero-sized tag: archer unit");

        /// <summary>Cast-only unit type tag (magic users).</summary>
        public static readonly ComponentMapping UnitClassCastOnly =
            new ComponentMapping("Components.CastOnlyUnit", "unit.class.cast_only",
                "Zero-sized tag: magic-only unit");

        /// <summary>High priority targeting marker.</summary>
        public static readonly ComponentMapping UnitHighPriority =
            new ComponentMapping("Components.HighPriorityUnit", "unit.class.high_priority",
                "Zero-sized tag: prioritized by enemy targeting");

        // ── Faction Tags ──

        /// <summary>Enemy faction tag. Absence implies player-owned.</summary>
        public static readonly ComponentMapping FactionEnemy =
            new ComponentMapping("Components.Enemy", "unit.faction.enemy",
                "Enemy base data (BlobAssetReference<EnemyBaseData>). " +
                "DINO has no explicit faction component — player vs enemy is the only split.");

        // ── Projectile Components ──

        /// <summary>Projectile base data (blob asset reference).</summary>
        public static readonly ComponentMapping ProjectileBase =
            new ComponentMapping("Components.ProjectileDataBase", "projectile.base",
                "Projectile data (BlobAssetReference<ProjectileData>)");

        /// <summary>Projectile runtime fly data. Field: damage (float).</summary>
        public static readonly ComponentMapping ProjectileDamage =
            new ComponentMapping("Components.RawComponents.ProjectileFlyData",
                "projectile.damage",
                "Runtime projectile damage value", "damage");

        /// <summary>Projectile gravity. Field: gravity (float).</summary>
        public static readonly ComponentMapping ProjectileGravity =
            new ComponentMapping("Components.RawComponents.ProjectileFlyData",
                "projectile.gravity",
                "Projectile gravity for ballistic arc", "gravity");

        // ──────────────────────────────────────────────
        //  Building Component Mappings
        // ──────────────────────────────────────────────

        /// <summary>Core building data.</summary>
        public static readonly ComponentMapping BuildingBase =
            new ComponentMapping("Components.BuildingBase", "building",
                "Base building component present on all buildings");

        /// <summary>Barracks building (note: game misspells as "Barraks").</summary>
        public static readonly ComponentMapping BuildingBarracks =
            new ComponentMapping("Components.Barraks", "building.type.barracks",
                "Barracks marker — note game-side typo 'Barraks'");

        /// <summary>Farm building.</summary>
        public static readonly ComponentMapping BuildingFarm =
            new ComponentMapping("Components.Farm", "building.type.farm",
                "Farm marker");

        /// <summary>House building.</summary>
        public static readonly ComponentMapping BuildingHouse =
            new ComponentMapping("Components.House", "building.type.house",
                "House marker (population housing)");

        /// <summary>House base data.</summary>
        public static readonly ComponentMapping BuildingHouseBase =
            new ComponentMapping("Components.HouseBase", "building.type.house_base",
                "House base data (capacity, etc.)");

        /// <summary>Granary building (food storage).</summary>
        public static readonly ComponentMapping BuildingGranary =
            new ComponentMapping("Components.Granary", "building.type.granary",
                "Food storage building");

        /// <summary>Hospital building.</summary>
        public static readonly ComponentMapping BuildingHospital =
            new ComponentMapping("Components.Hospital", "building.type.hospital",
                "Hospital marker");

        /// <summary>Hospital base data.</summary>
        public static readonly ComponentMapping BuildingHospitalBase =
            new ComponentMapping("Components.HospitalBase", "building.type.hospital_base",
                "Hospital base data (heal rate, capacity, etc.)");

        /// <summary>Forester house (wood production).</summary>
        public static readonly ComponentMapping BuildingForesterHouse =
            new ComponentMapping("Components.ForesterHouse", "building.type.forester",
                "Wood production building");

        /// <summary>Stone cutter (stone production).</summary>
        public static readonly ComponentMapping BuildingStoneCutter =
            new ComponentMapping("Components.StoneCutter", "building.type.stonecutter",
                "Stone production building");

        /// <summary>Iron mine.</summary>
        public static readonly ComponentMapping BuildingIronMine =
            new ComponentMapping("Components.IronMine", "building.type.ironmine",
                "Iron production building (finite deposit)");

        /// <summary>Infinite iron mine.</summary>
        public static readonly ComponentMapping BuildingInfiniteIronMine =
            new ComponentMapping("Components.InfiniteIronMine", "building.type.ironmine_infinite",
                "Iron production building (infinite deposit)");

        /// <summary>Soul mine.</summary>
        public static readonly ComponentMapping BuildingSoulMine =
            new ComponentMapping("Components.SoulMine", "building.type.soulmine",
                "Soul crystal production building");

        // ──────────────────────────────────────────────
        //  Resource Component Mappings
        //
        //  NOTE: These primary mappings are used by ResourceReader's fallback chain.
        //  If the exact type name or field does not exist in the running game version,
        //  ResourceReader automatically tries alternatives — see ResourceReader.cs.
        //  To identify the correct names for a given game build, check:
        //    BepInEx/dinoforge_debug.log   (logs "Available fields:" on every miss)
        //  All resource entity queries MUST use EntityQueryOptions.IncludePrefab —
        //  DINO marks all live entities (including resource singletons) as ECS Prefabs.
        // ──────────────────────────────────────────────

        /// <summary>Current food stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceFood =
            new ComponentMapping("Components.RawComponents.CurrentFood", "resource.current.food",
                "Current food resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current iron stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceIron =
            new ComponentMapping("Components.RawComponents.CurrentIron", "resource.current.iron",
                "Current iron resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current stone stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceStone =
            new ComponentMapping("Components.RawComponents.CurrentStone", "resource.current.stone",
                "Current stone resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current wood stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceWood =
            new ComponentMapping("Components.RawComponents.CurrentWood", "resource.current.wood",
                "Current wood resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current money (gold) stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceMoney =
            new ComponentMapping("Components.RawComponents.CurrentMoney", "resource.current.money",
                "Current money/gold resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current souls stockpile. Field: value (int). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceSouls =
            new ComponentMapping("Components.RawComponents.CurrentSouls", "resource.current.souls",
                "Current soul crystal resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to alternative type names if this does not resolve.", "value");

        /// <summary>Current bones stockpile. Field: valueContainer.value (nested CurrentResources). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceBones =
            new ComponentMapping("Components.RawComponents.CurrentBones", "resource.current.bones",
                "Current bones resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to flat 'value' and alternative type names if nested path fails.", "valueContainer.value");

        /// <summary>Current spirit stockpile. Field: valueContainer.value (nested CurrentResources). NOTE: type name unverified — see ResourceReader fallback chain.</summary>
        public static readonly ComponentMapping ResourceSpirit =
            new ComponentMapping("Components.RawComponents.CurrentSpirit", "resource.current.spirit",
                "Current spirit resource amount (singleton, prefab entity). " +
                "ResourceReader will fall back to flat 'value' and alternative type names if nested path fails.", "valueContainer.value");

        /// <summary>Food production source.</summary>
        public static readonly ComponentMapping ResourceFoodSource =
            new ComponentMapping("Components.FoodSource", "resource.source.food",
                "Entity produces food");

        /// <summary>Iron production source.</summary>
        public static readonly ComponentMapping ResourceIronSource =
            new ComponentMapping("Components.IronSource", "resource.source.iron",
                "Entity produces iron");

        /// <summary>Stone production source.</summary>
        public static readonly ComponentMapping ResourceStoneSource =
            new ComponentMapping("Components.StoneSource", "resource.source.stone",
                "Entity produces stone");

        /// <summary>Food storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceFoodStorage =
            new ComponentMapping("Components.FoodStorage", "resource.storage.food",
                "Entity stores food", "stored");

        /// <summary>Wood storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceWoodStorage =
            new ComponentMapping("Components.WoodStorage", "resource.storage.wood",
                "Entity stores wood", "stored");

        /// <summary>Stone storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceStoneStorage =
            new ComponentMapping("Components.StoneStorage", "resource.storage.stone",
                "Entity stores stone", "stored");

        /// <summary>Iron storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceIronStorage =
            new ComponentMapping("Components.IronStorage", "resource.storage.iron",
                "Entity stores iron", "stored");

        /// <summary>Bones storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceBonesStorage =
            new ComponentMapping("Components.BonesStorage", "resource.storage.bones",
                "Entity stores bones", "stored");

        /// <summary>Corpse storage. Field: stored (int).</summary>
        public static readonly ComponentMapping ResourceCorpseStorage =
            new ComponentMapping("Components.CorpseStorage", "resource.storage.corpse",
                "Entity stores corpses", "stored");

        // ──────────────────────────────────────────────
        //  Aggregate Lookup
        // ──────────────────────────────────────────────

        private static readonly Dictionary<string, ComponentMapping> _all;

        /// <summary>
        /// All registered mappings indexed by SDK model path.
        /// </summary>
        public static IReadOnlyDictionary<string, ComponentMapping> All => _all;

        static ComponentMap()
        {
            _all = new Dictionary<string, ComponentMapping>(StringComparer.OrdinalIgnoreCase);

            // Register every static ComponentMapping field in this class
            foreach (FieldInfo field in typeof(ComponentMap).GetFields(
                BindingFlags.Public | BindingFlags.Static))
            {
                if (field.FieldType == typeof(ComponentMapping))
                {
                    ComponentMapping? mapping = field.GetValue(null) as ComponentMapping;
                    if (mapping != null)
                    {
                        _all[mapping.SdkModelPath] = mapping;
                    }
                }
            }
        }

        /// <summary>
        /// Look up a mapping by its SDK model path.
        /// </summary>
        /// <param name="sdkPath">Dot-separated SDK model path (e.g. "unit.stats.hp").</param>
        /// <returns>The mapping, or null if not found.</returns>
        public static ComponentMapping? Find(string sdkPath)
        {
            _all.TryGetValue(sdkPath, out ComponentMapping? mapping);
            return mapping;
        }

        /// <summary>
        /// Look up a mapping by the ECS component type name.
        /// </summary>
        /// <param name="ecsTypeName">Full ECS type name (e.g. "Components.Health").</param>
        /// <returns>The first matching mapping, or null.</returns>
        public static ComponentMapping? FindByEcsType(string ecsTypeName)
        {
            foreach (ComponentMapping mapping in _all.Values)
            {
                if (string.Equals(mapping.EcsComponentType, ecsTypeName, StringComparison.Ordinal))
                    return mapping;
            }
            return null;
        }

        /// <summary>
        /// Attempt to resolve all CLR types and return a diagnostic summary.
        /// Useful for startup validation.
        /// </summary>
        /// <returns>Tuple of (resolved count, total count, list of unresolved type names).</returns>
        public static (int Resolved, int Total, List<string> Unresolved) ValidateResolution()
        {
            List<string> unresolved = new List<string>();
            int resolved = 0;

            foreach (ComponentMapping mapping in _all.Values)
            {
                if (mapping.ResolvedType != null)
                    resolved++;
                else
                    unresolved.Add(mapping.EcsComponentType);
            }

            return (resolved, _all.Count, unresolved);
        }
    }
}
