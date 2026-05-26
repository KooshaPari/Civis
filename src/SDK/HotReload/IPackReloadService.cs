#nullable enable

namespace DINOForge.SDK.HotReload
{
    /// <summary>
    /// Applies pack content changes for a specific pack root.
    /// </summary>
    internal interface IPackReloadService
    {
        /// <summary>
        /// Reloads content from a single pack directory.
        /// </summary>
        ContentLoadResult ReloadPack(string packDirectory);
    }
}
