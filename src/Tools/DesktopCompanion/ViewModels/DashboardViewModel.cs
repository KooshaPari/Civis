using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Dashboard page — shows a summary of loaded packs and errors.
    /// </summary>
    public sealed partial class DashboardViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly AppConfigService _configService;

        [ObservableProperty]
        public partial int LoadedCount { get; set; }

        [ObservableProperty]
        public partial int ErrorCount { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "Not loaded";

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        /// <summary>True when not loading — used to enable the Refresh button.</summary>
        public bool IsNotLoading => !IsLoading;

        partial void OnIsLoadingChanged(bool value) => OnPropertyChanged(nameof(IsNotLoading));

        [ObservableProperty]
        public partial bool HasErrors { get; set; }

        [ObservableProperty]
        public partial string PacksDirectory { get; set; } = "";

        /// <summary>Initializes a new instance of <see cref="DashboardViewModel"/>.</summary>
        public DashboardViewModel(IPackDataService packDataService, AppConfigService configService)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
        }

        /// <summary>Refreshes pack data from the configured packs directory.</summary>
        [RelayCommand]
        public async Task RefreshAsync()
        {
            IsLoading = true;
            StatusMessage = "Scanning packs…";

            try
            {
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                PacksDirectory = config.PacksDirectory;

                if (string.IsNullOrEmpty(PacksDirectory))
                {
                    StatusMessage = "Packs directory not configured — go to Settings.";
                    LoadedCount = 0;
                    ErrorCount = 0;
                    HasErrors = false;
                    return;
                }

                LoadResultViewModel result = await _packDataService
                    .LoadPacksAsync(PacksDirectory)
                    .ConfigureAwait(true);

                LoadedCount = result.LoadedCount;
                ErrorCount = result.ErrorCount;
                HasErrors = result.ErrorCount > 0;

                StatusMessage = result.IsSuccess
                    ? $"All {result.LoadedCount} pack(s) loaded OK"
                    : $"{result.LoadedCount} loaded, {result.ErrorCount} error(s)";
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error: {ex.Message}";
                HasErrors = true;
            }
            finally
            {
                IsLoading = false;
            }
        }
    }
}
