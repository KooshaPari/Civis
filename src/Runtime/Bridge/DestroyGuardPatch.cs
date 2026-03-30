#nullable enable
using System.Reflection;
using HarmonyLib;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Harmony patches that prevent Unity/DINO from destroying the DINOForge persistent
    /// root GameObject during scene transitions.
    ///
    /// DINO's scene management calls <c>SceneManager.LoadScene()</c> which triggers Unity
    /// to unload previous scenes and destroy all GameObjects — including those marked with
    /// <c>HideFlags.HideAndDontSave</c> and <c>DontDestroyOnLoad</c>. This patch intercepts
    /// <c>Object.Destroy(Object)</c> and <c>Object.DestroyImmediate(Object)</c> to skip
    /// destruction of any object named "DINOForge_Root" or carrying a <see cref="RuntimeDriver"/>.
    /// </summary>
    internal static class DestroyGuardPatch
    {
        private const string RootName = "DINOForge_Root";

        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.DestroyGuard");

        /// <summary>
        /// Apply all destroy-guard patches via the shared Harmony instance.
        /// </summary>
        internal static void Apply(Harmony harmony)
        {
            var prefix = new HarmonyMethod(typeof(DestroyGuardPatch)
                .GetMethod(nameof(DestroyPrefix), BindingFlags.Static | BindingFlags.NonPublic));

            // Patch Object.Destroy(Object obj) — called by Unity during scene unload
            MethodInfo? destroy = typeof(Object).GetMethod(
                nameof(Object.Destroy),
                new[] { typeof(Object) });
            if (destroy != null)
            {
                harmony.Patch(destroy, prefix: prefix);
                _log.LogInfo("[DestroyGuard] Patched Object.Destroy(Object)");
            }

            // Patch Object.Destroy(Object obj, float t) — timed destruction
            MethodInfo? destroyTimed = typeof(Object).GetMethod(
                nameof(Object.Destroy),
                new[] { typeof(Object), typeof(float) });
            if (destroyTimed != null)
            {
                var timedPrefix = new HarmonyMethod(typeof(DestroyGuardPatch)
                    .GetMethod(nameof(DestroyTimedPrefix), BindingFlags.Static | BindingFlags.NonPublic));
                harmony.Patch(destroyTimed, prefix: timedPrefix);
                _log.LogInfo("[DestroyGuard] Patched Object.Destroy(Object, float)");
            }

            // Patch Object.DestroyImmediate(Object obj) — synchronous destruction
            MethodInfo? destroyImmediate = typeof(Object).GetMethod(
                nameof(Object.DestroyImmediate),
                new[] { typeof(Object) });
            if (destroyImmediate != null)
            {
                harmony.Patch(destroyImmediate, prefix: prefix);
                _log.LogInfo("[DestroyGuard] Patched Object.DestroyImmediate(Object)");
            }

            // Patch Object.DestroyImmediate(Object obj, bool allowDestroyingAssets)
            MethodInfo? destroyImmediateAllow = typeof(Object).GetMethod(
                nameof(Object.DestroyImmediate),
                new[] { typeof(Object), typeof(bool) });
            if (destroyImmediateAllow != null)
            {
                var allowPrefix = new HarmonyMethod(typeof(DestroyGuardPatch)
                    .GetMethod(nameof(DestroyImmediateAllowPrefix), BindingFlags.Static | BindingFlags.NonPublic));
                harmony.Patch(destroyImmediateAllow, prefix: allowPrefix);
                _log.LogInfo("[DestroyGuard] Patched Object.DestroyImmediate(Object, bool)");
            }
        }

        /// <summary>
        /// Prefix for Object.Destroy(Object obj).
        /// Returns false (skip original) if the target is our persistent root.
        /// </summary>
        static bool DestroyPrefix(Object obj)
        {
            return !ShouldGuard(obj);
        }

        /// <summary>
        /// Prefix for Object.Destroy(Object obj, float t).
        /// </summary>
        static bool DestroyTimedPrefix(Object obj)
        {
            return !ShouldGuard(obj);
        }

        /// <summary>
        /// Prefix for Object.DestroyImmediate(Object obj, bool allowDestroyingAssets).
        /// </summary>
        static bool DestroyImmediateAllowPrefix(Object obj)
        {
            return !ShouldGuard(obj);
        }

        /// <summary>
        /// Returns true if the object should be protected from destruction.
        /// Checks the object name and walks up the transform hierarchy to catch
        /// child objects that are part of the DINOForge root.
        /// </summary>
        private static bool ShouldGuard(Object obj)
        {
            if (obj == null) return false;

            // Direct name check (fast path)
            if (obj.name == RootName)
            {
                _log.LogInfo($"[DestroyGuard] BLOCKED destruction of '{obj.name}'");
                return true;
            }

            // Check if it's a GameObject or Component attached to our root
            GameObject? go = null;
            if (obj is GameObject gameObj)
                go = gameObj;
            else if (obj is Component comp)
                go = comp.gameObject;

            if (go == null) return false;

            // Walk up the hierarchy to check if any parent is our root
            Transform? t = go.transform;
            while (t != null)
            {
                if (t.gameObject.name == RootName)
                {
                    _log.LogInfo($"[DestroyGuard] BLOCKED destruction of '{obj.name}' (child of {RootName})");
                    return true;
                }
                t = t.parent;
            }

            return false;
        }
    }
}
