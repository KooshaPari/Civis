using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Hosting;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace DINOForge.DesktopCompanion.Data;

/// <summary>
/// Watches the configured packs directory and reloads the shared
/// <see cref="PackListViewModel"/> with a 500ms debounce when pack files change.
/// </summary>
public sealed class PackFileWatcher : IHostedService, IDisposable
{
    private const int DebounceMilliseconds = 500;

    private readonly AppConfigService _configService;
    private readonly PackListViewModel _packListViewModel;
    private FileSystemWatcher? _watcher;
    private Timer? _reloadTimer;
    private readonly object _timerLock = new();
    private volatile bool _started;
    private bool _disposed;

    /// <summary>Initializes a new instance of <see cref="PackFileWatcher"/>.</summary>
    public PackFileWatcher(AppConfigService configService, PackListViewModel packListViewModel)
    {
        _configService = configService ?? throw new ArgumentNullException(nameof(configService));
        _packListViewModel = packListViewModel ?? throw new ArgumentNullException(nameof(packListViewModel));
    }

    /// <inheritdoc />
    public async Task StartAsync(CancellationToken cancellationToken)
    {
        if (_started)
            return;

        _started = true;

        AppConfig config = await _configService.LoadAsync().ConfigureAwait(false);
        if (string.IsNullOrWhiteSpace(config.PacksDirectory))
            return;

        if (!Directory.Exists(config.PacksDirectory))
        {
            return;
        }

        _watcher = new FileSystemWatcher(config.PacksDirectory)
        {
            IncludeSubdirectories = true,
            NotifyFilter = NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.LastWrite,
            EnableRaisingEvents = true,
            Filter = "*.yaml"
        };

        _watcher.Created += OnPackDirectoryChanged;
        _watcher.Changed += OnPackDirectoryChanged;
        _watcher.Deleted += OnPackDirectoryChanged;
        _watcher.Renamed += OnPackDirectoryChanged;
        _watcher.Error += OnWatcherError;
    }

    /// <inheritdoc />
    public Task StopAsync(CancellationToken cancellationToken)
    {
        _watcher?.Dispose();
        _watcher = null;
        _reloadTimer?.Dispose();
        _reloadTimer = null;
        return Task.CompletedTask;
    }

    private void OnPackDirectoryChanged(object sender, FileSystemEventArgs e)
    {
        DebouncedReload();
    }

    private void OnPackDirectoryChanged(object sender, RenamedEventArgs e)
    {
        DebouncedReload();
    }

    private void OnWatcherError(object sender, ErrorEventArgs e)
    {
        CompanionLogger.Append($"PackFileWatcher error: {e.GetException()}");
    }

    private void DebouncedReload()
    {
        lock (_timerLock)
        {
            _reloadTimer?.Dispose();
            _reloadTimer = new Timer(
                _ =>
                {
                    _ = ExecuteReloadAsync();
                },
                null,
                DebounceMilliseconds,
                Timeout.Infinite);
        }
    }

    private Task ExecuteReloadAsync()
    {
        if (Application.Current is not App app || app.CurrentWindowDispatcherQueue is null)
            return _packListViewModel.ReloadAsync();

        if (app.CurrentWindowDispatcherQueue.HasThreadAccess)
        {
            return _packListViewModel.ReloadAsync();
        }

        var completionSource = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        app.CurrentWindowDispatcherQueue.TryEnqueue(async () =>
        {
            try
            {
                await _packListViewModel.ReloadAsync().ConfigureAwait(false);
                completionSource.TrySetResult();
            }
            catch (Exception ex)
            {
                completionSource.TrySetException(ex);
            }
        });

        return completionSource.Task;
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed)
            return;

        _disposed = true;
        StopAsync(CancellationToken.None).GetAwaiter().GetResult();
    }
}
