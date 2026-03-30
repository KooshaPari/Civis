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
    /// View-model for the Browse page — loads pack catalog from local folder or HTTP URL,
    /// displays the list with search/filter support, and provides an install action.
    /// </summary>
    public sealed partial class BrowseViewModel : ObservableObject
    {
        private readonly IModCatalogService _catalogService;
        private readonly ILogger<BrowseViewModel>? _logger;
        private readonly AppConfigService _configService;
        private IReadOnlyList<CatalogEntry> _allEntries = Array.Empty<CatalogEntry>();

        [ObservableProperty]
        public partial ObservableCollection<CatalogEntry> Entries { get; set; } = new ObservableCollection<CatalogEntry>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "Enter a catalog URL or folder path";

        [ObservableProperty]
        public partial string CatalogSource { get; set; } = "";

        [ObservableProperty]
        public partial string SearchText { get; set; } = "";

        [ObservableProperty]
        public partial string FilterType { get; set; } = "All";

        [ObservableProperty]
        public partial CatalogEntry? SelectedEntry { get; set; }

        [ObservableProperty]
        public partial bool HasEntries => Entries.Count > 0;

        /// <summary>Initializes a new instance of <see cref="BrowseViewModel"/>.</summary>
        /// <param name="catalogService">The catalog service for loading pack metadata.</param>
        /// <param name="configService">The app config service for reading default paths.</param>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public BrowseViewModel(
            IModCatalogService catalogService,
            AppConfigService configService,
            ILogger<BrowseViewModel>? logger = null)
        {
            _catalogService = catalogService ?? throw new ArgumentNullException(nameof(catalogService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
            _logger = logger;
        }

        /// <summary>
        /// Loads the catalog from the specified source URI.
        /// </summary>
        [RelayCommand]
        public async Task LoadCatalogAsync()
        {
            if (string.IsNullOrWhiteSpace(CatalogSource))
            {
                StatusMessage = "Please enter a catalog URL or folder path.";
                return;
            }

            IsLoading = true;
            StatusMessage = "Loading catalog…";

            try
            {
                CatalogLoadResult result = await _catalogService
                    .LoadCatalogAsync(CatalogSource)
                    .ConfigureAwait(false);

                if (!result.IsSuccess)
                {
                    StatusMessage = string.Join("; ", result.Errors);
                    Entries.Clear();
                    _allEntries = Array.Empty<CatalogEntry>();
                    return;
                }

                _allEntries = result.Entries;

                // Mark installed packs
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                if (!string.IsNullOrEmpty(config.PacksDirectory))
                {
                    foreach (CatalogEntry entry in _allEntries)
                    {
                        // Simple check: if pack folder exists, mark as installed
                        string packPath = System.IO.Path.Combine(config.PacksDirectory, entry.Id);
                        entry.IsInstalled = System.IO.Directory.Exists(packPath);
                    }
                }

                ApplyFilters();
                StatusMessage = result.IsSuccess
                    ? $"{result.Entries.Count} pack(s) found in catalog"
                    : "No packs found";

                _logger?.LogInformation("Loaded {Count} catalog entries from {Source}", result.Entries.Count, result.SourceUri);
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error loading catalog: {ex.Message}";
                _logger?.LogError(ex, "Failed to load catalog from {CatalogSource}", CatalogSource);
            }
            finally
            {
                IsLoading = false;
                OnPropertyChanged(nameof(HasEntries));
            }
        }

        /// <summary>
        /// Applies the current search text and type filter to the catalog entries.
        /// </summary>
        partial void OnSearchTextChanged(string value) => ApplyFilters();

        /// <summary>
        /// Applies the current type filter to the catalog entries.
        /// </summary>
        partial void OnFilterTypeChanged(string value) => ApplyFilters();

        /// <summary>
        /// Refreshes the catalog from the current source.
        /// </summary>
        [RelayCommand]
        public async Task RefreshAsync()
        {
            await LoadCatalogAsync().ConfigureAwait(false);
        }

        /// <summary>
        /// Opens the download URL for the selected catalog entry in the default browser.
        /// </summary>
        [RelayCommand]
        public void InstallPack()
        {
            if (SelectedEntry == null || string.IsNullOrEmpty(SelectedEntry.DownloadUrl))
            {
                StatusMessage = "No download URL available for this pack.";
                return;
            }

            try
            {
                StatusMessage = $"Opening download page for {SelectedEntry.Name}…";
                System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                {
                    FileName = SelectedEntry.DownloadUrl,
                    UseShellExecute = true
                });
            }
            catch (Exception ex)
            {
                StatusMessage = $"Failed to open download page: {ex.Message}";
            }
        }

        private void ApplyFilters()
        {
            Entries.Clear();

            IQueryable<CatalogEntry> filtered = _allEntries.AsQueryable();

            // Apply type filter
            if (!string.IsNullOrEmpty(FilterType) && FilterType != "All")
            {
                filtered = filtered.Where(e =>
                    string.Equals(e.Type, FilterType, StringComparison.OrdinalIgnoreCase));
            }

            // Apply search filter
            if (!string.IsNullOrWhiteSpace(SearchText))
            {
                string search = SearchText.ToLowerInvariant();
                filtered = filtered.Where(e =>
                    e.Id.ToLowerInvariant().Contains(search) ||
                    e.Name.ToLowerInvariant().Contains(search) ||
                    (e.Description?.ToLowerInvariant().Contains(search) ?? false) ||
                    e.Author.ToLowerInvariant().Contains(search));
            }

            foreach (CatalogEntry entry in filtered)
            {
                Entries.Add(entry);
            }

            OnPropertyChanged(nameof(HasEntries));
        }
    }
}
