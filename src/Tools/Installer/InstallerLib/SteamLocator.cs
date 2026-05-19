using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Runtime.Versioning;
using System.Text;
using System.Text.RegularExpressions;

namespace DINOForge.Tools.Installer
{
    /// <summary>
    /// Locates Steam installations and game directories across platforms.
    /// Supports Windows (registry + default paths) and Linux (~/.steam/).
    /// </summary>
    public static class SteamLocator
    {
        /// <summary>
        /// DINO Steam AppId.
        /// </summary>
        public const int DinoAppId = 1272320;

        /// <summary>
        /// Default game directory name on disk.
        /// </summary>
        public const string DinoDirectoryName = "Diplomacy is Not an Option";

        /// <summary>
        /// Attempts to find the DINO game install directory.
        /// Checks Steam library folders via registry (Windows) or well-known paths (Linux).
        /// </summary>
        /// <returns>The full path to the DINO install directory, or null if not found.</returns>
        public static string? FindDinoInstallPath()
        {
            string? steamPath = FindSteamPath();
            if (steamPath == null)
                return null;

            IReadOnlyList<string> libraryFolders = GetLibraryFolders(steamPath);
            foreach (string library in libraryFolders)
            {
                string? gamePath = FindGameInLibrary(library, DinoAppId);
                if (gamePath != null)
                    return gamePath;
            }

            return null;
        }

        /// <summary>
        /// Finds a game install path by AppId within a specific Steam library folder.
        /// </summary>
        /// <param name="libraryPath">Root of a Steam library folder.</param>
        /// <param name="appId">Steam application ID to search for.</param>
        /// <returns>Full path to the game directory, or null.</returns>
        public static string? FindGameInLibrary(string libraryPath, int appId)
        {
            // Check if acf manifest exists for this appId
            string steamAppsPath = Path.Combine(libraryPath, "steamapps");
            if (!Directory.Exists(steamAppsPath))
                steamAppsPath = libraryPath; // libraryPath might already be the steamapps folder

            string acfPath = Path.Combine(steamAppsPath, $"appmanifest_{appId}.acf");
            if (File.Exists(acfPath))
            {
                string? installDir = ParseInstallDirFromAcf(acfPath);
                if (installDir != null)
                {
                    string fullPath = Path.Combine(steamAppsPath, "common", installDir);
                    if (Directory.Exists(fullPath))
                        return fullPath;
                }
            }

            // Fallback: check common directory name
            string commonPath = Path.Combine(steamAppsPath, "common", DinoDirectoryName);
            if (Directory.Exists(commonPath))
                return commonPath;

            return null;
        }

        /// <summary>
        /// Finds the Steam installation path.
        /// On Windows, reads from the registry. On Linux, checks well-known paths.
        /// </summary>
        /// <returns>Path to the Steam installation root, or null.</returns>
        public static string? FindSteamPath()
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                return FindSteamPathWindows();
            }

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            {
                return FindSteamPathLinux();
            }

            return null;
        }

        /// <summary>
        /// Finds Steam path on Windows via registry.
        /// </summary>
        [SupportedOSPlatform("windows")]
        internal static string? FindSteamPathWindows()
        {
            // Try 64-bit registry view first
            string[] registryPaths = new[]
            {
                @"SOFTWARE\WOW6432Node\Valve\Steam",
                @"SOFTWARE\Valve\Steam"
            };

            foreach (string regPath in registryPaths)
            {
                try
                {
                    using Microsoft.Win32.RegistryKey? key = Microsoft.Win32.Registry.LocalMachine.OpenSubKey(regPath);
                    string? installPath = key?.GetValue("InstallPath") as string;
                    if (!string.IsNullOrEmpty(installPath) && Directory.Exists(installPath))
                        return installPath;
                }
                catch
                {
                    // Registry access may fail, continue to next
                }
            }

            // Fallback to common install paths
            string[] defaultPaths = new[]
            {
                @"C:\Program Files (x86)\Steam",
                @"C:\Program Files\Steam"
            };

            foreach (string path in defaultPaths)
            {
                if (Directory.Exists(path))
                    return path;
            }

            return null;
        }

        /// <summary>
        /// Finds Steam path on Linux via well-known paths.
        /// </summary>
        internal static string? FindSteamPathLinux()
        {
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            string[] linuxPaths = new[]
            {
                Path.Combine(home, ".steam", "steam"),
                Path.Combine(home, ".local", "share", "Steam"),
                Path.Combine(home, ".steam", "debian-installation"),
                Path.Combine(home, "snap", "steam", "common", ".steam", "steam")
            };

            foreach (string path in linuxPaths)
            {
                if (Directory.Exists(path))
                    return path;
            }

            return null;
        }

        /// <summary>
        /// Parses Steam's libraryfolders.vdf to extract all library folder paths.
        /// Returns the Steam root plus any additional library folders.
        /// </summary>
        /// <param name="steamPath">Root Steam installation path.</param>
        /// <returns>List of library folder paths.</returns>
        public static IReadOnlyList<string> GetLibraryFolders(string steamPath)
        {
            List<string> folders = new List<string> { steamPath };

            string vdfPath = Path.Combine(steamPath, "steamapps", "libraryfolders.vdf");
            if (!File.Exists(vdfPath))
            {
                // Try alternate location
                vdfPath = Path.Combine(steamPath, "config", "libraryfolders.vdf");
            }

            if (!File.Exists(vdfPath))
                return folders;

            try
            {
                string content = File.ReadAllText(vdfPath, Encoding.UTF8);
                IReadOnlyList<string> parsed = ParseLibraryFoldersVdf(content);
                foreach (string folder in parsed)
                {
                    if (!folders.Any(f => string.Equals(f, folder, StringComparison.OrdinalIgnoreCase)))
                        folders.Add(folder);
                }
            }
            catch
            {
                // If parsing fails, return just the root
            }

            return folders;
        }

        /// <summary>
        /// Parses the content of a libraryfolders.vdf file to extract library paths.
        /// VDF format uses nested key-value pairs with "path" keys.
        /// </summary>
        /// <param name="vdfContent">Raw VDF file content.</param>
        /// <returns>List of library folder paths found in the VDF.</returns>
        public static IReadOnlyList<string> ParseLibraryFoldersVdf(string vdfContent)
        {
            List<string> paths = new List<string>();

            // Match "path" values in VDF format: "path"		"C:\\path\\to\\library"
            Regex pathRegex = new Regex(
                "\"path\"\\s+\"([^\"]+)\"",
                RegexOptions.IgnoreCase);

            MatchCollection matches = pathRegex.Matches(vdfContent);
            foreach (Match match in matches)
            {
                string path = match.Groups[1].Value
                    .Replace("\\\\", "\\"); // VDF escapes backslashes
                paths.Add(path);
            }

            return paths;
        }

        /// <summary>
        /// Parses the installdir value from a Steam ACF manifest file.
        /// </summary>
        /// <param name="acfPath">Path to the appmanifest_*.acf file.</param>
        /// <returns>The install directory name, or null.</returns>
        internal static string? ParseInstallDirFromAcf(string acfPath)
        {
            try
            {
                string content = File.ReadAllText(acfPath, Encoding.UTF8);
                Regex regex = new Regex("\"installdir\"\\s+\"([^\"]+)\"", RegexOptions.IgnoreCase);
                Match match = regex.Match(content);
                if (match.Success)
                    return match.Groups[1].Value;
            }
            catch
            {
                // Ignore parse errors
            }

            return null;
        }
    }
}
