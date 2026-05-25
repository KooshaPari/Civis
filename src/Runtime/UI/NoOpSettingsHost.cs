#nullable enable

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// No-op settings host used for UGUI-only flows where we intentionally avoid
    /// creating the IMGUI BepInEx settings panel.
    /// </summary>
    internal sealed class NoOpSettingsHost : IModSettingsHost
    {
        public bool IsVisible => false;

        public void Show()
        {
        }

        public void Hide()
        {
        }

        public void Toggle()
        {
        }
    }
}
