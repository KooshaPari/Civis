using System;
using System.Collections.Generic;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Skeleton system for injecting HUD elements into the game.
    /// This is a placeholder for future in-game HUD mods. In the full
    /// implementation, this will be an ECS system that runs in the
    /// simulation group and manages HUD entity lifecycle.
    /// </summary>
    /// <remarks>
    /// This does not inherit from Unity.Entities.SystemBase because the UI domain
    /// plugin targets netstandard2.0 without Unity ECS references.
    /// The Runtime layer will create the actual ECS system that delegates to this class.
    /// </remarks>
    public sealed class HUDInjectionSystem
    {
        private readonly List<HUDElementDefinition> _registeredElements = new List<HUDElementDefinition>();
        private bool _initialized;

        /// <summary>Whether the system has been initialized.</summary>
        public bool IsInitialized => _initialized;

        /// <summary>Number of registered HUD elements.</summary>
        public int ElementCount => _registeredElements.Count;

        /// <summary>
        /// Initializes the HUD injection system. Call once during startup.
        /// </summary>
        public void Initialize()
        {
            _registeredElements.Clear();
            _initialized = true;
        }

        /// <summary>
        /// Registers a HUD element definition for injection.
        /// </summary>
        /// <param name="element">The HUD element to register.</param>
        public void RegisterElement(HUDElementDefinition element)
        {
            if (!_initialized)
                throw new InvalidOperationException("HUDInjectionSystem has not been initialized. Call Initialize() first.");

            if (element == null) throw new ArgumentNullException(nameof(element));

            _registeredElements.Add(element);
        }

        /// <summary>
        /// Removes a HUD element by ID.
        /// </summary>
        /// <param name="elementId">The ID of the element to remove.</param>
        /// <returns>True if the element was found and removed.</returns>
        public bool UnregisterElement(string elementId)
        {
            return _registeredElements.RemoveAll(e =>
                string.Equals(e.Id, elementId, StringComparison.OrdinalIgnoreCase)) > 0;
        }

        /// <summary>
        /// Gets all registered HUD element definitions.
        /// </summary>
        public IReadOnlyList<HUDElementDefinition> GetElements()
        {
            return _registeredElements.AsReadOnly();
        }

        /// <summary>
        /// Placeholder update method. In the full implementation, this will be called
        /// each frame by the ECS system to update HUD element positions and visibility.
        /// </summary>
        public void Update()
        {
            if (!_initialized) return;

            // Placeholder: future implementation will iterate registered elements
            // and synchronize their state with the ECS world.
        }

        /// <summary>
        /// Shuts down the HUD injection system and clears all elements.
        /// </summary>
        public void Shutdown()
        {
            _registeredElements.Clear();
            _initialized = false;
        }
    }

    /// <summary>
    /// Defines a HUD element that can be injected into the game UI.
    /// </summary>
    public sealed class HUDElementDefinition
    {
        /// <summary>Unique identifier for this HUD element.</summary>
        public string Id { get; }

        /// <summary>Display name.</summary>
        public string Name { get; }

        /// <summary>The pack that registered this element.</summary>
        public string SourcePackId { get; }

        /// <summary>
        /// Anchor position on screen. One of: top-left, top-center, top-right,
        /// middle-left, middle-center, middle-right, bottom-left, bottom-center, bottom-right.
        /// </summary>
        public string Anchor { get; }

        /// <summary>Z-order for layering (higher = on top).</summary>
        public int ZOrder { get; }

        /// <summary>Whether this element is currently visible.</summary>
        public bool IsVisible { get; set; }

        /// <summary>
        /// Initializes a new instance of the <see cref="HUDElementDefinition"/> class.
        /// </summary>
        /// <param name="id">Unique identifier for this HUD element.</param>
        /// <param name="name">Display name.</param>
        /// <param name="sourcePackId">The pack that registered this element.</param>
        /// <param name="anchor">Anchor position on screen.</param>
        /// <param name="zOrder">Z-order for layering.</param>
        /// <param name="isVisible">Whether this element is currently visible.</param>
        public HUDElementDefinition(
            string id,
            string name,
            string sourcePackId,
            string anchor = "top-left",
            int zOrder = 0,
            bool isVisible = true)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Name = name ?? throw new ArgumentNullException(nameof(name));
            SourcePackId = sourcePackId ?? throw new ArgumentNullException(nameof(sourcePackId));
            Anchor = anchor;
            ZOrder = zOrder;
            IsVisible = isVisible;
        }
    }
}
