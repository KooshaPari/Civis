#nullable enable

namespace DINOForge.SDK.HotReload
{
    /// <summary>
    /// Resolves the affected pack root for a changed filesystem path.
    /// </summary>
    internal interface IPackRootResolver
    {
        /// <summary>
        /// Attempts to resolve the pack directory that owns the changed path.
        /// </summary>
        string? ResolvePackRoot(string changedPath, string packsRootDirectory);
    }
}
