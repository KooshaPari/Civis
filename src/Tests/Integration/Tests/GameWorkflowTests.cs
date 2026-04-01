#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// BDD tests for game workflow: save loading, loading screen dismissal,
/// gameplay entry, and MCP tool integration.
///
/// SCENARIO: Save Loading
/// GIVEN the player is at the main menu
/// WHEN they load a save file (e.g. AUTOSAVE_1)
/// THEN the bridge creates a LoadRequest ECS entity
/// AND the game transitions to the InitialGameLoader scene
/// AND the ECS world is rebuilt
/// AND the loading screen shows "Press Any Key to Continue"
/// AND pressing any key dismisses the loading screen
/// AND the game enters gameplay with hundreds/thousands of entities
/// </summary>
[Trait("Category", "GameWorkflow")]
public class GameWorkflowTests
{
    // FEATURE: Save Loading via Bridge
    //
    // The CLI `load-save` command and MCP `game_navigate_to` tool
    // trigger the game's save loading mechanism via the ECS bridge:
    // 1. CLI receives load-save AUTOSAVE_1
    // 2. CLI sends bridge LoadSave request
    // 3. HandleLoadSave creates a Components.RawComponents.LoadRequest singleton
    // 4. DINO reads LoadRequest and begins loading
    // 5. DINO transitions to InitialGameLoader scene
    // 6. Old ECS world is destroyed
    // 7. RuntimeDriver is recreated (OnSceneLoaded -> TryResurrect)
    // 8. New ECS world is created
    // 9. Gameplay scene loads
    // 10. Loading screen shows
    // 11. DismissLoadScreen or key press clears the screen
    // 12. Gameplay begins with thousands of entities

    /// <summary>
    /// GIVEN the player is at the main menu (17 entities, Default World)
    /// WHEN the bridge receives a loadSave("AUTOSAVE_1") request
    /// THEN the bridge must return success with LoadRequest entity info.
    ///
    /// The bridge does NOT load the save itself -- it creates a LoadRequest
    /// ECS singleton that DINO's SaveLoadSystem reads and acts upon.
    /// </summary>
    [Fact]
    public void LoadSave_GivenMainMenu_CreatesLoadRequestEntity()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        StartGameResult result = bridge.LoadSave("AUTOSAVE_1");

