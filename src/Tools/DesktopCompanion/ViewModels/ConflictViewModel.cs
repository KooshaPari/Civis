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
    /// View-model for the Conflict page — runs dependency resolution and conflict detection
    /// on installed packs, displaying issues and computed load order.
    /// </summary>
    public sealed partial class ConflictViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly ConflictDetectionService _conflictService;
        private readonly AppConfigService _configService;
        private readonly ILogger<ConflictViewModel>? _logger;

        [ObservableProperty]
        public partial ObservableCollection<ConflictItem> Conflicts { get; set; } = new ObservableCollection<ConflictItem>();

        [ObservableProperty]
        public partial ObservableCollection<DependencyIssue> DependencyIssues { get; set; } = new ObservableCollection<DependencyIssue>();

        [ObservableProperty]
        public partial ObservableCollection<DependencyTreeNode> DependencyTree { get; set; } = new ObservableCollection<DependencyTreeNode>();

        [ObservableProperty]
        public partial ObservableCollection<string> LoadOrder { get; set; } = new ObservableCollection<string>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "Click Analyze to check for conflicts";

        [ObservableProperty]
        public partial int ConflictCount { get; set; }

        [ObservableProperty]
        public partial int DependencyIssueCount { get; set; }

        [ObservableProperty]
        public partial bool HasIssues { get; set; }

        [ObservableProperty]
        public partial string SelectedPackId { get; set; } = "";

        private IReadOnlyList<PackViewModel> _allPacks = Array.Empty<PackViewModel>();

        /// <summary>Initializes a new instance of <see cref="ConflictViewModel"/>.</summary>
        /// <param name="packDataService">Service for loading installed packs.</param>
        /// <param name="conflictService">Service for detecting conflicts.</param>
        /// <param name="configService">App configuration service.</param>
        /// <param name="logger">Optional logger for diagnostic output.</param>
        public ConflictViewModel(
            IPackDataService packDataService,
            ConflictDetectionService conflictService,
            AppConfigService configService,
            ILogger<ConflictViewModel>? logger = null)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _conflictService = conflictService ?? throw new ArgumentNullException(nameof(conflictService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
            _logger = logger;
        }

        /// <summary>
        /// Analyzes installed packs for conflicts and dependency issues.
        /// </summary>
        [RelayCommand]
        public async Task AnalyzeConflictsAsync()
        {
            IsLoading = true;
            StatusMessage = "Loading installed packs…";

            try
            {
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                if (string.IsNullOrEmpty(config.PacksDirectory))
                {
                    StatusMessage = "Packs directory not configured — go to Settings.";
                    return;
                }

                LoadResultViewModel result = await _packDataService
                    .LoadPacksAsync(config.PacksDirectory)
                    .ConfigureAwait(true);

                _allPacks = result.Packs;
                StatusMessage = "Analyzing conflicts…";

                ConflictReport report = _conflictService.AnalyzeConflicts(_allPacks);

                // Update conflict list
                Conflicts.Clear();
                foreach (ConflictItem conflict in report.Conflicts)
                {
                    Conflicts.Add(conflict);
                }

                // Update dependency issues list
                DependencyIssues.Clear();
                foreach (DependencyIssue issue in report.MissingDependencies)
                {
                    DependencyIssues.Add(issue);
                }

                // Update load order
                LoadOrder.Clear();
                foreach (string packId in report.ComputedLoadOrder)
                {
                    LoadOrder.Add(packId);
                }

                ConflictCount = Conflicts.Count;
                DependencyIssueCount = DependencyIssues.Count;
                HasIssues = report.HasIssues;

                if (!report.IsSuccess)
                {
                    StatusMessage = $"{report.IssueCount} issue(s) detected among {result.Packs.Count} packs";
                }
                else
                {
                    StatusMessage = $"No issues — {result.Packs.Count} packs analyzed successfully";
                }

                _logger?.LogInformation(
                    "Conflict analysis complete: {ConflictCount} conflicts, {DepIssueCount} dependency issues",
                    ConflictCount,
                    DependencyIssueCount);
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error analyzing conflicts: {ex.Message}";
                _logger?.LogError(ex, "Failed to analyze conflicts");
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// Shows the dependency tree for the selected pack.
        /// </summary>
        partial void OnSelectedPackIdChanged(string value)
        {
            if (string.IsNullOrEmpty(value))
            {
                DependencyTree.Clear();
                return;
            }

            IReadOnlyList<DependencyTreeNode> nodes = _conflictService.GetDependencyTree(value, _allPacks);
            DependencyTree.Clear();
            foreach (DependencyTreeNode node in nodes)
            {
                DependencyTree.Add(node);
            }
        }

        /// <summary>
        /// Refreshes the conflict analysis.
        /// </summary>
        [RelayCommand]
        public async Task RefreshAsync()
        {
            await AnalyzeConflictsAsync().ConfigureAwait(false);
        }
    }
}
