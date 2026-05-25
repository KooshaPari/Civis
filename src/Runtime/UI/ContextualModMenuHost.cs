#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Routes mod-menu interactions to a true main-menu-native screen when the
    /// game's main menu is active, and falls back to the DFCanvas overlay elsewhere.
    /// </summary>
    internal sealed class ContextualModMenuHost : IModMenuHost
    {
        private readonly IModMenuHost _overlayHost;
        private readonly NativeMainMenuModMenu _nativeHost;
        private Action? _onReloadRequested;
        private Action<string, bool>? _onPackToggled;

        public ContextualModMenuHost(IModMenuHost overlayHost, NativeMainMenuModMenu nativeHost)
        {
            _overlayHost = overlayHost;
            _nativeHost = nativeHost;
        }

        public Action? OnReloadRequested
        {
            get => _onReloadRequested;
            set
            {
                _onReloadRequested = value;
                _overlayHost.OnReloadRequested = value;
                _nativeHost.OnReloadRequested = value;
            }
        }

        public Action<string, bool>? OnPackToggled
        {
            get => _onPackToggled;
            set
            {
                _onPackToggled = value;
                _overlayHost.OnPackToggled = value;
                _nativeHost.OnPackToggled = value;
            }
        }

        public bool IsVisible => _nativeHost.IsVisible || _overlayHost.IsVisible;

        public void Show()
        {
            if (_nativeHost.CanUseNativeScreen)
            {
                _overlayHost.Hide();
                _nativeHost.Show();
                return;
            }

            _overlayHost.Show();
        }

        public void Hide()
        {
            _nativeHost.Hide();
            _overlayHost.Hide();
        }

        public void Toggle()
        {
            if (_nativeHost.CanUseNativeScreen)
            {
                _overlayHost.Hide();
                _nativeHost.Toggle();
                return;
            }

            _overlayHost.Toggle();
        }

        public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            _overlayHost.SetPacks(packs);
            _nativeHost.SetPacks(packs);
        }

        public void SetStatus(string message, int errorCount = 0)
        {
            _overlayHost.SetStatus(message, errorCount);
            _nativeHost.SetStatus(message, errorCount);
        }
    }
}
