#nullable enable
// Iter-144 H7 instrumentation — Harmony Prefix+Postfix on Resources.UnloadUnusedAssets
// to confirm/refute gray-freeze hypothesis. See docs/sessions/iter144-h7-harmony-targets.md.
//
// Pre-existing DF analyzer warnings in this file's logging surface are tracked in
// Pattern Catalog #103/#106/#111 — this is diagnostic-only instrumentation.
#pragma warning disable DF0103 // local-time logging (UTC used; analyzer false positive on Stopwatch math)
using System;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Threading;
using HarmonyLib;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Iter-144 H7 instrumentation: Harmony Prefix+Postfix on
    /// <c>UnityEngine.Resources.UnloadUnusedAssets()</c>.
    ///
    /// Hypothesis: the InitialGameLoader → MainMenu wedge happens INSIDE Unity's
    /// automatic UnloadUnusedAssets pump (called by SceneManager.LoadScene single
    /// mode). 2s gap between scene-change event and gray-freeze matches GC +
    /// asset-graph traversal across 6 GB of Addressables.
    ///
    /// Probe semantics:
    /// - <see cref="Prefix"/> logs ENTER with truncated stack trace (5 frames).
    /// - <see cref="Postfix"/> logs EXIT with elapsed milliseconds.
    /// - If ENTER fires without matching EXIT in the log → H7 CONFIRMED.
    /// - If both ENTER and EXIT fire but pump still wedges → H7 REFUTED, move to H8.
    ///
    /// This patch is intentionally NON-INVASIVE — it never returns false from Prefix
    /// (the original call always runs). A follow-on patch can convert to skip-mode
    /// if H7 is confirmed.
    /// </summary>
    internal static class ResourcesUnloadGuardPatch
    {
        // DF1006-ok: ManualLogSource is owned by BepInEx; mirrors DestroyGuardPatch._log lifecycle.
#pragma warning disable DF1006
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.ResourcesUnload");
#pragma warning restore DF1006

        private static long s_enterTicks;
        private static int s_callCount;

        /// <summary>
        /// Apply the Resources.UnloadUnusedAssets probe via the shared Harmony instance.
        /// </summary>
        internal static void Apply(Harmony harmony)
        {
            try
            {
                MethodInfo? target = typeof(Resources).GetMethod(
                    nameof(Resources.UnloadUnusedAssets),
                    BindingFlags.Public | BindingFlags.Static,
                    binder: null,
                    types: Type.EmptyTypes,
                    modifiers: null);

                if (target == null)
                {
                    _log.LogWarning("[ResourcesUnload] Could not resolve Resources.UnloadUnusedAssets() — patch SKIPPED");
                    return;
                }

                var prefix = new HarmonyMethod(typeof(ResourcesUnloadGuardPatch)
                    .GetMethod(nameof(Prefix), BindingFlags.Static | BindingFlags.NonPublic));
                var postfix = new HarmonyMethod(typeof(ResourcesUnloadGuardPatch)
                    .GetMethod(nameof(Postfix), BindingFlags.Static | BindingFlags.NonPublic));

                harmony.Patch(target, prefix: prefix, postfix: postfix);
                _log.LogInfo("[ResourcesUnload] Patched Resources.UnloadUnusedAssets() — H7 probe active");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ResourcesUnload] Patch failed: {ex}");
            }
        }

        /// <summary>
        /// Prefix: record entry timestamp + log a truncated stack trace so we can see
        /// what triggered the call (Unity's post-LoadScene pump vs. an explicit user call).
        /// Always returns true (original call runs).
        /// </summary>
        private static void Prefix()
        {
            try
            {
                s_enterTicks = Stopwatch.GetTimestamp();
                int n = Interlocked.Increment(ref s_callCount);

                // Truncated stack trace — skip 2 frames (this method + Harmony stub),
                // capture next 5 frames for caller-context fingerprint.
                var stack = new StackTrace(skipFrames: 2, fNeedFileInfo: false).ToString();
                string[] lines = stack.Split('\n');
                string trimmed = string.Join("\n", lines.Take(6));

                _log.LogInfo($"[ResourcesUnload] ENTER call#{n} t={DateTime.UtcNow:o} from:\n{trimmed}");
            }
            catch (Exception ex)
            {
                // Defense-in-depth: probe must NEVER break the wrapped call.
                _log.LogWarning($"[ResourcesUnload] Prefix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }

        /// <summary>
        /// Postfix: log elapsed milliseconds. If this never fires after a matching
        /// Prefix ENTER line, the wedge is INSIDE UnloadUnusedAssets (H7 confirmed).
        /// </summary>
        private static void Postfix()
        {
            try
            {
                long enter = Interlocked.Read(ref s_enterTicks);
                double elapsedMs = (Stopwatch.GetTimestamp() - enter) * 1000.0 / Stopwatch.Frequency;
                _log.LogInfo($"[ResourcesUnload] EXIT call#{s_callCount} elapsed={elapsedMs:F2}ms t={DateTime.UtcNow:o}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ResourcesUnload] Postfix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }
    }
}
