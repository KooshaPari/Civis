namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Identifies where a registry entry originated, used to determine default priority.
    /// Higher numeric value = higher priority when resolving conflicts.
    /// </summary>
    public enum RegistrySource
    {
        /// <summary>Shipped with the base game; lowest priority.</summary>
        BaseGame = 0,

        /// <summary>Provided by the DINOForge framework itself.</summary>
        Framework = 1,

        /// <summary>Contributed by a domain plug-in (e.g. a gameplay systems mod).</summary>
        DomainPlugin = 2,

        /// <summary>Contributed by a content pack; highest priority.</summary>
        Pack = 3
    }
}
