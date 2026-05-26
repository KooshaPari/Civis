using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.UI.Models
{
    /// <summary>
    /// Defines a HUD (Heads-Up Display) element that appears in-game overlay.
    /// HUD elements can display health bars, resource counters, minimaps, unit portraits, alert banners, etc.
    /// </summary>
    public class HudElementDefinition : IValidatable
    {
        /// <summary>
        /// Unique identifier for this HUD element (e.g. "player-health-bar", "resource-counter").
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display type of this HUD element.
        /// Possible values: health_bar, resource_counter, minimap, unit_portrait, alert_banner, custom.
        /// </summary>
        public string Type { get; set; }

        /// <summary>
        /// Screen position anchor for this HUD element.
        /// Possible values: top_left, top_right, bottom_left, bottom_right, center.
        /// </summary>
        public string Position { get; set; }

        /// <summary>
        /// Width of the HUD element in pixels.
        /// </summary>
        public int Width { get; set; }

        /// <summary>
        /// Height of the HUD element in pixels.
        /// </summary>
        public int Height { get; set; }

        /// <summary>
        /// List of game states where this HUD element is visible.
        /// Possible values: gameplay, pause, main_menu.
        /// </summary>
        public List<string> VisibleIn { get; set; } = new List<string>(); // public-mutable-ok: UI content loader deserializes visibility states into a mutable list

        /// <summary>
        /// Dictionary of color overrides for this HUD element (property name to hex color code, e.g. "background" -> "#FF0000").
        /// </summary>
        public Dictionary<string, string> ColorOverrides { get; set; } = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase); // public-mutable-ok: UI content loader deserializes overrides into a mutable dictionary

        /// <summary>
        /// Optional opacity value (0.0 to 1.0) for the HUD element.
        /// </summary>
        public float Opacity { get; set; } = 1.0f;

        /// <summary>
        /// Optional description of what this HUD element displays.
        /// </summary>
        public string Description { get; set; } = string.Empty;

        /// <summary>
        /// Initializes a new HUD element definition with default values.
        /// </summary>
        public HudElementDefinition()
        {
            Id = string.Empty;
            Type = "custom";
            Position = "top_left";
            Width = 200;
            Height = 50;
        }

        /// <summary>
        /// Initializes a new HUD element definition with core properties.
        /// </summary>
        public HudElementDefinition(
            string id,
            string type,
            string position,
            int width,
            int height)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Type = type ?? throw new ArgumentNullException(nameof(type));
            Position = position ?? throw new ArgumentNullException(nameof(position));
            Width = width;
            Height = height;
        }

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the UIContentLoader deserialize site.
        /// Rejects blank id and out-of-range opacity.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "HudElementDefinition 'id' is required.", "non_empty"));

            if (Opacity < 0f || Opacity > 1f)
                errors.Add(new ValidationError("opacity", "HudElementDefinition 'opacity' must be in [0,1].", "range"));

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
