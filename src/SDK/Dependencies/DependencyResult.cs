using System.Collections.Generic;

namespace DINOForge.SDK.Dependencies
{
    /// <summary>
    /// Immutable result of a pack dependency resolution operation.
    /// Use <see cref="Success"/> or <see cref="Failure"/> factory methods to construct.
    /// </summary>
    public sealed class DependencyResult
    {
        /// <summary>Whether dependency resolution completed without errors.</summary>
        public bool IsSuccess { get; }

        /// <summary>The resolved load order when successful; empty on failure.</summary>
        public IReadOnlyList<PackManifest> LoadOrder { get; }

        /// <summary>Error messages describing resolution failures; empty on success.</summary>
        public IReadOnlyList<string> Errors { get; }

        private DependencyResult(bool isSuccess, IReadOnlyList<PackManifest> loadOrder, IReadOnlyList<string> errors)
        {
            IsSuccess = isSuccess;
            LoadOrder = loadOrder;
            Errors = errors;
        }

        /// <summary>
        /// Creates a successful result with the computed load order.
        /// </summary>
        /// <param name="loadOrder">The ordered list of packs to load.</param>
        public static DependencyResult Success(IReadOnlyList<PackManifest> loadOrder)
            => new DependencyResult(true, loadOrder, new List<string>().AsReadOnly());

        /// <summary>
        /// Creates a failed result with error descriptions.
        /// </summary>
        /// <param name="errors">The errors that prevented resolution.</param>
        public static DependencyResult Failure(IReadOnlyList<string> errors)
            => new DependencyResult(false, new List<PackManifest>().AsReadOnly(), errors);
    }
}
