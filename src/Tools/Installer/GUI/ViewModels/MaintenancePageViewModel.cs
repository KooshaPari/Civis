using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// The maintenance action the user has chosen when DINOForge is already installed.
/// </summary>
public enum MaintenanceAction
{
    /// <summary>No action selected yet.</summary>
    None,
    /// <summary>Repair: re-copy files, restore missing components.</summary>
    Repair,
    /// <summary>Update: install newer version over existing.</summary>
    Update,
    /// <summary>Uninstall: remove all DINOForge files.</summary>
    Uninstall
}

/// <summary>
/// ViewModel for the Maintenance mode selection screen.
/// Shown when DINOForge is already installed; lets the user choose
/// Repair, Update, or Uninstall instead of running the full wizard.
/// </summary>
public partial class MaintenancePageViewModel : ObservableObject
{
    [ObservableProperty]
    private string _installedVersion = string.Empty;

    [ObservableProperty]
    private string _currentVersion = "0.5.0";

    [ObservableProperty]
    private string _gamePath = string.Empty;

    /// <summary>Fires when the user chooses Repair.</summary>
    public event Action? RepairSelected;

    /// <summary>Fires when the user chooses Update.</summary>
    public event Action? UpdateSelected;

    /// <summary>Fires when the user chooses Uninstall.</summary>
    public event Action? UninstallSelected;

    /// <summary>Fires when the user cancels and wants to close the installer.</summary>
    public event Action? Cancelled;

    /// <summary>
    /// Whether an update is available (installed version is older than current installer version).
    /// </summary>
    public bool UpdateAvailable
    {
        get
        {
            if (Version.TryParse(CurrentVersion, out Version? current) &&
                Version.TryParse(InstalledVersion, out Version? installed))
            {
                return current > installed;
            }
            return false;
        }
    }

    /// <summary>
    /// Notifies the view that computed properties derived from InstalledVersion/CurrentVersion
    /// should be re-evaluated (e.g., <see cref="UpdateAvailable"/>).
    /// </summary>
    public void RefreshDerivedProperties() => OnPropertyChanged(nameof(UpdateAvailable));

    /// <summary>
    /// Selects the Repair action.
    /// </summary>
    [RelayCommand]
    public void SelectRepair() => RepairSelected?.Invoke();

    /// <summary>
    /// Selects the Update action.
    /// </summary>
    [RelayCommand]
    public void SelectUpdate() => UpdateSelected?.Invoke();

    /// <summary>
    /// Selects the Uninstall action.
    /// </summary>
    [RelayCommand]
    public void SelectUninstall() => UninstallSelected?.Invoke();

    /// <summary>
    /// Cancels maintenance and closes the installer.
    /// </summary>
    [RelayCommand]
    public void Cancel() => Cancelled?.Invoke();
}
