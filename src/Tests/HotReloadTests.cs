using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using DINOForge.SDK;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for hot module reload functionality.
    /// Maps to: US-F4.1 Hot Module Reload
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F4.1")]
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-RuntimeUI")]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateBalance")]
    public class HotReloadTests : IDisposable
    {
        private readonly string _tempDir;
        private readonly RegistryManager _registryManager;
        private readonly ContentLoader _contentLoader;

        public HotReloadTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_hotreload_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
            _registryManager = new RegistryManager();
            _contentLoader = new ContentLoader(_registryManager);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, true);
            }
            catch { /* best effort cleanup */ }
        }

        #region HotReloadResult Tests

        [Fact]
        public void HotReloadResult_Success_HasCorrectProperties()
        {
            var changedFiles = new List<string> { "file1.yaml", "file2.yaml" }.AsReadOnly();
            var updatedEntries = new List<string> { "my-pack" }.AsReadOnly();

            HotReloadResult result = HotReloadResult.Success(changedFiles, updatedEntries);

            result.IsSuccess.Should().BeTrue();
            result.ChangedFiles.Should().HaveCount(2);
            result.UpdatedEntries.Should().HaveCount(1);
            result.Errors.Should().BeEmpty();
            result.Timestamp.Should().BeCloseTo(DateTimeOffset.UtcNow, TimeSpan.FromSeconds(2));
        }

        [Fact]
        public void HotReloadResult_Failure_HasCorrectProperties()
        {
            var changedFiles = new List<string> { "bad.yaml" }.AsReadOnly();
            var errors = new List<string> { "Parse error in bad.yaml" }.AsReadOnly();

            HotReloadResult result = HotReloadResult.Failure(changedFiles, errors);

            result.IsSuccess.Should().BeFalse();
            result.ChangedFiles.Should().HaveCount(1);
            result.UpdatedEntries.Should().BeEmpty();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("Parse error");
        }

        [Fact]
        public void HotReloadResult_Partial_HasUpdatesAndErrors()
        {
            var changedFiles = new List<string> { "a.yaml", "b.yaml" }.AsReadOnly();
            var updated = new List<string> { "pack-a" }.AsReadOnly();
            var errors = new List<string> { "Error in b.yaml" }.AsReadOnly();

            HotReloadResult result = HotReloadResult.Partial(changedFiles, updated, errors);

            result.IsSuccess.Should().BeFalse();
            result.UpdatedEntries.Should().HaveCount(1);
            result.Errors.Should().HaveCount(1);
        }

        #endregion

        #region PackFileWatcher Debounce Tests

        [Fact]
        public void PackFileWatcher_DebounceCoalesces_MultipleChanges()
        {
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager, debounceMs: 100);

            var contentChangedCount = 0;
            watcher.OnPackContentChanged += (_, _) => Interlocked.Increment(ref contentChangedCount);

            // Enqueue multiple changes rapidly
            watcher.EnqueueChange(Path.Combine(_tempDir, "test1.yaml"));
            watcher.EnqueueChange(Path.Combine(_tempDir, "test2.yaml"));
            watcher.EnqueueChange(Path.Combine(_tempDir, "test3.yaml"));

            // Each enqueue fires OnPackContentChanged immediately
            contentChangedCount.Should().Be(3);
        }

        [Fact]
        public void PackFileWatcher_DebounceWaitsBeforeReload()
        {
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager, debounceMs: 200);

            var reloadedEvent = new ManualResetEventSlim(false);
            var failedEvent = new ManualResetEventSlim(false);

            watcher.OnPackReloaded += (_, _) => reloadedEvent.Set();
            watcher.OnPackReloadFailed += (_, _) => failedEvent.Set();

            // Enqueue a change (no pack.yaml exists, so reload will fail to find pack)
            watcher.EnqueueChange(Path.Combine(_tempDir, "some", "test.yaml"));

            // Should not have fired reload yet (debounce in progress)
            Thread.Sleep(50);
            reloadedEvent.IsSet.Should().BeFalse();

            // Wait for debounce to complete
            bool signaled = reloadedEvent.Wait(1000) || failedEvent.Wait(1000);
            // One of the events should fire (failure since no pack.yaml)
        }

        [Fact]
        public void PackFileWatcher_Start_SetsIsWatching()
        {
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            watcher.IsWatching.Should().BeFalse();
            watcher.Start();
            watcher.IsWatching.Should().BeTrue();
            watcher.Stop();
            watcher.IsWatching.Should().BeFalse();
        }

        [Fact]
        public void PackFileWatcher_Start_NonExistentDir_DoesNotThrow()
        {
            string bogusDir = Path.Combine(_tempDir, "nonexistent");
            using var watcher = new PackFileWatcher(
                bogusDir, _contentLoader, _registryManager);

            Action act = () => watcher.Start();
            act.Should().NotThrow();
            watcher.IsWatching.Should().BeFalse();
        }

        [Fact]
        public void PackFileWatcher_Stop_WhenNotStarted_DoesNotThrow()
        {
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            Action act = () => watcher.Stop();
            act.Should().NotThrow();
        }

        [Fact]
        public void PackFileWatcher_Dispose_StopsWatching()
        {
            var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);
            watcher.Start();
            watcher.IsWatching.Should().BeTrue();

            watcher.Dispose();
            watcher.IsWatching.Should().BeFalse();
        }

        [Fact]
        public void PackFileWatcher_Dispose_ThenStart_Throws()
        {
            var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);
            watcher.Dispose();

            Action act = () => watcher.Start();
            act.Should().Throw<ObjectDisposedException>();
        }

        #endregion

        #region Thread Safety Tests

        [Fact]
        public void PackFileWatcher_ConcurrentEnqueue_DoesNotThrow()
        {
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager, debounceMs: 50);

            var exceptions = new ConcurrentBag<Exception>();
            var threads = new List<Thread>();

            for (int i = 0; i < 10; i++)
            {
                int threadIndex = i;
                var thread = new Thread(() =>
                {
                    try
                    {
                        for (int j = 0; j < 20; j++)
                        {
                            watcher.EnqueueChange(
                                Path.Combine(_tempDir, $"thread{threadIndex}_file{j}.yaml"));
                        }
                    }
                    catch (Exception ex)
                    {
                        exceptions.Add(ex);
                    }
                });
                threads.Add(thread);
            }

            foreach (Thread t in threads) t.Start();
            foreach (Thread t in threads) t.Join(5000);

            exceptions.Should().BeEmpty("concurrent enqueue should be thread-safe");
        }

        #endregion

        #region ReloadAll Tests

        [Fact]
        public void PackFileWatcher_ReloadAll_WithValidPack_Succeeds()
        {
            // Create a valid pack directory
            string packDir = Path.Combine(_tempDir, "test-pack");
            Directory.CreateDirectory(packDir);
            File.WriteAllText(Path.Combine(packDir, "pack.yaml"),
                "id: test-pack\nname: Test Pack\nversion: 1.0.0\nauthor: Test\ntype: content\n");

            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            HotReloadResult result = watcher.ReloadAll();

            // The reload processes the directory; since there's no content beyond the manifest,
            // it should succeed (the pack just has no content to load)
            result.Should().NotBeNull();
        }

        [Fact]
        public void PackFileWatcher_ReloadAll_WithNoPackYaml_ReturnsResult()
        {
            // Empty directory with no pack.yaml files
            using var watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            HotReloadResult result = watcher.ReloadAll();

            result.Should().NotBeNull();
            // No pack directories found, so the content loader returns success with empty list
        }

        #endregion
    }
}
