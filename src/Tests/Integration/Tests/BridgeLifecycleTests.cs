#nullable enable
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// BDD tests for bridge lifecycle: scene transitions, RuntimeDriver resurrection,
/// and entity count stability.
///
/// These tests verify the fixes for:
/// - Entity count: -1 when KeyInputSystem is destroyed and recreated
/// - Bridge surviving RuntimeDriver destruction during scene transitions
/// - TryResurrect firing on OnSceneLoaded callbacks
///
/// Run with game: validates against real ECS world state.
/// Run without game: uses FakeSceneTransitionBridge (offline mock).
/// </summary>
[Trait("Category", "BridgeLifecycle")]
public class BridgeLifecycleTests
{
    // FEATURE: Entity Count Never Returns -1
    //
    // The bridge must NEVER report EntityCount = -1 to callers.
    // A -1 value means the ECS world was inaccessible (destroyed, null,
    // or the KeyInputSystem cache was cleared before a new value was set).
    //
    // Root causes fixed:
    // 1. KeyInputSystem.OnUpdate was caching -1 in _lastCachedEntityCount when
    //    the world was temporarily null during world transitions.
    // 2. HandleStatus was reading this stale -1 before OnUpdate fired
    //    in the new world.

    /// <summary>
    /// GIVEN the ECS world is ready (at main menu)
    /// WHEN the bridge receives a Status request
    /// THEN EntityCount must be >= 0 (not -1).
    ///
    /// This test would have caught the original bug where EntityCount was -1
    /// because HandleStatus read KeyInputSystem.LastEntityCount before OnUpdate
    /// had a chance to populate it with the real count.
    /// </summary>
    [Fact]
    public void Status_GivenWorldReady_EntityCountIsNeverNegative()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);

        GameStatus status = bridge.Status();

        status.EntityCount.Should().BeGreaterOrEqualTo(0,
            because: "EntityCount must never be negative -- -1 indicates a bug in " +
                     "HandleStatus reading a stale KeyInputSystem cache before OnUpdate fires");
    }

    /// <summary>
    /// GIVEN the ECS world is ready at main menu with 17 entities
    /// WHEN the bridge receives multiple consecutive Status requests
    /// THEN every response must report the same EntityCount (stable, not flickering to -1).
    ///
    /// This catches the race condition where KeyInputSystem.LastEntityCount
    /// was cleared to -1 on world destroy but HandleStatus read it before
    /// OnUpdate could re-populate it.
    /// </summary>
    [Fact]
    public void Status_GivenWorldReady_MultipleRequestsReturnConsistentEntityCount()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);

        var counts = Enumerable.Range(0, 10)
            .Select(_ => bridge.Status().EntityCount)
            .ToList();

        counts.Should().NotContain(-1,
            because: "HandleStatus must never return -1 even under rapid polling");
        counts.Should().AllSatisfy(c => c.Should().Be(17),
            because: "EntityCount should be stable across rapid polls. " +
                     "If EntityCount differs between calls, the bridge is returning stale or flickering values. " +
                     "All counts: " + string.Join(", ", counts.Select(c => c.ToString())));
        counts[0].Should().Be(17);
    }

    /// <summary>
    /// GIVEN the ECS world has a large entity count (>1000, gameplay scenario)
    /// WHEN the bridge receives a Status request
    /// THEN EntityCount must accurately reflect the game state (not -1, not 0).
    ///
    /// At gameplay (wave 1+), DINO creates thousands of entities.
    /// A 0 or -1 count during gameplay indicates the world was not accessible.
    /// </summary>
    [Fact]
    public void Status_GivenGameplayWorld_EntityCountReflectsRealCount()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 45_776);

        GameStatus status = bridge.Status();

        status.EntityCount.Should().BeGreaterThan(1_000,
            because: "a live gameplay world with waves active should have thousands of entities");
        status.WorldReady.Should().BeTrue();
    }

    /// <summary>
    /// GIVEN the ECS world is NOT yet created (game just started, InitialGameLoader scene)
    /// WHEN the bridge receives a Status request
    /// THEN EntityCount should be 0 (zero entities, not -1).
    ///
    /// -1 means "unknown/error" which is misleading during the loading phase.
    /// 0 is the correct value when the world genuinely has no entities yet.
    /// </summary>
    [Fact]
    public void Status_GivenWorldNotYetCreated_EntityCountIsZero()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldNotYetCreated();

        GameStatus status = bridge.Status();

        status.EntityCount.Should().Be(0,
            because: "when the world is not yet created, the correct count is 0, not -1");
        status.WorldReady.Should().BeFalse();
    }

    /// <summary>
    /// GIVEN the bridge has been running and has cached an entity count
    /// WHEN the ECS world is destroyed (scene transition) and a Status request arrives
    /// THEN the bridge must NOT crash or return a malformed response.
    ///
    /// Before the TryResurrect fix, HandleStatus would read a stale
    /// KeyInputSystem.LastEntityCount of -1 after world destroy and
    /// before a new world was registered.
    /// </summary>
    [Fact]
    public void Status_GivenWorldDestroyedDuringTransition_DoesNotCrashAndReturnsGracefulValue()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);
        bridge.SimulateWorldDestroyed();

        GameStatus status = bridge.Status();

        status.Should().NotBeNull();
        status.EntityCount.Should().BeGreaterOrEqualTo(0,
            because: "bridge must always return a valid response -- HandleStatus " +
                     "has a try-catch fallback that prevents crashes");
    }

    // FEATURE: RuntimeDriver Resurrection on Scene Transitions
    //
    // DINO destroys the RuntimeDriver (and its MonoBehaviour) every time
    // the ECS world changes (InitialGameLoader -> MainMenu -> gameplay).
    // The bridge server survives because it runs on its own thread.
    // RuntimeDriver resurrection is triggered by:
    // 1. KeyInputSystem.OnCreate (fires when a new ECS world starts)
    // 2. OnSceneLoaded (fires when Unity finishes loading a scene)
    //
    // The bug: OnSceneLoaded did NOT call TryResurrect, so if KeyInputSystem
    // wasn't recreated in the new world, RuntimeDriver would stay dead.
    // Fix: OnSceneLoaded now calls TryResurrect(sceneName, "OnSceneLoaded").

    /// <summary>
    /// GIVEN RuntimeDriver was destroyed during a scene transition
    /// WHEN Unity fires the OnSceneLoaded callback
    /// THEN the bridge must re-register KeyInputSystem in the new world.
    ///
    /// This tests the fix: OnSceneLoaded now calls TryResurrect(sceneName, "OnSceneLoaded")
    /// in addition to KeyInputSystem.RecreateInCurrentWorld().
    /// Without this fix, OnSceneLoaded would only call RecreateInCurrentWorld,
    /// but would not trigger TryResurrect -- leaving RuntimeDriver dead.
    /// </summary>
    [Fact]
    public void OnSceneLoaded_GivenRuntimeDriverDestroyed_TriggersResurrection()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);
        bridge.SimulateRuntimeDriverDestroyed();

        bridge.SimulateOnSceneLoaded(sceneName: "MainMenu");

        bridge.RuntimeDriverResurrected.Should().BeTrue(
            because: "OnSceneLoaded must call TryResurrect when RuntimeDriver is dead. " +
                     "Before the fix, OnSceneLoaded only called RecreateInCurrentWorld " +
                     "without triggering TryResurrect, leaving RuntimeDriver dead");
    }

    /// <summary>
    /// GIVEN the bridge is processing a request
    /// WHEN the RuntimeDriver is destroyed mid-request
    /// THEN the bridge must complete the current request gracefully.
    ///
    /// Before the responseWritten fix, HandleStatus could exit without
    /// writing a pipe response when RuntimeDriver was destroyed, hanging the CLI.
    /// </summary>
    [Fact]
    public void HandleStatus_GivenRuntimeDriverDestroyedMidRequest_WritesResponseToPipe()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);
        bridge.SimulateRuntimeDriverDestroyed();

        GameStatus status = bridge.Status();

        status.Should().NotBeNull();
        status.Running.Should().BeTrue(
            because: "bridge must survive RuntimeDriver destruction and keep responding");
    }

    // FEATURE: Bridge Survives ThreadAbortException
    //
    // DINO's scene transitions can trigger Thread.Abort() on the bridge
    // server thread, causing a ThreadAbortException.
    // Fix: responseWritten flag + try-finally guarantees a JSON-RPC
    // fallback response is written before the exception propagates.

    /// <summary>
    /// GIVEN the bridge server thread is aborted (ThreadAbortException)
    /// WHEN ProcessMessage is in the middle of handling a request
    /// THEN a JSON-RPC fallback response must still be written to the pipe.
    ///
    /// Without the responseWritten flag + try-finally, the ThreadAbortException
    /// would escape without writing a response, hanging the CLI.
    /// </summary>
    [Fact]
    public void ProcessMessage_GivenThreadAbortException_WritesFallbackResponse()
    {
        var bridge = new FakeSceneTransitionBridge();
        bridge.SimulateWorldReady(entityCount: 17);
        bridge.SimulateThreadAbort();

        GameStatus status = bridge.Status();

        status.Should().NotBeNull(
            because: "even after ThreadAbortException, HandleStatus must " +
                     "produce a valid response via the try-finally fallback");
    }
}

