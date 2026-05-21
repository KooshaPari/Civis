using System;
using System.Collections.Generic;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Manages mod menu state (open/closed, active panel) and coordinates
    /// between the overlay and settings panel. This is a pure state manager
    /// with no Unity dependencies, so it can be tested in isolation.
    /// </summary>
    public sealed class MenuManager
    {
        private readonly Dictionary<string, bool> _panelStates =
            new Dictionary<string, bool>(StringComparer.OrdinalIgnoreCase);

        /// <summary>Well-known panel name for the main mod list.</summary>
        public const string PanelModList = "mod_list";

        /// <summary>Well-known panel name for the settings view.</summary>
        public const string PanelSettings = "settings";

        /// <summary>Well-known panel name for the pack details view.</summary>
        public const string PanelPackDetails = "pack_details";

        /// <summary>Whether the mod menu is currently open.</summary>
        public bool IsMenuOpen { get; private set; }

        /// <summary>The name of the currently active panel.</summary>
        public string ActivePanel { get; private set; } = PanelModList;

        /// <summary>
        /// Opens the mod menu. If already open, does nothing.
        /// </summary>
        public void Open()
        {
            IsMenuOpen = true;
        }

        /// <summary>
        /// Closes the mod menu. If already closed, does nothing.
        /// </summary>
        public void Close()
        {
            IsMenuOpen = false;
        }

        /// <summary>
        /// Toggles the mod menu open/closed state.
        /// </summary>
        public void Toggle()
        {
            IsMenuOpen = !IsMenuOpen;
        }

        /// <summary>
        /// Switches the active panel to the specified panel name.
        /// </summary>
        /// <param name="panelName">The name of the panel to activate.</param>
        /// <exception cref="ArgumentException">If panelName is null or empty.</exception>
        public void SetActivePanel(string panelName)
        {
            if (string.IsNullOrWhiteSpace(panelName))
                throw new ArgumentException("Panel name is required.", nameof(panelName));

            ActivePanel = panelName;
        }

        /// <summary>
        /// Registers a custom panel name and its initial visibility state.
        /// </summary>
        /// <param name="panelName">The panel name to register.</param>
        /// <param name="initiallyVisible">Whether the panel should start visible.</param>
        public void RegisterPanel(string panelName, bool initiallyVisible = false)
        {
            if (string.IsNullOrWhiteSpace(panelName))
                throw new ArgumentException("Panel name is required.", nameof(panelName));

            _panelStates[panelName] = initiallyVisible;
        }

        /// <summary>
        /// Gets whether a registered panel is visible.
        /// Returns false for unregistered panels.
        /// </summary>
        /// <param name="panelName">The panel name to check.</param>
        /// <returns>True if the panel is registered and visible.</returns>
        public bool IsPanelVisible(string panelName)
        {
            return _panelStates.TryGetValue(panelName, out bool visible) && visible;
        }

        /// <summary>
        /// Sets the visibility of a registered panel.
        /// </summary>
        /// <param name="panelName">The panel name to update.</param>
        /// <param name="visible">Whether the panel should be visible.</param>
        public void SetPanelVisible(string panelName, bool visible)
        {
            if (string.IsNullOrWhiteSpace(panelName))
                throw new ArgumentException("Panel name is required.", nameof(panelName));

            _panelStates[panelName] = visible;
        }

        /// <summary>
        /// Gets all registered panel names and their visibility states.
        /// </summary>
        public IReadOnlyDictionary<string, bool> GetPanelStates()
        {
            var snapshot = new Dictionary<string, bool>(StringComparer.OrdinalIgnoreCase);
            foreach (var kvp in _panelStates) snapshot[kvp.Key] = kvp.Value;
            return snapshot;
        }
    }
}
