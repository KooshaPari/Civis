using CommunityToolkit.Mvvm.ComponentModel;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// ViewModel for the Options wizard page.
/// Exposes checkboxes for player and developer installation options.
/// </summary>
public partial class OptionsPageViewModel : ObservableObject
{
    /// <summary>
    /// Whether this install is in developer mode (shows dev-specific options).
    /// </summary>
    [ObservableProperty]
    private bool _isDevMode;

    // ── Player options ────────────────────────────────────────────────────────

    /// <summary>Whether to copy example packs to the packs directory.</summary>
    [ObservableProperty]
    private bool _installExamplePacks = true;

    /// <summary>Whether to create a Windows desktop shortcut for the game.</summary>
    [ObservableProperty]
    private bool _createDesktopShortcut = true;

    // ── Developer options ─────────────────────────────────────────────────────

    /// <summary>Whether to install SDK header files (dev mode).</summary>
    [ObservableProperty]
    private bool _installSdkHeaders = true;

    /// <summary>Whether to install the PackCompiler CLI tool (dev mode).</summary>
    [ObservableProperty]
    private bool _installPackCompiler = true;

    /// <summary>Whether to install the JSON schema files (dev mode).</summary>
    [ObservableProperty]
    private bool _installSchemas = true;

    /// <summary>Whether to install debug/inspection tools (dev mode).</summary>
    [ObservableProperty]
    private bool _installDebugTools;

    /// <summary>
    /// BepInEx version that will be installed or detected.
    /// </summary>
    [ObservableProperty]
    private string _bepInExVersion = "5.4.23.2 (x64)";
}