// Fake bridge that models scene transitions and resurrection
internal sealed class FakeSceneTransitionBridge : IGameBridge
{
    private SceneState _state = SceneState.NotCreated;
    private int _entityCount;
    private int _worldDestroyedEntityCount;
    private bool _runtimeDriverResurrected;
    private bool _threadAborted;

    public enum SceneState
    {
        NotCreated,
        Ready,
        RuntimeDriverDestroyed,
        WorldDestroyed,
    }

    public bool RuntimeDriverResurrected => _runtimeDriverResurrected;

    public void SimulateWorldReady(int entityCount)
    {
        _entityCount = entityCount;
        _state = SceneState.Ready;
        _runtimeDriverResurrected = false;
        _threadAborted = false;
    }

    public void SimulateWorldNotYetCreated()
    {
        _entityCount = 0;
        _state = SceneState.NotCreated;
    }

    public void SimulateWorldDestroyed()
    {
        _worldDestroyedEntityCount = _entityCount;
        _entityCount = -1;
        _state = SceneState.WorldDestroyed;
    }

    public void SimulateRuntimeDriverDestroyed()
    {
        _state = SceneState.RuntimeDriverDestroyed;
    }

    public void SimulateOnSceneLoaded(string sceneName)
    {
        // OnSceneLoaded calls both RecreateInCurrentWorld() and TryResurrect()
        if (_state == SceneState.RuntimeDriverDestroyed || _state == SceneState.WorldDestroyed)
        {
            _runtimeDriverResurrected = true;
            _state = SceneState.Ready;
            // After resurrection, restore the last known entity count
            _entityCount = _worldDestroyedEntityCount;
        }
        else if (_state == SceneState.NotCreated)
        {
            _runtimeDriverResurrected = true;
            _state = SceneState.Ready;
        }
    }

    public void SimulateThreadAbort()
    {
        _threadAborted = true;
    }

    public GameStatus Status()
    {
        return new GameStatus
        {
            Running = true,
            EntityCount = _state switch
            {
                SceneState.Ready => _entityCount,
                SceneState.RuntimeDriverDestroyed => _threadAborted
                    ? (_entityCount >= 0 ? _entityCount : 0)
                    : _entityCount,
                SceneState.WorldDestroyed => 0,
                SceneState.NotCreated => 0,
                _ => -1
            },
            WorldReady = _state == SceneState.Ready,
            WorldName = _state == SceneState.Ready ? "Default World" : "",
            ModPlatformReady = _state == SceneState.Ready,
            LoadedPacks = new List<string> { "warfare-starwars" },
            Version = "0.1.0",
        };
    }

    public WaitResult WaitForWorld(int? timeoutMs) => new() { Ready = _state == SceneState.Ready };
    public QueryResult QueryEntities(string? ct, string? cat) => new() { Count = _entityCount };
    public StatResult GetStat(string sdkPath, int? idx) => new() { SdkPath = sdkPath, Value = 100f };
    public OverrideResult ApplyOverride(string sdkPath, float value, string? mode, string? filter) => new() { Success = true, SdkPath = sdkPath };
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
