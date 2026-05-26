using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Settings page.
    /// </summary>
    public sealed partial class SettingsViewModel : ObservableObject
    {
        private readonly AppConfigService _configService;

        [ObservableProperty]
        public partial string PacksDirectory { get; set; } = "";

        [ObservableProperty]
        public partial string GameDirectory { get; set; } = "";

        [ObservableProperty]
        public partial int ReloadIntervalMs { get; set; } = 2000;

        [ObservableProperty]
        public partial string SaveStatus { get; set; } = "";

        [ObservableProperty]
        public partial bool IsSaving { get; set; }

        /// <summary>True when not saving — used to enable the Save button.</summary>
        public bool IsNotSaving => !IsSaving;

        partial void OnIsSavingChanged(bool value) => OnPropertyChanged(nameof(IsNotSaving));

        /// <summary>Initializes a new instance of <see cref="SettingsViewModel"/>.</summary>
        public SettingsViewModel(AppConfigService configService)
        {
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
        }

        /// <summary>Loads settings from disk.</summary>
        [RelayCommand]
        public async Task LoadAsync()
        {
            AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
            PacksDirectory = config.PacksDirectory;
            GameDirectory = config.GameDirectory;
            ReloadIntervalMs = config.ReloadIntervalMs;
        }

        /// <summary>Persists current settings to disk.</summary>
        [RelayCommand]
        public async Task SaveAsync()
        {
            IsSaving = true;
            SaveStatus = "Saving…";
            try
            {
                AppConfig config = new AppConfig
                {
                    PacksDirectory = PacksDirectory,
                    GameDirectory = GameDirectory,
                    ReloadIntervalMs = ReloadIntervalMs
                };
                await _configService.SaveAsync(config).ConfigureAwait(true);
                SaveStatus = "Settings saved.";
            }
            catch (Exception ex)
            {
                SaveStatus = $"Save failed: {ex.Message}";
            }
            finally
            {
                IsSaving = false;
            }
        }
    }
}
