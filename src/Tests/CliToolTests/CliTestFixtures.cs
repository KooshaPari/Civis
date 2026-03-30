#nullable enable
using DINOForge.Bridge.Protocol;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Shared test fixtures for CLI command tests (GameClient is sealed, so we test protocol types).
/// </summary>
public static class CliTestFixtures
{
    /// <summary>
    /// Creates a GameStatus with default values.
    /// </summary>
    public static GameStatus CreateGameStatus(
        bool running = true,
        bool worldReady = true,
        string worldName = "TestWorld",
        int entityCount = 500,
        bool modPlatformReady = true,
        string version = "0.15.0",
        string[]? loadedPacks = null)
    {
        return new GameStatus
        {
            Running = running,
            WorldReady = worldReady,
            WorldName = worldName,
            EntityCount = entityCount,
            ModPlatformReady = modPlatformReady,
            Version = version,
            LoadedPacks = new List<string>(loadedPacks ?? ["example-balance"])
        };
    }

    /// <summary>
    /// Creates a QueryResult with custom entities for testing.
    /// </summary>
    public static QueryResult CreateQueryResult(
        int count = 5,
        List<EntityInfo>? entities = null)
    {
        return new QueryResult
        {
            Count = count,
            Entities = entities ?? Enumerable.Range(0, count)
                .Select(i => new EntityInfo
                {
                    Index = i,
                    Components = ["Health", "Transform", "Faction"]
                })
                .ToList()
        };
    }

    /// <summary>
    /// Creates a QueryResult with a specified number of sample entities.
    /// </summary>
    public static QueryResult CreateQueryResultWithEntities(int entityCount = 5)
    {
        var entities = Enumerable.Range(0, entityCount)
            .Select(i => new EntityInfo
            {
                Index = i,
                Components = [typeof(object).Name, "Health", "Transform"]
            })
            .ToList();

        return new QueryResult
        {
            Count = entityCount,
            Entities = entities
        };
    }

    /// <summary>
    /// Creates an OverrideResult with custom values.
    /// </summary>
    public static OverrideResult CreateOverrideResult(
        bool success = true,
        string sdkPath = "unit.stats.hp",
        int modifiedCount = 42,
        string message = "Override applied")
    {
        return new OverrideResult
        {
            Success = success,
            SdkPath = sdkPath,
            ModifiedCount = modifiedCount,
            Message = message
        };
    }

    /// <summary>
    /// Creates a ResourceSnapshot with custom values.
    /// </summary>
    public static ResourceSnapshot CreateResourceSnapshot(
        int food = 100,
        int wood = 200,
        int stone = 150,
        int iron = 75,
        int money = 50,
        int souls = 0,
        int bones = 0,
        int spirit = 0)
    {
        return new ResourceSnapshot
        {
            Food = food,
            Wood = wood,
            Stone = stone,
            Iron = iron,
            Money = money,
            Souls = souls,
            Bones = bones,
            Spirit = spirit
        };
    }

    /// <summary>
    /// Creates a ReloadResult indicating success.
    /// </summary>
    public static ReloadResult CreateSuccessfulReloadResult(
        string[]? loadedPacks = null)
    {
        return new ReloadResult
        {
            Success = true,
            LoadedPacks = new List<string>(loadedPacks ?? ["example-balance"]),
            Errors = new List<string>()
        };
    }

    /// <summary>
    /// Creates a ReloadResult indicating failure.
    /// </summary>
    public static ReloadResult CreateFailedReloadResult(
        string[]? errors = null)
    {
        return new ReloadResult
        {
            Success = false,
            LoadedPacks = new List<string>(),
            Errors = new List<string>(errors ?? ["Pack not found"])
        };
    }
}
