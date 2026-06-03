#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Bridge;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.UI;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;
using UnityEngine.EventSystems;

namespace DINOForge.Runtime.Capture
{
    /// <summary>
    /// Scripts the multi-step native UI sequence that takes DINO from the main menu into an
    /// active gameplay/skirmish state, driving each transition through the in-process
    /// <see cref="EventSystemDriver"/> (issue #972) and capturing a verification frame at every
    /// step with <see cref="FrameCapture"/> (issue #980).
    ///
    /// <para>
    /// WHY THIS EXISTS: the bridge could already (a) fire a real EventSystem pointer click on a
    /// single <c>MainMenuButton:Selectable</c> and (b) reliably capture a PNG in any state — but
    /// nothing scripted the FULL sequence (PLAY → map/options → START → gameplay camera). RPCs
    /// that create the world-loading singleton load a world without firing the menu→level UI
    /// transition, so the gameplay camera was never reached and #980's capture had nothing
    /// in-game to photograph. This composes the existing primitives into one robust, parameterized
    /// routine the verify-agents can call once.
    /// </para>
    ///
    /// <para>
    /// ROBUSTNESS: every step resolves its target from an ORDERED list of candidate selectors and
    /// takes the first that resolves + is actionable (DINO's button labels vary by build / locale,
    /// so we never hard-bind to one). Between steps we WAIT FOR A CONDITION (next canvas appears,
    /// or a live ECS world with units becomes ready) rather than sleeping a fixed time. The
    /// map-select sub-flow is handled by an optional intermediate step whose absence is tolerated
    /// (some builds go straight from PLAY to START).
    /// </para>
    ///
    /// THREADING: main-thread only. Callers route via
    /// <see cref="MainThreadDispatcher.RunOnMainThread{T}"/>. Coroutine-based capture inside
    /// <see cref="FrameCapture"/> is itself main-thread safe.
    /// </summary>
    internal static class NavigationScripter
    {
        /// <summary>Entity count above which the active world is considered "in gameplay".</summary>
        private const int GameplayEntityThreshold = 5000;

        /// <summary>A single scripted step.</summary>
        internal sealed class Step
        {
            /// <summary>Human label, surfaced in the trace (e.g. "click PLAY").</summary>
            public string Name { get; set; } = "";

            /// <summary>
            /// Ordered candidate selectors. The first that resolves to an actionable node is
            /// clicked. Empty = no click (a pure wait/observe step).
            /// </summary>
            public IReadOnlyList<string> Candidates { get; set; } = Array.Empty<string>();

            /// <summary>Pointer event to fire on the resolved candidate ("press" = full lifecycle).</summary>
            public string PointerEvent { get; set; } = "press";

            /// <summary>
            /// After the click, wait until ANY of these selectors is visible (next-screen signal).
            /// Empty = no selector wait.
            /// </summary>
            public IReadOnlyList<string> WaitForVisible { get; set; } = Array.Empty<string>();

            /// <summary>After the click, wait for a live ECS world with units (gameplay reached).</summary>
            public bool WaitForWorld { get; set; }

            /// <summary>Max time to wait for this step's condition.</summary>
            public int WaitTimeoutMs { get; set; } = 8000;

            /// <summary>
            /// If true, the step is optional: a failure to resolve/click is logged and skipped,
            /// not treated as a blocking failure. Used for the map-select sub-flow which some
            /// builds omit.
            /// </summary>
            public bool Optional { get; set; }

            /// <summary>Settle delay (ms) AFTER the wait condition before the screenshot.</summary>
            public int SettleMs { get; set; } = 250;
        }

        /// <summary>A named, ordered collection of steps.</summary>
        internal sealed class Plan
        {
            public string Name { get; set; } = "";
            public IReadOnlyList<Step> Steps { get; set; } = Array.Empty<Step>();
        }

