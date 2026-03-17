using DINOForge.Tools.Installer;
using System;
using System.IO;
using System.IO.Compression;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Installer.Services;

/// <summary>
/// Options controlling what the installer should set up.
/// </summary>
public sealed class InstallOptions
{
    /// <summary>
    /// Full path to the DINO game directory.
    /// </summary>
    public required string GamePath { get; init; }

    /// <summary>
    /// When true, SDK headers, schemas, PackCompiler, and debug tools are copied.
    /// </summary>
    public bool IsDevMode { get; init; }

    /// <summary>
    /// When true, example packs are copied to the packs directory.
    /// </summary>
    public bool InstallExamplePacks { get; init; }

    /// <summary>
    /// When true, a desktop shortcut is created for the game.
    /// </summary>
    public bool CreateDesktopShortcut { get; init; }

    /// <summary>When true (dev mode), SDK header files are installed.</summary>
    public bool InstallSdkHeaders { get; init; }

    /// <summary>When true (dev mode), PackCompiler CLI is installed.</summary>
    public bool InstallPackCompiler { get; init; }

    /// <summary>When true (dev mode), JSON schemas are installed.</summary>
    public bool InstallSchemas { get; init; }

    /// <summary>When true (dev mode), debug tools are installed.</summary>
    public bool InstallDebugTools { get; init; }
}

/// <summary>
/// Options controlling what the uninstaller should remove.
/// </summary>
public sealed class UninstallOptions
{
    /// <summary>
    /// Full path to the DINO game directory.
    /// </summary>
    public required string GamePath { get; init; }

    /// <summary>
    /// When true, the dinoforge_packs/ directory is also removed.
    /// </summary>
    public bool RemovePacks { get; init; } = true;

    /// <summary>
    /// When true, the dinoforge_dumps/ directory is also removed.
    /// </summary>
    public bool RemoveDumps { get; init; } = true;

    /// <summary>
    /// When true, the dinoforge_dev/ directory is also removed.
    /// </summary>
    public bool RemoveDevAssets { get; init; } = true;
}

/// <summary>
/// Detects an existing DINOForge installation.
/// </summary>
public sealed class InstallDetector
{
    /// <summary>
    /// Returns true when DINOForge Runtime is already present in the given game directory.
    /// </summary>
    /// <param name="gamePath">Path to the DINO game directory.</param>
    public static bool IsInstalled(string gamePath)
    {
        if (string.IsNullOrWhiteSpace(gamePath)) return false;
        string runtimeDll = Path.Combine(gamePath, "BepInEx", "plugins", "DINOForge.Runtime.dll");
        return File.Exists(runtimeDll);
    }

    /// <summary>
    /// Reads the installed DINOForge version from <c>dinoforge_version.txt</c>
    /// next to the Runtime DLL, or falls back to the DLL's <see cref="FileVersionInfo"/>.
    /// Returns "unknown" if the version cannot be determined.
    /// </summary>
    /// <param name="gamePath">Path to the DINO game directory.</param>
    public static string GetInstalledVersion(string gamePath)
    {
        // Prefer the sidecar version file written during install
        string versionFile = Path.Combine(gamePath, "BepInEx", "plugins", "dinoforge_version.txt");
        if (File.Exists(versionFile))
        {
            try
            {
                string text = File.ReadAllText(versionFile).Trim();
                if (!string.IsNullOrEmpty(text))
                    return text;
            }
            catch { /* fall through */ }
        }

        // Fall back to FileVersionInfo on the DLL
        string runtimeDll = Path.Combine(gamePath, "BepInEx", "plugins", "DINOForge.Runtime.dll");
        if (File.Exists(runtimeDll))
        {
            try
            {
                System.Diagnostics.FileVersionInfo fvi = System.Diagnostics.FileVersionInfo.GetVersionInfo(runtimeDll);
                string? ver = fvi.ProductVersion ?? fvi.FileVersion;
                if (!string.IsNullOrEmpty(ver))
                    return ver;
            }
            catch { /* fall through */ }
        }

        return "unknown";
    }

    /// <summary>
    /// Gets the installer's own (current) version string.
    /// </summary>
    public static string GetCurrentVersion()
    {
        Version? v = Assembly.GetExecutingAssembly().GetName().Version;
        return v is null ? "0.0.0" : $"{v.Major}.{v.Minor}.{v.Build}";
    }
}

/// <summary>
/// Orchestrates the full DINOForge installation sequence.
/// Downloads BepInEx if needed, extracts it, copies DINOForge binaries,
/// and optionally installs dev tooling and example packs.
/// </summary>
public sealed class InstallerService
{
    private const string BepInExDownloadUrl =
        "https://github.com/BepInEx/BepInEx/releases/download/v5.4.23.2/BepInEx_x64_5.4.23.2.0.zip";

