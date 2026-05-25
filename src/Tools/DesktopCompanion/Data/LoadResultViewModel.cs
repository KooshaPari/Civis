using System.Collections.Generic;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Result of a pack-directory scan / load operation.
    /// </summary>
    public sealed class LoadResultViewModel
    {
        /// <summary>Number of packs successfully parsed from disk.</summary>
        public int LoadedCount { get; init; }

        /// <summary>Number of packs with parse/validation errors.</summary>
        public int ErrorCount { get; init; }

        /// <summary>Flat list of all error messages across all packs.</summary>
        public IReadOnlyList<string> Errors { get; init; } = System.Array.Empty<string>();

        /// <summary>All discovered packs (enabled and disabled).</summary>
        public IReadOnlyList<PackViewModel> Packs { get; init; } = System.Array.Empty<PackViewModel>();

        /// <summary>Whether the scan completed without any errors.</summary>
        public bool IsSuccess => ErrorCount == 0;
    }
}