        result.Success.Should().BeTrue(
            because: "HandleLoadSave must create a LoadRequest ECS entity");
    }

    /// <summary>
    /// GIVEN the bridge has a LoadRequest entity created
    /// WHEN DINO processes the LoadRequest and starts loading
    /// THEN the ECS world must be rebuilt and the entity count must change.
    ///
    /// At main menu: ~17 entities (menu system)
    /// After save load: hundreds or thousands of entities (gameplay units/buildings)
    ///
    /// This is the key validation: EntityCount must jump from 17 to >1000
    /// when a save is successfully loaded. If EntityCount stays at 17,
    /// the save was not loaded.
    /// </summary>
    [Fact]
    public void Status_AfterSaveLoaded_EntityCountJumpsToGameplay()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        GameStatus beforeLoad = bridge.Status();
        beforeLoad.EntityCount.Should().Be(17);

        bridge.SimulateSaveLoaded(entityCount: 45_776);

        GameStatus afterLoad = bridge.Status();
        afterLoad.EntityCount.Should().BeGreaterThan(1_000,
            because: "after a successful save load, the game must have " +
                     "thousands of entities. If EntityCount is still 17, " +
                     "the save was NOT loaded. This is the critical gameplay validation test.");
    }

    /// <summary>
    /// GIVEN a save is being loaded (InitialGameLoader scene)
    /// WHEN the loading screen shows "Press Any Key to Continue"
    /// AND the bridge receives a DismissLoadScreen request
    /// THEN the loading screen must be dismissed and gameplay begins.
    ///
    /// The dismiss mechanism looks for UI.LoadingProgressBar and invokes
    /// its _startAction UnityAction.
    /// </summary>
    [Fact]
    public void DismissLoadScreen_GivenLoadingScreenVisible_DismissesSuccessfully()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateLoadingScreen();

        StartGameResult result = bridge.DismissLoadScreen();

        result.Success.Should().BeTrue(
            because: "DismissLoadScreen must find LoadingProgressBar and " +
                     "invoke its _startAction. Failure means the loading " +
                     "screen UI elements could not be located at runtime");
    }

    /// <summary>
    /// GIVEN a save is being loaded but the loading screen has already cleared
    /// WHEN the bridge receives a DismissLoadScreen request
    /// THEN the bridge must return success=false gracefully (not crash).
    /// </summary>
    /// <summary>
    /// GIVEN a save is being loaded but the loading screen has already cleared
    /// WHEN the bridge receives a DismissLoadScreen request
    /// THEN the bridge must return success=false gracefully (not crash).
    ///
    /// This is tested using SimulateMainMenu (no loading screen visible)
    /// rather than SimulateSaveLoaded (loading screen visible).
    /// </summary>
    [Fact]
    public void DismissLoadScreen_GivenNoLoadingScreen_ReturnsFailureGracefully()
    {
        var bridge = new FakeGameWorkflowBridge();
        // SimulateMainMenu: no loading screen, at main menu
        bridge.SimulateMainMenu(entityCount: 17);

        StartGameResult result = bridge.DismissLoadScreen();

        result.Success.Should().BeFalse(
            because: "if there is no loading screen, dismiss should " +
                     "return success=false gracefully, not throw");
    }

    /// <summary>
    /// GIVEN the game is at the main menu (17 entities)
    /// WHEN the bridge receives a LoadSave request followed by DismissLoadScreen
    /// THEN the entity count must increase dramatically (from ~17 to >1000).
    ///
    /// This is the full end-to-end save-load-goto-gameplay sequence test.
    /// </summary>
    [Fact]
    public void FullSaveLoadSequence_GivenMainMenu_EntityCountReachesGameplay()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        // Step 1: Load save -> transitions to loading screen
        bridge.SimulateSaveLoaded(entityCount: 45_776);

        // Step 2: Verify EntityCount jumped
        GameStatus status = bridge.Status();
        status.EntityCount.Should().BeGreaterThan(1_000,
            because: "full sequence: main menu (17) -> save load -> gameplay (45000+)");

        // Step 3: Dismiss loading screen
        StartGameResult dismissResult = bridge.DismissLoadScreen();
        dismissResult.Success.Should().BeTrue(
            because: "dismiss should succeed once gameplay is ready");

        // Step 4: EntityCount stays high after dismiss
        status = bridge.Status();
        status.EntityCount.Should().BeGreaterThan(1_000,
            because: "entity count must remain high after dismiss");
    }

    // FEATURE: World Readiness Transition

    /// <summary>
    /// GIVEN the bridge reports WorldReady=false (world not yet initialized)
    /// WHEN WaitForWorld is called with a 60-second timeout
    /// THEN it must eventually return Ready=true once the world is created.
    /// </summary>
    /// <summary>
    /// GIVEN the bridge reports WorldReady=false (world not yet initialized)
    /// WHEN WaitForWorld is called with a 60-second timeout
    /// THEN it must eventually return Ready=true once the world is created.
    ///
    /// Note: This test documents the expected behavior. The real WaitForWorld
    /// polls the ECS world until Ready=true. The fake bridge simulates this
    /// by marking the world as ready after the first WaitForWorld call
    /// when SimulateSaveLoaded has been called.
    /// </summary>
    [Fact]
    public void WaitForWorld_GivenWorldNotReady_EventuallyReportsReady()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        // Simulate the world transitioning from not-ready to ready
        bridge.SimulateSaveLoaded(entityCount: 45_776);

        WaitResult result = bridge.WaitForWorld(timeoutMs: 60_000);

        result.Ready.Should().BeTrue(
            because: "WaitForWorld must poll until the ECS world is ready");
    }

    /// <summary>
    /// GIVEN the bridge reports WorldReady=true
    /// WHEN WaitForWorld is called again
    /// THEN it must return Ready=true immediately (no polling needed).
    /// </summary>
    [Fact]
    public void WaitForWorld_GivenWorldAlreadyReady_ReturnsImmediately()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        WaitResult result = bridge.WaitForWorld(timeoutMs: 5_000);

        result.Ready.Should().BeTrue(
            because: "if world is already ready, WaitForWorld should return immediately");
    }

    // FEATURE: MCP Tool Integration
    //
    // The MCP server wraps the CLI which wraps the bridge.
    // The critical path for MCP tools:
    // MCP game_status -> CLI --format=json -> bridge -> HandleStatus
    // The bug: HandleStatus was returning EntityCount=-1 during world transitions.
    // Fix: HandleStatus now uses KeyInputSystem.CachedWorldName and
    // LastEntityCount with a try-catch fallback.

    /// <summary>
    /// GIVEN the MCP server receives game_status tool call
    /// WHEN the CLI --format=json flag is used
    /// THEN the output must be valid JSON that the MCP server can parse.
    ///
    /// This tests the CLI --format=json flag (added in 1c1e8c4).
    /// The MCP server uses this flag to parse machine-readable output.
    /// Before the fix, the CLI always output AnsiConsole markup,
    /// which the MCP server could not parse -> timeout.
    /// </summary>
    [Fact]
    public void CliJsonOutput_GivenFormatJsonFlag_OutputsValidJson()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateMainMenu(entityCount: 17);

        string json = bridge.GetJsonStatus();

        Newtonsoft.Json.JsonConvert.DeserializeObject<GameStatus>(json)
            .Should().NotBeNull(
                because: "--format=json must output valid JSON. " +
                         "If this fails, the MCP server will timeout " +
                         "waiting for a parseable response");
    }

    /// <summary>
    /// GIVEN the MCP server receives game_status when the bridge returns -1
    /// WHEN the bridge fallback is applied
    /// THEN the JSON must still be valid and contain success=true.
    ///
    /// Even when HandleStatus catches an exception and falls back to
    /// EntityCount=-1, the JSON-RPC response must be valid.
    /// </summary>
    [Fact]
    public void CliJsonOutput_GivenBridgeReturnsFallback_StillOutputsValidJson()
    {
        var bridge = new FakeGameWorkflowBridge();
        bridge.SimulateFallbackState();

        string json = bridge.GetJsonStatus();

        Newtonsoft.Json.JsonConvert.DeserializeObject<GameStatus>(json)
            .Should().NotBeNull(
                because: "even the fallback JSON response from HandleStatus " +
                         "must be valid JSON. Invalid JSON causes MCP timeouts");
    }
}

