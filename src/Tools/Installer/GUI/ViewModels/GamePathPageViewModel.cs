using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.Tools.Installer;
using System.IO;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// ViewModel for the Game Path detection wizard page.
/// Uses <see cref="SteamLocator"/> to auto-detect the DINO install directory,
/// and exposes validation status for each expected component.
/// </summary>
public partial class GamePathPageViewModel : ObservableObject
{
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsPathValid))]
    [NotifyPropertyChangedFor(nameof(GameExeStatus))]
    [NotifyPropertyChangedFor(nameof(BepInExStatus))]
    private string _gamePath = string.Empty;

    [ObservableProperty]
    private bool _isAutoDetected;

    /// <summary>
    /// Gets whether the current game path passes basic validation.
    /// </summary>
    public bool IsPathValid => !string.IsNullOrEmpty(GamePath) && Directory.Exists(GamePath);

    /// <summary>
    /// Gets a status string for the game executable check.
    /// </summary>
    public string GameExeStatus
    {
        get
        {
            if (!IsPathValid) return "?";
            bool found = File.Exists(Path.Combine(GamePath, "Diplomacy is Not an Option.exe"))
                      || File.Exists(Path.Combine(GamePath, "Diplomacy is Not an Option.x86_64"));
            return found ? "Found" : "Not found";
        }
    }

    /// <summary>
    /// Gets a status string for the BepInEx check.
    /// </summary>
    public string BepInExStatus
    {
        get
        {
            if (!IsPathValid) return "?";
            bool found = Directory.Exists(Path.Combine(GamePath, "BepInEx"));
            return found ? "Installed" : "Not installed (will be downloaded)";
        }
    }

    /// <summary>
    /// Whether the game exe status is positive.
    /// </summary>
    public bool GameExeOk => GameExeStatus.StartsWith("Found");

    /// <summary>
    /// Whether the BepInEx status is positive.
    /// </summary>
    public bool BepInExOk => BepInExStatus.StartsWith("Installed");

    /// <summary>
    /// Delegate for opening a folder picker dialog (injected by the View).
    /// </summary>
    public System.Func<System.Threading.Tasks.Task<string?>>? BrowseDialogOpener { get; set; }

    /// <summary>
    /// Attempts to auto-detect the DINO install path via Steam.
    /// </summary>
    public void AutoDetect()
    {
        string? found = SteamLocator.FindDinoInstallPath();
        if (found is not null)
        {
            GamePath = found;
            IsAutoDetected = true;
        }
    }

    /// <summary>
    /// Opens the folder browser dialog (via the injected opener) so the user
    /// can manually select the game directory.
    /// </summary>
    [RelayCommand]
    public async System.Threading.Tasks.Task BrowseAsync()
    {
        if (BrowseDialogOpener is null) return;
        string? result = await BrowseDialogOpener().ConfigureAwait(false);
        if (result is not null)
        {
            GamePath = result;
            IsAutoDetected = false;
            OnPropertyChanged(nameof(IsPathValid));
            OnPropertyChanged(nameof(GameExeStatus));
            OnPropertyChanged(nameof(BepInExStatus));
            OnPropertyChanged(nameof(GameExeOk));
            OnPropertyChanged(nameof(BepInExOk));
        }
    }
}
