#nullable enable

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Abstracts the runtime settings host so settings rendering can move between
    /// IMGUI, custom UGUI, and native-hosted implementations without changing callers.
    /// </summary>
    public interface IModSettingsHost
    {
        /// <summary>Whether the settings host is currently visible.</summary>
        bool IsVisible { get; }

        /// <summary>Shows the settings host.</summary>
        void Show();

        /// <summary>Hides the settings host.</summary>
        void Hide();

        /// <summary>Toggles the settings host visibility.</summary>
        void Toggle();
    }
}
