#nullable enable
using HarmonyLib;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Harmony patches to prevent DINO's UiGrid (or any UI update loop) from overwriting
    /// the "Mods" label on our repurposed Options button.
    ///
    /// Strategy: patch <see cref="UnityEngine.UI.Text.text"/> setter and
    /// <see cref="TMPro.TMP_Text.text"/> setter. When the target <see cref="GameObject"/>
    /// is our repurposed Mods button AND the incoming value is "OPTIONS" (case-insensitive),
    /// substitute "Mods" instead.
    ///
    /// This is intentionally narrow: we only intercept the exact game-object name
    /// <c>NativeMenuInjector.RepurposedModsButtonGoName</c>, so the patch is transparent
    /// to every other Text component in the game.
    /// </summary>
    [HarmonyPatch]
    internal static class ModsButtonTextPatch
    {
        // ──────────────────────────────────────────────────────────────────────
        // UnityEngine.UI.Text (UGUI legacy text)
        // ──────────────────────────────────────────────────────────────────────

        [HarmonyPatch(typeof(Text), nameof(Text.text), MethodType.Setter)]
        [HarmonyPrefix]
        static bool PatchUguiTextSetter(Text __instance, ref string value)
        {
            return ApplyModsSubstitution(__instance?.gameObject, ref value);
        }

        // ──────────────────────────────────────────────────────────────────────
        // TMPro.TMP_Text (TextMeshPro — used by DINO's main menu)
        // ──────────────────────────────────────────────────────────────────────

        [HarmonyPatch(typeof(TMPro.TMP_Text), nameof(TMPro.TMP_Text.text), MethodType.Setter)]
        [HarmonyPrefix]
        static bool PatchTmpTextSetter(TMPro.TMP_Text __instance, ref string value)
        {
            return ApplyModsSubstitution(__instance?.gameObject, ref value);
        }

        // ──────────────────────────────────────────────────────────────────────
        // Shared substitution logic
        // ──────────────────────────────────────────────────────────────────────

        /// <summary>
        /// If <paramref name="go"/> is part of our repurposed Mods button GameObject
        /// (matched by name on its own transform or its parent) and <paramref name="value"/>
        /// is "OPTIONS" (case-insensitive), replaces the value with "Mods".
        /// </summary>
        /// <returns>Always <c>true</c> — the original setter always runs; we only alter the value.</returns>
        private static bool ApplyModsSubstitution(GameObject? go, ref string value)
        {
            string? targetName = NativeMenuInjector.RepurposedModsButtonGoName;
            if (targetName == null || go == null) return true;

            // The Text component may be a child of the button GameObject, so walk up one level.
            bool isTarget = go.name == targetName
                            || (go.transform.parent != null && go.transform.parent.name == targetName);

            if (isTarget && string.Compare(value, "OPTIONS", System.StringComparison.OrdinalIgnoreCase) == 0)
            {
                value = "Mods";
            }
            return true;
        }
    }
}
