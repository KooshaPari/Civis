using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DINOForge.Installer.Services;
using DINOForge.Tools.Installer;
using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// Installation phase displayed on the progress page.
/// </summary>
public enum InstallPhase
{
    /// <summary>Not yet started.</summary>
    Idle,
    /// <summary>Installation in progress.</summary>
    Running,
    /// <summary>Installation succeeded.</summary>
    Success,
    /// <summary>Installation failed with an error.</summary>
    Error
}

/// <summary>
/// ViewModel for the Progress / Completion wizard page.
/// Drives the installation sequence and surfaces log output.
/// </summary>
public partial class ProgressPageViewModel : ObservableObject
{
    private readonly InstallerService _service = new InstallerService();
    private CancellationTokenSource? _cts;

    [ObservableProperty]
    private InstallPhase _phase = InstallPhase.Idle;

    [ObservableProperty]
    private bool _isIndeterminate = true;

    [ObservableProperty]
    private double _progressValue;

    [ObservableProperty]
    private string _statusText = "Ready to install.";

    [ObservableProperty]
    private string _errorMessage = string.Empty;

    [ObservableProperty]
    private bool _hasError;

    [ObservableProperty]
    private string _gamePath = string.Empty;

    /// <summary>
    /// Scrollable log lines surfaced to the View.
    /// </summary>
    public ObservableCollection<string> LogLines { get; } = new ObservableCollection<string>();

    /// <summary>
    /// Whether to show the post-install action buttons.
    /// </summary>
    public bool ShowCompletionButtons => Phase is InstallPhase.Success or InstallPhase.Error;

    /// <summary>
    /// Starts the install sequence with the provided options.
    /// </summary>
    /// <param name="options">Install options.</param>
    public async Task RunInstallAsync(InstallOptions options)
    {
        _cts = new CancellationTokenSource();
        Phase = InstallPhase.Running;
        HasError = false;
        ErrorMessage = string.Empty;
        LogLines.Clear();
        StatusText = "Installing DINOForge...";
        GamePath = options.GamePath;

        Progress<string> progress = new Progress<string>(line =>
        {
            LogLines.Add(line);
        });

        try
        {
            InstallStatus status = await _service.InstallAsync(options, progress, _cts.Token).ConfigureAwait(false);

            if (status.IsFullyInstalled)
            {
                Phase = InstallPhase.Success;
                StatusText = "Installation complete!";
            }
            else
            {
                Phase = InstallPhase.Error;
                StatusText = "Installation finished with issues.";
                HasError = true;
                ErrorMessage = "Some components could not be verified. See log for details.";
            }
        }
        catch (OperationCanceledException)
        {
            Phase = InstallPhase.Error;
            StatusText = "Installation was cancelled.";
            HasError = true;
            ErrorMessage = "The installation was cancelled by the user.";
        }
        catch (FileNotFoundException ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Installation failed — installer package incomplete.";
            HasError = true;
            ErrorMessage = ex.Message;
            LogLines.Add($"ERROR: {ex.Message}");
        }
        catch (Exception ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Installation failed.";
            HasError = true;
            ErrorMessage = ex.Message;
            LogLines.Add($"ERROR: {ex.Message}");
        }
        finally
        {
            IsIndeterminate = false;
            ProgressValue = 100;
            OnPropertyChanged(nameof(ShowCompletionButtons));
        }
    }

