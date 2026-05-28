namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Pack tier classification representing the scope and content type of a pack.
    /// Used to display visual badges and enable category-specific features in the mod menu.
    /// </summary>
    public enum PackTier
    {
        /// <summary>Adds new mechanics/systems (e.g., aerial combat, naval). Doesn't replace vanilla content.</summary>
        EngineExtension,

        /// <summary>Adds units/buildings/factions on top of existing systems.</summary>
        Content,

        /// <summary>Replaces all vanilla content with a complete themed conversion (e.g., Star Wars).</summary>
        TotalConversion,

        /// <summary>The vanilla-DINO baseline (auto-detected, can't be disabled).</summary>
        Baseline
    }
}
