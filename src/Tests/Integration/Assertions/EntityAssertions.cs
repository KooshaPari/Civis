#nullable enable
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;

namespace DINOForge.Tests.Integration.Assertions;

/// <summary>
/// FluentAssertions-style extension methods for asserting game state
/// through the <see cref="GameClient"/>.
/// </summary>
public static class EntityAssertions
{
    /// <summary>
    /// Asserts that the game world contains at least <paramref name="minCount"/> entities
    /// matching the specified component type.
    /// </summary>
    /// <param name="client">The connected game client.</param>
    /// <param name="componentType">The component type to query.</param>
    /// <param name="minCount">Minimum number of expected entities.</param>
    public static async Task ShouldHaveEntities(this GameClient client, string componentType, int minCount)
    {
        QueryResult result = await client.QueryEntitiesAsync(componentType: componentType).ConfigureAwait(true);
        result.Count.Should().BeGreaterThanOrEqualTo(minCount,
            $"expected at least {minCount} entities with component '{componentType}'");
    }

    /// <summary>
    /// Asserts that a stat at the given SDK path has the expected value
    /// within the specified tolerance.
    /// </summary>
    /// <param name="client">The connected game client.</param>
    /// <param name="sdkPath">The SDK path to read (e.g. "unit.stats.hp").</param>
    /// <param name="expected">The expected stat value.</param>
    /// <param name="tolerance">Acceptable tolerance for float comparison.</param>
    public static async Task ShouldHaveStat(this GameClient client, string sdkPath, float expected, float tolerance = 0.01f)
    {
        StatResult result = await client.GetStatAsync(sdkPath).ConfigureAwait(true);
        result.Value.Should().BeApproximately(expected, tolerance,
            $"expected stat '{sdkPath}' to be approximately {expected}");
    }

    /// <summary>
    /// Asserts that the game has at least <paramref name="minValue"/> of the specified resource type.
    /// </summary>
    /// <param name="client">The connected game client.</param>
    /// <param name="type">The resource type name (food, wood, stone, iron, money, souls, bones, spirit).</param>
    /// <param name="minValue">Minimum expected resource value.</param>
    public static async Task ShouldHaveResources(this GameClient client, string type, int minValue)
    {
        ResourceSnapshot resources = await client.GetResourcesAsync().ConfigureAwait(true);
        int actual = type.ToLowerInvariant() switch
        {
            "food" => resources.Food,
            "wood" => resources.Wood,
            "stone" => resources.Stone,
            "iron" => resources.Iron,
            "money" or "gold" => resources.Money,
            "souls" => resources.Souls,
            "bones" => resources.Bones,
            "spirit" => resources.Spirit,
            _ => throw new ArgumentException($"Unknown resource type: {type}", nameof(type))
        };

        actual.Should().BeGreaterThanOrEqualTo(minValue,
            $"expected at least {minValue} of resource '{type}'");
    }
}
