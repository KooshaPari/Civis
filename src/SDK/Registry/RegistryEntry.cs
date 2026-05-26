namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Wraps a registry data value with the metadata needed for conflict resolution and auditing.
    /// </summary>
    /// <typeparam name="T">The type of the registered data object.</typeparam>
    public sealed class RegistryEntry<T>
    {
        /// <summary>
        /// Namespaced identifier in the form <c>pack_id:entry_id</c>
        /// (e.g. <c>base_game:raptor</c>).
        /// </summary>
        public string Id { get; }

        /// <summary>The registered data object.</summary>
        public T Data { get; }

        /// <summary>Which layer of the loading stack contributed this entry.</summary>
        public RegistrySource Source { get; }

        /// <summary>
        /// Effective load priority. Derived from <see cref="Source"/> (×1000) plus the
        /// pack's <c>load_order</c> value, so that source tier always outranks intra-tier ordering.
        /// </summary>
        public int Priority { get; }

        /// <summary>The <c>id</c> of the pack that registered this entry.</summary>
        public string SourcePackId { get; }

        /// <summary>
        /// Creates a new registry entry with computed priority.
        /// </summary>
        /// <param name="id">Namespaced entry identifier.</param>
        /// <param name="data">The content data object.</param>
        /// <param name="source">Which loading layer contributed this entry.</param>
        /// <param name="sourcePackId">The pack that registered this entry.</param>
        /// <param name="loadOrder">Intra-tier ordering value.</param>
        public RegistryEntry(string id, T data, RegistrySource source, string sourcePackId, int loadOrder = 100)
        {
            Id = id;
            Data = data;
            Source = source;
            SourcePackId = sourcePackId;
            // Tier occupies the thousands column; load_order refines within the tier.
            Priority = ((int)source * 1000) + loadOrder;
        }
    }
}
