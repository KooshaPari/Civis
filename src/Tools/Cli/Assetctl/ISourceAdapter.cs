#nullable enable
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Interface for asset source adapters that search and retrieve assets from various sources.
/// </summary>
public interface ISourceAdapter
{
    /// <summary>
    /// Searches for assets matching the specified query.
    /// </summary>
    /// <param name="query">The search query string.</param>
    /// <param name="cancellationToken">A cancellation token to observe.</param>
    /// <returns>An enumerable of asset candidates matching the query.</returns>
    Task<IEnumerable<AssetCandidate>> SearchAsync(string query, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets an asset by its unique identifier.
    /// </summary>
    /// <param name="id">The source-specific asset identifier.</param>
    /// <param name="cancellationToken">A cancellation token to observe.</param>
    /// <returns>The asset candidate if found; otherwise, null.</returns>
    Task<AssetCandidate?> GetByIdAsync(string id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets the name of the source this adapter handles.
    /// </summary>
    string SourceName { get; }

    /// <summary>
    /// Gets a value indicating whether this adapter supports searching.
    /// </summary>
    bool SupportsSearch { get; }

    /// <summary>
    /// Gets a value indicating whether this adapter supports direct ID lookup.
    /// </summary>
    bool SupportsGetById { get; }
}
