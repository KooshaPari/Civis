#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration;

/// <summary>
/// Parallel game E2E tests using DINOBox container infrastructure.
///
/// This test class demonstrates:
/// - Creating N isolated game instances in parallel
/// - Launching them concurrently with <30s total cold start
/// - Each instance has a unique pipe name for bridge communication
/// - Tests execute against real game instances without interference
///
/// USAGE:
/// ```
/// dotnet test src/Tests/Integration/DINOForge.Tests.Integration.csproj --filter "Parallel"
/// ```
///
/// NOTE: This test class is skipped in CI/CD environments where DINOBox infrastructure
/// (G:\dino_boxes) is not available. Initialization is deferred to avoid timeouts.
/// </summary>
[Trait("Category", "E2E")]
[Trait("Category", "Parallel")]
[Trait("Journey", "Journey-ParallelGameTesting")]
public class ParallelGameTestsWithHarness : IAsyncLifetime
{
    private readonly GameTestContainerHarness _harness;
    private List<GameTestContainerHarness.GameContainer>? _pool;
    private bool _disposed;
    private bool _infrastructureAvailable;

    public ParallelGameTestsWithHarness()
    {
        _harness = new GameTestContainerHarness(@"G:\dino_boxes");
        // Check if DINOBox infrastructure exists; if not, skip initialization silently
        _infrastructureAvailable = Directory.Exists(@"G:\dino_boxes");
    }

    public async Task InitializeAsync()
    {
        // Guard: skip initialization if infrastructure is not available
        if (!_infrastructureAvailable)
        {
            return;
        }

        // Create pool of 4 isolated instances
        _pool = await _harness.CreatePoolAsync(4).ConfigureAwait(true);
        _pool.Should().NotBeEmpty().And.HaveCount(4);

        // Verify pool structure
        foreach (var container in _pool)
        {
            container.BoxPath.Should().NotBeNullOrEmpty();
            container.PipeName.Should().StartWith("dinoforge-game-bridge-");
            container.SaveDir.Should().NotBeNullOrEmpty();
        }
    }

    public async Task DisposeAsync()
    {
        if (!_disposed)
        {
            await _harness.DisposeAsync().ConfigureAwait(true);
            _disposed = true;
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// Launch all game instances in parallel and verify connectivity.
    /// Target: <30s cold start for 4 instances.
    /// </summary>
    [Fact(Skip = "Requires game install and MCP server")]
    public async Task ParallelLaunchTest_AllInstancesConnectWithin30Seconds()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        // Launch all instances in parallel
        var startTime = DateTime.UtcNow;

        var launchTasks = _pool!.Select(container =>
            _harness.LaunchInstanceAsync(container, timeoutSeconds: 30)
        ).ToList();

        var results = await Task.WhenAll(launchTasks).ConfigureAwait(true);
        var elapsed = DateTime.UtcNow - startTime;

        // All launches should succeed
        results.Should().AllSatisfy(r => r.Should().BeTrue("All instances should launch and connect within timeout"));

        // Total time should be <30s
        elapsed.TotalSeconds.Should().BeLessThan(30, "All 4 instances should launch in parallel within 30s");
    }

    /// <summary>
    /// Verify pipe name isolation - each instance has a unique pipe.
    /// </summary>
    [Fact(Skip = "Requires DINOBox infrastructure at G:\\dino_boxes")]
    public void PipeNameIsolationTest_UniqueNamesPerInstance()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        var pipeNames = _pool!.Select(c => c.PipeName).ToList();

        // All should be unique
        pipeNames.Distinct().Should().HaveCount(pipeNames.Count,
            "Each container should have a unique pipe name");

        // All should follow pattern
        pipeNames.Should().AllSatisfy(pipe =>
            pipe.Should().Match("dinoforge-game-bridge-*"));
    }

