#nullable enable
// Iter-144 H9 instrumentation — Harmony Prefix+Postfix on AssetBundle.LoadFromFile(string).
// H7+H8 refuted (no ENTER fires on Unity destroy chain). Evidence: PackUnitSpawner.Initialize
// fires immediately before the wedge, so the suspect is mod-side pack-recreation. The first
// blocking sync I/O on pack reload is AssetBundle.LoadFromFile — probe ENTER/EXIT to determine
// if a bundle load is the wedge site.
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
    /// Iter-144 H9 instrumentation: Harmony Prefix+Postfix on
    /// <c>UnityEngine.AssetBundle.LoadFromFile(string)</c> (static method, single-arg overload).
    ///
    /// Probe semantics:
    /// - <see cref="Prefix"/> logs ENTER with the file path arg + truncated stack trace.
    /// - <see cref="Postfix"/> logs EXIT with elapsed milliseconds.
    /// - If ENTER fires without matching EXIT in the log → wedge target IDENTIFIED.
    /// </summary>
    internal static class AssetBundleLoadGuardPatch
    {
#pragma warning disable DF1006
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.AssetBundleLoad");
#pragma warning restore DF1006

        private static long s_enterTicks;
        private static int s_callCount;

        internal static void Apply(Harmony harmony)
        {
            try
            {
                MethodInfo? target = typeof(AssetBundle).GetMethod(
                    nameof(AssetBundle.LoadFromFile),
                    BindingFlags.Public | BindingFlags.Static,
                    binder: null,
                    types: new[] { typeof(string) },
                    modifiers: null);

                if (target == null)
                {
                    _log.LogWarning("[AssetBundleLoad] Could not resolve AssetBundle.LoadFromFile(string) — patch SKIPPED");
                    return;
                }

                var prefix = new HarmonyMethod(typeof(AssetBundleLoadGuardPatch)
                    .GetMethod(nameof(Prefix), BindingFlags.Static | BindingFlags.NonPublic));
                var postfix = new HarmonyMethod(typeof(AssetBundleLoadGuardPatch)
                    .GetMethod(nameof(Postfix), BindingFlags.Static | BindingFlags.NonPublic));

                harmony.Patch(target, prefix: prefix, postfix: postfix);
                _log.LogInfo("[AssetBundleLoad] Patched AssetBundle.LoadFromFile(string) — H9 probe active");
            }
            catch (Exception ex)
            {
                _log.LogError($"[AssetBundleLoad] Patch failed: {ex}");
            }
        }

        private static void Prefix(string path)
        {
            try
            {
                s_enterTicks = Stopwatch.GetTimestamp();
                int n = Interlocked.Increment(ref s_callCount);

                var stack = new StackTrace(skipFrames: 2, fNeedFileInfo: false).ToString();
                string[] lines = stack.Split('\n');
                string trimmed = string.Join("\n", lines.Take(6));

                _log.LogInfo($"[AssetBundleLoad] ENTER call#{n} path='{path}' t={DateTime.UtcNow:o} thread={Thread.CurrentThread.ManagedThreadId} from:\n{trimmed}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[AssetBundleLoad] Prefix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }

        private static void Postfix()
        {
            try
            {
                long enter = Interlocked.Read(ref s_enterTicks);
                double elapsedMs = (Stopwatch.GetTimestamp() - enter) * 1000.0 / Stopwatch.Frequency;
                _log.LogInfo($"[AssetBundleLoad] EXIT call#{s_callCount} elapsed={elapsedMs:F2}ms t={DateTime.UtcNow:o}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[AssetBundleLoad] Postfix-instrumentation threw: {ex}"); // pattern-96-ok: full ex rendered via ToString()
            }
        }
    }
}
