using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.Json;

namespace DINOForge.Tools.Installer
{
    /// <summary>
    /// Shared install-layout rules and lifecycle helpers for DINOForge-managed files.
    /// </summary>
    public static class InstallLifecycle
    {
        /// <summary>Installer manifest file written into the plugins directory.</summary>
        public const string ManifestFileName = "dinoforge.install_manifest.json";

        /// <summary>Version sidecar written into the plugins directory.</summary>
        public const string VersionFileName = "dinoforge_version.txt";

        private static readonly string[] ManagedPluginFiles =
        {
            "DINOForge.Runtime.dll",
            "DINOForge.Runtime.pdb",
            "DINOForge.Runtime.deps.json",
            "DINOForge.SDK.dll",
            "DINOForge.SDK.pdb",
            "DINOForge.SDK.xml",
            "DINOForge.Bridge.Protocol.dll",
            "DINOForge.Bridge.Protocol.pdb",
            VersionFileName,
        };

        private static readonly string[] LegacyPluginFiles =
        [
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.Runtime.dll",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.Runtime.pdb",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.Runtime.deps.json",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.SDK.dll",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.SDK.pdb",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.SDK.xml",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.Bridge.Protocol.dll",
            $"BepInEx{Path.DirectorySeparatorChar}ecs_plugins{Path.DirectorySeparatorChar}DINOForge.Bridge.Protocol.pdb",
            $"BepInEx{Path.DirectorySeparatorChar}plugins{Path.DirectorySeparatorChar}DINOForge.Runtime.dll.bak",
        ];

        /// <summary>Gets the BepInEx root directory.</summary>
        public static string GetBepInExDirectory(string gamePath) => Path.Combine(gamePath, "BepInEx");

        /// <summary>Gets the canonical plugins directory.</summary>
        public static string GetPluginsDirectory(string gamePath) => Path.Combine(GetBepInExDirectory(gamePath), "plugins");

        /// <summary>Gets the canonical packs directory used by the runtime.</summary>
        public static string GetPacksDirectory(string gamePath) => Path.Combine(GetBepInExDirectory(gamePath), "dinoforge_packs");

        /// <summary>Gets the legacy packs directory used by older installer flows.</summary>
        public static string GetLegacyPacksDirectory(string gamePath) => Path.Combine(gamePath, "dinoforge_packs");

        /// <summary>Gets the installer manifest path.</summary>
        public static string GetManifestPath(string gamePath) => Path.Combine(GetPluginsDirectory(gamePath), ManifestFileName);

        /// <summary>
        /// Inspects a game directory for DINOForge install health, duplicate assemblies, and stale legacy files.
        /// </summary>
        public static InstallInspection Inspect(string gamePath)
        {
            List<string> issues = new List<string>();
            List<string> warnings = new List<string>();
            List<string> legacyArtifacts = new List<string>();
            List<string> managedFiles = new List<string>();

            if (string.IsNullOrWhiteSpace(gamePath) || !Directory.Exists(gamePath))
            {
                issues.Add($"Game directory does not exist: {gamePath}");
                return new InstallInspection(gamePath, false, false, false, "unknown", null, issues, warnings, legacyArtifacts, managedFiles);
            }

            string pluginsDir = GetPluginsDirectory(gamePath);
            string runtimePath = Path.Combine(pluginsDir, "DINOForge.Runtime.dll");
            bool runtimeInstalled = File.Exists(runtimePath);

            string version = InstallDetector.GetInstalledVersion(gamePath);
            string manifestPath = GetManifestPath(gamePath);
            bool manifestPresent = File.Exists(manifestPath);

            foreach (string fileName in ManagedPluginFiles)
            {
                string fullPath = Path.Combine(pluginsDir, fileName);
                if (File.Exists(fullPath))
                {
                    managedFiles.Add(fullPath);
                }
            }

            string uiAssetsDir = Path.Combine(pluginsDir, "dinoforge-ui-assets");
            if (Directory.Exists(uiAssetsDir))
            {
                managedFiles.Add(uiAssetsDir);
            }

            foreach (string relativePath in LegacyPluginFiles)
            {
                string fullPath = Path.Combine(gamePath, relativePath);
                if (File.Exists(fullPath))
                {
                    legacyArtifacts.Add(fullPath);
                }
            }

            string legacyPacksDir = GetLegacyPacksDirectory(gamePath);
            if (Directory.Exists(legacyPacksDir))
            {
                warnings.Add($"Legacy packs directory present: {legacyPacksDir}");
                if (!Directory.Exists(GetPacksDirectory(gamePath)))
                {
                    issues.Add("Only the legacy packs directory is present. Packs should be under BepInEx/dinoforge_packs.");
                }
            }

            if (legacyArtifacts.Count > 0)
            {
                issues.Add("Legacy DINOForge plugin artifacts detected in deprecated locations.");
            }

            if (manifestPresent)
            {
                InstallManifest? manifest = TryReadManifest(gamePath);
                if (manifest != null)
                {
                    foreach (InstalledFileRecord file in manifest.Files)
                    {
                        string fullPath = Path.Combine(gamePath, file.RelativePath);
                        if (!File.Exists(fullPath))
                        {
                            issues.Add($"Managed file missing from install manifest: {file.RelativePath}");
                        }
                    }
                }
                else
                {
                    warnings.Add($"Install manifest is unreadable: {manifestPath}");
                }
            }
            else if (runtimeInstalled)
            {
                warnings.Add("Install manifest missing from plugins directory.");
            }

            return new InstallInspection(
                gamePath,
                Directory.Exists(pluginsDir),
                runtimeInstalled,
                manifestPresent,
                version,
                runtimeInstalled ? runtimePath : null,
                issues,
                warnings,
                legacyArtifacts,
                managedFiles);
        }

