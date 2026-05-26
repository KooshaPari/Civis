#nullable enable
// Iter-144 H8 instrumentation — Harmony Prefix+Postfix on SceneManager.UnloadSceneAsync(Scene).
// Companion to ResourcesUnloadGuardPatch (H7 refuted). New evidence: wedge happens during
// MainMenu transition + pack-recreation. Scene unload of InitialGameLoader is a candidate.
#pragma warning disable DF0103
using System;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Threading;
using HarmonyLib;
using UnityEngine.SceneManagement;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Iter-144 H8 instrumentation: Harmony Prefix+Postfix on
    /// <c>UnityEngine.SceneManagement.SceneManager.UnloadSceneAsync(Scene)</c> (static).
    ///
    /// Probe semantics:
    /// - <see cref="Prefix"/> logs ENTER with scene name + truncated stack trace.
    /// - <see cref="Postfix"/> logs EXIT with elapsed milliseconds.
    /// - If ENTER fires without matching EXIT → wedge target IDENTIFIED.
    /// </summary>
    internal static class SceneUnloadGuardPatch
    {
#pragma warning disable DF1006
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.SceneUnload");
#pragma warning restore DF1006

        private static long s_enterTicks;
        private static int s_callCount;

        internal static void Apply(Harmony harmony)
        {
            try
            {
                MethodInfo? target = typeof(SceneManager).GetMethod(
                    nameof(SceneManager.UnloadSceneAsync),
                    BindingFlags.Public | BindingFlags.Static,
                    binder: null,
                    types: new[] { typeof(Scene) },
                    modifiers: null);

                if (target == null)
                {
                    _log.LogWarning("[SceneUnload] Could not resolve SceneManager.UnloadSceneAsync(Scene) — patch SKIPPED");
                    return;
                }

                var prefix = new HarmonyMethod(typeof(SceneUnloadGuardPatch)
                    .GetMethod(nameof(Prefix), BindingFlags.Static | BindingFlags.NonPublic));
                var postfix = new HarmonyMethod(typeof(SceneUnloadGuardPatch)
                    .GetMethod(nameof(Postfix), BindingFlags.Static | BindingFlags.NonPublic));

                harmony.Patch(target, prefix: prefix, postfix: postfix);
                _log.LogInfo("[SceneUnload] Patched SceneManager.UnloadSceneAsync(Scene) — H8 probe active");
            }
            catch (Exception ex)
            {
                _log.LogError($"[SceneUnload] Patch failed: {ex}");
            }
        }

        private static void Prefix(Scene scene)
        {
            try
            {
                s_enterTicks = Stopwatch.GetTimestamp();
                int n = Interlocked.Increment(ref s_callCount);

                string sceneName;
                try { sceneName = scene.IsValid() ? scene.name : "<invalid>"; }
                catch { sceneName = "<unreadable>"; }

                var stack = new StackTrace(skipFrames: 2, fNeedFileInfo: false).ToString();
                string[] lines = stack.Split('\n');
                string trimmed = string.Join("\n", lines.Take(6));

                _log.LogInfo($"[SceneUnload] ENTER call#{n} scene='{sceneName}' t={DateTime.UtcNow:o} from:\n{trimmed}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[SceneUnload] Prefix-instrumentation threw: {ex}"); // pattern-96-ok
            }
        }

        private static void Postfix()
        {
            try
            {
                long enter = Interlocked.Read(ref s_enterTicks);
                double elapsedMs = (Stopwatch.GetTimestamp() - enter) * 1000.0 / Stopwatch.Frequency;
                _log.LogInfo($"[SceneUnload] EXIT call#{s_callCount} elapsed={elapsedMs:F2}ms t={DateTime.UtcNow:o}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[SceneUnload] Postfix-instrumentation threw: {ex}"); // pattern-96-ok
            }
        }
    }
}
