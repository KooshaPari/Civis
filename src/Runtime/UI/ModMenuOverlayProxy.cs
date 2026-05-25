#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Legacy adapter that lets older call sites wrap a <see cref="ModMenuPanel"/>
    /// behind the new <see cref="IModMenuHost"/> contract without depending on
    /// <see cref="ModMenuOverlay"/> inheritance.
    /// </summary>
    internal sealed class ModMenuOverlayProxy : IModMenuHost
    {
        private readonly ModMenuPanel _target;

        public ModMenuOverlayProxy(ModMenuPanel target)
        {
            _target = target;
        }

        /// <inheritdoc />
        public Action? OnReloadRequested
        {
            get => _target.OnReloadRequested;
            set => _target.OnReloadRequested = value;
        }

        /// <inheritdoc />
        public Action<string, bool>? OnPackToggled
        {
            get => _target.OnPackToggled;
            set => _target.OnPackToggled = value;
        }

        /// <inheritdoc />
        public bool IsVisible => _target.IsVisible;

        /// <inheritdoc />
        public void Show() => _target.Show();

        /// <inheritdoc />
        public void Hide() => _target.Hide();

        /// <inheritdoc />
        public void Toggle() => _target.Toggle();

        /// <inheritdoc />
        public void SetPacks(IEnumerable<PackDisplayInfo> packs) => _target.SetPacks(packs);

        /// <inheritdoc />
        public void SetStatus(string message, int errorCount = 0) => _target.SetStatus(message, errorCount);
    }
}