    private static readonly HttpClient _http = new HttpClient
    {
        Timeout = TimeSpan.FromMinutes(5)
    };

    /// <summary>
    /// Runs the full installation sequence, reporting progress via <paramref name="log"/>.
    /// </summary>
    /// <param name="options">Installation options.</param>
    /// <param name="log">Progress reporter for log messages.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The final <see cref="InstallStatus"/> after verification.</returns>
    public async Task<InstallStatus> InstallAsync(
        InstallOptions options,
        IProgress<string> log,
        CancellationToken cancellationToken = default)
    {
        string gamePath = options.GamePath;
        string bepInExDir = Path.Combine(gamePath, "BepInEx");
        string pluginsDir = Path.Combine(bepInExDir, "plugins");
        string packsDir = InstallLifecycle.GetPacksDirectory(gamePath);

        // Step 1 — Ensure BepInEx is present
        if (!Directory.Exists(bepInExDir) || !File.Exists(Path.Combine(gamePath, "winhttp.dll")))
        {
            log.Report("BepInEx not found — downloading...");
            await DownloadAndExtractBepInExAsync(gamePath, log, cancellationToken).ConfigureAwait(false);
        }
        else
        {
            log.Report("BepInEx already installed — skipping download.");
        }

        int removedLegacy = InstallLifecycle.CleanupLegacyArtifacts(gamePath, log.Report);
        if (removedLegacy > 0)
        {
            log.Report($"Cleaned {removedLegacy} legacy DINOForge artifact(s).");
        }

        if (InstallLifecycle.MigrateLegacyPacks(gamePath, log.Report))
        {
            log.Report("Migrated packs from legacy root-level directory.");
        }

        // Step 2 — Ensure plugins directory exists
        Directory.CreateDirectory(pluginsDir);
        log.Report("Ensured BepInEx/plugins/ directory exists.");

        // Step 3 — Copy DINOForge runtime binaries (required — abort if not next to installer)
        string baseDir = AppContext.BaseDirectory;
        CopyRequired(Path.Combine(baseDir, "DINOForge.Runtime.dll"), Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), log);
        CopyRequired(Path.Combine(baseDir, "DINOForge.SDK.dll"), Path.Combine(pluginsDir, "DINOForge.SDK.dll"), log);

        // Write version sidecar so future runs can detect the installed version
        string versionFile = Path.Combine(pluginsDir, "dinoforge_version.txt");
        string installerVersion = InstallDetector.GetCurrentVersion();
        try
        {
            await File.WriteAllTextAsync(versionFile, installerVersion, cancellationToken).ConfigureAwait(false);
            log.Report($"Wrote version file: {installerVersion}");
        }
        catch (Exception ex)
        {
            log.Report($"  Warning: could not write version file: {ex.Message}");
        }

        try
        {
            string manifestPath = InstallLifecycle.WriteManifest(gamePath, installerVersion);
            log.Report($"Wrote install manifest: {manifestPath}");
        }
        catch (Exception ex)
        {
            log.Report($"  Warning: could not write install manifest: {ex.Message}");
        }

        // Step 4 — Example packs
        if (options.InstallExamplePacks)
        {
            Directory.CreateDirectory(packsDir);
            log.Report("Installing example packs...");
            string packsSource = Path.Combine(baseDir, "packs");
            if (Directory.Exists(packsSource))
                CopyDirectory(packsSource, packsDir, log);
            else
                log.Report("  Warning: packs/ not found next to installer — skipping.");
        }

        // Step 5 — Dev mode extras
        if (options.IsDevMode)
        {
            string devDir = Path.Combine(gamePath, "dinoforge_dev");
            Directory.CreateDirectory(devDir);

            if (options.InstallSdkHeaders)
            {
                log.Report("Installing SDK headers...");
                string sdkSource = Path.Combine(baseDir, "sdk");
                CopyDirectoryIfExists(sdkSource, Path.Combine(devDir, "sdk"), log);
            }

            if (options.InstallPackCompiler)
            {
                log.Report("Installing PackCompiler CLI...");
                string compilerSource = Path.Combine(baseDir, "PackCompiler");
                CopyDirectoryIfExists(compilerSource, Path.Combine(devDir, "PackCompiler"), log);
            }

            if (options.InstallSchemas)
            {
                log.Report("Installing JSON schemas...");
                string schemasSource = Path.Combine(baseDir, "schemas");
                CopyDirectoryIfExists(schemasSource, Path.Combine(devDir, "schemas"), log);
            }

            if (options.InstallDebugTools)
            {
                log.Report("Installing debug tools...");
                string debugSource = Path.Combine(baseDir, "DebugTools");
                CopyDirectoryIfExists(debugSource, Path.Combine(devDir, "DebugTools"), log);
            }
        }

