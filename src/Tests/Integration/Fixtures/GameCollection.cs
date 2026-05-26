#nullable enable
using Xunit;

namespace DINOForge.Tests.Integration.Fixtures;

/// <summary>
/// xUnit collection definition that shares a single <see cref="GameFixture"/>
/// across all integration tests in the "Game" collection.
/// </summary>
[CollectionDefinition("Game")]
public class GameCollection : ICollectionFixture<GameFixture> { }
