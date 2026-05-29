#nullable enable
using System;
using System.IO;
using BepInEx.Logging;

namespace DINOForge.Runtime.HotReload
{
    /// <summary>
    /// Classifies a hot-module-reload signal into one of three tiers and
    /// performs the appropriate action, delegating UI feedback to the
    /// <see cref="IHmrUiActions"/> abstraction (satisfied by DFCanvas/ModMenuPanel).
    ///
    /// Tier 1 — pack YAML change (pack.yaml or content subdirectory):
    ///   Calls <see cref="IHmrPackActions.TriggerPackReload"/> and shows a toast.
    ///   No game restart required.
    ///
    /// Tier 2 — bundle / asset change (packs/*/assets/):
    ///   Shows a modal confirmation: "Asset changes detected. Assets require returning
    ///   to the main menu for the swap to take effect. Reload now?"
    ///   On confirm: calls <see cref="IHmrPackActions.TriggerSceneReload"/>.
    ///
    /// Tier 3 — Runtime DLL updated on disk (newer hash than loaded assembly):
    ///   Informational only — shows a toast: "DLL updated. Restart game to apply."
    /// </summary>
    public sealed class HmrTieredReloader
    {
        private readonly ManualLogSource _log;
        private readonly IHmrPackActions _packActions;
        private readonly IHmrUiActions _uiActions;
        private readonly string _runtimeDllPath;

        // Captured hash of the loaded assembly (computed once on construction).
        private readonly string _loadedDllHash;

        /// <summary>
        /// Creates a new tiered reloader.
        /// </summary>
        /// <param name="log">BepInEx logger.</param>
        /// <param name="packActions">Callbacks to execute pack-reload or scene-reload.</param>
        /// <param name="uiActions">Callbacks to show toast or confirmation dialog.</param>
        /// <param name="runtimeDllPath">Absolute path to the deployed DINOForge.Runtime.dll on disk.</param>
        public HmrTieredReloader(
            ManualLogSource log,
            IHmrPackActions packActions,
            IHmrUiActions uiActions,
            string runtimeDllPath)
        {
            _log = log ?? throw new ArgumentNullException(nameof(log));
            _packActions = packActions ?? throw new ArgumentNullException(nameof(packActions));
            _uiActions = uiActions ?? throw new ArgumentNullException(nameof(uiActions));
            _runtimeDllPath = runtimeDllPath ?? string.Empty;

            // Capture hash of currently-loaded assembly (not the on-disk copy).
            _loadedDllHash = ComputeLoadedDllHash();
        }

        /// <summary>
        /// Classifies the changed file path and executes the appropriate tier action.
        /// </summary>
        /// <param name="changedPath">
        /// The file or directory path that triggered the signal, or an empty string
        /// when the signal file itself was the only indicator (tier falls back to Tier 1).
        /// </param>
        public void Handle(string changedPath)
        {
            HmrTier tier = ClassifyPath(changedPath);
            _log.LogInfo($"[HmrTieredReloader] Path='{changedPath}' → Tier {(int)tier} ({tier})");

            switch (tier)
            {
                case HmrTier.PackYaml:
                    HandleTier1();
                    break;

                case HmrTier.Asset:
                    HandleTier2();
                    break;

                case HmrTier.RuntimeDll:
                    HandleTier3();
                    break;
            }
        }

        /// <summary>
        /// Handles the HMR signal when no specific changed-path is known.
        /// Checks whether the DLL on disk is newer than the loaded assembly;
        /// if so escalates to Tier 3, otherwise falls back to Tier 1.
        /// </summary>
        public void HandleUnknown()
        {
            if (IsDllUpdatedOnDisk())
            {
                _log.LogInfo("[HmrTieredReloader] DLL on disk is newer — escalating to Tier 3.");
                HandleTier3();
            }
            else
            {
                _log.LogInfo("[HmrTieredReloader] No DLL change detected — defaulting to Tier 1 pack reload.");
                HandleTier1();
            }
        }

        // ── Tier classification ───────────────────────────────────────────────────