    /// <summary>
    /// Verify symlink structure - no asset duplication.
    /// </summary>
    [Fact(Skip = "Requires DINOBox infrastructure at G:\\dino_boxes")]
    public void SymlinkValidationTest_NoAssetDuplication()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        foreach (var container in _pool!)
        {
            var dataDir = System.IO.Path.Combine(container.BoxPath, "Diplomacy is Not an Option_Data");
            var managedLink = System.IO.Path.Combine(dataDir, "Managed");

            // Managed directory should exist and be readable
            System.IO.Directory.Exists(dataDir).Should().BeTrue(
                $"Box {container.Index} should have _Data directory");

            System.IO.Directory.Exists(managedLink).Should().BeTrue(
                $"Box {container.Index} should have Managed directory/symlink");
        }
    }

    /// <summary>
    /// Verify box structure and configuration files.
    /// </summary>
    [Fact(Skip = "Requires DINOBox infrastructure at G:\\dino_boxes")]
    public void BoxStructureTest_AllRequiredFilesPresent()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        foreach (var container in _pool!)
        {
            // Game exe
            System.IO.File.Exists(container.ExePath).Should().BeTrue(
                $"Box {container.Index} should have game executable");

            // BepInEx structure
            System.IO.Directory.Exists(container.BepInExDir).Should().BeTrue(
                $"Box {container.Index} should have BepInEx directory");

            var configFile = System.IO.Path.Combine(
                container.BepInExDir, "config", "BepInEx.cfg");
            System.IO.File.Exists(configFile).Should().BeTrue(
                $"Box {container.Index} should have BepInEx.cfg");

            // Verify pipe name in config
            var configContent = System.IO.File.ReadAllText(configFile, System.Text.Encoding.UTF8);
            configContent.Should().Contain(container.PipeName,
                $"Box {container.Index} BepInEx.cfg should contain its pipe name");

            // Save directory
            System.IO.Directory.Exists(container.SaveDir).Should().BeTrue(
                $"Box {container.Index} should have save directory");
        }
    }

    /// <summary>
    /// Demonstrate ECS world readiness polling (not sleep-based).
    /// </summary>
    [Fact(Skip = "Requires game instance running")]
    public async Task WorldReadinessPollTest_DetectsEntityCountWithoutSleep()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        // Launch first instance
        var container = _pool![0];
        var launched = await _harness.LaunchInstanceAsync(container, timeoutSeconds: 30).ConfigureAwait(true);
        launched.Should().BeTrue("Instance should launch successfully");

        // Poll for world readiness (should detect >0 entities, not just sleep)
        var ready = await _harness.WaitForWorldAsync(container, timeoutSeconds: 60).ConfigureAwait(true);
        ready.Should().BeTrue("ECS world should become ready");

        // Get client and verify entity count
        var client = _harness.GetClient(container);
        client.Should().NotBeNull("Client should be connected");

        var status = await client!.StatusAsync().ConfigureAwait(true);
        status.Should().NotBeNull();
        status!.EntityCount.Should().BeGreaterThan(0, "World should have entities");
    }

    /// <summary>
    /// Concurrent box operations test - create and manage multiple boxes simultaneously.
    /// </summary>
    [Fact(Skip = "Requires DINOBox infrastructure at G:\\dino_boxes")]
    public async Task ConcurrentOperationsTest_MultipleContainersWorkIndependently()
    {
        if (!_infrastructureAvailable)
        {
            return; // Skip silently if DINOBox infrastructure not available
        }

        _pool.Should().NotBeNull();

        // Simulate concurrent queries on different containers
        var tasks = _pool!.Select(async container =>
        {
            // Each "operation" is independent
            var boxValid = System.IO.Directory.Exists(container.BoxPath);
            var exeValid = System.IO.File.Exists(container.ExePath);
            await Task.Delay(10).ConfigureAwait(true); // Simulate async work
            return boxValid && exeValid;
        }).ToList();

        var results = await Task.WhenAll(tasks).ConfigureAwait(true);
        results.Should().AllSatisfy(r => r.Should().BeTrue("All concurrent operations should succeed"));
    }
}
