#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Runtime.UI;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime
{
    /// <summary>
    /// IMGUI debug overlay MonoBehaviour showing ECS world state,
    /// loaded packs, and system status.
    /// Toggled via F9. Lives on the persistent DINOForge_Root GameObject.
    /// Window is 350 px wide (dev-only tool). Keyboard shortcuts strip is shown at the top.
    /// </summary>
    public class DebugOverlayBehaviour : MonoBehaviour
    {
        /// <summary>
        /// Static singleton — set in Awake, cleared in OnDestroy.
        /// Allows KeyInputSystem (ECS, survives scene transitions) to toggle
        /// the overlay without a MonoBehaviour reference chain.
        /// </summary>
        public static DebugOverlayBehaviour? Instance { get; private set; }

        private bool _visible;
        private Vector2 _scrollPosition;
        private Rect _windowRect = new Rect(10, 10, 350, 580);
        private bool _showSystems;
        private bool _showArchetypes;
        private bool _showErrors;
        private ModPlatform? _modPlatform;

        /// <summary>Whether the debug overlay is currently visible.</summary>
        public bool IsVisible => _visible;

        /// <summary>
        /// Provides a reference to the ModPlatform for status display.
        /// </summary>
        /// <param name="modPlatform">The mod platform orchestrator.</param>
        public void SetModPlatform(ModPlatform? modPlatform)
        {
            _modPlatform = modPlatform;
        }

        /// <summary>
        /// Toggles the debug overlay visibility.
        /// Called by RuntimeDriver.Update() which owns all F-key handling.
        /// </summary>
        public void Toggle()
        {
            _visible = !_visible;
        }

        private void Awake() => Instance = this;
        private void OnDestroy() { if (Instance == this) Instance = null; }

        // ── Unity lifecycle ────────────────────────────────────────────────────────

        private void OnGUI()
        {
            if (!_visible) return;

            string header = BuildHeader();
            _windowRect = GUI.Window(9999, _windowRect, DrawWindow, header, DinoForgeStyle.WindowStyle);
        }

        // ── Header ─────────────────────────────────────────────────────────────────

        private string BuildHeader()
        {
            World? defaultWorld = null;
            try { defaultWorld = World.DefaultGameObjectInjectionWorld; }
            catch { /* not ready */ }

            if (defaultWorld != null && defaultWorld.IsCreated)
            {
                int entityCount = 0;
                int systemCount = 0;
                try
                {
                    NativeArray<Entity> entities = defaultWorld.EntityManager.GetAllEntities(Allocator.Temp);
                    entityCount = entities.Length;
                    entities.Dispose();
                    systemCount = defaultWorld.Systems.Count;
                }
                catch { /* ignore */ }

                return $"[DF] DEBUG  {defaultWorld.Name}  {entityCount} ent / {systemCount} sys";
            }

            return $"[DF] DEBUG v{PluginInfo.VERSION}  waiting for ECS";
        }

        // ── Window drawing ─────────────────────────────────────────────────────────

        private void DrawWindow(int windowId)
        {
            // ── Keyboard shortcuts strip at the TOP ───────────────────────────────
            GUILayout.BeginHorizontal(DinoForgeStyle.BoxStyle);
            GUILayout.Label("F8 = Dump  |  F9 = Debug  |  F10 = Menu", DinoForgeStyle.SectionLabelStyle);
            GUILayout.EndHorizontal();

            GUILayout.Space(4);

            _scrollPosition = GUILayout.BeginScrollView(_scrollPosition);

            // ── Platform status ───────────────────────────────────────────────────
            GUILayout.Label("Platform Status", DinoForgeStyle.HeaderStyle);

            if (_modPlatform != null)
            {
                DrawStatusRow("Initialized", _modPlatform.IsInitialized);
                DrawStatusRow("World Ready", _modPlatform.IsWorldReady);
                GUILayout.Label($"  Packs: {_modPlatform.PacksDirectory}", DinoForgeStyle.BodyLabelStyle);

                if (_modPlatform.ContentLoader != null)
                {
                    int errorCount = _modPlatform.ContentLoader.LastLoadErrorCount;
                    if (errorCount > 0)
                    {
                        GUILayout.Label($"  Load Errors: {errorCount}", DinoForgeStyle.ErrorLabelStyle);
                    }
                }
            }
            else
            {
                GUILayout.Label("  ModPlatform: not available", DinoForgeStyle.BodyLabelStyle);
            }

            GUILayout.Space(8);

            // ── ECS Worlds ────────────────────────────────────────────────────────
            GUILayout.Label("ECS Worlds", DinoForgeStyle.HeaderStyle);

            if (World.All.Count > 0)
            {
                foreach (World world in World.All)
                {
                    if (!world.IsCreated) continue;

                    GUILayout.Label($"  {world.Name}", DinoForgeStyle.BodyLabelStyle);

                    try
                    {
                        EntityManager em = world.EntityManager;
                        NativeArray<Entity> entities = em.GetAllEntities(Allocator.Temp);
                        int systemCount = world.Systems.Count;
                        GUILayout.Label($"    Entities: {entities.Length}  Systems: {systemCount}", DinoForgeStyle.BodyLabelStyle);
                        entities.Dispose();
                    }
                    catch (Exception ex)
                    {
                        GUILayout.Label($"    Error: {ex.Message}", DinoForgeStyle.ErrorLabelStyle);
                    }

                    GUILayout.Space(3);
                }
            }
            else
            {
                GUILayout.Label("  No worlds found.", DinoForgeStyle.BodyLabelStyle);
            }

            GUILayout.Space(8);

            // ── Collapsible: Systems ──────────────────────────────────────────────
            string sysArrow = _showSystems ? "▼" : "▶";
            if (GUILayout.Button($"{sysArrow}  Show Systems", DinoForgeStyle.ButtonStyle))
            {
                _showSystems = !_showSystems;
            }

            if (_showSystems)
            {
                DrawSystems();
            }

            GUILayout.Space(4);

            // ── Collapsible: Archetypes ────────────────────────────────────────────
            string archArrow = _showArchetypes ? "▼" : "▶";
            if (GUILayout.Button($"{archArrow}  Show Archetypes (top 20)", DinoForgeStyle.ButtonStyle))
            {
                _showArchetypes = !_showArchetypes;
            }

            if (_showArchetypes)
            {
                DrawArchetypes();
            }

            // ── Collapsible: Errors ───────────────────────────────────────────────
            if (_modPlatform?.ContentLoader != null && _modPlatform.ContentLoader.LastLoadErrorCount > 0)
            {
                GUILayout.Space(4);
                int errCount = _modPlatform.ContentLoader.LastLoadErrorCount;
                string errArrow = _showErrors ? "▼" : "▶";
                if (GUILayout.Button($"{errArrow}  Show Errors ({errCount})", DinoForgeStyle.ButtonStyle))
                {
                    _showErrors = !_showErrors;
                }

                if (_showErrors)
                {
                    DrawErrors();
                }
            }

            GUILayout.EndScrollView();
            GUI.DragWindow();
        }

        // ── Section renderers ──────────────────────────────────────────────────────

        private static void DrawStatusRow(string label, bool value)
        {
            GUILayout.BeginHorizontal();
            GUILayout.Label($"  {label}:", DinoForgeStyle.BodyLabelStyle, GUILayout.Width(110));
            GUILayout.Label(value ? "ON" : "OFF", value ? DinoForgeStyle.SuccessLabelStyle : DinoForgeStyle.WarningLabelStyle);
            GUILayout.EndHorizontal();
        }

        private void DrawSystems()
        {
            if (World.All.Count == 0) return;

            foreach (World world in World.All)
            {
                if (!world.IsCreated) continue;

                try
                {
                    var systems = world.Systems;
                    int limit = Math.Min(systems.Count, 30);
                    for (int i = 0; i < limit; i++)
                    {
                        ComponentSystemBase system = systems[i];
                        bool on = system.Enabled;
                        GUILayout.BeginHorizontal();
                        DinoForgeStyle.StatusBadge(on ? "ON" : "OFF", on ? DinoForgeStyle.Success : DinoForgeStyle.TextMuted, 30f);
                        GUILayout.Space(4);
                        GUILayout.Label(system.GetType().Name, DinoForgeStyle.BodyLabelStyle);
                        GUILayout.EndHorizontal();
                    }
                }
                catch { /* ignore */ }
            }
        }

        private void DrawArchetypes()
        {
            if (World.All.Count == 0) return;

            foreach (World world in World.All)
            {
                if (!world.IsCreated) continue;

                try
                {
                    EntityManager em = world.EntityManager;
                    NativeArray<Entity> entities = em.GetAllEntities(Allocator.Temp);

                    Dictionary<string, int> archetypeCounts = new Dictionary<string, int>();

                    foreach (Entity entity in entities)
                    {
                        try
                        {
                            NativeArray<ComponentType> types = em.GetComponentTypes(entity, Allocator.Temp);
                            string key = string.Join(", ", types
                                .Select(t => t.GetManagedType()?.Name ?? "?")
                                .OrderBy(n => n)
                                .Take(5));
                            if (types.Length > 5) key += $" (+{types.Length - 5} more)";

                            if (!archetypeCounts.ContainsKey(key))
                                archetypeCounts[key] = 0;
                            archetypeCounts[key]++;
                            types.Dispose();
                        }
                        catch { /* ignore */ }
                    }

                    foreach (KeyValuePair<string, int> kvp in archetypeCounts.OrderByDescending(k => k.Value).Take(20))
                    {
                        GUILayout.Label($"  [{kvp.Value}x] {kvp.Key}", DinoForgeStyle.BodyLabelStyle);
                    }

                    entities.Dispose();
                }
                catch { /* ignore */ }
            }
        }

        private void DrawErrors()
        {
            if (_modPlatform?.ContentLoader == null) return;

            IReadOnlyList<string>? errors = _modPlatform.ContentLoader.LastLoadErrors;
            if (errors == null || errors.Count == 0)
            {
                GUILayout.Label("  (No errors available)", DinoForgeStyle.BodyLabelStyle);
                return;
            }

            // Copy to clipboard button
            if (GUILayout.Button("Copy to Clipboard", DinoForgeStyle.ButtonStyle, GUILayout.Width(140)))
            {
                GUIUtility.systemCopyBuffer = string.Join("\n", errors);
            }

            int maxShow = Math.Min(10, errors.Count);
            for (int i = 0; i < maxShow; i++)
            {
                string error = errors[i];
                string display = error.Length > 100 ? error.Substring(0, 97) + "..." : error;
                GUILayout.Label($"  - {display}", DinoForgeStyle.ErrorLabelStyle);
            }

            if (errors.Count > 10)
            {
                GUILayout.Label($"  ... and {errors.Count - 10} more (see F10 mod menu)", DinoForgeStyle.WarningLabelStyle);
            }
        }
    }
}