        /// <summary>
        /// Removes known legacy DINOForge runtime artifacts from deprecated locations.
        /// </summary>
        public static int CleanupLegacyArtifacts(string gamePath, Action<string>? log = null)
        {
            int removedCount = 0;

            foreach (string relativePath in LegacyPluginFiles)
            {
                string fullPath = Path.Combine(gamePath, relativePath);
                if (!File.Exists(fullPath))
                {
                    continue;
                }

                File.Delete(fullPath);
                removedCount++;
                log?.Invoke($"Removed legacy file: {fullPath}");
            }

            return removedCount;
        }

        /// <summary>
        /// Migrates packs from the legacy root-level directory into the canonical BepInEx packs directory.
        /// </summary>
        public static bool MigrateLegacyPacks(string gamePath, Action<string>? log = null)
        {
            string legacyPacksDir = GetLegacyPacksDirectory(gamePath);
            if (!Directory.Exists(legacyPacksDir))
            {
                return false;
            }

            string packsDir = GetPacksDirectory(gamePath);
            Directory.CreateDirectory(packsDir);
            CopyDirectoryContents(legacyPacksDir, packsDir);
            Directory.Delete(legacyPacksDir, recursive: true);
            log?.Invoke($"Migrated legacy packs directory to: {packsDir}");
            return true;
        }

        /// <summary>
        /// Removes managed files recorded in the install manifest, falling back to known plugin files.
        /// </summary>
        public static int RemoveManagedFiles(string gamePath, Action<string>? log = null)
        {
            int removedCount = 0;
            InstallManifest? manifest = TryReadManifest(gamePath);
            HashSet<string> targets = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            if (manifest != null)
            {
                foreach (InstalledFileRecord file in manifest.Files)
                {
                    targets.Add(Path.Combine(gamePath, file.RelativePath));
                }
            }
            else
            {
                string pluginsDir = GetPluginsDirectory(gamePath);
                foreach (string fileName in ManagedPluginFiles)
                {
                    targets.Add(Path.Combine(pluginsDir, fileName));
                }

                targets.Add(Path.Combine(pluginsDir, "dinoforge-ui-assets"));
            }

            foreach (string path in targets.OrderByDescending(p => p.Length))
            {
                if (File.Exists(path))
                {
                    File.Delete(path);
                    removedCount++;
                    log?.Invoke($"Deleted managed file: {path}");
                }
                else if (Directory.Exists(path))
                {
                    Directory.Delete(path, recursive: true);
                    removedCount++;
                    log?.Invoke($"Deleted managed directory: {path}");
                }
            }

            string manifestPath = GetManifestPath(gamePath);
            if (File.Exists(manifestPath))
            {
                File.Delete(manifestPath);
                removedCount++;
                log?.Invoke($"Deleted install manifest: {manifestPath}");
            }

            return removedCount;
        }

