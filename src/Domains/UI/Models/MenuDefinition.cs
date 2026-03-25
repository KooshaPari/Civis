using System;
using System.Collections.Generic;

namespace DINOForge.Domains.UI.Models
{
    /// <summary>
    /// Defines a menu that appears in the UI system.
    /// Menus contain items that can navigate to other menus, toggle packs, reload content, or perform custom actions.
    /// </summary>
    public class MenuDefinition
    {
        /// <summary>
        /// Unique identifier for this menu (e.g. "main-menu", "mods-submenu", "settings").
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display title for this menu.
        /// </summary>
        public string Title { get; set; }

        /// <summary>
        /// Parent menu ID that this menu navigates back to.
        /// Empty or null indicates this is a root menu.
        /// </summary>
        public string ParentMenuId { get; set; } = string.Empty;

        /// <summary>
        /// List of menu items in this menu.
        /// </summary>
        public List<MenuItemDefinition> Items { get; set; } = new List<MenuItemDefinition>();

        /// <summary>
        /// Background color of this menu in hex format (e.g. "#1A1A1A").
        /// </summary>
        public string BackgroundColor { get; set; } = "#1A1A1A";

        /// <summary>
        /// Font size for menu items (in pixels or relative units).
        /// </summary>
        public int FontSize { get; set; } = 16;

        /// <summary>
        /// Optional description of what this menu represents.
        /// </summary>
        public string Description { get; set; } = string.Empty;

        /// <summary>
        /// Initializes a new menu definition with default values.
        /// </summary>
        public MenuDefinition()
        {
            Id = string.Empty;
            Title = string.Empty;
        }

        /// <summary>
        /// Initializes a new menu definition with core properties.
        /// </summary>
        public MenuDefinition(
            string id,
            string title,
            string parentMenuId = "")
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Title = title ?? throw new ArgumentNullException(nameof(title));
            ParentMenuId = parentMenuId ?? string.Empty;
        }
    }
}
