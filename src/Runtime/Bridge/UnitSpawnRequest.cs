namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// A request to spawn a unit from a pack definition at a given world position.
    /// Used internally by PackUnitSpawner to queue spawn requests from the API layer.
    /// </summary>
    public struct UnitSpawnRequest
    {
        /// <summary>
        /// The pack unit definition ID (e.g. "modern-warfare:m1_abrams").
        /// Must exist in the loaded pack registries.
        /// </summary>
        public string UnitDefinitionId { get; set; }

        /// <summary>
        /// World X coordinate (left-right).
        /// </summary>
        public float X { get; set; }

        /// <summary>
        /// World Y coordinate (vertical altitude). Default 0 (ground level).
        /// Aerial units set this to their CruiseAltitude via AerialUnitMapper.
        /// </summary>
        public float Y { get; set; }

        /// <summary>
        /// World Z coordinate (forward-backward).
        /// </summary>
        public float Z { get; set; }

        /// <summary>
        /// Whether this unit belongs to the enemy faction (true) or player faction (false).
        /// Corresponds to presence of Components.Enemy on the spawned entity.
        /// </summary>
        public bool IsEnemy { get; set; }

        public UnitSpawnRequest(string unitDefinitionId, float x, float z, bool isEnemy = false, float y = 0f)
        {
            UnitDefinitionId = unitDefinitionId;
            X = x;
            Y = y;
            Z = z;
            IsEnemy = isEnemy;
        }

        public override string ToString()
        {
            string faction = IsEnemy ? "enemy" : "player";
            return $"UnitSpawnRequest({UnitDefinitionId}, pos=({X}, {Y}, {Z}), faction={faction})";
        }
    }
}
