using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Pack List page — mirrors the F10 mod menu pack list.
    /// Supports enable/disable toggles persisted via <see cref="DisabledPacksService"/>.
    /// Runs conflict detection to show warning icons on packs with issues.
    /// </summary>
    public sealed partial class PackListViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly DisabledPacksService _disabledPacksService;
        private readonly AppConfigService _configService;
        private readonly ConflictDetectionService _conflictService;
        private string _packsDirectory = "";

        [ObservableProperty]
        public partial ObservableCollection<PackViewModel> Packs { get; set; } = new ObservableCollection<PackViewModel>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "";

        [ObservableProperty]
        public partial PackViewModel? SelectedPack { get; set; }

        [ObservableProperty]
        public partial int ConflictCount { get; set; }

        [ObservableProperty]
        public partial int DependencyIssueCount { get; set; }

        /// <summary>Initializes a new instance of <see cref="PackListViewModel"/>.</summary>
        /// <param name="packDataService">Service for loading pack data.</param>
        /// <param name="disabledPacksService">Service for persisting disabled pack state.</param>
        /// <param name="configService">App configuration service.</param>
        /// <param name="conflictService">Service for detecting pack conflicts.</param>
        public PackListViewModel(
            IPackDataService packDataService,
            DisabledPacksService disabledPacksService,
            AppConfigService configService,
            ConflictDetectionService conflictService)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _disabledPacksService = disabledPacksService ?? throw new ArgumentNullException(nameof(disabledPacksService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
            _conflictService = conflictService ?? throw new ArgumentNullException(nameof(conflictService));
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

                // Run conflict detection on loaded packs
                RunConflictDetection();

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

        /// <summary>
        /// Runs conflict detection on the currently loaded packs and updates UI state.
        /// </summary>
        public void RunConflictDetection()
        {
            if (Packs.Count == 0)
            {
                ConflictCount = 0;
                DependencyIssueCount = 0;
                return;
            }

            // Clear previous conflict states
            foreach (PackViewModel pack in Packs)
            {
                pack.HasConflict = false;
                pack.HasMissingDependency = false;
            }

            ConflictReport report = _conflictService.AnalyzeConflicts(Packs);
            ConflictCount = report.Conflicts.Count;
            DependencyIssueCount = report.MissingDependencies.Count;

            // Mark packs with conflicts
            HashSet<string> conflictPackIds = new HashSet<string>(
                report.Conflicts.SelectMany(c => c.PackIds),
                StringComparer.OrdinalIgnoreCase);

            foreach (PackViewModel pack in Packs)
            {
                if (conflictPackIds.Contains(pack.Id))
                    pack.HasConflict = true;
            }

            // Mark packs with missing dependencies
            HashSet<string> missingDepPackIds = new HashSet<string>(
                report.MissingDependencies.Select(d => d.PackId),
                StringComparer.OrdinalIgnoreCase);

            foreach (PackViewModel pack in Packs)
            {
                if (missingDepPackIds.Contains(pack.Id))
                    pack.HasMissingDependency = true;
            }
        }

        private async void OnPackPropertyChanged(object? sender, PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(PackViewModel.Enabled))
            {
                await PersistDisabledPacksAsync().ConfigureAwait(false);
                // Re-run conflict detection when enabled/disabled state changes
                RunConflictDetection();
            }
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
