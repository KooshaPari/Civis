using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.UI.Models;

namespace DINOForge.Domains.UI.Registries
{
    /// <summary>
    /// Registry of menu definitions. Manages all menus registered across all packs,
    /// supporting navigation, hierarchies, and menu composition.
    /// </summary>
    public class MenuRegistry
    {
        private readonly Dictionary<string, MenuDefinition> _menus =
            new Dictionary<string, MenuDefinition>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered menus.
        /// </summary>
        public IReadOnlyList<MenuDefinition> All => _menus.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered menus.
        /// </summary>
        public int Count => _menus.Count;

        /// <summary>
        /// Retrieve a menu definition by its identifier.
        /// </summary>
        /// <param name="id">Menu identifier (e.g. "main-menu", "mods-submenu").</param>
        /// <returns>The matching menu definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no menu with the given id exists.</exception>
        public MenuDefinition GetMenu(string id)
        {
            if (_menus.TryGetValue(id, out MenuDefinition? menu))
                return menu;

            throw new KeyNotFoundException($"No menu registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a menu definition by its identifier.
        /// </summary>
        /// <param name="id">Menu identifier.</param>
        /// <param name="menu">The matching menu definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetMenu(string id, out MenuDefinition? menu)
        {
            return _menus.TryGetValue(id, out menu);
        }

        /// <summary>
        /// Check if a menu with the given identifier is registered.
        /// </summary>
        /// <param name="id">Menu identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _menus.ContainsKey(id);
        }

        /// <summary>
        /// Register a menu definition.
        /// </summary>
        /// <param name="menu">The menu definition to register.</param>
        public void Register(MenuDefinition menu)
        {
            if (menu == null) throw new ArgumentNullException(nameof(menu));
            if (string.IsNullOrWhiteSpace(menu.Id)) throw new ArgumentException("Menu ID cannot be empty.", nameof(menu));
            _menus[menu.Id] = menu;
        }

        /// <summary>
        /// Unregister a menu by identifier.
        /// </summary>
        /// <param name="id">Menu identifier.</param>
        /// <returns>True if a menu was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _menus.Remove(id);
        }

        /// <summary>
        /// Get all root menus (those with no parent or empty parent ID).
        /// </summary>
        /// <returns>A list of root menu definitions.</returns>
        public IReadOnlyList<MenuDefinition> GetRootMenus()
        {
            return _menus.Values
                .Where(m => string.IsNullOrEmpty(m.ParentMenuId))
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Get all child menus for a given parent menu.
        /// </summary>
        /// <param name="parentMenuId">The parent menu identifier.</param>
        /// <returns>A list of menu definitions that have this parent.</returns>
        public IReadOnlyList<MenuDefinition> GetChildMenus(string parentMenuId)
        {
            if (string.IsNullOrWhiteSpace(parentMenuId))
                return new List<MenuDefinition>().AsReadOnly();

            return _menus.Values
                .Where(m => string.Equals(m.ParentMenuId, parentMenuId, StringComparison.OrdinalIgnoreCase))
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Validate menu hierarchy for cycles and broken references.
        /// </summary>
        /// <returns>A list of validation errors (empty if valid).</returns>
        public IReadOnlyList<string> ValidateHierarchy()
        {
            List<string> errors = new List<string>();

            // Check for broken parent references
            foreach (MenuDefinition menu in _menus.Values)
            {
                if (!string.IsNullOrEmpty(menu.ParentMenuId) && !_menus.ContainsKey(menu.ParentMenuId))
                {
                    errors.Add($"Menu '{menu.Id}' references non-existent parent menu '{menu.ParentMenuId}'.");
                }
            }

            // Check for cycles
            foreach (MenuDefinition menu in _menus.Values)
            {
                if (HasCycle(menu.Id))
                {
                    errors.Add($"Menu hierarchy contains a cycle starting from '{menu.Id}'.");
                }
            }

            return errors.AsReadOnly();
        }

        private bool HasCycle(string menuId, HashSet<string>? visited = null)
        {
            visited ??= new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            if (visited.Contains(menuId))
                return true;

            if (!_menus.TryGetValue(menuId, out MenuDefinition? menu) || string.IsNullOrEmpty(menu.ParentMenuId))
                return false;

            visited.Add(menuId);
            return HasCycle(menu.ParentMenuId, visited);
        }
    }
}
