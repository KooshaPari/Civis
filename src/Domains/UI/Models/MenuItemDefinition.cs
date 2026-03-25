using System;

namespace DINOForge.Domains.UI.Models
{
    /// <summary>
    /// Defines a single menu item that appears in a menu.
    /// Menu items can navigate to other menus, toggle packs, reload content, or execute custom actions.
    /// </summary>
    public class MenuItemDefinition
    {
        /// <summary>
        /// Unique identifier for this menu item within its parent menu.
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display label shown to the user for this menu item.
        /// </summary>
        public string Label { get; set; }

        /// <summary>
        /// Action type this menu item performs.
        /// Possible values: navigate, toggle_pack, reload_packs, quit, custom.
        /// </summary>
        public string Action { get; set; }

        /// <summary>
        /// Target for the action (e.g. menu ID for navigate, pack ID for toggle_pack, custom action name).
        /// </summary>
        public string Target { get; set; } = string.Empty;

        /// <summary>
        /// Optional condition that determines if this menu item is enabled/clickable.
        /// Can be a simple predicate string or expression (e.g. "pack_loaded:warfare-modern").
        /// </summary>
        public string EnabledCondition { get; set; } = string.Empty;

        /// <summary>
        /// Optional description of what this menu item does.
        /// </summary>
        public string Description { get; set; } = string.Empty;

        /// <summary>
        /// Initializes a new menu item definition with default values.
        /// </summary>
        public MenuItemDefinition()
        {
            Id = string.Empty;
            Label = string.Empty;
            Action = "custom";
        }

        /// <summary>
        /// Initializes a new menu item definition with core properties.
        /// </summary>
        public MenuItemDefinition(
            string id,
            string label,
            string action,
            string target = "")
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Label = label ?? throw new ArgumentNullException(nameof(label));
            Action = action ?? throw new ArgumentNullException(nameof(action));
            Target = target ?? string.Empty;
        }
    }
}
