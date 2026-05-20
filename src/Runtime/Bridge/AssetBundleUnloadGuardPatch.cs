#nullable enable
// Iter-144 H8 instrumentation — Harmony Prefix+Postfix on AssetBundle.Unload(bool).
// Companion to ResourcesUnloadGuardPatch (H7 refuted: no ENTER/EXIT lines).
// New evidence: fallback thread alive 6s post-OnDestroy, PackUnitSpawner.Initialize
// fires (pack-recreation), then wedge. AssetBundle.Unload(true) is a top H8 candidate
// because pack-reload would Unload all current bundles before reloading.
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
    /// Iter-144 H8 instrumentation: Harmony Prefix+Postfix on
    /// <c>UnityEngine.AssetBundle.Unload(bool)</c> (instance method).
    ///
    /// Probe semantics:
    /// - <see cref="Prefix"/> logs ENTER with the <c>unloadAllLoadedObjects</c> arg + truncated stack trace.
    /// - <see cref="Postfix"/> logs EXIT with elapsed milliseconds.
    /// - If ENTER fires without matching EXIT in the log → wedge target IDENTIFIED.
    /// </summary>
    internal static class AssetBundleUnloadGuardPatch
    {
#pragma warning disable DF1006
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.AssetBundleUnload");
#pragma warning restore DF1006

        private static long s_enterTicks;
        private static int s_callCount;

        internal static void Apply(Harmony harmony)
        {
            try
            {
                MethodInfo? target = typeof(AssetBundle).GetMethod(
                    nameof(AssetBundle.Unload),
                    BindingFlags.Public | BindingFlags.Instance,
                    binder: null,
                    types: new[] { typeof(bool) },
                    modifiers: null);

                if (target == null)
                {
                    _log.LogWarning("[AssetBundleUnload] Could not resolve AssetBundle.Unload(bool) — patch SKIPPED");
                    return;
                }

                var prefix = new HarmonyMethod(typeof(AssetBundleUnloadGuardPatch)
                    .GetMethod(nameof(Prefix), BindingFlags.Static | BindingFlags.NonPublic));
                var postfix = new HarmonyMethod(typeof(AssetBundleUnloadGuardPatch)
                    .GetMethod(nameof(Postfix), BindingFlags.Static | BindingFlags.NonPublic));

                harmony.Patch(target, prefix: prefix, postfix: postfix);
                _log.LogInfo("[AssetBundleUnload] Patched AssetBundle.Unload(bool) — H8 probe active");
            }
            catch (Exception ex)
            {
                _log.LogError($"[AssetBundleUnload] Patch failed: {ex}");
            }
        }

        private static void Prefix(AssetBundle __instance, bool unloadAllLoadedObjects)
        {
            try
            {
                s_enterTicks = Stopwatch.GetTimestamp();
                int n = Interlocked.Increment(ref s_callCount);

                string bundleName;
                try { bundleName = __instance != null ? __instance.name : "<null>"; }
                catch { bundleName = "<unreadable>"; }

                var stack = new StackTrace(skipFrames: 2, fNeedFileInfo: false).ToString();
                string[] lines = stack.Split('\n');
                string trimmed = string.Join("\n", lines.Take(6));

                _log.LogInfo($"[AssetBundleUnload] ENTER call#{n} bundle='{bundleName}' unloadAll={unloadAllLoadedObjects} t={DateTime.UtcNow:o} from:\n{trimmed}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[AssetBundleUnload] Prefix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }

        private static void Postfix()
        {
            try
            {
                long enter = Interlocked.Read(ref s_enterTicks);
                double elapsedMs = (Stopwatch.GetTimestamp() - enter) * 1000.0 / Stopwatch.Frequency;
                _log.LogInfo($"[AssetBundleUnload] EXIT call#{s_callCount} elapsed={elapsedMs:F2}ms t={DateTime.UtcNow:o}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[AssetBundleUnload] Postfix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }
    }
}
