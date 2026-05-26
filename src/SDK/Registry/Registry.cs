using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Generic content registry that stores entries keyed by string ID, supporting
    /// priority-based conflict resolution and multi-source registration.
    /// </summary>
    /// <typeparam name="T">The type of content stored in this registry.</typeparam>
    public sealed class Registry<T> : IRegistry<T>
    {
        private readonly Dictionary<string, List<RegistryEntry<T>>> _entries =
            new Dictionary<string, List<RegistryEntry<T>>>(StringComparer.Ordinal);

        // #771: case-insensitive shadow set for detecting case-variant collisions.
        // IDs in DINOForge should use lowercase-kebab; registering both 'Clone-Trooper' and
        // 'clone-trooper' is almost always a bug because the case-sensitive primary store
        // will silently accept both as distinct entries while pack authors expect one.
        private readonly HashSet<string> _shadow = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        private readonly Action<string> _log;

        /// <summary>Creates a Registry with an optional warning logger sink (used for case-variant detection).</summary>
        public Registry(Action<string>? log = null)
        {
            _log = log ?? (_ => { });
        }

        /// <inheritdoc />
        public void Register(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100)
        {
            RegistryEntry<T> registryEntry = new RegistryEntry<T>(id, entry, source, sourcePackId, loadOrder);

            if (!_entries.TryGetValue(id, out List<RegistryEntry<T>>? list))
            {
                list = new List<RegistryEntry<T>>();
                _entries[id] = list;
            }

            // #771: case-variant collision detection. _shadow.Add returns false if a case-
            // insensitive duplicate already exists; if the case-sensitive store does NOT yet
            // contain this exact id, the only way the shadow can already have it is via a
            // different-casing entry — emit a warning.
            if (!_shadow.Add(id) && list.Count == 0)
            {
                _log($"[Registry<{typeof(T).Name}>] Case-variant collision: '{id}' conflicts with an existing case-variant ID. IDs should use lowercase-kebab convention.");
            }

            list.Add(registryEntry);
            list.Sort((a, b) => b.Priority.CompareTo(a.Priority));
        }

        /// <inheritdoc />
        public T? Get(string id)
        {
            if (_entries.TryGetValue(id, out List<RegistryEntry<T>>? list) && list.Count > 0)
                return list[0].Data;
            return default;
        }

        /// <inheritdoc />
        public bool Contains(string id)
        {
            return _entries.ContainsKey(id) && _entries[id].Count > 0;
        }

        /// <inheritdoc />
        public IReadOnlyDictionary<string, RegistryEntry<T>> All
        {
            get
            {
                Dictionary<string, RegistryEntry<T>> result = new Dictionary<string, RegistryEntry<T>>(StringComparer.Ordinal);
                foreach (KeyValuePair<string, List<RegistryEntry<T>>> kvp in _entries)
                {
                    if (kvp.Value.Count > 0)
                        result[kvp.Key] = kvp.Value[0];
                }
                return result;
            }
        }

        /// <inheritdoc />
        public void Override(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100)
        {
            Register(id, entry, source, sourcePackId, loadOrder);
        }

        /// <inheritdoc />
        public IReadOnlyList<RegistryConflict> DetectConflicts()
        {
            List<RegistryConflict> conflicts = new List<RegistryConflict>();

            foreach (KeyValuePair<string, List<RegistryEntry<T>>> kvp in _entries)
            {
                List<RegistryEntry<T>> list = kvp.Value;
                if (list.Count < 2)
                    continue;

                int topPriority = list[0].Priority;
                List<RegistryEntry<T>> tied = list.Where(e => e.Priority == topPriority).ToList();

                if (tied.Count >= 2)
                {
                    // Extract conflicting pack IDs from tied entries
                    List<string> conflictingPackIds = tied
                        .Select(e => e.SourcePackId)
                        .ToList();

                    conflicts.Add(new RegistryConflict(
                        kvp.Key,
                        conflictingPackIds.AsReadOnly(),
                        $"Entry '{kvp.Key}' has {tied.Count} entries with equal priority {topPriority}."));
                }
            }

            return conflicts;
        }
    }
}
