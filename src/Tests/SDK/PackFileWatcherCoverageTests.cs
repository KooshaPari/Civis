#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK;

[Trait("Category", "SDK")]
public sealed class PackFileWatcherCoverageTests : IDisposable
{
    private readonly List<string> _tempDirectories = new();

    [Fact]
    public void Constructor_WithNullPacksDirectory_Throws()
    {
        Action action = () => new PackFileWatcher(
            null!,
            CreateContentLoader(),
            new RegistryManager());

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("packsDirectory");
    }

    [Fact]
    public void Constructor_WithNullContentLoader_Throws()
    {
        Action action = () => new PackFileWatcher(
            CreateDirectory(),
            null!,
            new RegistryManager());

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("contentLoader");
    }

    [Fact]
    public void Constructor_WithNullRegistryManager_Throws()
    {
        Action action = () => new PackFileWatcher(
            CreateDirectory(),
            CreateContentLoader(),
            null!);

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("registryManager");
    }

    [Fact]
    public void Constructor_WithZeroDebounce_InitializesAndLeavesWatcherStopped()
    {
        string packsDirectory = CreateDirectory();
        List<string> logs = new();

        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager(),
            log: logs.Add,
            debounceMs: 0);

        watcher.IsWatching.Should().BeFalse();
        logs.Should().BeEmpty();
    }

    [Fact]
    public void Start_WithMissingDirectory_LogsAndLeavesWatcherStopped()
    {
        string packsDirectory = Path.Combine(Path.GetTempPath(), "dinoforge_packwatch_missing_" + Guid.NewGuid().ToString("N"));
        List<string> logs = new();
        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager(),
            log: logs.Add);

        watcher.Start();

        watcher.IsWatching.Should().BeFalse();
        logs.Should().ContainSingle();
        logs[0].ToLowerInvariant().Should().Contain("does not exist");
    }

    [Fact]
    public void Start_Stop_And_StartAgain_AreIdempotent()
    {
        string packsDirectory = CreateDirectory();
        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager());

        watcher.Start();
        watcher.IsWatching.Should().BeTrue();

        watcher.Start();
        watcher.IsWatching.Should().BeTrue();

        watcher.Stop();
        watcher.IsWatching.Should().BeFalse();

        watcher.Stop();
        watcher.IsWatching.Should().BeFalse();
    }

    [Fact]
    public void Stop_WithoutStart_DoesNotThrow()
    {
        string packsDirectory = CreateDirectory();
        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager());

        watcher.Stop();

        watcher.IsWatching.Should().BeFalse();
    }

    [Fact]
    public void ReloadAll_OnEmptyPacksRoot_ReturnsSuccessWithRootPath()
    {
        string packsDirectory = CreateDirectory();
        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager());

        HotReloadResult result = watcher.ReloadAll();

        result.IsSuccess.Should().BeTrue();
        result.ChangedFiles.Should().ContainSingle().Which.Should().Be(packsDirectory);
        result.UpdatedEntries.Should().BeEmpty();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void EventSubscriptions_CanBeAddedAndRemoved()
    {
        string packsDirectory = CreateDirectory();
        using PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager());

        EventHandler<PackContentChangedEventArgs> contentChanged = (_, _) => { };
        EventHandler<HotReloadResult> reloaded = (_, _) => { };
        EventHandler<HotReloadResult> failed = (_, _) => { };

        watcher.OnPackContentChanged += contentChanged;
        watcher.OnPackReloaded += reloaded;
        watcher.OnPackReloadFailed += failed;

        watcher.OnPackContentChanged -= contentChanged;
        watcher.OnPackReloaded -= reloaded;
        watcher.OnPackReloadFailed -= failed;
    }

    [Fact]
    public void PackContentChangedEventArgs_StoresPathAndTimestamp()
    {
        string filePath = Path.Combine(CreateDirectory(), "pack.yaml");
        DateTimeOffset before = DateTimeOffset.UtcNow;

        PackContentChangedEventArgs args = new(filePath);

        args.FilePath.Should().Be(filePath);
        args.Timestamp.Should().BeOnOrAfter(before);
        args.Timestamp.Should().BeOnOrBefore(DateTimeOffset.UtcNow);
    }

    [Fact]
    public void Dispose_IsIdempotentAndStopsWatching()
    {
        string packsDirectory = CreateDirectory();
        PackFileWatcher watcher = new(
            packsDirectory,
            CreateContentLoader(),
            new RegistryManager());

        watcher.Start();
        watcher.IsWatching.Should().BeTrue();

        watcher.Dispose();
        watcher.IsWatching.Should().BeFalse();

        watcher.Dispose();
        watcher.IsWatching.Should().BeFalse();
    }

    public void Dispose()
    {
        foreach (string directory in _tempDirectories)
        {
            try
            {
                if (Directory.Exists(directory))
                {
                    Directory.Delete(directory, recursive: true);
                }
            }
            catch
            {
                // Best-effort cleanup for temp test artifacts.
            }
        }

        _tempDirectories.Clear();
    }

    private static ContentLoader CreateContentLoader()
    {
        return new ContentLoader(new RegistryManager(), schemaValidator: null, log: null);
    }

    private string CreateDirectory()
    {
        string directory = Path.Combine(Path.GetTempPath(), "dinoforge_packwatch_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        _tempDirectories.Add(directory);
        return directory;
    }
}
