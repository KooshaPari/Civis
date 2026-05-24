#nullable enable
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;

namespace DINOForge.SDK.HotReload
{
    /// <summary>
    /// Watches the packs directory for YAML file changes and triggers re-parse,
    /// re-validation, and registry updates. Uses <see cref="FileSystemWatcher"/>
    /// with debouncing to coalesce rapid edits (ADR-008: wrap, don't handroll).
    /// </summary>
    public class PackFileWatcher : IDisposable
    {
        private FileSystemWatcher? _watcher;
        private readonly string _packsDirectory;
        private readonly IPackReloadService _packReloadService;
        private readonly IPackRootResolver _packRootResolver;
        private readonly Action<string> _log;
        private readonly int _debounceMs;

        private readonly ConcurrentDictionary<string, DateTimeOffset> _pendingChanges =
            new ConcurrentDictionary<string, DateTimeOffset>(StringComparer.OrdinalIgnoreCase);
        private Timer? _debounceTimer;
        private readonly object _lock = new object();
        private bool _disposed;

        /// <summary>Raised when pack content files change on disk.</summary>
        public event EventHandler<PackContentChangedEventArgs>? OnPackContentChanged; // public-field-ok: events are intrinsically encapsulated (only += and -= allowed)

        /// <summary>Raised after packs are successfully reloaded.</summary>
        public event EventHandler<HotReloadResult>? OnPackReloaded; // public-field-ok: events are intrinsically encapsulated (only += and -= allowed)

        /// <summary>Raised when a reload attempt fails.</summary>
        public event EventHandler<HotReloadResult>? OnPackReloadFailed; // public-field-ok: events are intrinsically encapsulated (only += and -= allowed)

        /// <summary>
        /// Initializes a new <see cref="PackFileWatcher"/>.
        /// </summary>
        /// <param name="packsDirectory">Root directory containing pack subdirectories.</param>
        /// <param name="contentLoader">The content loader used to re-parse packs.</param>
        /// <param name="registryManager">The registry manager to update on reload.</param>
        /// <param name="schemaValidator">Optional schema validator for re-validation.</param>
        /// <param name="log">Logging callback.</param>
        /// <param name="debounceMs">Debounce window in milliseconds (default 15000; floor 500).</param>
        public PackFileWatcher(
            string packsDirectory,
            ContentLoader contentLoader,
            RegistryManager registryManager,
            ISchemaValidator? schemaValidator = null,
            Action<string>? log = null,
            int debounceMs = 15000)
        {
            _packsDirectory = packsDirectory ?? throw new ArgumentNullException(nameof(packsDirectory));
            _packReloadService = contentLoader ?? throw new ArgumentNullException(nameof(contentLoader));
            _packRootResolver = new FileSystemPackRootResolver();
            _ = registryManager ?? throw new ArgumentNullException(nameof(registryManager));
            _ = schemaValidator;
            _log = log ?? (_ => { });
            _debounceMs = debounceMs > 0 ? debounceMs : 15000;
        }

        /// <summary>
        /// Starts watching the packs directory for YAML file changes.
        /// </summary>
        public void Start()
        {
            if (_disposed) throw new ObjectDisposedException(nameof(PackFileWatcher));
            if (_watcher != null) return;

            if (!Directory.Exists(_packsDirectory))
            {
                _log($"[PackFileWatcher] Packs directory does not exist: {_packsDirectory}");
                return;
            }

            _watcher = new FileSystemWatcher(_packsDirectory)
            {
                IncludeSubdirectories = true,
                NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.Size,
                // Reduce missed events when many watchers run under xUnit parallel (Pattern #108).
                InternalBufferSize = 65536,
            };

            // Watch YAML files
            _watcher.Filter = "*.*";
            _watcher.Changed += OnFileChanged;
            _watcher.Created += OnFileChanged;
            _watcher.Renamed += OnFileRenamed;
            _watcher.EnableRaisingEvents = true;

            _log($"[PackFileWatcher] Watching packs directory: {_packsDirectory}");
        }

        /// <summary>
        /// Stops watching and releases the <see cref="FileSystemWatcher"/>.
        /// </summary>
        public void Stop()
        {
            if (_watcher != null)
            {
                _watcher.EnableRaisingEvents = false;
                _watcher.Changed -= OnFileChanged;
                _watcher.Created -= OnFileChanged;
                _watcher.Renamed -= OnFileRenamed;
                _watcher.Dispose();
                _watcher = null;
            }

            lock (_lock)
            {
                _debounceTimer?.Dispose();
                _debounceTimer = null;
            }

            _log("[PackFileWatcher] Stopped watching.");
        }

        /// <summary>
        /// Gets whether the watcher is currently active.
        /// </summary>
        public bool IsWatching => _watcher != null && _watcher.EnableRaisingEvents;

