#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.SDK.HotReload
{
    /// <summary>
    /// Immutable result of a hot-reload operation on one or more pack files.
    /// </summary>
    public sealed class HotReloadResult
    {
        /// <summary>Whether the reload completed without errors.</summary>
        public bool IsSuccess { get; }

        /// <summary>Files that were detected as changed.</summary>
        public IReadOnlyList<string> ChangedFiles { get; }

        /// <summary>Registry entry IDs that were updated as a result of the reload.</summary>
        public IReadOnlyList<string> UpdatedEntries { get; }

        /// <summary>Errors encountered during the reload attempt.</summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>Timestamp when the reload was initiated.</summary>
        public DateTimeOffset Timestamp { get; }

        private HotReloadResult(
            bool isSuccess,
            IReadOnlyList<string> changedFiles,
            IReadOnlyList<string> updatedEntries,
            IReadOnlyList<string> errors,
            DateTimeOffset timestamp)
        {
            IsSuccess = isSuccess;
            ChangedFiles = changedFiles;
            UpdatedEntries = updatedEntries;
            Errors = errors;
            Timestamp = timestamp;
        }

        /// <summary>Creates a successful reload result.</summary>
        /// <param name="changedFiles">The files that changed.</param>
        /// <param name="updatedEntries">The registry entries that were refreshed.</param>
        public static HotReloadResult Success(
            IReadOnlyList<string> changedFiles,
            IReadOnlyList<string> updatedEntries)
            => new HotReloadResult(true, changedFiles, updatedEntries,
                new List<string>().AsReadOnly(), DateTimeOffset.UtcNow);

        /// <summary>Creates a failed reload result.</summary>
        /// <param name="changedFiles">The files that changed.</param>
        /// <param name="errors">Errors that prevented a successful reload.</param>
        public static HotReloadResult Failure(
            IReadOnlyList<string> changedFiles,
            IReadOnlyList<string> errors)
            => new HotReloadResult(false, changedFiles,
                new List<string>().AsReadOnly(), errors, DateTimeOffset.UtcNow);

        /// <summary>Creates a partial reload result: some entries updated but errors occurred.</summary>
        /// <param name="changedFiles">The files that changed.</param>
        /// <param name="updatedEntries">The registry entries that were refreshed.</param>
        /// <param name="errors">Errors encountered for some files.</param>
        public static HotReloadResult Partial(
            IReadOnlyList<string> changedFiles,
            IReadOnlyList<string> updatedEntries,
            IReadOnlyList<string> errors)
            => new HotReloadResult(false, changedFiles, updatedEntries, errors, DateTimeOffset.UtcNow);
    }
}
