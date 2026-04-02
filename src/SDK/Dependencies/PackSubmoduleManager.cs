#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Diagnostics.CodeAnalysis;
using System.Threading.Tasks;

namespace DINOForge.SDK.Dependencies
{
    /// <summary>
    /// Represents a pack submodule entry from .gitmodules.
    /// </summary>
    [ExcludeFromCodeCoverage] // Simple data class
    public class PackSubmoduleEntry
    {
        /// <summary>
        /// The submodule path (e.g., "packs/warfare-modern").
        /// </summary>
        public string Path { get; set; } = string.Empty;

        /// <summary>
        /// The repository URL (e.g., "https://github.com/KooshaPari/warfare-modern").
        /// </summary>
        public string Url { get; set; } = string.Empty;
    }

    /// <summary>
    /// Manages pack repositories as git submodules with support for adding, listing, updating,
    /// and lock file generation for reproducible builds.
    /// </summary>
    [ExcludeFromCodeCoverage] // Requires git subprocess — integration tests only
    public class PackSubmoduleManager
    {
        private readonly string _workingDirectory;

        /// <summary>
        /// Initializes a new instance of the PackSubmoduleManager.
        /// </summary>
        /// <param name="workingDirectory">The git repository root directory. If null, uses the current directory.</param>
        public PackSubmoduleManager(string? workingDirectory = null)
        {
            _workingDirectory = workingDirectory ?? Directory.GetCurrentDirectory();
        }

