#nullable enable
using System;
using Unity.Entities;
using UnityEngine;
using UnityEngine.LowLevel;
using HarmonyLib;
using DINOForge.Runtime;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS system that handles F9/F10 key input and owns the IMGUI overlay.
    /// ECS systems survive DINO's scene transitions (unlike MonoBehaviours).
    ///
    /// Uses a Harmony postfix on PlayerLoop.SetPlayerLoop to re-inject our
    /// DINOForgeKeyLoop entry every time DINO rebuilds the player loop.
    /// This ensures F9/F10 work regardless of DINO's boot cycle.
    /// </summary>
    [AlwaysUpdateSystem]
    [UpdateInGroup(typeof(InitializationSystemGroup))]
    public class KeyInputSystem : SystemBase
    {
        /// <summary>Called when F9 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF9Pressed;

        /// <summary>Called when F10 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF10Pressed;

        private int _updateFrame;

        private static int _playerLoopTickCount = 0;
        private static bool _harmonyPatched = false;

        protected override void OnCreate()
        {
            WriteDebug("KeyInputSystem.OnCreate");
            // Patch PlayerLoop.SetPlayerLoop with Harmony so we re-inject after every
            // time DINO rebuilds its player loop (happens during world tear-down/creation).
            PatchPlayerLoopSetPlayerLoop();
            // Also inject immediately in case the loop is already set.
            InjectIntoPlayerLoop();
        }

        protected override void OnUpdate()
        {
            try
            {
                _updateFrame++;
                // Log every frame for first 5 frames, then every 600
                if (_updateFrame <= 5 || _updateFrame % 600 == 0)
                    WriteDebug($"OnUpdate frame={_updateFrame} PersistentRoot={(Plugin.PersistentRoot != null ? "alive" : "null")}");

                // Resurrect PersistentRoot if needed (runs on main thread — safe to call Unity APIs)
                if (Plugin.NeedsResurrection && Plugin.PersistentRoot == null)
                {
                    WriteDebug("Resurrection triggered from KeyInputSystem (main thread)");
                    Plugin.NeedsResurrection = false;
                    try
                    {
                        GameObject root = new GameObject("DINOForge_Root");
                        root.hideFlags = HideFlags.HideAndDontSave;
                        UnityEngine.Object.DontDestroyOnLoad(root);
                        Plugin.PersistentRoot = root;
                        RuntimeDriver driver = root.AddComponent<RuntimeDriver>();
                        driver.Initialize(Plugin.ResurrectionLog, Plugin.ResurrectionConfig, Plugin.ResurrectionDump, Plugin.ResurrectionDumpPath);
                        WriteDebug("Resurrection complete — new root created from KeyInputSystem");
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"Resurrection FAILED in KeyInputSystem: {ex.Message}");
                    }
                }

                if (Input.GetKeyDown(KeyCode.F9))
                {
                    WriteDebug("F9 pressed");
                    OnF9Pressed?.Invoke();
                }

                if (Input.GetKeyDown(KeyCode.F10))
                {
                    WriteDebug("F10 pressed");
                    OnF10Pressed?.Invoke();
                }
            }
            catch { }
        }


        private static void PatchPlayerLoopSetPlayerLoop()
        {
            if (_harmonyPatched) return;
            try
            {
                var harmony = new Harmony("dinoforge.keyinput.playerloop");
                var original = typeof(PlayerLoop).GetMethod(
                    nameof(PlayerLoop.SetPlayerLoop),
                    System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (original == null)
                {
                    WriteDebug("KeyInputSystem: PlayerLoop.SetPlayerLoop method not found via reflection");
                    return;
                }
                var postfix = typeof(KeyInputSystem).GetMethod(
                    nameof(SetPlayerLoopPostfix),
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                harmony.Patch(original, postfix: new HarmonyMethod(postfix));
                _harmonyPatched = true;
                WriteDebug("KeyInputSystem: Harmony postfix on PlayerLoop.SetPlayerLoop applied");
            }
            catch (Exception ex)
            {
                WriteDebug($"KeyInputSystem: Harmony patch failed: {ex.GetType().Name}: {ex.Message}");
            }
        }

        // Postfix called after every PlayerLoop.SetPlayerLoop — re-injects our entry.
        private static void SetPlayerLoopPostfix() => InjectIntoPlayerLoop();

        private static volatile bool _injectingNow = false;

        private static void InjectIntoPlayerLoop()
        {
            // Reentrancy guard: our own SetPlayerLoop call would re-trigger the Harmony postfix
            if (_injectingNow) return;
            _injectingNow = true;
            try
            {
                var loop = PlayerLoop.GetCurrentPlayerLoop();
                var rootSubsystems = new System.Collections.Generic.List<PlayerLoopSystem>(loop.subSystemList ?? System.Array.Empty<PlayerLoopSystem>());

                // Find the Update subsystem
                int updateIdx = -1;
                for (int i = 0; i < rootSubsystems.Count; i++)
                {
                    if (rootSubsystems[i].type == typeof(UnityEngine.PlayerLoop.Update))
                    {
                        updateIdx = i;
                        break;
                    }
                }

                if (updateIdx < 0)
                {
                    WriteDebug($"KeyInputSystem: WARNING - Update subsystem not found. Root subsystems: {rootSubsystems.Count}");
                    // Fallback: inject directly into root
                    var entry = new PlayerLoopSystem { type = typeof(DINOForgeKeyLoop), updateDelegate = PlayerLoopTick };
                    rootSubsystems.Add(entry);
                    loop.subSystemList = rootSubsystems.ToArray();
                    PlayerLoop.SetPlayerLoop(loop);
                    WriteDebug("KeyInputSystem: PlayerLoop injection into ROOT (fallback) successful");
                    return;
                }

                var updateSystem = rootSubsystems[updateIdx];
                var updateSubsystems = new System.Collections.Generic.List<PlayerLoopSystem>(updateSystem.subSystemList ?? System.Array.Empty<PlayerLoopSystem>());

                // Remove any existing DINOForgeKeyLoop entries (avoid duplicates on re-inject)
                updateSubsystems.RemoveAll(s => s.type == typeof(DINOForgeKeyLoop));

                // Append our entry
                updateSubsystems.Add(new PlayerLoopSystem { type = typeof(DINOForgeKeyLoop), updateDelegate = PlayerLoopTick });
                updateSystem.subSystemList = updateSubsystems.ToArray();
                rootSubsystems[updateIdx] = updateSystem;

                loop.subSystemList = rootSubsystems.ToArray();
                PlayerLoop.SetPlayerLoop(loop);

                WriteDebug($"KeyInputSystem: PlayerLoop injection successful (Update has {updateSubsystems.Count} subsystems)");
            }
            catch (Exception ex)
            {
                WriteDebug($"KeyInputSystem: PlayerLoop injection failed: {ex.GetType().Name}: {ex.Message}");
            }
            finally
            {
                _injectingNow = false;
            }
        }

        private static void PlayerLoopTick()
        {
            try
            {
                _playerLoopTickCount++;

                // Heartbeat: first tick + every 600 ticks (~10s at 60fps)
                if (_playerLoopTickCount == 1 || _playerLoopTickCount % 600 == 0)
                {
                    WriteDebug($"[KeyInputSystem] PlayerLoop tick #{_playerLoopTickCount}");
                }

                // Check F9
                if (Input.GetKeyDown(KeyCode.F9))
                {
                    WriteDebug("F9 pressed (from PlayerLoop)");
                    OnF9Pressed?.Invoke();
                }

                // Check F10
                if (Input.GetKeyDown(KeyCode.F10))
                {
                    WriteDebug("F10 pressed (from PlayerLoop)");
                    OnF10Pressed?.Invoke();
                }

                // Check resurrection
                if (Plugin.NeedsResurrection && Plugin.PersistentRoot == null)
                {
                    WriteDebug("Resurrection triggered from PlayerLoopTick");
                    Plugin.TryResurrect("(PlayerLoopTick)", "PlayerLoopTick");
                }
            }
            catch (Exception ex)
            {
                try
                {
                    WriteDebug($"PlayerLoopTick exception: {ex.Message}\n{ex.StackTrace}");
                }
                catch { }
            }
        }

        private struct DINOForgeKeyLoop { }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                System.IO.File.AppendAllText(debugLog, $"[{System.DateTime.Now}] [{nameof(KeyInputSystem)}] {msg}\n");
            }
            catch { }
        }
    }
}
