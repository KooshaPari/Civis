using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.Tools.Installer;
using System;
using System.Threading.Tasks;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// ViewModel for the Welcome wizard page.
/// Handles install-mode selection and update banner display.
/// </summary>
public partial class WelcomePageViewModel : ObservableObject
{
    private readonly UpdateChecker _updateChecker = new UpdateChecker();

    /// <inheritdoc cref="InstallMode"/>
    public enum InstallMode { NotSelected, Player, Developer }

    [ObservableProperty]
    private InstallMode _selectedMode = InstallMode.NotSelected;

    [ObservableProperty]
    private bool _hasUpdate;

    [ObservableProperty]
    private string _updateVersion = string.Empty;

    [ObservableProperty]
    private string _updateUrl = string.Empty;

    [ObservableProperty]
    private bool _isCheckingUpdate;

    [ObservableProperty]
    private string _currentVersion = "0.5.0";

    /// <summary>
    /// Fires when the user wants to proceed as a player.
    /// </summary>
    public event Action? PlayerModeSelected;

    /// <summary>
    /// Fires when the user wants to proceed as a developer.
    /// </summary>
    public event Action? DevModeSelected;

    /// <summary>
    /// Kicks off an update check and populates the update banner if needed.
    /// </summary>
    [RelayCommand]
    public async Task CheckForUpdatesAsync()
    {
        IsCheckingUpdate = true;
        HasUpdate = false;

        try
        {
            UpdateInfo info = await _updateChecker.CheckAsync().ConfigureAwait(false);
            CurrentVersion = info.CurrentVersion;

            if (info.HasUpdate)
            {
                HasUpdate = true;
                UpdateVersion = info.LatestVersion;
                UpdateUrl = info.ReleaseUrl;
            }
        }
        finally
        {
            IsCheckingUpdate = false;
        }
    }

    /// <summary>
    /// Opens the update release page in the default browser.
    /// </summary>
    [RelayCommand]
    public void OpenUpdateUrl()
    {
        if (!string.IsNullOrEmpty(UpdateUrl))
        {
            using var _ = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = UpdateUrl,
                UseShellExecute = true
            });
        }
    }

    /// <summary>
    /// Selects Player mode and fires navigation event.
    /// </summary>
    [RelayCommand]
    public void SelectPlayerMode()
    {
        SelectedMode = InstallMode.Player;
        PlayerModeSelected?.Invoke();
    }

    /// <summary>
    /// Selects Developer mode and fires navigation event.
    /// </summary>
    [RelayCommand]
    public void SelectDevMode()
    {
        SelectedMode = InstallMode.Developer;
        DevModeSelected?.Invoke();
    }
}
