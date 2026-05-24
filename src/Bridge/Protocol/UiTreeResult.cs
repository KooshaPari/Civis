#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of a live Unity UI tree snapshot request.
    /// </summary>
    public sealed class UiTreeResult
    {
        /// <summary>Whether the snapshot was captured successfully.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>Human-readable status message for the snapshot request.</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";

        /// <summary>Optional selector string echoed from the request.</summary>
        [JsonProperty("selector")]
        public string? Selector { get; set; }

        /// <summary>UTC timestamp when the snapshot was generated.</summary>
        [JsonProperty("generatedAtUtc")]
        public string GeneratedAtUtc { get; set; } = "";

        /// <summary>Total number of nodes included in the snapshot tree.</summary>
        [JsonProperty("nodeCount")]
        public int NodeCount { get; set; }

        /// <summary>Synthetic root node for the complete UI hierarchy snapshot.</summary>
        [JsonProperty("root")]
        public UiNode Root { get; set; } = new UiNode();
    }

    /// <summary>
    /// A single node in the live Unity UI tree snapshot.
    /// </summary>
    public sealed class UiNode
    {
        /// <summary>Stable identifier for this node within the snapshot.</summary>
        [JsonProperty("id")]
        public string Id { get; set; } = "";

        /// <summary>Stable slash-delimited hierarchy path for this node.</summary>
        [JsonProperty("path")]
        public string Path { get; set; } = "";

        /// <summary>Unity object name for this node.</summary>
        [JsonProperty("name")]
        public string Name { get; set; } = "";

        /// <summary>Best-effort visible label text associated with this node.</summary>
        [JsonProperty("label")]
        public string Label { get; set; } = "";

        /// <summary>Semantic role inferred from Unity UI components.</summary>
        [JsonProperty("role")]
        public string Role { get; set; } = "";

        /// <summary>Primary Unity component type used to classify this node.</summary>
        [JsonProperty("componentType")]
        public string ComponentType { get; set; } = "";

        /// <summary>Whether the node's GameObject is active in hierarchy.</summary>
        [JsonProperty("active")]
        public bool Active { get; set; }

        /// <summary>Whether the node should be considered visible.</summary>
        [JsonProperty("visible")]
        public bool Visible { get; set; }

        /// <summary>Whether the node is currently interactable.</summary>
        [JsonProperty("interactable")]
        public bool Interactable { get; set; }

        /// <summary>Whether the node can receive UI raycasts.</summary>
        [JsonProperty("raycastTarget")]
        public bool RaycastTarget { get; set; }

        /// <summary>Best-effort screen-space bounds for this node.</summary>
        [JsonProperty("bounds")]
        public UiBounds? Bounds { get; set; }

        /// <summary>
        /// Best-effort UGUI button/text styling captured for bridge assertions (SPEC-002 F-03).
        /// Populated for UGUI Button nodes when a live tree snapshot is taken.
        /// </summary>
        [JsonProperty("style", NullValueHandling = NullValueHandling.Ignore)]
        public UiStyleSnapshot? Style { get; set; }

        /// <summary>Child nodes in hierarchy order.</summary>
        [JsonProperty("children")]
        public List<UiNode> Children { get; set; } = new List<UiNode>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    /// <summary>
    /// Subset of Unity UGUI button/text styling exposed for live UI tree comparisons.
    /// Mirrors fields copied by <c>NativeMenuInjector.SyncButtonVisualStyle</c>.
    /// </summary>
    public sealed class UiStyleSnapshot
    {
        /// <summary>UGUI Selectable transition mode name.</summary>
        [JsonProperty("transition")]
        public string Transition { get; set; } = "";

        /// <summary>Normal-state color from the button <c>ColorBlock</c> (#RRGGBBAA).</summary>
        [JsonProperty("normalColor")]
        public string NormalColor { get; set; } = "";

        /// <summary>Highlighted-state color from the button <c>ColorBlock</c> (#RRGGBBAA).</summary>
        [JsonProperty("highlightedColor")]
        public string HighlightedColor { get; set; } = "";

        /// <summary>Pressed-state color from the button <c>ColorBlock</c> (#RRGGBBAA).</summary>
        [JsonProperty("pressedColor")]
        public string PressedColor { get; set; } = "";

        /// <summary>Disabled-state color from the button <c>ColorBlock</c> (#RRGGBBAA).</summary>
        [JsonProperty("disabledColor")]
        public string DisabledColor { get; set; } = "";

        /// <summary>Label font size from legacy Text or TextMeshPro.</summary>
        [JsonProperty("fontSize", NullValueHandling = NullValueHandling.Ignore)]
        public int? FontSize { get; set; }

        /// <summary>Label text color (#RRGGBBAA).</summary>
        [JsonProperty("textColor", NullValueHandling = NullValueHandling.Ignore)]
        public string? TextColor { get; set; }

        /// <summary>Target graphic tint when the button uses a UGUI Image.</summary>
        [JsonProperty("imageColor", NullValueHandling = NullValueHandling.Ignore)]
        public string? ImageColor { get; set; }
    }

    /// <summary>
    /// Screen-space bounds for a UI node.
    /// </summary>
    public sealed class UiBounds
    {
        /// <summary>Left coordinate in screen space.</summary>
        [JsonProperty("x")]
        public float X { get; set; }

        /// <summary>Top coordinate in screen space.</summary>
        [JsonProperty("y")]
        public float Y { get; set; }

        /// <summary>Width in screen space.</summary>
        [JsonProperty("width")]
        public float Width { get; set; }

        /// <summary>Height in screen space.</summary>
        [JsonProperty("height")]
        public float Height { get; set; }
    }
}
