using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Registry;
using DINOForge.Tests.Support;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="PackFileWatcher"/> covering lifecycle, debounce, and event firing.
    /// </summary>
    [Collection(Collections.FileSystemWatcher)]
    public class PackFileWatcherTests : IDisposable
    {
        private readonly string _tempDir;
        private readonly RegistryManager _registryManager;
        private readonly ContentLoader _contentLoader;

        public PackFileWatcherTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_watcher_tests_" + Guid.NewGuid().ToString("N"));
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
            catch { /* best-effort cleanup */ }
        }

        [Fact]
        public void WatcherCreated_WithValidDirectory_DoesNotThrow()
        {
            // Act
            Action act = () =>
            {
                using PackFileWatcher watcher = new PackFileWatcher(
                    _tempDir, _contentLoader, _registryManager);
            };

            // Assert
            act.Should().NotThrow();
        }

        [Fact]
        public void WatcherStarted_ThenStopped_DoesNotThrow()
        {
            // Arrange
            using PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            // Act
            Action act = () =>
            {
                watcher.Start();
                watcher.Stop();
            };

            // Assert
            act.Should().NotThrow();
            watcher.IsWatching.Should().BeFalse();
        }

        [Fact]
        public void WatcherDisposed_DoesNotThrow()
        {
            // Arrange
            PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);
            watcher.Start();

            // Act
            Action act = () => watcher.Dispose();

            // Assert
            act.Should().NotThrow();
            watcher.IsWatching.Should().BeFalse();
        }

        [Fact]
        public void Start_WithValidDirectory_SetsIsWatching()
        {
            // Arrange
            using PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            // Act
            watcher.Start();

            // Assert
            watcher.IsWatching.Should().BeTrue();
        }

        [Fact]
        public void Stop_WhenNotStarted_DoesNotThrow()
        {
            // Arrange
            using PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);

            // Act
            Action act = () => watcher.Stop();

            // Assert
            act.Should().NotThrow();
        }

        [Fact]
        public void Start_AfterDispose_ThrowsObjectDisposedException()
        {
            // Arrange
            PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager);
            watcher.Dispose();

            // Act
            Action act = () => watcher.Start();

            // Assert
            act.Should().Throw<ObjectDisposedException>();
        }

        [Fact]
        public void EnqueueChange_YamlFile_FiresOnPackContentChangedEvent()
        {
            // Arrange
            using PackFileWatcher watcher = new PackFileWatcher(
                _tempDir, _contentLoader, _registryManager, debounceMs: 100);

            string? receivedPath = null;
            watcher.OnPackContentChanged += (_, args) => receivedPath = args.FilePath;

            string yamlPath = Path.Combine(_tempDir, "test.yaml");

            // Act
            watcher.EnqueueChange(yamlPath);

            // Assert — event fires synchronously on EnqueueChange
            receivedPath.Should().Be(yamlPath);
        }

        [Fact]
        public async Task FileChanged_TriggersCallback_WithDebounce()
        {
            const int debounceMs = 200;
            TimeSpan waitTimeout = TimeSpan.FromSeconds(15);

            // Arrange
            string packsDir = Path.Combine(_tempDir, "packs");
            Directory.CreateDirectory(packsDir);

            // Create a valid pack so the reload can succeed
            string packDir = Path.Combine(packsDir, "watch-test-pack");
            Directory.CreateDirectory(packDir);
            string packYaml = Path.Combine(packDir, "pack.yaml");
            File.WriteAllText(packYaml,
                "id: watch-test-pack\nname: Watch Test\nversion: 1.0.0\nauthor: Test\ntype: content\n");

            ContentLoader loader = new ContentLoader(_registryManager);
            using PackFileWatcher watcher = new PackFileWatcher(
                packsDir, loader, _registryManager, debounceMs: debounceMs);

            int reloadCount = 0;
            int contentChangedCount = 0;
            watcher.OnPackReloaded += (_, _) => Interlocked.Increment(ref reloadCount);
            watcher.OnPackReloadFailed += (_, _) => Interlocked.Increment(ref reloadCount);
            watcher.OnPackContentChanged += (_, _) => Interlocked.Increment(ref contentChangedCount);

            watcher.Start();
            watcher.IsWatching.Should().BeTrue();

            // Pattern #108: warm-up proves FileSystemWatcher + debounce before the assertion write.
            // Touch existing pack.yaml first (Changed is more reliable than Create on Windows FSW).
            bool warmupReloaded = false;
            for (int attempt = 0; attempt < 3 && !warmupReloaded; attempt++)
            {
                int contentBefore = Volatile.Read(ref contentChangedCount);
                int reloadBefore = Volatile.Read(ref reloadCount);

                if (attempt == 0)
                {
                    File.SetLastWriteTimeUtc(packYaml, DateTime.UtcNow);
                }
                else
                {
                    string warmupFile = Path.Combine(packDir, $"_fsw_warmup_{attempt}.yaml");
                    File.WriteAllText(warmupFile, $"id: fsw-warmup-{attempt}\n");
                }

                bool fswDelivered = await TestWait.UntilAsync(
                    () => Volatile.Read(ref contentChangedCount) > contentBefore,
                    TimeSpan.FromSeconds(5));
                if (!fswDelivered)
                {
                    continue;
                }

                warmupReloaded = await TestWait.UntilAsync(
                    () => Volatile.Read(ref reloadCount) > reloadBefore,
                    waitTimeout);
            }

            warmupReloaded.Should().BeTrue(
                "warm-up write should complete debounce and fire a reload callback");

            int reloadCountBeforeTrigger = Volatile.Read(ref reloadCount);

            // Act — second write must trigger another debounced reload
            string yamlFile = Path.Combine(packDir, "trigger.yaml");
            File.WriteAllText(yamlFile, "id: trigger\n");

            // Assert — predicate-based wait (no blind Task.Delay)
            bool triggered = await TestWait.UntilAsync(
                () => Volatile.Read(ref reloadCount) > reloadCountBeforeTrigger,
                waitTimeout);

            triggered.Should().BeTrue(
                "the watcher should fire its reload callback after debounce on a second write");
        }

        [Fact]
        public void NullPacksDirectory_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => new PackFileWatcher(null!, _contentLoader, _registryManager);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void NullContentLoader_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => new PackFileWatcher(_tempDir, null!, _registryManager);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void NullRegistryManager_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => new PackFileWatcher(_tempDir, _contentLoader, null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }
    }
}