        /// <summary>
        /// Default skirmish/sandbox plan. Each step lists several candidate labels because DINO's
        /// native main-menu button labels differ across builds (PLAY / NEW GAME / FREE PLAY /
        /// SANDBOX / SKIRMISH all observed). Resolution stops at the first actionable match.
        /// </summary>
        internal static Plan SkirmishPlan() => new Plan
        {
            Name = "skirmish",
            Steps = new List<Step>
            {
                // Step 1: top-level entry — open the play/new-game flow.
                new Step
                {
                    Name = "open play menu",
                    Candidates = new[]
                    {
                        "label=Skirmish", "label=Free Play", "label=Sandbox",
                        "label=New Game", "label=Play", "label=Single",
                        "name=PlayButton", "name=NewGameButton", "name=SkirmishButton",
                    },
                    PointerEvent = "press",
                    // Next screen: a map/scenario list or an options/start panel.
                    WaitForVisible = new[]
                    {
                        "label=Start", "label=Begin", "label=Play",
                        "label=Map", "label=Scenario", "name=MapSelect",
                        "name=StartButton", "role=button",
                    },
                    WaitTimeoutMs = 10000,
                },

                // Step 2 (optional): map / scenario select sub-flow. Tolerated if absent.
                new Step
                {
                    Name = "select map/scenario",
                    Candidates = new[]
                    {
                        "name=MapSelect", "label=Map", "label=Scenario",
                        "label=Random", "label=Continent", "role=toggle",
                    },
                    PointerEvent = "press",
                    Optional = true,
                    WaitForVisible = new[] { "label=Start", "label=Begin", "name=StartButton" },
                    WaitTimeoutMs = 6000,
                },

                // Step 3: commit — start/begin the match.
                new Step
                {
                    Name = "start match",
                    Candidates = new[]
                    {
                        "label=Start", "label=Begin", "label=Confirm",
                        "label=Play", "name=StartButton", "name=BeginButton",
                    },
                    PointerEvent = "press",
                    // Gameplay reached: live world spins up with units.
                    WaitForWorld = true,
                    WaitTimeoutMs = 60000,
                    SettleMs = 1500,
                },
            },
        };

        /// <summary>
        /// Execute a navigation plan. MAIN THREAD ONLY. Captures a screenshot into
        /// <paramref name="screenshotDir"/> after every step (named
        /// <c>nav_&lt;plan&gt;_&lt;index&gt;_&lt;step&gt;.png</c>) and a final
        /// <c>gameplay-camera-reached.png</c> when a world is reached.
        /// </summary>
        public static NavigationResult Run(Plan plan, string screenshotDir, string? finalShotPath)
        {
            var steps = new List<NavigationStepResult>();
            var result = new NavigationResult { Plan = plan.Name, Steps = steps };

            if (EventSystem.current == null)
            {
                result.Success = false;
                result.Message = "EventSystem.current is null — no active UI to drive (is the game at a menu?).";
                result.FinalState = "no-eventsystem";
                return result;
            }

            try
            {
                if (!string.IsNullOrEmpty(screenshotDir))
                    Directory.CreateDirectory(screenshotDir);
            }
            catch (Exception ex)
            {
                DebugLog.Write("NavigationScripter", $"[NavigationScripter] mkdir '{screenshotDir}' failed: {ex.Message}");
            }

            for (int i = 0; i < plan.Steps.Count; i++)
            {
                Step step = plan.Steps[i];
                var sr = new NavigationStepResult { Name = step.Name };
                steps.Add(sr);
                DebugLog.Write("NavigationScripter", $"[NavigationScripter] step {i} '{step.Name}' (event={step.PointerEvent}, optional={step.Optional})");

                // ── Resolve + click the first actionable candidate ──────────────
                bool clicked = step.Candidates.Count == 0; // no-candidate step = pure wait
                foreach (string candidate in step.Candidates)
                {
                    Transform? target = UiSelectorEngine.ResolveTarget(candidate, out int matchCount);
                    if (target == null) continue;
                    if (!UiSelectorEngine.IsTargetActionable(target, out string reason))
                    {
                        DebugLog.Write("NavigationScripter", $"[NavigationScripter]   candidate '{candidate}' matched({matchCount}) but not actionable: {reason}");
                        continue;
                    }

                    UiActionResult click = EventSystemDriver.Drive(candidate, step.PointerEvent);
                    if (click.Success)
                    {
                        clicked = true;
                        sr.ResolvedSelector = candidate;
                        sr.Detail = click.Message;
                        DebugLog.Write("NavigationScripter", $"[NavigationScripter]   clicked via '{candidate}': {click.Message}");
                        break;
                    }
                    DebugLog.Write("NavigationScripter", $"[NavigationScripter]   candidate '{candidate}' click had no effect: {click.Message}");
                }

                if (!clicked)
                {
                    if (step.Optional)
                    {
                        sr.Success = true; // optional + unresolved = skipped, not a failure
                        sr.Detail = "optional step skipped (no candidate resolved)";
                        sr.WaitCondition = "skipped";
                        DebugLog.Write("NavigationScripter", $"[NavigationScripter]   optional step skipped");
                        sr.Screenshot = CaptureStep(screenshotDir, plan.Name, i, step.Name);
                        continue;
                    }

                    sr.Success = false;
                    sr.Detail = "no candidate selector resolved to an actionable node";
                    sr.Screenshot = CaptureStep(screenshotDir, plan.Name, i, step.Name);
                    result.BlockedAtStep = i;
                    result.Success = false;
                    result.FinalState = step.Name;
                    result.Message = $"Blocked at step {i} ('{step.Name}'): could not resolve any of [{string.Join(", ", step.Candidates)}].";
                    Finalize(result);
                    return result;
                }

                // ── Wait for the next-screen / world-ready condition ────────────
                bool waitOk = true;
                if (step.WaitForWorld)
                {
                    sr.WaitCondition = $"world-ready (>{GameplayEntityThreshold} entities)";
                    waitOk = WaitForGameplayWorld(step.WaitTimeoutMs);
                }
                else if (step.WaitForVisible.Count > 0)
                {
                    sr.WaitCondition = $"any-visible [{string.Join(", ", step.WaitForVisible)}]";
                    waitOk = WaitForAnyVisible(step.WaitForVisible, step.WaitTimeoutMs);
                }
                else
                {
                    sr.WaitCondition = "settle-only";
                }
                sr.WaitSatisfied = waitOk;

                // Settle so the freshly-shown screen finishes its open animation before capture.
                if (step.SettleMs > 0)
                    System.Threading.Thread.Sleep(step.SettleMs);

                sr.Screenshot = CaptureStep(screenshotDir, plan.Name, i, step.Name);

                if (!waitOk && !step.Optional)
                {
                    sr.Success = false;
                    sr.Detail = (sr.Detail.Length > 0 ? sr.Detail + "; " : "") + "wait condition not met before timeout";
                    result.BlockedAtStep = i;
                    result.Success = false;
                    result.FinalState = step.Name;
                    result.Message = $"Blocked at step {i} ('{step.Name}'): clicked '{sr.ResolvedSelector}' but wait condition '{sr.WaitCondition}' was not satisfied.";
                    Finalize(result);
                    return result;
                }

                sr.Success = true;
            }

            // ── Whole plan executed ─────────────────────────────────────────────
            result.Success = true;
            result.BlockedAtStep = -1;
            Finalize(result);
            result.FinalState = result.EntityCount > GameplayEntityThreshold ? "gameplay" : "menu";
            result.Message = result.EntityCount > GameplayEntityThreshold
                ? $"Reached gameplay: world '{result.WorldName}' with {result.EntityCount} entities."
                : $"Plan completed but no gameplay world detected ({result.EntityCount} entities).";

            // Final, named gameplay-camera capture.
            if (!string.IsNullOrEmpty(finalShotPath))
            {
                FrameCapture.Result fc = FrameCapture.Capture(finalShotPath!);
                DebugLog.Write("NavigationScripter", $"[NavigationScripter] final capture -> {finalShotPath} success={fc.Success} {fc.Width}x{fc.Height}");
            }

            return result;
        }

