#nullable enable
// Iter-144 H8 instrumentation — Harmony Prefix+Postfix on Unity.Entities.World.Dispose().
// Companion to ResourcesUnloadGuardPatch (H7 refuted). New evidence: pack-recreation happens
// after MainMenu transition. World.Dispose() teardown of the Default World (45K entities)
// during scene transition is a top candidate for the wedge.
#pragma warning disable DF0103
using System;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Threading;
using HarmonyLib;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Iter-144 H8 instrumentation: Harmony Prefix+Postfix on
    /// <c>Unity.Entities.World.Dispose()</c> (instance, parameterless).
    ///
    /// Probe semantics:
    /// - <see cref="Prefix"/> logs ENTER with world name + truncated stack trace.
    /// - <see cref="Postfix"/> logs EXIT with elapsed milliseconds.
    /// - If ENTER fires without matching EXIT → wedge target IDENTIFIED.
    ///
    /// Note: Unity.Entities.World type is resolved by name (the assembly is loaded by the
    /// game; we cannot take a direct typeof() reference without a project ref). If the
    /// type cannot be resolved, the patch is logged-and-skipped (defense-in-depth).
    /// </summary>
    internal static class WorldDisposeGuardPatch
    {
#pragma warning disable DF1006
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.WorldDispose");
#pragma warning restore DF1006

        private static long s_enterTicks;
        private static int s_callCount;
        private static PropertyInfo? s_nameProp;

        internal static void Apply(Harmony harmony)
        {
            try
            {
                Type? worldType = AppDomain.CurrentDomain.GetAssemblies()
                    .Select(a =>
                    {
                        try { return a.GetType("Unity.Entities.World", throwOnError: false); }
                        catch { return null; } // safe-swallow: type lookup can fail on partially loaded assemblies during startup
                    })
                    .FirstOrDefault(t => t != null);

                if (worldType == null)
                {
                    _log.LogWarning("[WorldDispose] Could not resolve Unity.Entities.World — patch SKIPPED");
                    return;
                }

                s_nameProp = worldType.GetProperty("Name", BindingFlags.Public | BindingFlags.Instance);

                MethodInfo? target = worldType.GetMethod(
                    "Dispose",
                    BindingFlags.Public | BindingFlags.Instance,
                    binder: null,
                    types: Type.EmptyTypes,
                    modifiers: null);

                if (target == null)
                {
                    _log.LogWarning("[WorldDispose] Could not resolve World.Dispose() — patch SKIPPED");
                    return;
                }

                var prefix = new HarmonyMethod(typeof(WorldDisposeGuardPatch)
                    .GetMethod(nameof(Prefix), BindingFlags.Static | BindingFlags.NonPublic));
                var postfix = new HarmonyMethod(typeof(WorldDisposeGuardPatch)
                    .GetMethod(nameof(Postfix), BindingFlags.Static | BindingFlags.NonPublic));

                harmony.Patch(target, prefix: prefix, postfix: postfix);
                _log.LogInfo("[WorldDispose] Patched Unity.Entities.World.Dispose() — H8 probe active");
            }
            catch (Exception ex)
            {
                _log.LogError($"[WorldDispose] Patch failed: {ex}");
            }
        }

        private static void Prefix(object __instance)
        {
            try
            {
                s_enterTicks = Stopwatch.GetTimestamp();
                int n = Interlocked.Increment(ref s_callCount);

                string worldName;
                try { worldName = s_nameProp?.GetValue(__instance) as string ?? "<unknown>"; }
                catch { worldName = "<unreadable>"; }

                var stack = new StackTrace(skipFrames: 2, fNeedFileInfo: false).ToString();
                string[] lines = stack.Split('\n');
                string trimmed = string.Join("\n", lines.Take(6));

                _log.LogInfo($"[WorldDispose] ENTER call#{n} world='{worldName}' t={DateTime.UtcNow:o} from:\n{trimmed}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[WorldDispose] Prefix-instrumentation threw: {ex}"); // pattern-96-ok
            }
        }

        private static void Postfix()
        {
            try
            {
                long enter = Interlocked.Read(ref s_enterTicks);
                double elapsedMs = (Stopwatch.GetTimestamp() - enter) * 1000.0 / Stopwatch.Frequency;
                _log.LogInfo($"[WorldDispose] EXIT call#{s_callCount} elapsed={elapsedMs:F2}ms t={DateTime.UtcNow:o}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[WorldDispose] Postfix-instrumentation threw: {ex}"); // pattern-96-ok
            }
        }
    }
}
