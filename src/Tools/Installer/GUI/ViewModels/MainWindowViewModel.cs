using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.Installer.Services;
using System;
using System.ComponentModel;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// Represents the wizard page currently displayed.
/// </summary>
public enum WizardPage
{
    /// <summary>Welcome / mode selection screen.</summary>
    Welcome = 0,
    /// <summary>Game path detection screen.</summary>
    GamePath = 1,
    /// <summary>Installation options screen.</summary>
    Options = 2,
    /// <summary>Progress / completion screen.</summary>
    Progress = 3,
    /// <summary>Maintenance mode (Repair / Update / Uninstall) shown when already installed.</summary>
    Maintenance = 4
}

/// <summary>
/// Root ViewModel for the installer main window.
/// Acts as a wizard state machine, hosting the four page ViewModels and
/// managing navigation between them.
/// On startup, auto-detects the game path and — if DINOForge is already installed —
/// routes directly to the Maintenance page instead of the normal install wizard.
/// </summary>
public partial class MainWindowViewModel : ObservableObject, IDisposable
{
    private readonly WelcomePageViewModel _welcomeVm;
    private readonly GamePathPageViewModel _gamePathVm;
    private readonly OptionsPageViewModel _optionsVm;
    private readonly ProgressPageViewModel _progressVm;
    private readonly MaintenancePageViewModel _maintenanceVm;
    private bool _disposed;

    [ObservableProperty]
    private WizardPage _currentPage = WizardPage.Welcome;

    [ObservableProperty]
    private ObservableObject _currentPageViewModel;

    /// <summary>
    /// Initializes child ViewModels and wires navigation events.
    /// When an existing installation is detected via auto-detected game path,
    /// the wizard starts on the Maintenance page instead of Welcome.
    /// </summary>
    public MainWindowViewModel()
    {
        _welcomeVm = new WelcomePageViewModel();
        _gamePathVm = new GamePathPageViewModel();
        _optionsVm = new OptionsPageViewModel();
        _progressVm = new ProgressPageViewModel();
        _maintenanceVm = new MaintenancePageViewModel();

        _currentPageViewModel = _welcomeVm;

        // Wire up welcome page navigation (Pattern #105: named handlers so Dispose
        // can unsubscribe by reference; lambda captures cannot be unsubscribed).
        _welcomeVm.PlayerModeSelected += OnWelcomePlayerModeSelected;
        _welcomeVm.DevModeSelected += OnWelcomeDevModeSelected;

        // Re-evaluate CanGoNext whenever GamePath changes so the Next button
        // enables/disables live as the user types or auto-detect resolves.
        // Pattern #105: named handler so Dispose can unsubscribe by reference.
        _gamePathVm.PropertyChanged += OnGamePathPropertyChanged;

        // Auto-detect on first visit to game path page
        _gamePathVm.AutoDetect();

        // Wire maintenance page events (Pattern #105: stored references for Dispose).
        _maintenanceVm.RepairSelected += OnRepairSelected;
        _maintenanceVm.UpdateSelected += OnUpdateSelected;
        _maintenanceVm.UninstallSelected += OnUninstallSelected;
        _maintenanceVm.Cancelled += OnMaintenanceCancelled;

        // After auto-detect, check if DINOForge is already present
        CheckAndOfferMaintenance(_gamePathVm.GamePath);
    }

