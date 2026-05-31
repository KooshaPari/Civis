#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of a scripted menu→gameplay navigation run driven by the in-process
    /// NavigationScripter. Captures whether the full sequence reached the target state
    /// plus a per-step trace (which selector resolved, whether the wait condition was
    /// satisfied, and the screenshot captured at that step) so verify-agents can see
    /// exactly where a flow stalled.
    /// </summary>
    public sealed class NavigationResult
    {
        /// <summary>Whether the full sequence reached the requested target state.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>Human-readable summary (or the reason the flow stopped).</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";

        /// <summary>Plan name that was executed (e.g. "skirmish").</summary>
        [JsonProperty("plan")]
        public string Plan { get; set; } = "";

        /// <summary>Final reported state — "gameplay" when a live world was reached, else last step name.</summary>
        [JsonProperty("finalState")]
        public string FinalState { get; set; } = "";

        /// <summary>Entity count of the active world at completion (0 when no world).</summary>
        [JsonProperty("entityCount")]
        public int EntityCount { get; set; }

        /// <summary>Active world name at completion, when a world is ready.</summary>
        [JsonProperty("worldName")]
        public string WorldName { get; set; } = "";

        /// <summary>Index (0-based) of the step that blocked, or -1 when the whole plan succeeded.</summary>
        [JsonProperty("blockedAtStep")]
        public int BlockedAtStep { get; set; } = -1;

        /// <summary>Per-step trace, in execution order.</summary>
        [JsonProperty("steps")]
        public IReadOnlyList<NavigationStepResult> Steps { get; set; } = new List<NavigationStepResult>();
    }

    /// <summary>One executed navigation step's outcome.</summary>
    public sealed class NavigationStepResult
    {
        /// <summary>Human label for the step (e.g. "click PLAY").</summary>
        [JsonProperty("name")]
        public string Name { get; set; } = "";

        /// <summary>Whether the step completed (action fired AND wait condition satisfied).</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>The candidate selector that actually resolved + fired (empty if none).</summary>
        [JsonProperty("resolvedSelector")]
        public string ResolvedSelector { get; set; } = "";

        /// <summary>Whether the post-action wait condition was satisfied.</summary>
        [JsonProperty("waitSatisfied")]
        public bool WaitSatisfied { get; set; }

        /// <summary>Description of the wait condition that was applied.</summary>
        [JsonProperty("waitCondition")]
        public string WaitCondition { get; set; } = "";

        /// <summary>Screenshot path captured at the end of this step (empty if none / failed).</summary>
        [JsonProperty("screenshot")]
        public string Screenshot { get; set; } = "";

        /// <summary>Diagnostic detail for the step.</summary>
        [JsonProperty("detail")]
        public string Detail { get; set; } = "";
    }
}
