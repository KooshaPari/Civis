using System.Collections.Generic;
using System.Threading.Tasks;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Abstraction for loading pack metadata from a catalog source.
    /// Supports local filesystem paths (file://) and HTTP(S) URLs.
    /// </summary>
    public interface IModCatalogService
    {
        /// <summary>
        /// Loads pack entries from the specified catalog source.
        /// </summary>
        /// <param name="catalogUri">
        /// URI to the catalog. Supports:
        /// - Local folder path (e.g. "C:\Packs" or "file:///C:/Packs")
        /// - HTTP/HTTPS URL to a JSON catalog file (e.g. "https://example.com/catalog.json")
        /// </param>
        /// <returns>A result containing all discovered catalog entries and any errors.</returns>
        Task<CatalogLoadResult> LoadCatalogAsync(string catalogUri);
    }

    /// <summary>
    /// Result of a catalog load operation.
    /// </summary>
    public sealed class CatalogLoadResult
    {
        /// <summary>All pack entries discovered in the catalog.</summary>
        public IReadOnlyList<CatalogEntry> Entries { get; init; } = System.Array.Empty<CatalogEntry>();

        /// <summary>Error messages encountered during loading.</summary>
        public IReadOnlyList<string> Errors { get; init; } = System.Array.Empty<string>();

        /// <summary>The source URI that was loaded.</summary>
        public string SourceUri { get; init; } = "";

        /// <summary>Whether the catalog loaded without errors.</summary>
        public bool IsSuccess => Errors.Count == 0;
    }
}
