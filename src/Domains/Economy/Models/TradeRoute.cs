namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// Defines a trade route that converts one resource into another at a specified exchange rate.
    /// In DINO, gold is obtained exclusively through trading via the merchant dirigible,
    /// making trade routes critical for late-game economy.
    /// </summary>
    public sealed class TradeRoute
    {
        /// <summary>
        /// Unique identifier for this trade route.
        /// </summary>
        public string Id { get; set; } = "";

        /// <summary>
        /// Display name of this trade route.
        /// </summary>
        public string DisplayName { get; set; } = "";

        /// <summary>
        /// The resource type being sold/given (e.g. "wood", "food").
        /// Must be one of: food, wood, stone, iron, gold.
        /// </summary>
        public string SourceResource { get; set; } = "";

        /// <summary>
        /// The resource type being received (e.g. "gold", "iron").
        /// Must be one of: food, wood, stone, iron, gold.
        /// </summary>
        public string TargetResource { get; set; } = "";

        /// <summary>
        /// How many units of source resource are needed to produce one unit of target resource.
        /// Higher values mean worse exchange rates. For example, 10.0 means 10 wood per 1 gold.
        /// </summary>
        public float ExchangeRate { get; set; } = 1.0f;

        /// <summary>
        /// Cooldown in game ticks between trade executions on this route.
        /// Prevents instant infinite conversion loops.
        /// </summary>
        public int CooldownTicks { get; set; } = 60;

        /// <summary>
        /// Maximum amount of source resource that can be traded per execution.
        /// Zero means unlimited.
        /// </summary>
        public int MaxPerTransaction { get; set; } = 0;

        /// <summary>
        /// Whether this trade route is currently enabled.
        /// </summary>
        public bool Enabled { get; set; } = true;
    }
}
