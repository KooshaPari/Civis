using Unity.Entities;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// ECS component that marks a unit or building as capable of engaging aerial targets.
    /// Units with this component can target entities with <see cref="AerialUnitComponent"/>.
    ///
    /// Attach to ground units and buildings that should act as anti-air defenses.
    /// The range and damage bonus are applied by AerialTargetingSystem when selecting targets.
    /// </summary>
    public struct AntiAirComponent : IComponentData
    {
        /// <summary>
        /// Maximum range in world units at which this unit can engage aerial targets.
        /// </summary>
        public float AntiAirRange;

        /// <summary>
        /// Damage multiplier applied when attacking aerial targets (e.g. 1.5f = +50% damage vs aerial).
        /// </summary>
        public float AntiAirDamageBonus;
    }
}
