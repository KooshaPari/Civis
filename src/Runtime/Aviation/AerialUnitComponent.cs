using Unity.Entities;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// ECS component that marks an entity as an aerial unit.
    /// Added by <see cref="AerialUnitMapper"/> when a unit definition has the "Aerial" behavior tag.
    ///
    /// Processed each frame by <see cref="AerialMovementSystem"/> for altitude maintenance
    /// and by <see cref="AerialSpawnSystem"/> for initial altitude placement.
    /// </summary>
    public struct AerialUnitComponent : IComponentData
    {
        /// <summary>
        /// Target altitude in world units above ground (Y axis).
        /// </summary>
        public float CruiseAltitude;

        /// <summary>
        /// Speed at which the unit climbs toward its cruise altitude (world units/second).
        /// </summary>
        public float AscendSpeed;

        /// <summary>
        /// Speed at which the unit descends when attacking or landing (world units/second).
        /// </summary>
        public float DescendSpeed;

        /// <summary>
        /// When true, the unit descends toward ground for an attack run.
        /// AerialMovementSystem will drive Y toward 0 when this is set.
        /// </summary>
        public bool IsAttacking;
    }
}
