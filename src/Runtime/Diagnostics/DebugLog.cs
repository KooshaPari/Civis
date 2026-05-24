using System;
using System.IO;
using System.Text;

namespace DINOForge.Runtime.Diagnostics
{
    /// <summary>
    /// Centralized debug logging helper for the DINOForge Runtime plugin.
    ///
    /// <para>
    /// Owns the canonical <c>dinoforge_debug.log</c> file under the BepInEx root, with:
    /// <list type="bullet">
    ///   <item>Thread-safe append (single lock around rotation + write).</item>
    ///   <item>Pattern #232 rotation at 100 MB (rename to <c>.1</c>, overwriting any prior) — prevents the
    ///     iter-142 3.3 GB incident.</item>
    ///   <item>Pattern #106 explicit UTF-8 encoding (no locale-default ambiguity).</item>
    ///   <item>Safe-swallow of I/O exceptions with Console.Error fallback (logging must never throw).</item>
    /// </list>
    /// </para>
    ///
    /// <para>
    /// Pattern #644 PHASE 1: replaces the 24 ad-hoc <c>WriteDebug</c> helpers scattered across
    /// <c>src/Runtime/</c>. PHASE 2 (separate dispatch) will sweep the call sites and delete the locals.
    /// </para>
    /// </summary>
    public static class DebugLog
    {
        // Pattern #232: rotate log at 100MB to prevent unbounded growth (iter-142 3.3GB incident).
        private const long RotationThresholdBytes = 100L * 1024L * 1024L;

        private const string LogFileName = "dinoforge_debug.log";

        // Pattern #113/Pattern #232: serialize all rotate+append operations across threads.
        private static readonly object _lock = new object();

        /// <summary>
        /// Append a single line to the DINOForge debug log.
        /// </summary>
        /// <param name="category">Short category tag (e.g. "VanillaCatalog", "AssetSwap"). Wrapped in brackets.</param>
        /// <param name="msg">Free-form message. A trailing newline is added automatically.</param>
        /// <remarks>
        /// Format: <c>&lt;ISO8601-UTC&gt; [category] &lt;msg&gt;\n</c>.
        /// Never throws; I/O failures are reported to <see cref="Console.Error"/> as a fallback for visibility.
        /// </remarks>
        public static void Write(string category, string msg)
        {
            // Defensive normalization — callers shouldn't trip on null.
            string cat = category ?? string.Empty;
            string body = msg ?? string.Empty;
            string line = "[" + DateTime.UtcNow.ToString("o") + "] [" + cat + "] " + body + "\n";

            try
            {
                string logPath = ResolveLogPath();

                lock (_lock)
                {
                    RotateIfNeeded(logPath);

                    // Pattern #106: explicit UTF-8 encoding for portable log output.
                    File.AppendAllText(logPath, line, Encoding.UTF8);
                }
            }
            catch (Exception ex)
            {
                // Logging must never throw. Fall through to stderr so the failure is at least visible
                // in BepInEx console / attached debugger output.
                try
                {
                    Console.Error.WriteLine("[DebugLog] I/O failure: " + ex.GetType().Name + ": " + ex.Message);
                    Console.Error.WriteLine("[DebugLog] dropped line: " + line);
                }
                catch
                {
                    // safe-swallow: stderr itself failed; nothing more we can do without recursing.
                }
            }
        }

        /// <summary>
        /// Resolve the absolute path of the debug log file under the BepInEx root.
        /// Falls back to <c>%TEMP%/DINOForge</c> when BepInEx paths are unavailable (e.g. unit tests).
        /// </summary>
        private static string ResolveLogPath()
        {
            // BepInEx.Paths.BepInExRootPath is the canonical convention used by the existing
            // WriteDebug helpers (see VanillaCatalog.cs line 370 etc.).
            string? root = BepInEx.Paths.BepInExRootPath;
            if (string.IsNullOrEmpty(root))
            {
                root = Path.Combine(Path.GetTempPath(), "DINOForge");
                Directory.CreateDirectory(root);
            }

            return Path.Combine(root, LogFileName);
        }

        /// <summary>
        /// Pattern #232: rotate to <c>.1</c> (overwriting any prior) when the current log meets/exceeds
        /// the threshold. Best-effort; failures are swallowed so the append still attempts.
        /// </summary>
        private static void RotateIfNeeded(string logPath)
        {
            try
            {
                var info = new FileInfo(logPath);
                if (info.Exists && info.Length >= RotationThresholdBytes)
                {
                    string rotated = logPath + ".1";
                    if (File.Exists(rotated))
                    {
                        File.Delete(rotated);
                    }
                    File.Move(logPath, rotated);
                }
            }
            catch
            {
                // safe-swallow: rotation is best-effort; the append below may still succeed
                // (or fail loudly via the outer catch to stderr).
            }
        }
    }
}
