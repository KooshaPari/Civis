using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="PackFileWatcher"/> covering lifecycle, debounce, and event firing.
    /// </summary>
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
            // Arrange
            string packsDir = Path.Combine(_tempDir, "packs");
            Directory.CreateDirectory(packsDir);

            // Create a valid pack so the reload can succeed
            string packDir = Path.Combine(packsDir, "watch-test-pack");
            Directory.CreateDirectory(packDir);
            File.WriteAllText(Path.Combine(packDir, "pack.yaml"),
                "id: watch-test-pack\nname: Watch Test\nversion: 1.0.0\nauthor: Test\ntype: content\n");

            ContentLoader loader = new ContentLoader(_registryManager);
            using PackFileWatcher watcher = new PackFileWatcher(
                packsDir, loader, _registryManager, debounceMs: 200);

            TaskCompletionSource<bool> tcs = new TaskCompletionSource<bool>();
            watcher.OnPackReloaded += (_, _) => tcs.TrySetResult(true);
            watcher.OnPackReloadFailed += (_, _) => tcs.TrySetResult(false);

            // Act — write a YAML file inside the pack directory to trigger the watcher
            watcher.Start();
            // Pattern #108: Windows FileSystemWatcher needs a generous settle window
            // before the OS hooks in, especially under push-time lefthook CPU contention
            // (xUnit parallel + build + test concurrency).
            await Task.Delay(250);

            string yamlFile = Path.Combine(packDir, "trigger.yaml");
            File.WriteAllText(yamlFile, "id: trigger\n");

            // Assert — wait up to 15 seconds for debounce + reload.
            // Original 3s was too tight under Windows FSW + xUnit parallel + lefthook
            // CPU contention (push v7 flake). 15s ceiling still fails fast on real bugs
            // while absorbing OS-level FSW latency spikes (200-2000ms observed).
            bool completed = await Task.WhenAny(tcs.Task, Task.Delay(15000)) == tcs.Task;
            // The watcher fires; success or failure both indicate the callback fired
            completed.Should().BeTrue("the watcher should fire its reload callback within 15 seconds");
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
