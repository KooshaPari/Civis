using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.DesktopCompanion.Data;

namespace DINOForge.DesktopCompanion.ViewModels
{
    /// <summary>
    /// View-model for the Asset Browser page.
    /// Discovers and displays asset bundles across all installed packs.
    /// </summary>
    public sealed partial class AssetBrowserViewModel : ObservableObject
    {
        private readonly AppConfigService _configService;
        private string _packsDirectory = "";

        [ObservableProperty]
        public partial ObservableCollection<PackAssetGroup> AssetGroups { get; set; } = new ObservableCollection<PackAssetGroup>();

        [ObservableProperty]
        public partial bool IsLoading { get; set; }

        [ObservableProperty]
        public partial string StatusMessage { get; set; } = "";

        [ObservableProperty]
        public partial PackAssetGroup? SelectedGroup { get; set; }

        [ObservableProperty]
        public partial BundleEntry? SelectedBundle { get; set; }

        [ObservableProperty]
        public partial string SearchText { get; set; } = "";

        /// <summary>Initializes a new instance of <see cref="AssetBrowserViewModel"/>.</summary>
        public AssetBrowserViewModel(AppConfigService configService)
        {
            _configService = configService ?? throw new ArgumentNullException(nameof(configService));
        }

        /// <summary>Loads asset bundles from all packs in the configured packs directory.</summary>
        [RelayCommand]
        public async Task ReloadAsync()
        {
            IsLoading = true;
            StatusMessage = "Scanning packs for assets…";

            try
            {
                AppConfig config = await _configService.LoadAsync().ConfigureAwait(true);
                _packsDirectory = config.PacksDirectory;

                if (string.IsNullOrEmpty(_packsDirectory))
                {
                    StatusMessage = "Packs directory not configured — go to Settings.";
                    AssetGroups.Clear();
                    return;
                }

                AssetGroups.Clear();

                var packDirs = Directory.EnumerateDirectories(_packsDirectory);
                int groupsFound = 0;
                int bundlesFound = 0;

                foreach (string packDir in packDirs)
                {
                    string packName = Path.GetFileName(packDir);
                    string assetsDir = Path.Combine(packDir, "assets");
                    string bundlesDir = Path.Combine(assetsDir, "bundles");

                    if (!Directory.Exists(bundlesDir))
                        continue;

                    var group = new PackAssetGroup
                    {
                        PackId = packName,
                        PackName = packName,
                        PackPath = packDir
                    };

                    // Try to read pack.yaml to get the human-readable name and version
                    string packYamlPath = Path.Combine(packDir, "pack.yaml");
                    if (File.Exists(packYamlPath))
                    {
                        try
                        {
                            string yamlContent = File.ReadAllText(packYamlPath);
                            // Simple extraction of name and version fields
                            var nameMatch = System.Text.RegularExpressions.Regex.Match(yamlContent, @"name:\s*(.+?)(?:\n|$)");
                            if (nameMatch.Success)
                                group.PackName = nameMatch.Groups[1].Value.Trim();

                            var versionMatch = System.Text.RegularExpressions.Regex.Match(yamlContent, @"version:\s*(.+?)(?:\n|$)");
                            if (versionMatch.Success)
                                group.PackVersion = versionMatch.Groups[1].Value.Trim();
                        }
                        catch
                        {
                            // Ignore YAML parsing errors; use defaults
                        }
                    }

                    // Scan bundles
                    var bundleFiles = Directory.EnumerateFiles(bundlesDir)
                        .Where(f => !f.EndsWith(".manifest", StringComparison.OrdinalIgnoreCase))
                        .ToList();

                    foreach (string bundleFile in bundleFiles)
                    {
                        string fileName = Path.GetFileName(bundleFile);
                        var fileInfo = new FileInfo(bundleFile);
                        string manifestPath = bundleFile + ".manifest";

                        var entry = new BundleEntry
                        {
                            FileName = fileName,
                            SizeBytes = fileInfo.Length,
                            AssetCount = File.Exists(manifestPath) ? 1 : 0, // Simplified; real count would require reading manifest
                            ManifestPath = File.Exists(manifestPath) ? manifestPath : null,
                            FullPath = bundleFile
                        };

                        group.Bundles.Add(entry);
                        bundlesFound++;
                    }

                    if (group.Bundles.Count > 0)
                    {
                        AssetGroups.Add(group);
                        groupsFound++;
                    }
                }

                StatusMessage = groupsFound > 0
                    ? $"{groupsFound} pack(s) with assets • {bundlesFound} bundle(s) found"
                    : "No asset bundles found.";
            }
            catch (Exception ex)
            {
                StatusMessage = $"Error loading assets: {ex.Message}";
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>Filters asset groups by search text (pack name or bundle file name).</summary>
        [RelayCommand]
        public void FilterAssets(string? searchText)
        {
            SearchText = searchText ?? "";
            // In a real implementation, this would filter AssetGroups or provide a filtered collection.
            // For simplicity, the UI can bind directly to AssetGroups and implement filtering in the DataTemplate or View logic.
        }
    }
}