        /// <summary>
        /// Writes a new install manifest for the managed files currently present in the install.
        /// </summary>
        public static string WriteManifest(string gamePath, string installerVersion)
        {
            string pluginsDir = GetPluginsDirectory(gamePath);
            Directory.CreateDirectory(pluginsDir);

            InstallManifest manifest = CreateManifest(gamePath, installerVersion);
            string manifestPath = GetManifestPath(gamePath);
            string json = JsonSerializer.Serialize(manifest, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(manifestPath, json);
            return manifestPath;
        }

        /// <summary>
        /// Reads the install manifest when present and parseable.
        /// </summary>
        public static InstallManifest? TryReadManifest(string gamePath)
        {
            string manifestPath = GetManifestPath(gamePath);
            if (!File.Exists(manifestPath))
            {
                return null;
            }

            try
            {
                string json = File.ReadAllText(manifestPath);
                return JsonSerializer.Deserialize<InstallManifest>(json);
            }
            catch
            {
                return null;
            }
        }

        private static InstallManifest CreateManifest(string gamePath, string installerVersion)
        {
            string pluginsDir = GetPluginsDirectory(gamePath);
            List<InstalledFileRecord> files = new List<InstalledFileRecord>();

            foreach (string fileName in ManagedPluginFiles)
            {
                string fullPath = Path.Combine(pluginsDir, fileName);
                if (!File.Exists(fullPath))
                {
                    continue;
                }

                files.Add(CreateRecord(gamePath, fullPath));
            }

            string uiAssetsDir = Path.Combine(pluginsDir, "dinoforge-ui-assets");
            if (Directory.Exists(uiAssetsDir))
            {
                foreach (string filePath in Directory.GetFiles(uiAssetsDir, "*", SearchOption.AllDirectories))
                {
                    files.Add(CreateRecord(gamePath, filePath));
                }
            }

            return new InstallManifest
            {
                InstallerVersion = installerVersion,
                InstalledAtUtc = DateTime.UtcNow.ToString("O"),
                Files = files
            };
        }

        private static InstalledFileRecord CreateRecord(string gamePath, string fullPath)
        {
            FileInfo fileInfo = new FileInfo(fullPath);
            return new InstalledFileRecord
            {
                RelativePath = Path.GetRelativePath(gamePath, fullPath),
                Size = fileInfo.Length,
                Sha256 = ComputeSha256(fullPath)
            };
        }

        private static string ComputeSha256(string filePath)
        {
            using FileStream stream = File.OpenRead(filePath);
            using SHA256 sha256 = SHA256.Create();
            byte[] hash = sha256.ComputeHash(stream);
            return Convert.ToHexString(hash);
        }

        private static void CopyDirectoryContents(string sourceDir, string destinationDir)
        {
            foreach (string directory in Directory.GetDirectories(sourceDir, "*", SearchOption.AllDirectories))
            {
                string relativeDir = Path.GetRelativePath(sourceDir, directory);
                Directory.CreateDirectory(Path.Combine(destinationDir, relativeDir));
            }

            foreach (string filePath in Directory.GetFiles(sourceDir, "*", SearchOption.AllDirectories))
            {
                string relativePath = Path.GetRelativePath(sourceDir, filePath);
                string destinationPath = Path.Combine(destinationDir, relativePath);
                string? destinationParent = Path.GetDirectoryName(destinationPath);
                if (!string.IsNullOrEmpty(destinationParent))
                {
                    Directory.CreateDirectory(destinationParent);
                }

                File.Copy(filePath, destinationPath, overwrite: true);
            }
        }
    }

    /// <summary>
    /// Structured install health report for a game directory.
    /// </summary>
    public sealed class InstallInspection
    {
        public InstallInspection(
            string gamePath,
            bool pluginsDirectoryPresent,
            bool runtimeInstalled,
            bool manifestPresent,
            string installedVersion,
            string? primaryRuntimePath,
            IReadOnlyList<string> issues,
            IReadOnlyList<string> warnings,
            IReadOnlyList<string> legacyArtifacts,
            IReadOnlyList<string> managedFiles)
        {
            GamePath = gamePath;
            PluginsDirectoryPresent = pluginsDirectoryPresent;
            RuntimeInstalled = runtimeInstalled;
            ManifestPresent = manifestPresent;
            InstalledVersion = installedVersion;
            PrimaryRuntimePath = primaryRuntimePath;
            Issues = issues;
            Warnings = warnings;
            LegacyArtifacts = legacyArtifacts;
            ManagedFiles = managedFiles;
        }

        public string GamePath { get; }
        public bool PluginsDirectoryPresent { get; }
        public bool RuntimeInstalled { get; }
        public bool ManifestPresent { get; }
        public string InstalledVersion { get; }
        public string? PrimaryRuntimePath { get; }
        public IReadOnlyList<string> Issues { get; }
        public IReadOnlyList<string> Warnings { get; }
        public IReadOnlyList<string> LegacyArtifacts { get; }
        public IReadOnlyList<string> ManagedFiles { get; }
        public bool IsHealthy => RuntimeInstalled && Issues.Count == 0;
    }

    /// <summary>
    /// JSON manifest stored alongside installed DINOForge plugin files.
    /// </summary>
    public sealed class InstallManifest
    {
        public string SchemaVersion { get; set; } = "1";
        public string InstallerVersion { get; set; } = "unknown";
        public string InstalledAtUtc { get; set; } = string.Empty;
        public List<InstalledFileRecord> Files { get; set; } = new List<InstalledFileRecord>();
    }

    /// <summary>
    /// Single managed file entry recorded in the install manifest.
    /// </summary>
    public sealed class InstalledFileRecord
    {
        public string RelativePath { get; set; } = string.Empty;
        public long Size { get; set; }
        public string Sha256 { get; set; } = string.Empty;
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
                    FileVersionInfo fvi = FileVersionInfo.GetVersionInfo(runtimeDll);
                    string? ver = fvi.ProductVersion ?? fvi.FileVersion;
                    if (!string.IsNullOrEmpty(ver))
                        return ver;
                }
                catch { /* fall through */ }
            }

            return "unknown";
        }
    }
}