        /// <summary>
        /// Manually triggers a reload of all packs. Useful for "Reload" buttons.
        /// </summary>
        /// <returns>The result of the reload operation.</returns>
        public HotReloadResult ReloadAll()
        {
            _log("[PackFileWatcher] Manual reload of all packs triggered.");
            return ExecuteReload(new List<string> { _packsDirectory });
        }

        private void OnFileChanged(object sender, FileSystemEventArgs e)
        {
            if (!IsYamlFile(e.FullPath)) return;
            EnqueueChange(e.FullPath);
        }

        private void OnFileRenamed(object sender, RenamedEventArgs e)
        {
            if (!IsYamlFile(e.FullPath)) return;
            EnqueueChange(e.FullPath);
        }

        private static bool IsYamlFile(string path)
        {
            string ext = Path.GetExtension(path);
            return string.Equals(ext, ".yaml", StringComparison.OrdinalIgnoreCase)
                || string.Equals(ext, ".yml", StringComparison.OrdinalIgnoreCase);
        }

        /// <summary>
        /// Enqueues a file change and resets the debounce timer.
        /// Thread-safe: called from FileSystemWatcher thread pool threads.
        /// </summary>
        internal void EnqueueChange(string filePath)
        {
            _pendingChanges[filePath] = DateTimeOffset.UtcNow;

            OnPackContentChanged?.Invoke(this, new PackContentChangedEventArgs(filePath));

            lock (_lock)
            {
                _debounceTimer?.Dispose();
                _debounceTimer = new Timer(OnDebounceElapsed, null, _debounceMs, Timeout.Infinite);
            }
        }

        private void OnDebounceElapsed(object? state)
        {
            // Drain pending changes without Clear() — concurrent EnqueueChange during drain
            // must survive for the next debounce tick (Pattern #51 / TRUTH_TABLE #175).
            List<string> changedFiles = new List<string>();
            foreach (string key in _pendingChanges.Keys.ToList())
            {
                if (_pendingChanges.TryRemove(key, out _))
                {
                    changedFiles.Add(key);
                }
            }

            if (changedFiles.Count == 0) return;

            _log($"[PackFileWatcher] Debounce elapsed. Processing {changedFiles.Count} changed file(s).");

            HotReloadResult result = ExecuteReload(changedFiles);

            if (result.IsSuccess)
            {
                OnPackReloaded?.Invoke(this, result);
            }
            else
            {
                OnPackReloadFailed?.Invoke(this, result);
            }
        }

        private HotReloadResult ExecuteReload(List<string> changedFiles)
        {
            List<string> errors = new List<string>();
            List<string> updatedEntries = new List<string>();

            try
            {
                // Determine which pack directories are affected
                HashSet<string> affectedPackDirs = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
                foreach (string file in changedFiles)
                {
                    string? packDir = _packRootResolver.ResolvePackRoot(file, _packsDirectory);
                    if (packDir != null)
                    {
                        affectedPackDirs.Add(packDir);
                    }
                }

                foreach (string packDir in affectedPackDirs)
                {
                    try
                    {
                        ContentLoadResult loadResult = _packReloadService.ReloadPack(packDir);
                        if (loadResult.IsSuccess)
                        {
                            updatedEntries.AddRange(loadResult.LoadedPacks);
                            _log($"[PackFileWatcher] Reloaded pack from: {packDir}");
                        }
                        else
                        {
                            errors.AddRange(loadResult.Errors);
                        }
                    }
                    catch (Exception ex)
                    {
                        errors.Add($"Failed to reload pack at {packDir}: {ex.Message}");
                    }
                }
            }
            catch (Exception ex)
            {
                errors.Add($"Hot reload failed: {ex.Message}");
            }

            IReadOnlyList<string> changedFilesRo = changedFiles.AsReadOnly();
            IReadOnlyList<string> updatedRo = updatedEntries.AsReadOnly();
            IReadOnlyList<string> errorsRo = errors.AsReadOnly();

            if (errors.Count == 0)
            {
                return HotReloadResult.Success(changedFilesRo, updatedRo);
            }

            if (updatedEntries.Count > 0)
            {
                return HotReloadResult.Partial(changedFilesRo, updatedRo, errorsRo);
            }

            return HotReloadResult.Failure(changedFilesRo, errorsRo);
        }
        /// <inheritdoc />
        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;

            if (_watcher != null)
            {
                _watcher.Changed -= OnFileChanged;
                _watcher.Created -= OnFileChanged;
                _watcher.Renamed -= OnFileRenamed;
            }

            Stop();
        }
    }

    /// <summary>
    /// Event data for when a pack content file changes on disk.
    /// </summary>
    public sealed class PackContentChangedEventArgs : EventArgs
    {
        /// <summary>Full path of the changed file.</summary>
        public string FilePath { get; }

        /// <summary>When the change was detected.</summary>
        public DateTimeOffset Timestamp { get; }

        /// <summary>
        /// Creates a new instance with the given file path.
        /// </summary>
        public PackContentChangedEventArgs(string filePath)
        {
            FilePath = filePath;
            Timestamp = DateTimeOffset.UtcNow;
        }
    }
}