        internal static HmrTier ClassifyPath(string path)
        {
            if (string.IsNullOrEmpty(path))
                return HmrTier.PackYaml;

            string normalized = path.Replace('\\', '/');

            // Tier 3: explicit DLL path indicator
            if (normalized.EndsWith(".dll", StringComparison.OrdinalIgnoreCase)
                && normalized.IndexOf("DINOForge.Runtime", StringComparison.OrdinalIgnoreCase) >= 0)
            {
                return HmrTier.RuntimeDll;
            }

            // Tier 2: anything under a pack's assets/ subdirectory
            // Matches: packs/<pack>/assets/, dinoforge_packs/<pack>/assets/, or assets/bundles/
            bool isAssetPath =
                normalized.IndexOf("/assets/", StringComparison.OrdinalIgnoreCase) >= 0 ||
                normalized.IndexOf("/assets\\", StringComparison.OrdinalIgnoreCase) >= 0 ||
                (normalized.IndexOf("assets/bundles", StringComparison.OrdinalIgnoreCase) >= 0) ||
                (normalized.IndexOf("assets\\bundles", StringComparison.OrdinalIgnoreCase) >= 0);

            if (isAssetPath)
                return HmrTier.Asset;

            // Tier 1: everything else (pack.yaml, content YAML files, manifests)
            return HmrTier.PackYaml;
        }

        // ── Tier 1: pack YAML reload ──────────────────────────────────────────────

        private void HandleTier1()
        {
            _log.LogInfo("[HmrTieredReloader] Tier 1: triggering pack YAML reload.");
            try
            {
                _packActions.TriggerPackReload();
                _uiActions.ShowToast("Packs reloaded (Tier 1 — no restart needed).", HmrToastKind.Info);
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[HmrTieredReloader] Tier 1 reload failed: {ex}");
                _uiActions.ShowToast($"Pack reload failed: {ex.Message}", HmrToastKind.Error);
            }
        }

        // ── Tier 2: asset / bundle change ─────────────────────────────────────────

        private void HandleTier2()
        {
            _log.LogInfo("[HmrTieredReloader] Tier 2: asset change detected — showing confirmation prompt.");
            const string message =
                "Asset changes detected.\n\n" +
                "Asset swaps require returning to the main menu for the new bundles to take effect.\n\n" +
                "Reload now? (Returns to main menu and re-loads the scene.)";

            _uiActions.ShowConfirmDialog(
                message,
                onConfirm: () =>
                {
                    _log.LogInfo("[HmrTieredReloader] Tier 2: user confirmed — triggering scene reload.");
                    try
                    {
                        _packActions.TriggerSceneReload();
                    }
                    catch (Exception ex)
                    {
                        _log.LogWarning($"[HmrTieredReloader] Tier 2 scene reload failed: {ex}");
                        _uiActions.ShowToast($"Scene reload failed: {ex.Message}", HmrToastKind.Error);
                    }
                },
                onCancel: () =>
                {
                    _log.LogInfo("[HmrTieredReloader] Tier 2: user cancelled — asset swap deferred.");
                    _uiActions.ShowToast(
                        "Asset update deferred. Restart or return to main menu manually to apply.",
                        HmrToastKind.Warning);
                });
        }

        // ── Tier 3: DLL updated ───────────────────────────────────────────────────

        private void HandleTier3()
        {
            _log.LogInfo("[HmrTieredReloader] Tier 3: DLL updated on disk — informational toast only.");
            _uiActions.ShowToast(
                "DINOForge DLL updated. Restart the game to apply the new version.",
                HmrToastKind.Warning);
        }

        // ── DLL change detection ──────────────────────────────────────────────────

        private bool IsDllUpdatedOnDisk()
        {
            if (string.IsNullOrEmpty(_runtimeDllPath) || !File.Exists(_runtimeDllPath))
                return false;

            try
            {
                string diskHash = ComputeFileHash(_runtimeDllPath);
                bool changed = !string.Equals(diskHash, _loadedDllHash, StringComparison.OrdinalIgnoreCase);
                _log.LogInfo($"[HmrTieredReloader] DLL hash: loaded={_loadedDllHash}, disk={diskHash}, changed={changed}");
                return changed;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[HmrTieredReloader] DLL hash check failed: {ex}");
                return false;
            }
        }

