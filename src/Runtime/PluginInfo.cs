namespace DINOForge.Runtime
{
    /// <summary>
    /// Plugin metadata constants.
    /// </summary>
    public static class PluginInfo
    {
        public const string GUID = "com.dinoforge.runtime";
        public const string NAME = "DINOForge Runtime";
        public const string VERSION = "0.25.0-dev";

        /// <summary>
        /// Strict numeric version (Major.Minor.Build) for [BepInPlugin] attributes.
        /// BepInEx 5.4's chainloader uses System.Version.Parse, which rejects
        /// pre-release suffixes (e.g. "-dev"). Use this constant ONLY for the
        /// BepInPlugin attribute; use VERSION everywhere else.
        /// </summary>
        public const string BEPINEX_VERSION = "0.25.0";
    }
}