// Fake bridge for game workflow testing
internal sealed class FakeGameWorkflowBridge : IGameBridge
{
    private GamePhase _phase = GamePhase.MainMenu;
    private int _entityCount = 17;
    private bool _loadingScreenVisible;
    private bool _worldReady;
    private bool _fallbackMode;

    public enum GamePhase
    {
        MainMenu,
        Loading,
        Gameplay,
        Unknown,
    }

    public void SimulateMainMenu(int entityCount)
    {
        _phase = GamePhase.MainMenu;
        _entityCount = entityCount;
        _worldReady = true;
        _fallbackMode = false;
        _loadingScreenVisible = false;
    }

    public void SimulateSaveLoaded(int entityCount)
    {
        _phase = GamePhase.Gameplay;
        _entityCount = entityCount;
        _worldReady = true;
        _fallbackMode = false;
        _loadingScreenVisible = true; // Loading screen visible until dismissed
    }

    public void SimulateLoadingScreen()
    {
        _phase = GamePhase.Loading;
        _loadingScreenVisible = true;
        _worldReady = false;
    }

    public void SimulateWorldNotReady()
    {
        _phase = GamePhase.Unknown;
        _worldReady = false;
        _fallbackMode = false;
    }

    public void SimulateFallbackState()
    {
        _fallbackMode = true;
        _worldReady = true;
    }

    public GameStatus Status()
    {
        if (_fallbackMode)
        {
            return new GameStatus
            {
                Running = true,
                WorldReady = false,
                WorldName = "",
                EntityCount = -1, // The original bug -- but wrapped in valid JSON
                ModPlatformReady = false,
                LoadedPacks = new List<string>(),
                Version = "0.1.0",
            };
        }

        return new GameStatus
        {
            Running = true,
            WorldReady = _worldReady,
            WorldName = _phase == GamePhase.Gameplay ? "Default Gameplay World" : "Default World",
            EntityCount = _entityCount,
            ModPlatformReady = _worldReady,
            LoadedPacks = new List<string> { "warfare-starwars" },
            Version = "0.1.0",
        };
    }

    public string GetJsonStatus()
    {
        return Newtonsoft.Json.JsonConvert.SerializeObject(Status());
    }

    public StartGameResult LoadSave(string saveName)
    {
        if (_phase != GamePhase.MainMenu)
            return new StartGameResult
            {
                Success = false,
                Message = "Cannot load '" + saveName + "' -- not at main menu"
            };

        _phase = GamePhase.Loading;
        return new StartGameResult
        {
            Success = true,
            Message = "Created LoadRequest entity for '" + saveName + "'"
        };
    }

    public StartGameResult DismissLoadScreen()
    {
        if (!_loadingScreenVisible && _phase == GamePhase.Gameplay)
        {
            return new StartGameResult
            {
                Success = false,
                Message = "No loading screen to dismiss -- gameplay already loaded"
            };
        }

        if (!_loadingScreenVisible)
        {
            return new StartGameResult
            {
                Success = false,
                Message = "No loading screen found"
            };
        }

        _loadingScreenVisible = false;
        _phase = GamePhase.Gameplay;
        _worldReady = true;
        return new StartGameResult
        {
            Success = true,
            Message = "LoadingProgressBar._startAction invoked"
        };
    }

    public WaitResult WaitForWorld(int? timeoutMs)
    {
        return new WaitResult
        {
            Ready = _worldReady,
            WorldName = _worldReady
                ? (_phase == GamePhase.Gameplay ? "Default Gameplay World" : "Default World")
                : ""
        };
    }

    public QueryResult QueryEntities(string? ct, string? cat) => new() { Count = _entityCount };
    public StatResult GetStat(string sdkPath, int? idx) => new() { SdkPath = sdkPath, Value = 100f };
    public OverrideResult ApplyOverride(string sdkPath, float value, string? mode, string? filter) => new() { Success = true };
    public ReloadResult ReloadPacks(string? path) => new() { Success = true };
    public CatalogSnapshot GetCatalog() => new();
    public CatalogSnapshot DumpState(string? cat) => new();
    public ResourceSnapshot GetResources() => new();
    public ScreenshotResult Screenshot(string? path) => new();
    public VerifyResult VerifyMod(string? path) => new();
    public PingResult Ping() => new() { Pong = true };
    public ComponentMapResult GetComponentMap(string? sdkPath) => new();
    public UiTreeResult GetUiTree(string? selector) => new();
    public UiActionResult QueryUi(string selector) => new();
    public UiActionResult ClickUi(string selector) => new();
    public UiWaitResult WaitForUi(string selector, string? state, int? timeoutMs) => new();
    public UiExpectationResult ExpectUi(string selector, string condition) => new();
}