    /// <summary>
    /// Named handler for <see cref="_gamePathVm"/> PropertyChanged so it can be
    /// unsubscribed by reference in <see cref="Dispose"/> (Pattern #105 — event
    /// subscription lifecycle symmetry; lambda captures cannot be unsubscribed).
    /// </summary>
    private void OnGamePathPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(GamePathPageViewModel.GamePath)
                           or nameof(GamePathPageViewModel.IsPathValid)
                           or nameof(GamePathPageViewModel.GameExeOk))
        {
            OnPropertyChanged(nameof(CanGoNext));
            GoNextCommand.NotifyCanExecuteChanged();

            // If the user changes the path, re-check for existing installation
            CheckAndOfferMaintenance(_gamePathVm.GamePath);
        }
    }

    /// <summary>
    /// Named handler for <see cref="WelcomePageViewModel.PlayerModeSelected"/>
    /// (Pattern #105 — stored delegate reference so Dispose can unsubscribe).
    /// </summary>
    private void OnWelcomePlayerModeSelected()
    {
        _optionsVm.IsDevMode = false;
        NavigateTo(WizardPage.GamePath);
    }

    /// <summary>
    /// Named handler for <see cref="WelcomePageViewModel.DevModeSelected"/>
    /// (Pattern #105 — stored delegate reference so Dispose can unsubscribe).
    /// </summary>
    private void OnWelcomeDevModeSelected()
    {
        _optionsVm.IsDevMode = true;
        NavigateTo(WizardPage.GamePath);
    }

    /// <summary>
    /// Named handler for <see cref="MaintenancePageViewModel.Cancelled"/> so it
    /// can be unsubscribed by reference in <see cref="Dispose"/>.
    /// Uses Avalonia's <see cref="IClassicDesktopStyleApplicationLifetime.Shutdown"/>
    /// for an orderly app exit instead of <c>Environment.Exit(0)</c>, which bypasses
    /// finalizers and Avalonia teardown.
    /// </summary>
    private void OnMaintenanceCancelled()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.Shutdown(0);
            return;
        }

        // Fallback for non-desktop lifetimes (e.g. headless tests) — last resort.
        System.Environment.Exit(0);
    }

    /// <summary>
    /// Unsubscribes from all child ViewModel events to prevent leaks (Pattern #105).
    /// </summary>
    public void Dispose()
    {
        if (_disposed) return;

        _welcomeVm.PlayerModeSelected -= OnWelcomePlayerModeSelected;
        _welcomeVm.DevModeSelected -= OnWelcomeDevModeSelected;
        _gamePathVm.PropertyChanged -= OnGamePathPropertyChanged;
        _maintenanceVm.RepairSelected -= OnRepairSelected;
        _maintenanceVm.UpdateSelected -= OnUpdateSelected;
        _maintenanceVm.UninstallSelected -= OnUninstallSelected;
        _maintenanceVm.Cancelled -= OnMaintenanceCancelled;

        _disposed = true;
        GC.SuppressFinalize(this);
    }

    /// <summary>
    /// Checks whether DINOForge is already installed at the given game path.
    /// If so, populates the MaintenanceViewModel and navigates to the Maintenance page.
    /// </summary>
    private void CheckAndOfferMaintenance(string gamePath)
    {
        if (string.IsNullOrWhiteSpace(gamePath)) return;
        if (!InstallDetector.IsInstalled(gamePath)) return;

        _maintenanceVm.GamePath = gamePath;
        _maintenanceVm.InstalledVersion = InstallDetector.GetInstalledVersion(gamePath);
        _maintenanceVm.CurrentVersion = InstallDetector.GetCurrentVersion();
        _maintenanceVm.RefreshDerivedProperties();

        NavigateTo(WizardPage.Maintenance);
    }

    private void OnRepairSelected()
    {
        // Build repair options: same as a fresh install using the existing game path,
        // dev mode defaults to false (user can't change it from maintenance screen).
        InstallOptions opts = new InstallOptions
        {
            GamePath = _maintenanceVm.GamePath,
            IsDevMode = false,
            InstallExamplePacks = false,
            CreateDesktopShortcut = false,
            InstallSdkHeaders = false,
            InstallPackCompiler = false,
            InstallSchemas = false,
            InstallDebugTools = false
        };
        NavigateTo(WizardPage.Progress);
        _ = _progressVm.RunRepairAsync(opts);
    }

    private void OnUpdateSelected()
    {
        // Update = same as repair but confirms version bump via the version file
        InstallOptions opts = new InstallOptions
        {
            GamePath = _maintenanceVm.GamePath,
            IsDevMode = false,
            InstallExamplePacks = false,
            CreateDesktopShortcut = false,
            InstallSdkHeaders = false,
            InstallPackCompiler = false,
            InstallSchemas = false,
            InstallDebugTools = false
        };
        NavigateTo(WizardPage.Progress);
        _ = _progressVm.RunRepairAsync(opts);
    }

    private void OnUninstallSelected()
    {
        UninstallOptions opts = new UninstallOptions
        {
            GamePath = _maintenanceVm.GamePath,
            RemovePacks = true,
            RemoveDumps = true,
            RemoveDevAssets = true
        };
        NavigateTo(WizardPage.Progress);
        _ = _progressVm.RunUninstallAsync(opts);
    }

    /// <summary>
    /// Exposes the Welcome page ViewModel (used by WelcomePage view).
    /// </summary>
    public WelcomePageViewModel WelcomeVm => _welcomeVm;

    /// <summary>
    /// Exposes the GamePath page ViewModel.
    /// </summary>
    public GamePathPageViewModel GamePathVm => _gamePathVm;

    /// <summary>
    /// Exposes the Options page ViewModel.
    /// </summary>
    public OptionsPageViewModel OptionsVm => _optionsVm;

    /// <summary>
    /// Exposes the Progress page ViewModel.
    /// </summary>
    public ProgressPageViewModel ProgressVm => _progressVm;

    /// <summary>
    /// Exposes the Maintenance page ViewModel.
    /// </summary>
    public MaintenancePageViewModel MaintenanceVm => _maintenanceVm;

    /// <summary>
    /// Whether the bottom navigation bar should be shown at all.
    /// Hidden on Welcome (uses its own big mode buttons), Maintenance, and Progress pages.
    /// </summary>
    public bool ShowNavBar =>
        CurrentPage != WizardPage.Welcome &&
        CurrentPage != WizardPage.Progress &&
        CurrentPage != WizardPage.Maintenance;

    /// <summary>
    /// Whether the Back navigation button should be visible.
    /// Not shown on the Welcome, Progress, or Maintenance pages.
    /// </summary>
    public bool CanGoBack =>
        CurrentPage > WizardPage.Welcome &&
        CurrentPage != WizardPage.Progress &&
        CurrentPage != WizardPage.Maintenance;

    /// <summary>
    /// Whether the Next / Install navigation button should be visible and enabled.
    /// Hidden on Progress and Maintenance pages (those use their own action buttons).
    /// On the GamePath page the game exe must be found before proceeding.
    /// </summary>
    public bool CanGoNext
    {
        get
        {
            if (CurrentPage >= WizardPage.Progress) return false;
            if (CurrentPage == WizardPage.Maintenance) return false;
            if (CurrentPage == WizardPage.GamePath)
                return _gamePathVm.IsPathValid && _gamePathVm.GameExeOk;
            return true;
        }
    }

    /// <summary>
    /// Label for the primary action button (changes to "Install" on options page).
    /// </summary>
    public string NextButtonLabel => CurrentPage == WizardPage.Options ? "Install" : "Next";

    /// <summary>
    /// Navigates to the previous wizard page.
    /// </summary>
    [RelayCommand]
    public void GoBack()
    {
        if (CurrentPage > WizardPage.Welcome)
            NavigateTo(CurrentPage - 1);
    }

    /// <summary>
    /// Navigates to the next wizard page, or triggers install on the options page.
    /// Synchronous navigation uses a plain RelayCommand so the button is never
    /// disabled mid-navigation. Install uses a fire-and-forget Task.
    /// </summary>
    [RelayCommand(CanExecute = nameof(CanGoNext))]
    public void GoNext()
    {
        if (CurrentPage == WizardPage.Options)
        {
            NavigateTo(WizardPage.Progress);
            InstallOptions opts = BuildInstallOptions();
            // Fire-and-forget: progress page handles completion via its own state
            _ = _progressVm.RunInstallAsync(opts);
        }
        else if (CurrentPage < WizardPage.Progress)
        {
            NavigateTo(CurrentPage + 1);
        }
    }

    /// <summary>
    /// Navigates directly to a specific wizard page.
    /// </summary>
    private void NavigateTo(WizardPage page)
    {
        CurrentPage = page;
        CurrentPageViewModel = page switch
        {
            WizardPage.Welcome => _welcomeVm,
            WizardPage.GamePath => _gamePathVm,
            WizardPage.Options => _optionsVm,
            WizardPage.Progress => _progressVm,
            WizardPage.Maintenance => _maintenanceVm,
            _ => _welcomeVm
        };

        OnPropertyChanged(nameof(ShowNavBar));
        OnPropertyChanged(nameof(CanGoBack));
        OnPropertyChanged(nameof(CanGoNext));
        OnPropertyChanged(nameof(NextButtonLabel));
        GoBackCommand.NotifyCanExecuteChanged();
        GoNextCommand.NotifyCanExecuteChanged();
    }

    /// <summary>
    /// Builds <see cref="InstallOptions"/> from the current ViewModel state.
    /// </summary>
    private InstallOptions BuildInstallOptions()
    {
        return new InstallOptions
        {
            GamePath = _gamePathVm.GamePath,
            IsDevMode = _optionsVm.IsDevMode,
            InstallExamplePacks = _optionsVm.InstallExamplePacks,
            CreateDesktopShortcut = _optionsVm.CreateDesktopShortcut,
            InstallSdkHeaders = _optionsVm.InstallSdkHeaders,
            InstallPackCompiler = _optionsVm.InstallPackCompiler,
            InstallSchemas = _optionsVm.InstallSchemas,
            InstallDebugTools = _optionsVm.InstallDebugTools
        };
    }
}
