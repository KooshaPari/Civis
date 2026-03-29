#nullable enable
using System.Reflection;
using HarmonyLib;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Harmony patches to prevent DINO's UiGrid from overwriting the "Mods" label on our
    /// repurposed Options button.
    ///
    /// DINO's UiGrid calls <see cref="TMPro.TMP_Text.SetText(string,bool)"/> (not the .text
    /// property setter) to restore button labels every frame.  We patch that method directly.
    ///
    /// We also patch <see cref="UnityEngine.UI.Text.text"/> setter as a belt-and-suspenders
    /// fallback for legacy UGUI Text components.
    ///
    /// NOTE: Patching <c>TMP_Text.set_text</c> (virtual) fails on Mono/HarmonyX with a
    /// patching exception, so we avoid that and rely on the concrete <c>SetText</c> overloads.
    /// </summary>
    internal static class ModsButtonTextPatch
    {
        private static readonly BepInEx.Logging.ManualLogSource _log =
            BepInEx.Logging.Logger.CreateLogSource("DINOForge.ModsButtonTextPatch");

        /// <summary>
        /// Apply all patches manually so we can control exactly which methods are patched
        /// and avoid the <c>set_text</c> virtual patching exception.
        /// </summary>
        internal static void Apply(Harmony harmony)
        {
            var prefix = new HarmonyMethod(typeof(ModsButtonTextPatch)
                .GetMethod(nameof(SetTextPrefix), BindingFlags.Static | BindingFlags.NonPublic));

            // TMP_Text.SetText(string, bool) — primary path used by UiGrid
            var setTextStrBool = typeof(TMPro.TMP_Text).GetMethod(
                nameof(TMPro.TMP_Text.SetText),
                new[] { typeof(string), typeof(bool) });
            if (setTextStrBool != null)
            {
                harmony.Patch(setTextStrBool, prefix: prefix);
                _log.LogInfo("[ModsButtonTextPatch] Patched TMP_Text.SetText(string,bool)");
            }
            else
            {
                _log.LogWarning("[ModsButtonTextPatch] Could not find TMP_Text.SetText(string,bool)");
            }

            // UGUI legacy Text.set_text — fallback
            var uguiSetter = typeof(Text).GetProperty(nameof(Text.text))?.GetSetMethod();
            if (uguiSetter != null)
            {
                var uguiPrefix = new HarmonyMethod(typeof(ModsButtonTextPatch)
                    .GetMethod(nameof(UguiPrefix), BindingFlags.Static | BindingFlags.NonPublic));
                harmony.Patch(uguiSetter, prefix: uguiPrefix);
                _log.LogInfo("[ModsButtonTextPatch] Patched UnityEngine.UI.Text.set_text");
            }
        }

        // ── Prefix for TMP_Text.SetText(string sourceText, bool syncTextInputBox) ──
        // Parameter name must match the IL parameter name exactly ("sourceText").
        static bool SetTextPrefix(TMPro.TMP_Text __instance, ref string sourceText)
        {
            return ApplySubstitution(__instance?.gameObject, ref sourceText, "SetText(str,bool)");
        }

        // ── Prefix for UnityEngine.UI.Text.set_text ──
        static bool UguiPrefix(Text __instance, ref string value)
        {
            return ApplySubstitution(__instance?.gameObject, ref value, "UGUI.set_text");
        }

        private static bool ApplySubstitution(GameObject? go, ref string value, string src)
        {
            string? targetName = NativeMenuInjector.RepurposedModsButtonGoName;
            if (targetName == null || go == null) return true;

            if (string.Compare(value, "OPTIONS", System.StringComparison.OrdinalIgnoreCase) == 0)
            {
                // Walk up full hierarchy to find the button root.
                Transform? t = go.transform;
                while (t != null)
                {
                    if (t.name == targetName)
                    {
                        _log.LogInfo($"[{src}] INTERCEPTED OPTIONS→Mods on '{go.name}'");
                        value = "Mods";
                        return true;
                    }
                    t = t.parent;
                }
                // Log near-misses for diagnosis when target is set but hierarchy doesn't match.
                _log.LogInfo($"[{src}] OPTIONS on GO='{go.name}' parent='{go.transform.parent?.name}' (target='{targetName}' not in hierarchy)");
            }
            return true;
        }
    }
}