        // ── Wait helpers (condition-based, not fixed sleeps) ────────────────────

        private static bool WaitForAnyVisible(IReadOnlyList<string> selectors, int timeoutMs)
        {
            int elapsed = 0;
            const int poll = 150;
            while (elapsed <= timeoutMs)
            {
                foreach (string sel in selectors)
                {
                    UiWaitResult w = UiSelectorEngine.EvaluateState(sel, "visible");
                    if (w.Ready) return true;
                }
                System.Threading.Thread.Sleep(poll);
                elapsed += poll;
            }
            return false;
        }

        private static bool WaitForGameplayWorld(int timeoutMs)
        {
            int elapsed = 0;
            const int poll = 250;
            while (elapsed <= timeoutMs)
            {
                if (ActiveEntityCount() > GameplayEntityThreshold)
                    return true;
                System.Threading.Thread.Sleep(poll);
                elapsed += poll;
            }
            return false;
        }

        /// <summary>Entity count of the default ECS world (0 when no live world). Main thread only.</summary>
        private static int ActiveEntityCount()
        {
            try
            {
                World? world = World.DefaultGameObjectInjectionWorld;
                if (world == null || !world.IsCreated) return 0;
                EntityQuery q = world.EntityManager.CreateEntityQuery(
                    new EntityQueryDesc { Options = EntityQueryOptions.IncludePrefab });
                return q.CalculateEntityCount();
            }
            catch (Exception ex)
            {
                DebugLog.Write("NavigationScripter", $"[NavigationScripter] entity-count read failed: {ex.Message}");
                return 0;
            }
        }

        private static void Finalize(NavigationResult result)
        {
            result.EntityCount = ActiveEntityCount();
            try
            {
                World? world = World.DefaultGameObjectInjectionWorld;
                result.WorldName = (world != null && world.IsCreated) ? world.Name : "";
            }
            catch { result.WorldName = ""; /* safe-swallow: name read is best-effort diagnostic */ }
        }

        private static string CaptureStep(string dir, string plan, int index, string stepName)
        {
            if (string.IsNullOrEmpty(dir)) return "";
            string safe = new string(stepName.Select(c => char.IsLetterOrDigit(c) ? c : '_').ToArray());
            string path = Path.Combine(dir, $"nav_{plan}_{index:D2}_{safe}.png");
            try
            {
                FrameCapture.Result fc = FrameCapture.Capture(path);
                return fc.Success ? path : "";
            }
            catch (Exception ex)
            {
                DebugLog.Write("NavigationScripter", $"[NavigationScripter] step capture failed: {ex.Message}");
                return "";
            }
        }
    }
}