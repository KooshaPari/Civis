using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;
using Microsoft.Extensions.Logging;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Update page — compares installed pack versions against catalog versions
    /// and displays which packs have updates available.
    /// </summary>
    public sealed partial class UpdateViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly IModCatalogService _catalogService;
        private readonly UpdateCheckService _updateCheckService;
        private readonly AppConfigService _configService;
        private readonly ILogger<UpdateViewModel>? _logger;

        [ObservableProperty]
        public partial ObservableCollection<UpdateInfo> Updates { get; set; } = new ObservableCollection<UpdateInfo>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "Select a catalog to check for updates";

        [ObservableProperty]
        public partial string CatalogSource { get; set; } = "";

        [ObservableProperty]
        public partial int TotalInstalled { get; set; }

        [ObservableProperty]
        public partial int UpdatesAvailable { get; set; }

        [ObservableProperty]
        public partial UpdateInfo? SelectedUpdate { get; set; }

        /// <summary>Initializes a new instance of <see cref="UpdateViewModel"/>.</summary>
        /// <param name="packDataService">Service for loading installed packs.</param>
        /// <param name="catalogService">Service for loading catalog data.</param>
        /// <param name="updateCheckService">Service for comparing versions.</param>
        /// <param name="configService">App configuration service.</param>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public UpdateViewModel(
            IPackDataService packDataService,
            IModCatalogService catalogService,
            UpdateCheckService updateCheckService,
            AppConfigService configService,
            ILogger<UpdateViewModel>? logger = null)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _catalogService = catalogService ?? throw new ArgumentNullException(nameof(catalogService));
            _updateCheckService = updateCheckService ?? throw new ArgumentNullException(nameof(updateCheckService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
            _logger = logger;
        }

        /// <summary>
        /// Checks for updates by loading the catalog and comparing versions.
        /// </summary>
        [RelayCommand]
        public async Task CheckForUpdatesAsync()
        {
            if (string.IsNullOrWhiteSpace(CatalogSource))
            {
                StatusMessage = "Please enter a catalog URL or folder path.";
                return;
            }

            IsLoading = true;
            StatusMessage = "Loading installed packs…";

            try
            {
                // Step 1: Get installed packs
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                if (string.IsNullOrEmpty(config.PacksDirectory))
                {
                    StatusMessage = "Packs directory not configured — go to Settings.";
                    return;
                }

                LoadResultViewModel installResult = await _packDataService
                    .LoadPacksAsync(config.PacksDirectory)
                    .ConfigureAwait(true);

                TotalInstalled = installResult.Packs.Count;
                StatusMessage = "Loading catalog…";

                // Step 2: Load catalog
                CatalogLoadResult catalogResult = await _catalogService
                    .LoadCatalogAsync(CatalogSource)
                    .ConfigureAwait(true);

                if (!catalogResult.IsSuccess)
                {
                    StatusMessage = $"Catalog error: {string.Join("; ", catalogResult.Errors)}";
                    return;
                }

                // Step 3: Compare versions
                IReadOnlyList<UpdateInfo> updates = _updateCheckService.CheckForUpdates(
                    installResult.Packs,
                    catalogResult.Entries);

                Updates.Clear();
                foreach (UpdateInfo update in updates)
                {
                    Updates.Add(update);
                }

                UpdatesAvailable = Updates.Count;
                StatusMessage = UpdatesAvailable > 0
                    ? $"{UpdatesAvailable} update(s) available for {TotalInstalled} installed packs"
                    : $"All {TotalInstalled} installed packs are up to date";

                _logger?.LogInformation(
                    "Update check complete: {UpdateCount} updates for {InstalledCount} packs",
                    UpdatesAvailable,
                    TotalInstalled);
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error checking for updates: {ex.Message}";
                _logger?.LogError(ex, "Failed to check for updates");
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// Opens the download page for the selected update.
        /// </summary>
        [RelayCommand]
        public void DownloadUpdate()
        {
            if (SelectedUpdate == null || string.IsNullOrEmpty(SelectedUpdate.DownloadUrl))
            {
                StatusMessage = "No download URL available for this update.";
                return;
            }

            try
            {
                StatusMessage = $"Opening download page for {SelectedUpdate.PackName} {SelectedUpdate.AvailableVersion}…";
                System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                {
                    FileName = SelectedUpdate.DownloadUrl,
                    UseShellExecute = true
                });
            }
            catch (Exception ex)
            {
                StatusMessage = $"Failed to open download page: {ex.Message}";
            }
        }

        /// <summary>
        /// Refreshes the update check from the current catalog source.
        /// </summary>
        [RelayCommand]
        public async Task RefreshAsync()
        {
            await CheckForUpdatesAsync().ConfigureAwait(false);
        }
    }
}
