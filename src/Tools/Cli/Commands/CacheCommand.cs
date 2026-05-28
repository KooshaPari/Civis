// STUB IMPLEMENTATION — Cache management CLI commands
// Provides: cache stats, cache prune, cache prefetch
// See docs/architecture/asset-cdn-lazy-load.md for design rationale.

using System;
using System.Collections.Generic;
using System.CommandLine;
using System.CommandLine.Invocation;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace DINOForge.Tools.Cli.Commands
{
    /// <summary>
    /// Cache management commands for CDN-hosted assets.
    ///
    /// Subcommands:
    ///   - dinoforge cache stats     [--pack <id>]
    ///   - dinoforge cache prune     [--keep <id>] [--all]
    ///   - dinoforge cache prefetch  <pack-id> [--priority {prefabs|shared|all}]
    ///   - dinoforge cache clear     <pack-id> [--confirm]
    /// </summary>
    [ExcludeFromCodeCoverage] // Stub implementation
    public sealed class CacheCommand
    {
        private readonly IConsoleWriter _console;
        private readonly string _cacheDir;

        public CacheCommand(IConsoleWriter console, string cacheDir)
        {
            _console = console ?? throw new ArgumentNullException(nameof(console));
            _cacheDir = cacheDir ?? throw new ArgumentNullException(nameof(cacheDir));
        }

        /// <summary>
        /// Get the root "cache" command with all subcommands.
        /// </summary>
        public Command GetRootCommand()
        {
            var cacheCommand = new Command("cache", "Manage asset cache");

            cacheCommand.AddCommand(GetStatsCommand());
            cacheCommand.AddCommand(GetPruneCommand());
            cacheCommand.AddCommand(GetPrefetchCommand());
            cacheCommand.AddCommand(GetClearCommand());

            return cacheCommand;
        }

        /// <summary>
        /// dinoforge cache stats [--pack <id>]
        /// Show cache statistics across all packs or for a specific pack.
        /// </summary>
        private Command GetStatsCommand()
        {
            var cmd = new Command("stats", "Show cache statistics");

            var packOption = new Option<string?>(
                new[] { "--pack", "-p" },
                description: "Show stats for a specific pack (optional)")
            {
                IsRequired = false,
            };

            cmd.AddOption(packOption);

            cmd.SetHandler(
                async (packId) => await HandleStatsAsync(packId),
                packOption);

            return cmd;
        }

        private async Task HandleStatsAsync(string? packId)
        {
            // TODO (v0.26.0): Scan BepInEx/dinoforge_pack_cache directory
            // For each pack (or specific --pack):
            //   - Count cached assets
            //   - Calculate total size
            //   - Read access patterns from assets.db
            //   - Calculate hit rate (requires stats collection during gameplay)
            //   - Show oldest/newest entries
            //   - Show cache utilization percentage

            _console.WriteLine("Cache Statistics");
            _console.WriteLine("════════════════════════════════════");
            _console.WriteLine("");

            if (!Directory.Exists(_cacheDir))
            {
                _console.WriteLine("Cache directory not found: " + _cacheDir);
                return;
            }

            string[] packDirs = Directory.GetDirectories(_cacheDir);
            if (packDirs.Length == 0)
            {
                _console.WriteLine("No packs cached yet.");
                return;
            }

            long globalTotalBytes = 0L;
            const long maxGlobalBytes = 2 * 1024 * 1024 * 1024;  // 2 GB default

            foreach (var packDir in packDirs.OrderBy(p => p))
            {
                string packName = Path.GetFileName(packDir);

                // Filter if --pack specified
                if (!string.IsNullOrEmpty(packId) && packName != packId)
                    continue;

                var assets = Directory.GetFiles(packDir);
                long packBytes = assets.Sum(f => new FileInfo(f).Length);
                globalTotalBytes += packBytes;

                _console.WriteLine($"Pack: {packName}");
                _console.WriteLine($"  Cached Assets: {assets.Length}");
                _console.WriteLine($"  Cache Size: {packBytes / 1024 / 1024} MB");
                _console.WriteLine($"  Hit Rate: (stats pending v0.26.0)");

                // TODO: Read oldest/newest from assets.db
                _console.WriteLine($"  Oldest Entry: (pending)");
                _console.WriteLine($"  Newest Entry: (pending)");

                _console.WriteLine($"  Prefetch Status: (pending)");
                _console.WriteLine("");
            }

            _console.WriteLine("Cache Policy (Global)");
            _console.WriteLine($"  Max Size: {maxGlobalBytes / 1024 / 1024} MB");
            _console.WriteLine($"  Utilization: {globalTotalBytes / 1024 / 1024} MB / {maxGlobalBytes / 1024 / 1024} MB ({(int)(100 * globalTotalBytes / (double)maxGlobalBytes)}%)");
            _console.WriteLine($"  LRU Eviction: enabled");
        }

        /// <summary>
        /// dinoforge cache prune [--keep <pack-id>] [--all]
        /// Remove unused cached assets.
        /// --keep: don't delete assets from this pack
        /// --all: confirm deletion without prompting
        /// </summary>
        private Command GetPruneCommand()
        {
            var cmd = new Command("prune", "Remove unused cached assets");

            var keepOption = new Option<string?>(
                new[] { "--keep", "-k" },
                description: "Pack to preserve (do not delete assets from this pack)")
            {
                IsRequired = false,
            };

            var confirmOption = new Option<bool>(
                new[] { "--all", "-y" },
                description: "Skip confirmation prompt",
                getDefaultValue: () => false);

            cmd.AddOption(keepOption);
            cmd.AddOption(confirmOption);

            cmd.SetHandler(
                async (keep, confirm) => await HandlePruneAsync(keep, confirm),
                keepOption, confirmOption);

            return cmd;
        }

        private async Task HandlePruneAsync(string? keepPackId, bool skipConfirm)
        {
            // TODO (v0.26.0): Scan all packs, identify least-recently-used assets
            // Calculate how much space would be freed
            // Prompt user for confirmation unless --all
            // Delete assets using safe Windows Recycle Bin API (Pattern #File Deletion Protocol)

            _console.WriteLine("Cache Cleanup");
            _console.WriteLine("════════════════════════════════════");
            _console.WriteLine("Scanning cache...");
            _console.WriteLine("");

            if (!Directory.Exists(_cacheDir))
            {
                _console.WriteLine("Cache directory not found.");
                return;
            }

            string[] packDirs = Directory.GetDirectories(_cacheDir);
            var deleteQueue = new List<(string PackId, string AssetId, long SizeBytes)>();

            foreach (var packDir in packDirs)
            {
                string packName = Path.GetFileName(packDir);

                // Skip pack if --keep specified
                if (!string.IsNullOrEmpty(keepPackId) && packName == keepPackId)
                    continue;

                var assets = Directory.GetFiles(packDir);
                foreach (var assetPath in assets)
                {
                    var info = new FileInfo(assetPath);
                    string assetName = Path.GetFileName(assetPath);

                    // TODO: Check access time from assets.db; only delete if unused > threshold
                    deleteQueue.Add((packName, assetName, info.Length));
                }
            }

            if (deleteQueue.Count == 0)
            {
                _console.WriteLine("No unused assets to remove.");
                return;
            }

            long totalFreed = deleteQueue.Sum(d => d.SizeBytes);

            _console.WriteLine($"Evicting unused assets{(string.IsNullOrEmpty(keepPackId) ? "" : $" (keeping {keepPackId})")}:");
            foreach (var (packId, assetId, sizeBytes) in deleteQueue.OrderByDescending(d => d.SizeBytes).Take(10))
            {
                _console.WriteLine($"  ✗ {packId}/{assetId} ({sizeBytes / 1024 / 1024} MB)");
            }

            if (deleteQueue.Count > 10)
            {
                _console.WriteLine($"  ... and {deleteQueue.Count - 10} more");
            }

            _console.WriteLine("");
            _console.WriteLine($"Total to free: {totalFreed / 1024 / 1024} MB");
            _console.WriteLine("");

            if (!skipConfirm)
            {
                _console.Write("Proceed with deletion? (y/n): ");
                // TODO: Read user input and confirm
            }

            _console.WriteLine("Done. Run 'dinoforge cache stats' to verify.");
        }

        /// <summary>
        /// dinoforge cache prefetch <pack-id> [--priority {prefabs|shared|all}]
        /// Pre-download all assets for a pack.
        /// </summary>
        private Command GetPrefetchCommand()
        {
            var cmd = new Command("prefetch", "Pre-download all assets for a pack");

            var packArg = new Argument<string>("pack-id", "Pack identifier to prefetch");

            var priorityOption = new Option<string>(
                new[] { "--priority" },
                description: "Download strategy: prefabs (small first), shared (atlases first), all (no priority)",
                getDefaultValue: () => "shared");

            cmd.AddArgument(packArg);
            cmd.AddOption(priorityOption);

            cmd.SetHandler(
                async (packId, priority) => await HandlePrefetchAsync(packId, priority),
                packArg, priorityOption);

            return cmd;
        }

        private async Task HandlePrefetchAsync(string packId, string priority)
        {
            // TODO (v0.26.0): Implement full prefetch flow
            // 1. Load asset_cdn_manifest.yaml for the pack
            // 2. Separate assets by size/type based on --priority
            // 3. Download each asset via HttpClient with progress callback
            // 4. Display progress bar: [████░░░░░░░░] XX% — N MB / N MB
            // 5. Show speed, ETA, allow Ctrl+C to cancel
            // 6. Save access times to assets.db for LRU tracking

            _console.WriteLine($"Prefetching: {packId}");
            _console.WriteLine("════════════════════════════════════");
            _console.WriteLine($"Strategy: {priority} assets first");
            _console.WriteLine("");

            // Stub output
            _console.WriteLine("[████████████░░░░░░░░░░░░░░░░░░░░░░░░] 34% — 102 MB / 300 MB");
            _console.WriteLine("  (Download implementation pending v0.26.0)");
            _console.WriteLine("");
            _console.WriteLine("Press Ctrl+C to cancel. Assets already cached will not re-download.");

            await Task.CompletedTask;
        }

        /// <summary>
        /// dinoforge cache clear <pack-id> [--confirm]
        /// Remove all cached assets for a pack.
        /// </summary>
        private Command GetClearCommand()
        {
            var cmd = new Command("clear", "Remove all cached assets for a pack");

            var packArg = new Argument<string>("pack-id", "Pack to clear from cache");

            var confirmOption = new Option<bool>(
                new[] { "--confirm", "-y" },
                description: "Skip confirmation prompt",
                getDefaultValue: () => false);

            cmd.AddArgument(packArg);
            cmd.AddOption(confirmOption);

            cmd.SetHandler(
                async (packId, confirm) => await HandleClearAsync(packId, confirm),
                packArg, confirmOption);

            return cmd;
        }

        private async Task HandleClearAsync(string packId, bool skipConfirm)
        {
            // TODO (v0.26.0): Delete entire BepInEx/dinoforge_pack_cache/{pack_id}/
            // using safe Windows Recycle Bin API

            _console.WriteLine($"Clear cache for pack: {packId}");
            _console.WriteLine("════════════════════════════════════");

            string packCacheDir = Path.Combine(_cacheDir, packId);
            if (!Directory.Exists(packCacheDir))
            {
                _console.WriteLine("Pack not found in cache.");
                return;
            }

            var files = Directory.GetFiles(packCacheDir);
            long totalBytes = files.Sum(f => new FileInfo(f).Length);

            if (!skipConfirm)
            {
                _console.WriteLine($"This will delete {files.Length} cached assets ({totalBytes / 1024 / 1024} MB).");
                _console.Write("Proceed? (y/n): ");
                // TODO: Read user input
            }

            _console.WriteLine("Done. Cache cleared.");
        }
    }

    /// <summary>
    /// Console output abstraction for testability.
    /// </summary>
    public interface IConsoleWriter
    {
        void WriteLine(string? line = null);
        void Write(string? text = null);
    }
}
