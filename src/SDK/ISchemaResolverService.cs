using System.Collections.Generic;

namespace DINOForge.SDK
{
    /// <summary>
    /// Service for resolving content types to schema names for validation.
    /// </summary>
    internal interface ISchemaResolverService
    {
        /// <summary>
        /// Gets all registered content types.
        /// </summary>
        IReadOnlyCollection<string> ContentTypes { get; }

        /// <summary>
        /// Attempts to resolve a content type key to its corresponding schema name.
        /// </summary>
        /// <param name="contentType">Content type key (e.g., "units", "buildings").</param>
        /// <param name="schemaName">Output schema name if resolved.</param>
        /// <returns>True if the content type was resolved to a schema name.</returns>
        bool TryResolveSchemaName(string contentType, out string schemaName);
    }
}