        // Step 6 — Desktop shortcut
        if (options.CreateDesktopShortcut)
        {
            log.Report("Creating desktop shortcut...");
            TryCreateDesktopShortcut(gamePath, log);
        }

        // Step 7 — Verify installation
        log.Report("Verifying installation...");
        InstallStatus status = InstallVerifier.Verify(gamePath);

        if (status.IsFullyInstalled)
        {
            log.Report("Installation complete! All components verified.");
        }
        else
        {
            log.Report("Installation finished with warnings:");
            foreach (string issue in status.Issues)
                log.Report($"  - {issue}");
        }

        return status;
    }

    /// <summary>
    /// Downloads BepInEx from the hardcoded release URL and extracts it into
    /// the game directory.
    /// </summary>
    private static async Task DownloadAndExtractBepInExAsync(
        string gamePath,
        IProgress<string> log,
        CancellationToken cancellationToken)
    {
        string tmpZip = Path.Combine(Path.GetTempPath(), "BepInEx_x64.zip");

        log.Report($"Downloading BepInEx from {BepInExDownloadUrl}...");
        byte[] data = await _http.GetByteArrayAsync(new Uri(BepInExDownloadUrl), cancellationToken).ConfigureAwait(false);
        await File.WriteAllBytesAsync(tmpZip, data, cancellationToken).ConfigureAwait(false);
        log.Report($"Downloaded {data.Length / 1024} KB — extracting...");

        ZipFile.ExtractToDirectory(tmpZip, gamePath, overwriteFiles: true);
        log.Report("BepInEx extracted.");

        try { File.Delete(tmpZip); } catch { /* cleanup best-effort */ }
    }

    /// <summary>
    /// Copies a required file, throwing <see cref="FileNotFoundException"/> when the source
    /// is absent. Use for core DLLs that must be present for install/repair to succeed.
    /// </summary>
    /// <exception cref="FileNotFoundException">
    /// Thrown when <paramref name="src"/> does not exist next to the installer.
    /// The message includes actionable guidance for the user.
    /// </exception>
    private static void CopyRequired(string src, string dst, IProgress<string> log)
    {
        if (!File.Exists(src))
        {
            string fileName = Path.GetFileName(src);
            string message =
                $"{fileName} was not found next to the installer executable. " +
                $"Re-download the full installer package — the DLL must be in the same folder as this .exe.";
            log.Report($"  ERROR: {message}");
            throw new FileNotFoundException(message, src);
        }

        File.Copy(src, dst, overwrite: true);
        log.Report($"Copied {Path.GetFileName(src)}.");
    }

    /// <summary>
    /// Copies a file if it exists, logging the action.
    /// Use for optional components (packs, dev extras) where absence is acceptable.
    /// </summary>
    private static void CopyIfExists(string src, string dst, IProgress<string> log)
    {
        if (File.Exists(src))
        {
            File.Copy(src, dst, overwrite: true);
            log.Report($"Copied {Path.GetFileName(src)}.");
        }
        else
        {
            log.Report($"  Warning: {Path.GetFileName(src)} not found next to installer — skipping.");
        }
    }

    /// <summary>
    /// Recursively copies a directory, logging individual file copies.
    /// </summary>
    private static void CopyDirectory(string srcDir, string dstDir, IProgress<string> log)
    {
        Directory.CreateDirectory(dstDir);
        foreach (string file in Directory.GetFiles(srcDir, "*", SearchOption.AllDirectories))
        {
            string relative = Path.GetRelativePath(srcDir, file);
            string dest = Path.Combine(dstDir, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(dest)!);
            File.Copy(file, dest, overwrite: true);
            log.Report($"  Copied {relative}");
        }
    }

    /// <summary>
    /// Copies a directory only if it exists; otherwise logs a warning.
    /// </summary>
    private static void CopyDirectoryIfExists(string srcDir, string dstDir, IProgress<string> log)
    {
        if (Directory.Exists(srcDir))
            CopyDirectory(srcDir, dstDir, log);
        else
            log.Report($"  Warning: {Path.GetFileName(srcDir)}/ not found next to installer — skipping.");
    }

    /// <summary>
    /// Removes DINOForge files from the game directory, reporting progress via <paramref name="log"/>.
    /// All file I/O errors are caught and surfaced as log messages rather than thrown,
    /// so the caller can decide how to handle partial uninstalls.
    /// </summary>
    /// <param name="options">Uninstall options.</param>
    /// <param name="log">Progress reporter for log messages.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>True when all targeted files/directories were removed without errors.</returns>
    public Task<bool> UninstallAsync(
        UninstallOptions options,
        IProgress<string> log,
        CancellationToken cancellationToken = default)
    {
        bool success = true;
        string gamePath = options.GamePath;

        log.Report("Starting DINOForge uninstall...");

        try
        {
            int removedManaged = InstallLifecycle.RemoveManagedFiles(gamePath, log.Report);
            log.Report($"Removed {removedManaged} managed DINOForge install artifact(s).");
        }
        catch (Exception ex)
        {
            success = false;
            log.Report($"  ERROR removing managed DINOForge files: {ex.Message}");
        }

        try
        {
            int removedLegacy = InstallLifecycle.CleanupLegacyArtifacts(gamePath, log.Report);
            if (removedLegacy > 0)
            {
                log.Report($"Removed {removedLegacy} legacy DINOForge artifact(s).");
            }
        }
        catch (Exception ex)
        {
            success = false;
            log.Report($"  ERROR removing legacy DINOForge files: {ex.Message}");
        }

        // Remove packs directory
        if (options.RemovePacks)
        {
            success &= TryDeleteDirectory(InstallLifecycle.GetPacksDirectory(gamePath), log);
            success &= TryDeleteDirectory(InstallLifecycle.GetLegacyPacksDirectory(gamePath), log);
        }

        // Remove dumps directory
        if (options.RemoveDumps)
            success &= TryDeleteDirectory(Path.Combine(gamePath, "dinoforge_dumps"), log);

        // Remove dev assets directory
        if (options.RemoveDevAssets)
            success &= TryDeleteDirectory(Path.Combine(gamePath, "dinoforge_dev"), log);

        if (success)
            log.Report("Uninstall complete. DINOForge has been removed.");
        else
            log.Report("Uninstall finished with some errors (see above). Try running as Administrator.");

        return Task.FromResult(success);
    }

    /// <summary>
    /// Deletes a single file if it exists, logging success or warning on failure.
    /// </summary>
    private static bool TryDeleteFile(string path, IProgress<string> log)
    {
        if (!File.Exists(path))
        {
            log.Report($"  Skipped (not found): {Path.GetFileName(path)}");
            return true;
        }

        try
        {
            File.Delete(path);
            log.Report($"  Deleted: {Path.GetFileName(path)}");
            return true;
        }
        catch (IOException ex)
        {
            log.Report($"  ERROR deleting {Path.GetFileName(path)}: {ex.Message}. Try running as Administrator.");
            return false;
        }
    }

    /// <summary>
    /// Recursively deletes a directory if it exists, logging success or warning on failure.
    /// </summary>
    private static bool TryDeleteDirectory(string path, IProgress<string> log)
    {
        if (!Directory.Exists(path))
        {
            log.Report($"  Skipped (not found): {Path.GetFileName(path)}/");
            return true;
        }

        try
        {
            Directory.Delete(path, recursive: true);
            log.Report($"  Deleted directory: {Path.GetFileName(path)}/");
            return true;
        }
        catch (IOException ex)
        {
            log.Report($"  ERROR deleting {Path.GetFileName(path)}/: {ex.Message}. Try running as Administrator.");
            return false;
        }
    }

    /// <summary>
    /// Creates a Windows .url desktop shortcut pointing to the game folder.
    /// Falls back gracefully if the shortcut cannot be created.
    /// </summary>
    private static void TryCreateDesktopShortcut(string gamePath, IProgress<string> log)
    {
        try
        {
            string exePath = Path.Combine(gamePath, "Diplomacy is Not an Option.exe");
            if (!File.Exists(exePath))
            {
                log.Report("  Could not create shortcut: game exe not found.");
                return;
            }

            string desktop = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory);
            string shortcutPath = Path.Combine(desktop, "Diplomacy is Not an Option.url");

            // Write a simple URL-style shortcut that Windows Explorer understands
            File.WriteAllText(shortcutPath,
                $"[InternetShortcut]{Environment.NewLine}URL=file:///{exePath.Replace('\\', '/')}{Environment.NewLine}");

            log.Report($"  Desktop shortcut created: {shortcutPath}");
        }
        catch (Exception ex)
        {
            log.Report($"  Warning: could not create desktop shortcut: {ex.Message}");
        }
    }
}
