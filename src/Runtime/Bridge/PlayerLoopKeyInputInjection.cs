#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.LowLevel;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// SPEC-004 PlayerLoop injection and re-injection after <see cref="PlayerLoop.SetPlayerLoop"/>.
    /// Extracted as a testable seam (KIS-IT2, KIS-IT4); mirrors <c>Plugin_complete.cs</c> / SPEC-004.
    /// </summary>
    internal static class PlayerLoopKeyInputInjection
    {
        /// <summary>Marker type for Plugin.InjectPlayerLoopUpdate (SPEC-004 Path 2).</summary>
        internal struct DINOForgeUpdateMarker { }

        /// <summary>Marker type for KeyInputSystem.InjectIntoPlayerLoop (SPEC-004 Path 3).</summary>
        internal struct DINOForgeKeyLoopMarker { }

        private static bool _reinjecting;

        /// <summary>True while <see cref="OnAfterSetPlayerLoop"/> is running (re-entrancy guard).</summary>
        internal static bool IsReinjecting => _reinjecting;

        /// <summary>
        /// Harmony postfix on <see cref="PlayerLoop.SetPlayerLoop"/>: re-append injected entries after DINO rebuilds the loop.
        /// </summary>
        internal static void OnAfterSetPlayerLoop(Func<bool> inject)
        {
            if (_reinjecting)
            {
                return;
            }

            _reinjecting = true;
            try
            {
                inject();
            }
            finally
            {
                _reinjecting = false;
            }
        }

        /// <summary>
        /// Returns whether <paramref name="markerType"/> appears under Unity's Update subsystem.
        /// </summary>
        internal static bool ContainsMarkerInUpdate(PlayerLoopSystem loop, Type markerType)
        {
            if (loop.subSystemList == null)
            {
                return false;
            }

            for (int i = 0; i < loop.subSystemList.Length; i++)
            {
                PlayerLoopSystem subsystem = loop.subSystemList[i];
                if (subsystem.type != typeof(UnityEngine.PlayerLoop.Update))
                {
                    continue;
                }

                if (subsystem.subSystemList == null)
                {
                    return false;
                }

                for (int j = 0; j < subsystem.subSystemList.Length; j++)
                {
                    if (subsystem.subSystemList[j].type == markerType)
                    {
                        return true;
                    }
                }

                return false;
            }

            return false;
        }

        /// <summary>
        /// Appends <paramref name="markerType"/> to the Update subsystem without calling <see cref="PlayerLoop.SetPlayerLoop"/>.
        /// </summary>
        internal static bool TryBuildInjectedLoop(
            PlayerLoopSystem loop,
            Type markerType,
            PlayerLoopSystem.UpdateFunction? updateDelegate,
            out PlayerLoopSystem injectedLoop)
        {
            injectedLoop = loop;
            if (loop.subSystemList == null || loop.subSystemList.Length == 0)
            {
                return false;
            }

            var newSubsystems = new List<PlayerLoopSystem>(loop.subSystemList);
            int updateSystemIndex = -1;
            for (int i = 0; i < newSubsystems.Count; i++)
            {
                if (newSubsystems[i].type == typeof(UnityEngine.PlayerLoop.Update))
                {
                    updateSystemIndex = i;
                    break;
                }
            }

            if (updateSystemIndex < 0)
            {
                return false;
            }

            PlayerLoopSystem updateSystem = newSubsystems[updateSystemIndex];
            var updateSubsystems = new List<PlayerLoopSystem>(
                updateSystem.subSystemList ?? Array.Empty<PlayerLoopSystem>());

            for (int j = 0; j < updateSubsystems.Count; j++)
            {
                if (updateSubsystems[j].type == markerType)
                {
                    injectedLoop = loop;
                    return true;
                }
            }

            updateSubsystems.Add(new PlayerLoopSystem
            {
                type = markerType,
                updateDelegate = updateDelegate,
            });

            updateSystem.subSystemList = updateSubsystems.ToArray();
            newSubsystems[updateSystemIndex] = updateSystem;
            injectedLoop = loop;
            injectedLoop.subSystemList = newSubsystems.ToArray();
            return true;
        }

        /// <summary>
        /// Simulates DINO evicting injected entries by stripping marker types from the Update subsystem.
        /// </summary>
        internal static PlayerLoopSystem EvictMarkersFromUpdate(PlayerLoopSystem loop, params Type[] markerTypes)
        {
            if (loop.subSystemList == null || markerTypes.Length == 0)
            {
                return loop;
            }

            var root = new List<PlayerLoopSystem>(loop.subSystemList);
            for (int i = 0; i < root.Count; i++)
            {
                if (root[i].type != typeof(UnityEngine.PlayerLoop.Update))
                {
                    continue;
                }

                PlayerLoopSystem updateSystem = root[i];
                if (updateSystem.subSystemList == null)
                {
                    break;
                }

                var kept = new List<PlayerLoopSystem>();
                for (int j = 0; j < updateSystem.subSystemList.Length; j++)
                {
                    PlayerLoopSystem entry = updateSystem.subSystemList[j];
                    bool isMarker = false;
                    for (int m = 0; m < markerTypes.Length; m++)
                    {
                        if (entry.type == markerTypes[m])
                        {
                            isMarker = true;
                            break;
                        }
                    }

                    if (!isMarker)
                    {
                        kept.Add(entry);
                    }
                }

                updateSystem.subSystemList = kept.ToArray();
                root[i] = updateSystem;
                break;
            }

            loop.subSystemList = root.ToArray();
            return loop;
        }

        /// <summary>
        /// Injects <paramref name="markerType"/> into the current player loop and applies it via <see cref="PlayerLoop.SetPlayerLoop"/>.
        /// </summary>
        internal static bool InjectIntoCurrentPlayerLoop(
            Type markerType,
            PlayerLoopSystem.UpdateFunction? updateDelegate)
        {
            PlayerLoopSystem current = PlayerLoop.GetCurrentPlayerLoop();
            if (!TryBuildInjectedLoop(current, markerType, updateDelegate, out PlayerLoopSystem injected))
            {
                return false;
            }

            PlayerLoop.SetPlayerLoop(injected);
            return ContainsMarkerInUpdate(PlayerLoop.GetCurrentPlayerLoop(), markerType);
        }
    }
}
