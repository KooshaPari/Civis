namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Persisted application configuration model.
    /// Serialized to/from AppConfig.json in the app's local data folder.
    /// </summary>
    public sealed class AppConfig
    {
        /// <summary>Absolute path to the packs root directory.</summary>
        public string PacksDirectory { get; set; } = "";

        /// <summary>Absolute path to the game installation directory (optional).</summary>
        public string GameDirectory { get; set; } = "";

        /// <summary>Auto-refresh interval in milliseconds (0 = disabled).</summary>
        public int ReloadIntervalMs { get; set; } = 2000;
    }
}
