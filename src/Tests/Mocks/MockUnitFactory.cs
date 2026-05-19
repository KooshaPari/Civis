#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock implementation of <see cref="IUnitFactory"/> for unit testing.
/// Provides configurable spawn behavior with call tracking.
/// NOT thread-safe; intended for single-threaded test fixtures only.
/// </summary>
public class MockUnitFactory : IUnitFactory
{
    private readonly Dictionary<string, bool> _spawnableUnits;
    private readonly List<SpawnRequest> _spawnRequests;

    /// <summary>
    /// Number of times <see cref="CanSpawn"/> has been called.
    /// </summary>
    public int CanSpawnCount { get; set; }

    /// <summary>
    /// Number of times <see cref="RequestSpawn"/> has been called.
    /// </summary>
    public int RequestSpawnCount { get; set; }

    /// <summary>
    /// Gets all spawn requests submitted to this factory.
    /// </summary>
    public IReadOnlyList<SpawnRequest> SpawnRequests => _spawnRequests.AsReadOnly();

    /// <summary>
    /// Creates a new mock unit factory with no pre-configured spawnable units.
    /// </summary>
    public MockUnitFactory()
    {
        _spawnableUnits = new Dictionary<string, bool>(StringComparer.Ordinal);
        _spawnRequests = new List<SpawnRequest>();
    }

    /// <summary>
    /// Registers a unit definition as spawnable or not.
    /// </summary>
    /// <param name="unitDefinitionId">The unit definition ID.</param>
    /// <param name="canSpawn">Whether the unit can be spawned.</param>
    public void SetSpawnable(string unitDefinitionId, bool canSpawn)
    {
        _spawnableUnits[unitDefinitionId] = canSpawn;
    }

    public bool CanSpawn(string unitDefinitionId)
    {
        CanSpawnCount++;
        return _spawnableUnits.TryGetValue(unitDefinitionId, out var result) && result;
    }

    public void RequestSpawn(string unitDefinitionId, float x, float z)
    {
        RequestSpawnCount++;
        _spawnRequests.Add(new SpawnRequest { UnitDefinitionId = unitDefinitionId, X = x, Z = z });
    }

    /// <summary>
    /// Clears all recorded spawn requests.
    /// </summary>
    public void ClearRequests()
    {
        _spawnRequests.Clear();
    }

    /// <summary>
    /// Represents a single spawn request submitted to the factory.
    /// </summary>
    public class SpawnRequest
    {
        /// <summary>The unit definition ID requested for spawn.</summary>
        public required string UnitDefinitionId { get; init; }

        /// <summary>The world X coordinate for spawn.</summary>
        public float X { get; init; }

        /// <summary>The world Z coordinate for spawn.</summary>
        public float Z { get; init; }
    }
}
