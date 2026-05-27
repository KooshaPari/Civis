#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Main-thread helpers to open the native pause menu for GameLaunch NATIVE-004 tests.
    /// Uses inactive object search and reflection because pause UI is often disabled until toggled.
    /// </summary>
    internal static class PauseMenuBridgeHelper
    {
        private static readonly string[] PauseRootNames =
        {
            "PauseMenu",
            "Pause",
            "PauseUI",
            "PauseScreen",
            "GamePause",
        };

        private static readonly (string TypeHint, string[] Methods)[] PauseInvokers =
        {
            ("PauseMenu", new[] { "Show", "Open", "Toggle", "TogglePause" }),
            ("PauseMenuManager", new[] { "TogglePause", "Show", "Open", "Pause" }),
            ("GamePause", new[] { "Toggle", "Pause", "Show" }),
            ("PauseManager", new[] { "TogglePause", "Toggle", "Show" }),
            ("Pause", new[] { "Toggle", "Show", "Open" }),
            ("GameState", new[] { "TogglePause", "Pause", "SetPaused" }),
        };

        /// <summary>Opens pause menu via hierarchy activation and pause-manager reflection.</summary>
        public static (bool success, string message) TryOpenPauseMenu()
        {
            var actions = new List<string>();

            try
            {
                if (TryActivatePauseHierarchyByResumeButton(actions))
                {
                    bool earlyVisible = IsPauseMenuVisible();
                    if (earlyVisible)
                    {
                        TryInjectModsOnActivePauseMenu(actions);
                        return (true, $"pauseVisible=true; {string.Join("; ", actions)}");
                    }
                }

                foreach (string rootName in PauseRootNames)
                {
                    GameObject? found = GameObject.Find(rootName);
                    if (found != null)
                    {
                        ActivateHierarchy(found);
                        actions.Add($"Find+activate:{found.name}");
                    }
                }

                foreach (GameObject go in Resources.FindObjectsOfTypeAll<GameObject>())
                {
                    if (go == null)
                    {
                        continue;
                    }

                    string goName = go.name ?? "";
                    foreach (string rootName in PauseRootNames)
                    {
                        if (goName.IndexOf(rootName, StringComparison.OrdinalIgnoreCase) < 0)
                        {
                            continue;
                        }

                        if (!go.activeInHierarchy)
                        {
                            ActivateHierarchy(go);
                            actions.Add($"Resources+activate:{goName}");
                        }
                    }
                }

                foreach ((string typeHint, string[] methods) in PauseInvokers)
                {
                    TryInvokePauseTargets(typeHint, methods, actions);
                }

                TryInvokePauseTypesFromGameAssemblies(actions);

                bool visible = IsPauseMenuVisible();
                if (visible)
                {
                    TryInjectModsOnActivePauseMenu(actions);
                }

                string summary = actions.Count > 0
                    ? string.Join("; ", actions)
                    : "no pause targets matched";

                DebugLog.Write("PauseMenuBridgeHelper",
                    $"[PauseMenuBridgeHelper] TryOpenPauseMenu visible={visible} actions={summary}");

                return (visible, $"pauseVisible={visible}; {summary}");
            }
            catch (Exception ex)
            {
                DebugLog.Write("PauseMenuBridgeHelper", $"[PauseMenuBridgeHelper] TryOpenPauseMenu failed: {ex}");
                return (false, $"{ex.GetType().Name}: {ex.Message}");
            }
        }

        public static bool IsPauseMenuVisible()
        {
            foreach (Button btn in Resources.FindObjectsOfTypeAll<Button>())
            {
                if (btn == null || !btn.gameObject.activeInHierarchy)
                {
                    continue;
                }

                string label = GetButtonLabel(btn);
                if (IsPauseResumeLabel(label))
                {
                    return true;
                }
            }

            return false;
        }

        private static bool TryActivatePauseHierarchyByResumeButton(List<string> actions)
        {
            bool activated = false;

            foreach (Button btn in Resources.FindObjectsOfTypeAll<Button>())
            {
                if (btn == null)
                {
                    continue;
                }

                string label = GetButtonLabel(btn);
                if (!IsPauseResumeLabel(label))
                {
                    continue;
                }

                ActivateHierarchy(btn.gameObject);
                actions.Add($"ResumeButton+activate:{btn.gameObject.name}");
                activated = true;
            }

            return activated;
        }

        private static void TryInvokePauseTargets(string typeHint, string[] methods, List<string> actions)
        {
            foreach (MonoBehaviour mb in Resources.FindObjectsOfTypeAll<MonoBehaviour>())
            {
                if (mb == null)
                {
                    continue;
                }

                string typeName = mb.GetType().Name;
                string goName = mb.gameObject.name;
                bool matches = typeName.IndexOf(typeHint, StringComparison.OrdinalIgnoreCase) >= 0
                    || goName.IndexOf(typeHint, StringComparison.OrdinalIgnoreCase) >= 0;
                if (!matches)
                {
                    continue;
                }

                if (!mb.gameObject.activeInHierarchy)
                {
                    ActivateHierarchy(mb.gameObject);
                    actions.Add($"activate-before-invoke:{typeName}@{goName}");
                }

                foreach (string methodName in methods)
                {
                    if (!TryInvokeVoidMethod(mb, methodName, out string invokeDetail))
                    {
                        continue;
                    }

                    actions.Add(invokeDetail);
                }
            }
        }

        private static void TryInvokePauseTypesFromGameAssemblies(List<string> actions)
        {
            string[] methodNames = { "TogglePause", "Toggle", "Pause", "Show", "Open" };

            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                string asmName = asm.GetName().Name ?? "";
                if (!asmName.StartsWith("Assembly-CSharp", StringComparison.Ordinal)
                    && !asmName.StartsWith("DNO.", StringComparison.Ordinal)
                    && !asmName.StartsWith("Door407.", StringComparison.Ordinal))
                {
                    continue;
                }

                Type[] types;
                try
                {
                    types = asm.GetTypes();
                }
                catch (ReflectionTypeLoadException ex)
                {
                    types = ex.Types?.Where(t => t != null).Cast<Type>().ToArray()
                        ?? Array.Empty<Type>();
                }

                foreach (Type type in types)
                {
                    if (!typeof(MonoBehaviour).IsAssignableFrom(type))
                    {
                        continue;
                    }

                    string typeName = type.Name;
                    if (typeName.IndexOf("Pause", StringComparison.OrdinalIgnoreCase) < 0)
                    {
                        continue;
                    }

                    foreach (UnityEngine.Object obj in Resources.FindObjectsOfTypeAll(type))
                    {
                        if (obj is not MonoBehaviour mb)
                        {
                            continue;
                        }

                        if (!mb.gameObject.activeInHierarchy)
                        {
                            ActivateHierarchy(mb.gameObject);
                            actions.Add($"asm-activate:{typeName}@{mb.gameObject.name}");
                        }

                        foreach (string methodName in methodNames)
                        {
                            if (!TryInvokeVoidMethod(mb, methodName, out string invokeDetail))
                            {
                                continue;
                            }

                            actions.Add(invokeDetail);
                        }
                    }
                }
            }
        }

        private static void TryInjectModsOnActivePauseMenu(List<string> actions)
        {
            foreach (MonoBehaviour mb in Resources.FindObjectsOfTypeAll<MonoBehaviour>())
            {
                if (mb == null)
                {
                    continue;
                }

                if (!string.Equals(mb.GetType().Name, "NativeMenuInjector", StringComparison.Ordinal))
                {
                    continue;
                }

                if (!mb.gameObject.activeInHierarchy)
                {
                    ActivateHierarchy(mb.gameObject);
                }

                if (!TryInvokeVoidMethod(mb, "TryInjectMenuButton", out string injectDetail))
                {
                    continue;
                }

                actions.Add(injectDetail);
            }
        }

        private static void ActivateHierarchy(GameObject go)
        {
            Transform? current = go.transform;
            while (current != null)
            {
                if (!current.gameObject.activeSelf)
                {
                    current.gameObject.SetActive(true);
                }

                current = current.parent;
            }
        }

        private static bool TryInvokeVoidMethod(MonoBehaviour mb, string methodName, out string detail)
        {
            detail = "";
            MethodInfo? mi = mb.GetType().GetMethod(
                methodName,
                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance,
                null,
                Type.EmptyTypes,
                null);
            if (mi == null || mi.ReturnType != typeof(void))
            {
                return false;
            }

            try
            {
                mi.Invoke(mb, null);
            }
            catch (Exception ex)
            {
                detail = $"{mb.GetType().Name}.{methodName}() failed: {ex.GetType().Name}";
                return false;
            }

            detail = $"{mb.GetType().Name}.{methodName}() on '{mb.gameObject.name}'";
            return true;
        }

        private static string GetButtonLabel(Button btn)
        {
            Text? legacy = btn.GetComponentInChildren<Text>(true);
            if (!string.IsNullOrWhiteSpace(legacy?.text))
            {
                return legacy.text.Trim();
            }

            var tmp = btn.GetComponentInChildren<TMPro.TMP_Text>(true);
            return (tmp?.text ?? "").Trim();
        }

        private static bool IsPauseResumeLabel(string label) =>
            label.Equals("Resume", StringComparison.OrdinalIgnoreCase)
            || label.Equals("Continue", StringComparison.OrdinalIgnoreCase)
            || label.IndexOf("resume", StringComparison.OrdinalIgnoreCase) >= 0;
    }
}
