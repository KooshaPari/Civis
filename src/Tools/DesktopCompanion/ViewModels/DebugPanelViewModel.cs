using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Debug Panel page — mirrors the F9 in-game panel sections.
    /// Sections: Platform Status, ECS Info (simulated), Pack Info, System State, Errors.
    /// Data is sourced from the filesystem (no live ECS connection required).
    /// </summary>
    public sealed partial class DebugPanelViewModel : ObservableObject
    {
        private readonly IPackDataService _packDataService;
        private readonly AppConfigService _configService;

        [ObservableProperty]
        public partial ObservableCollection<DebugSectionViewModel> Sections { get; set; } =
            new ObservableCollection<DebugSectionViewModel>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        /// <summary>Initializes a new instance of <see cref="DebugPanelViewModel"/>.</summary>
        public DebugPanelViewModel(IPackDataService packDataService, AppConfigService configService)
        {
            _packDataService = packDataService ?? throw new ArgumentNullException(nameof(packDataService));
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
            BuildPlaceholderSections();
        }

        /// <summary>Refreshes all debug sections from filesystem data.</summary>
        [RelayCommand]
        public async Task RefreshAsync()
        {
            IsLoading = true;
            try
            {
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                LoadResultViewModel result = string.IsNullOrEmpty(config.PacksDirectory)
                    ? new LoadResultViewModel { ErrorCount = 1, Errors = new[] { "Packs directory not configured" } }
                    : await _packDataService.LoadPacksAsync(config.PacksDirectory).ConfigureAwait(true);

                Sections.Clear();

                // 1 — Platform Status
                DebugSectionViewModel platformSection = new DebugSectionViewModel { SectionName = "Platform Status" };
                platformSection.Lines.Add($"Initialized: true (companion mode)");
                platformSection.Lines.Add($"World Ready: false (no live game)");
                platformSection.Lines.Add($"Packs Dir: {TruncatePath(config.PacksDirectory, 50)}");
                platformSection.Lines.Add($"Load Errors: {result.ErrorCount}");
                Sections.Add(platformSection);

                // 2 — ECS Info (static — no live connection)
                DebugSectionViewModel ecsSection = new DebugSectionViewModel { SectionName = "ECS Info" };
                ecsSection.Lines.Add("(Companion mode — game not running)");
                ecsSection.Lines.Add("Launch game with DINOForge installed to see live ECS data.");
                ecsSection.Lines.Add("Known ECS worlds when connected: Default, Streaming, Presentation, …");
                Sections.Add(ecsSection);

                // 3 — Pack Info
                DebugSectionViewModel packSection = new DebugSectionViewModel { SectionName = "Pack Info" };
                packSection.Lines.Add($"Discovered: {result.Packs.Count} pack(s)");
                packSection.Lines.Add($"Loaded OK:  {result.LoadedCount}");
                packSection.Lines.Add($"Errors:     {result.ErrorCount}");
                foreach (PackViewModel pack in result.Packs)
                {
                    packSection.Lines.Add($"  [{(pack.Enabled ? "ON" : "OFF")}] {pack.Id} v{pack.Version} ({pack.Type})");
                }
                Sections.Add(packSection);

                // 4 — System State
                DebugSectionViewModel sysSection = new DebugSectionViewModel { SectionName = "System State" };
                sysSection.Lines.Add($"App version: {typeof(DebugPanelViewModel).Assembly.GetName().Version}");
                sysSection.Lines.Add($"OS: {Environment.OSVersion}");
                sysSection.Lines.Add($"Runtime: {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}");
                sysSection.Lines.Add($"Config file: AppConfig.json");
                Sections.Add(sysSection);

                // 5 — Errors
                DebugSectionViewModel errSection = new DebugSectionViewModel { SectionName = $"Errors ({result.ErrorCount})" };
                if (result.ErrorCount == 0)
                {
                    errSection.Lines.Add("(No errors)");
                }
                else
                {
                    foreach (string err in result.Errors)
                    {
                        errSection.Lines.Add($"  • {err}");
                    }
                }
                Sections.Add(errSection);
            }
            finally
            {
                IsLoading = false;
            }
        }

        private void BuildPlaceholderSections()
        {
            string[] names = { "Platform Status", "ECS Info", "Pack Info", "System State", "Errors (0)" };
            foreach (string name in names)
            {
                DebugSectionViewModel section = new DebugSectionViewModel { SectionName = name };
                section.Lines.Add("Click Refresh to load data…");
                Sections.Add(section);
            }
        }

        private static string TruncatePath(string? path, int maxLen)
        {
            if (string.IsNullOrEmpty(path)) return "(none)";
            return path.Length <= maxLen ? path : "…" + path.Substring(path.Length - maxLen);
        }
    }
}
