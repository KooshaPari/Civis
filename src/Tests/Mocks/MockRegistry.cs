#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Registry;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock implementation of <see cref="IRegistry{T}"/> for unit testing.
/// Tracks all registrations, overrides, and call counts for behavior verification.
/// NOT thread-safe; intended for single-threaded test fixtures only.
/// </summary>
/// <typeparam name="T">The type of content managed by this mock registry.</typeparam>
public class MockRegistry<T> : IRegistry<T>
{
    private readonly Dictionary<string, RegistryEntry<T>> _entries;
    private readonly Dictionary<string, List<RegistryEntry<T>>> _allByKey;

    /// <summary>
    /// Number of times <see cref="Register"/> has been called.
    /// </summary>
    public int RegisterCount { get; set; }

    /// <summary>
    /// Number of times <see cref="Get"/> has been called.
    /// </summary>
    public int GetCount { get; set; }

    /// <summary>
    /// Number of times <see cref="Contains"/> has been called.
    /// </summary>
    public int ContainsCount { get; set; }

    /// <summary>
    /// Number of times <see cref="Override"/> has been called.
    /// </summary>
    public int OverrideCount { get; set; }

    /// <summary>
    /// Number of times <see cref="DetectConflicts"/> has been called.
    /// </summary>
    public int DetectConflictsCount { get; set; }

    /// <summary>
    /// Creates a new mock registry with an empty in-memory store.
    /// </summary>
    public MockRegistry()
    {
        _entries = new Dictionary<string, RegistryEntry<T>>(StringComparer.Ordinal);
        _allByKey = new Dictionary<string, List<RegistryEntry<T>>>(StringComparer.Ordinal);
    }

    /// <summary>
    /// Registers an entry. Stores it in memory and tracks highest-priority entry per ID.
    /// Increments <see cref="RegisterCount"/>.
    /// </summary>
    public void Register(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100)
    {
        RegisterCount++;

        var regEntry = new RegistryEntry<T>(id, entry, source, sourcePackId, loadOrder);

        if (!_allByKey.ContainsKey(id))
            _allByKey[id] = [];

        _allByKey[id].Add(regEntry);

        // Update highest-priority entry for this ID
        var highest = _allByKey[id].OrderByDescending(e => e.Priority).First();
        _entries[id] = highest;
    }

    /// <summary>
    /// Retrieves the highest-priority entry for the given ID, or default if not found.
    /// Increments <see cref="GetCount"/>.
    /// </summary>
    public T? Get(string id)
    {
        GetCount++;
        return _entries.TryGetValue(id, out var entry) ? entry.Data : default;
    }

    /// <summary>
    /// Returns true if an entry with the given ID exists.
    /// Increments <see cref="ContainsCount"/>.
    /// </summary>
    public bool Contains(string id)
    {
        ContainsCount++;
        return _entries.ContainsKey(id);
    }

    /// <summary>
    /// Gets a snapshot of all entries, returning only the highest-priority entry per ID.
    /// </summary>
    public IReadOnlyDictionary<string, RegistryEntry<T>> All => _entries.AsReadOnly();

    /// <summary>
    /// Registers an override entry, treating it the same as a normal registration.
    /// Increments <see cref="OverrideCount"/>.
    /// </summary>
    public void Override(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100)
    {
        OverrideCount++;
        Register(id, entry, source, sourcePackId, loadOrder);
    }

    /// <summary>
    /// Detects entries where multiple registrations share the same top priority.
    /// Increments <see cref="DetectConflictsCount"/>.
    /// </summary>
    public IReadOnlyList<RegistryConflict> DetectConflicts()
    {
        DetectConflictsCount++;

        var conflicts = new List<RegistryConflict>();

        foreach (var kvp in _allByKey)
        {
            var id = kvp.Key;
            var candidates = kvp.Value;

            if (candidates.Count < 2)
                continue;

            var maxPriority = candidates.Max(e => e.Priority);
            var tied = candidates.Where(e => e.Priority == maxPriority).ToList();

            if (tied.Count > 1)
            {
                var conflictingPackIds = tied.Select(e => e.SourcePackId).Distinct(StringComparer.Ordinal).ToList();
                conflicts.Add(new RegistryConflict(
                    id,
                    conflictingPackIds,
                    $"Entry '{id}' has {tied.Count} registrations at priority {maxPriority}"));
            }
        }

        return conflicts;
    }
}