    /// <summary>
    /// Runs the repair sequence: identical to a fresh install but always overwrites existing files.
    /// </summary>
    /// <param name="options">Install options populated with the existing game path.</param>
    public async Task RunRepairAsync(InstallOptions options)
    {
        _cts = new CancellationTokenSource();
        Phase = InstallPhase.Running;
        HasError = false;
        ErrorMessage = string.Empty;
        LogLines.Clear();
        StatusText = "Repairing DINOForge...";
        GamePath = options.GamePath;

        IProgress<string> progress = new Progress<string>(line => LogLines.Add(line));

        try
        {
            progress.Report("Repair mode: re-copying all DINOForge files...");
            InstallStatus status = await _service.InstallAsync(options, progress, _cts.Token).ConfigureAwait(false);

            if (status.IsFullyInstalled)
            {
                Phase = InstallPhase.Success;
                StatusText = "Repair complete!";
            }
            else
            {
                Phase = InstallPhase.Error;
                StatusText = "Repair finished with issues.";
                HasError = true;
                ErrorMessage = "Some components could not be verified after repair. See log for details.";
            }
        }
        catch (OperationCanceledException)
        {
            Phase = InstallPhase.Error;
            StatusText = "Repair was cancelled.";
            HasError = true;
            ErrorMessage = "The repair was cancelled by the user.";
        }
        catch (FileNotFoundException ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Repair failed — installer package incomplete.";
            HasError = true;
            ErrorMessage = ex.Message;
            LogLines.Add($"ERROR: {ex.Message}");
        }
        catch (IOException ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Repair failed.";
            HasError = true;
            ErrorMessage = $"Could not complete repair: {ex.Message}. Try running as Administrator.";
            LogLines.Add($"ERROR: {ex.Message}");
        }
        catch (Exception ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Repair failed.";
            HasError = true;
            ErrorMessage = ex.Message;
            LogLines.Add($"ERROR: {ex.Message}");
        }
        finally
        {
            IsIndeterminate = false;
            ProgressValue = 100;
            OnPropertyChanged(nameof(ShowCompletionButtons));
        }
    }

    /// <summary>
    /// Runs the uninstall sequence.
    /// </summary>
    /// <param name="options">Uninstall options.</param>
    public async Task RunUninstallAsync(UninstallOptions options)
    {
        _cts = new CancellationTokenSource();
        Phase = InstallPhase.Running;
        HasError = false;
        ErrorMessage = string.Empty;
        LogLines.Clear();
        StatusText = "Uninstalling DINOForge...";
        GamePath = options.GamePath;

        IProgress<string> progress = new Progress<string>(line => LogLines.Add(line));

        try
        {
            bool ok = await _service.UninstallAsync(options, progress, _cts.Token).ConfigureAwait(false);

            if (ok)
            {
                Phase = InstallPhase.Success;
                StatusText = "Uninstall complete.";
            }
            else
            {
                Phase = InstallPhase.Error;
                StatusText = "Uninstall finished with errors.";
                HasError = true;
                ErrorMessage = "Some files could not be removed. Try running as Administrator.";
            }
        }
        catch (OperationCanceledException)
        {
            Phase = InstallPhase.Error;
            StatusText = "Uninstall was cancelled.";
            HasError = true;
            ErrorMessage = "The uninstall was cancelled by the user.";
        }
        catch (IOException ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Uninstall failed.";
            HasError = true;
            ErrorMessage = $"Could not complete uninstall: {ex.Message}. Try running as Administrator.";
            LogLines.Add($"ERROR: {ex.Message}");
        }
        catch (Exception ex)
        {
            Phase = InstallPhase.Error;
            StatusText = "Uninstall failed.";
            HasError = true;
            ErrorMessage = ex.Message;
            LogLines.Add($"ERROR: {ex.Message}");
        }
        finally
        {
            IsIndeterminate = false;
            ProgressValue = 100;
            OnPropertyChanged(nameof(ShowCompletionButtons));
        }
    }

    /// <summary>
    /// Launches the game executable.
    /// </summary>
    [RelayCommand]
    public void LaunchGame()
    {
        string exe = Path.Combine(GamePath, "Diplomacy is Not an Option.exe");
        if (File.Exists(exe))
        {
            Process.Start(new ProcessStartInfo { FileName = exe, UseShellExecute = true });
        }
    }

    /// <summary>
    /// Opens the game install folder in Windows Explorer.
    /// </summary>
    [RelayCommand]
    public void OpenInstallFolder()
    {
        if (Directory.Exists(GamePath))
        {
            Process.Start(new ProcessStartInfo { FileName = GamePath, UseShellExecute = true });
        }
    }

    /// <summary>
    /// Cancels an in-progress installation.
    /// </summary>
    [RelayCommand]
    public void Cancel()
    {
        _cts?.Cancel();
    }
}