        // ── Hash utilities ────────────────────────────────────────────────────────

        private string ComputeLoadedDllHash()
        {
            try
            {
                string location = typeof(HmrTieredReloader).Assembly.Location;
                if (string.IsNullOrEmpty(location) || !File.Exists(location))
                {
                    _log.LogInfo("[HmrTieredReloader] Assembly.Location unavailable — DLL change detection disabled.");
                    return string.Empty;
                }

                return ComputeFileHash(location);
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[HmrTieredReloader] Could not compute loaded DLL hash: {ex}");
                return string.Empty;
            }
        }

        /// <summary>
        /// Computes a lightweight CRC-32-like hash for change detection only.
        /// We avoid System.Security.Cryptography to stay netstandard2.0 compatible
        /// without additional references; a simple XOR + length check is sufficient
        /// since we only need to detect that the file bytes differ, not prove integrity.
        /// </summary>
        private static string ComputeFileHash(string filePath)
        {
            // Read up to 64 KB from the start and end of the file for a fast "changed?" check.
            const int SampleSize = 65536; // 64 KB
            using (FileStream fs = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
            {
                long length = fs.Length;
                byte[] buf = new byte[Math.Min(SampleSize * 2, (int)Math.Min(length, int.MaxValue))];
                int bytesRead = fs.Read(buf, 0, buf.Length);

                // Combine file length + XOR of all sampled bytes into a compact hex string.
                uint xorAccum = (uint)(length & 0xFFFFFFFF) ^ (uint)(length >> 32);
                for (int i = 0; i < bytesRead; i++)
                {
                    xorAccum = (xorAccum << 3) | (xorAccum >> 29); // rotate left 3
                    xorAccum ^= buf[i];
                }

                return length.ToString("X16") + xorAccum.ToString("X8");
            }
        }
    }

    /// <summary>Tier classification for an HMR event.</summary>
    public enum HmrTier
    {
        /// <summary>Tier 1: pack YAML / manifest change — in-memory reload, no restart.</summary>
        PackYaml = 1,

        /// <summary>Tier 2: pack asset / bundle change — requires scene reload with confirmation.</summary>
        Asset = 2,

        /// <summary>Tier 3: Runtime DLL updated — informational only, full restart required.</summary>
        RuntimeDll = 3,
    }

    /// <summary>Toast severity for HMR notifications.</summary>
    public enum HmrToastKind
    {
        /// <summary>Informational (green).</summary>
        Info,
        /// <summary>Advisory (yellow).</summary>
        Warning,
        /// <summary>Error (red).</summary>
        Error,
    }

    /// <summary>
    /// Callbacks the <see cref="HmrTieredReloader"/> calls to drive pack / scene state.
    /// Implement this in the hosting MonoBehaviour layer so the reloader stays Unity-free.
    /// </summary>
    public interface IHmrPackActions
    {
        /// <summary>Triggers a Tier-1 YAML-only pack reload (in-memory, no restart).</summary>
        void TriggerPackReload();

        /// <summary>
        /// Triggers a Tier-2 scene reload: loads Scene 1 (MainMenu) so asset bundles
        /// are re-evaluated on re-enter of gameplay.
        /// </summary>
        void TriggerSceneReload();
    }

    /// <summary>
    /// Callbacks the <see cref="HmrTieredReloader"/> calls to display UI feedback.
    /// Implement against DFCanvas/ModMenuPanel in RuntimeDriver.
    /// </summary>
    public interface IHmrUiActions
    {
        /// <summary>Shows a transient toast notification.</summary>
        void ShowToast(string message, HmrToastKind kind);

        /// <summary>
        /// Shows a modal confirmation dialog.
        /// <paramref name="onConfirm"/> is called if the user accepts.
        /// <paramref name="onCancel"/> is called if the user dismisses.
        /// </summary>
        void ShowConfirmDialog(string message, Action onConfirm, Action onCancel);
    }
}