        /// <summary>
        /// Adds a pack repository as a git submodule.
        /// </summary>
        /// <param name="repoUrl">The repository URL (HTTPS or SSH format).</param>
        /// <param name="path">Optional submodule path. Defaults to packs/{repo-name} if not specified.</param>
        /// <returns>A task representing the asynchronous operation.</returns>
        /// <exception cref="InvalidOperationException">Thrown if git command fails.</exception>
        public Task AddPackAsync(string repoUrl, string? path = null)
        {
            if (string.IsNullOrWhiteSpace(repoUrl))
                throw new ArgumentException("Repository URL cannot be empty.", nameof(repoUrl));

            // Extract repo name from URL if path not specified
            if (string.IsNullOrWhiteSpace(path))
            {
                string repoName = ExtractRepoName(repoUrl);
                path = System.IO.Path.Combine("packs", repoName);
            }

            // Normalize path separators for git
            path = path.Replace("\\", "/");

            // Run: git submodule add <repoUrl> <path>
            ProcessStartInfo psi = new ProcessStartInfo
            {
                FileName = "git",
                Arguments = $"submodule add \"{repoUrl}\" \"{path}\"",
                WorkingDirectory = _workingDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            return RunGitCommandAsync(psi, "git submodule add");
        }

        /// <summary>
        /// Lists all pack submodules from the .gitmodules file.
        /// </summary>
        /// <returns>A list of PackSubmoduleEntry objects representing each pack submodule.</returns>
        public List<PackSubmoduleEntry> ListPacks()
        {
            string gitmodulesPath = System.IO.Path.Combine(_workingDirectory, ".gitmodules");

            if (!File.Exists(gitmodulesPath))
                return new List<PackSubmoduleEntry>();

            var entries = new List<PackSubmoduleEntry>();
            var pathToUrl = new Dictionary<string, string>();

            // Parse .gitmodules file
            string content = File.ReadAllText(gitmodulesPath);
            string currentPath = string.Empty;

            foreach (string line in content.Split(new[] { "\r\n", "\r", "\n" }, StringSplitOptions.None))
            {
                string trimmed = line.Trim();

                // Extract path = value
                if (trimmed.StartsWith("path = "))
                {
                    currentPath = trimmed.Substring(7).Trim();
                }
                // Extract url = value
                else if (trimmed.StartsWith("url = "))
                {
                    string url = trimmed.Substring(6).Trim();
                    if (!string.IsNullOrEmpty(currentPath))
                    {
                        pathToUrl[currentPath] = url;
                        currentPath = string.Empty;
                    }
                }
            }

            // Create entries for packs/ submodules only
            foreach (var kvp in pathToUrl)
            {
                if (kvp.Key.StartsWith("packs/", StringComparison.OrdinalIgnoreCase))
                {
                    entries.Add(new PackSubmoduleEntry
                    {
                        Path = kvp.Key,
                        Url = kvp.Value
                    });
                }
            }

            return entries;
        }

        /// <summary>
        /// Updates all pack submodules to their latest remote versions.
        /// </summary>
        /// <returns>A task representing the asynchronous operation.</returns>
        /// <exception cref="InvalidOperationException">Thrown if git command fails.</exception>
        public Task UpdateAllAsync()
        {
            ProcessStartInfo psi = new ProcessStartInfo
            {
                FileName = "git",
                Arguments = "submodule update --remote",
                WorkingDirectory = _workingDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            return RunGitCommandAsync(psi, "git submodule update");
        }

        /// <summary>
        /// Generates a packs.lock file with submodule paths and their current commit SHAs for reproducible builds.
        /// </summary>
        /// <returns>A task representing the asynchronous operation.</returns>
        /// <exception cref="InvalidOperationException">Thrown if git command fails.</exception>
        public async Task GenerateLockFile()
        {
            var lockEntries = new List<string>();
            lockEntries.Add("# packs.lock - generated by dinoforge pack lock");
            lockEntries.Add(string.Format("# Generated: {0:yyyy-MM-dd HH:mm:ss} UTC", DateTime.UtcNow));
            lockEntries.Add("");

            var packs = ListPacks();
            foreach (var pack in packs)
            {
                string sha = await GetSubmoduleCommitShaAsync(pack.Path);
                lockEntries.Add(string.Format("{0} {1}", pack.Path, sha));
            }

            string lockPath = System.IO.Path.Combine(_workingDirectory, "packs.lock");
            File.WriteAllLines(lockPath, lockEntries);
        }

        /// <summary>
        /// Reads and parses the packs.lock file.
        /// </summary>
        /// <returns>A dictionary mapping pack paths to their commit SHAs.</returns>
        public Dictionary<string, string> ReadLockFile()
        {
            string lockPath = System.IO.Path.Combine(_workingDirectory, "packs.lock");

            if (!File.Exists(lockPath))
                return new Dictionary<string, string>();

            var entries = new Dictionary<string, string>();

            foreach (string line in File.ReadAllLines(lockPath))
            {
                // Skip comments and empty lines
                string trimmed = line.Trim();
                if (trimmed.StartsWith("#") || string.IsNullOrWhiteSpace(trimmed))
                    continue;

                string[] parts = trimmed.Split(new[] { ' ' }, StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length == 2)
                {
                    entries[parts[0]] = parts[1];
                }
            }

            return entries;
        }

        /// <summary>
        /// Extracts the repository name from a git URL.
        /// </summary>
        /// <param name="repoUrl">The repository URL.</param>
        /// <returns>The extracted repository name (without .git suffix).</returns>
        private static string ExtractRepoName(string repoUrl)
        {
            // Remove trailing slash
            string url = repoUrl.TrimEnd('/');

            // Get last path component
            string name = url.Split('/').Last();

            // Remove .git suffix if present
            if (name.EndsWith(".git", StringComparison.OrdinalIgnoreCase))
                name = name.Substring(0, name.Length - 4);

            return name;
        }

        /// <summary>
        /// Gets the current commit SHA of a submodule.
        /// </summary>
        /// <param name="submodulePath">The path to the submodule (e.g., "packs/warfare-modern").</param>
        /// <returns>The commit SHA (short form, 12 characters).</returns>
        /// <exception cref="InvalidOperationException">Thrown if git command fails.</exception>
        private async Task<string> GetSubmoduleCommitShaAsync(string submodulePath)
        {
            // Normalize path separators
            submodulePath = submodulePath.Replace("\\", "/");

            string fullPath = System.IO.Path.Combine(_workingDirectory, submodulePath);
            ProcessStartInfo psi = new ProcessStartInfo
            {
                FileName = "git",
                Arguments = string.Format("-C \"{0}\" rev-parse --short HEAD", fullPath),
                WorkingDirectory = _workingDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            return await RunGitCommandWithOutputAsync(psi, string.Format("git rev-parse for {0}", submodulePath));
        }

        /// <summary>
        /// Runs a git command asynchronously and returns output.
        /// </summary>
        private static async Task<string> RunGitCommandWithOutputAsync(ProcessStartInfo psi, string commandName)
        {
            Process process = Process.Start(psi);
            if (process == null)
                throw new InvalidOperationException("Failed to start git process");

            // Use Task.Run to avoid deadlocks with synchronous reads in async context
            Task<string> outputTask = Task.Run(() => process.StandardOutput.ReadToEnd());
            Task<string> errorTask = Task.Run(() => process.StandardError.ReadToEnd());

            process.WaitForExit();

            string output = await outputTask;
            string error = await errorTask;

            if (process.ExitCode != 0)
                throw new InvalidOperationException(string.Format("{0} failed: {1}", commandName, error));

            return output.Trim();
        }

        /// <summary>
        /// Runs a git command asynchronously.
        /// </summary>
        private static Task RunGitCommandAsync(ProcessStartInfo psi, string commandName)
        {
            return Task.Run(() =>
            {
                Process process = Process.Start(psi);
                if (process == null)
                    throw new InvalidOperationException("Failed to start git process");

                process.WaitForExit();

                if (process.ExitCode != 0)
                {
                    string error = process.StandardError.ReadToEnd();
                    throw new InvalidOperationException(string.Format("{0} failed: {1}", commandName, error));
                }
            });
        }
    }
}
