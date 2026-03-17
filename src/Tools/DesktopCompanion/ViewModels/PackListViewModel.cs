using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Pack List page — mirrors the F10 mod menu pack list.
    /// Supports enable/disable toggles persisted via <see cref="DisabledPacksService"/>.
    /// </summary>
    public sealed partial class PackListViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly DisabledPacksService _disabledPacksService;
        private readonly AppConfigService _configService;
        private string _packsDirectory = "";

        [ObservableProperty]
        public partial ObservableCollection<PackViewModel> Packs { get; set; } = new ObservableCollection<PackViewModel>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "";

        [ObservableProperty]
        public partial PackViewModel? SelectedPack { get; set; }

        /// <summary>Initializes a new instance of <see cref="PackListViewModel"/>.</summary>
        public PackListViewModel(
            IPackDataService packDataService,
            DisabledPacksService disabledPacksService,
            AppConfigService configService)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _disabledPacksService = disabledPacksService ?? throw new ArgumentNullException(nameof(disabledPacksService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
        }

        /// <summary>Loads packs from disk and applies disabled state from JSON.</summary>
        [RelayCommand]
        public async Task ReloadAsync()
        {
            IsLoading = true;
            StatusMessage = "Loading packs…";

            try
            {
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                _packsDirectory = config.PacksDirectory;

                if (string.IsNullOrEmpty(_packsDirectory))
                {
                    StatusMessage = "Packs directory not configured — go to Settings.";
                    // Unsubscribe all before clearing
                    foreach (PackViewModel p in Packs)
                        p.PropertyChanged -= OnPackPropertyChanged;
                    Packs.Clear();
                    return;
                }

                LoadResultViewModel result = await _packDataService
                    .LoadPacksAsync(_packsDirectory)
                    .ConfigureAwait(true);

                HashSet<string> disabled = await _disabledPacksService
                    .LoadAsync(_packsDirectory)
                    .ConfigureAwait(true);

                // Unsubscribe old items
                foreach (PackViewModel p in Packs)
                    p.PropertyChanged -= OnPackPropertyChanged;
                Packs.Clear();

                foreach (PackViewModel pack in result.Packs)
                {
                    pack.Enabled = !disabled.Contains(pack.Id);
                    pack.PropertyChanged += OnPackPropertyChanged;
                    Packs.Add(pack);
                }

                StatusMessage = result.IsSuccess
                    ? $"{result.LoadedCount} pack(s) discovered"
                    : $"{result.LoadedCount} packs, {result.ErrorCount} error(s)";
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error loading packs: {ex.Message}";
            }
            finally
            {
                IsLoading = false;
            }
        }

        private async void OnPackPropertyChanged(object? sender, PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(PackViewModel.Enabled))
                await PersistDisabledPacksAsync().ConfigureAwait(false);
        }

        private async Task PersistDisabledPacksAsync()
        {
            if (string.IsNullOrEmpty(_packsDirectory)) return;

            List<string> disabled = new List<string>();
            foreach (PackViewModel p in Packs)
            {
                if (!p.Enabled)
                    disabled.Add(p.Id);
            }

            try
            {
                await _disabledPacksService.SaveAsync(_packsDirectory, disabled).ConfigureAwait(false);
            }
            catch (Exception ex)
            {
                StatusMessage = $"Failed to save disabled packs: {ex.Message}";
            }
        }
    }
}
