// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 2 of #193 SDK split — third runtime adapter.

#nullable enable
using System;
using DINOForge.SDK.UI.Native;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="INativeButtonAdapter"/> contract.
    /// Wraps <see cref="NativeUiHelper"/> for find / clone / relabel of vanilla DINO buttons.
    /// </summary>
    /// <remarks>
    /// Naming note: the class is <c>NativeButtonAdapter</c> while the interface is
    /// <c>INativeButtonAdapter</c>; this matches the singleton-per-interface pattern from
    /// <see cref="NativeLabelGuardAdapter"/> and avoids the awkward "Adapter ... Adapter"
    /// duplication. The class is not the interface.
    ///
    /// Cloning preserves the source button's visual style via <see cref="NativeUiHelper.CloneButton"/>.
    /// The cloned button has its <c>onClick</c> wired to the supplied delegate, replacing
    /// any inherited persistent listeners.
    /// </remarks>
    public sealed class NativeButtonAdapter : INativeButtonAdapter
    {
        private static NativeButtonAdapter? _instance;

        /// <summary>Singleton accessor — adapter holds no per-call state.</summary>
        public static NativeButtonAdapter Instance => _instance ??= new NativeButtonAdapter();

        private NativeButtonAdapter() { }

        /// <inheritdoc />
        public NativeButtonHandle? FindButtonByText(NativeCanvasHandle canvas, string text)
        {
            if (canvas is null) throw new ArgumentNullException(nameof(canvas));
            if (text is null) throw new ArgumentNullException(nameof(text));

            return FindButtonByTextCore(canvas, text);
        }

        /// <inheritdoc />
        public NativeButtonHandle Clone(NativeButtonHandle source, string newLabel, Action onClick)
        {
            if (source is null) throw new ArgumentNullException(nameof(source));
            if (newLabel is null) throw new ArgumentNullException(nameof(newLabel));
            if (onClick is null) throw new ArgumentNullException(nameof(onClick));

            return CloneCore(source, newLabel, onClick);
        }

        /// <inheritdoc />
        public void SetLabel(NativeButtonHandle button, string text)
        {
            if (button is null) throw new ArgumentNullException(nameof(button));
            if (text is null) throw new ArgumentNullException(nameof(text));

            SetLabelCore(button, text);
        }

        // ------------------------------------------------------------------ //
        // *Core methods isolate the UnityEngine.UI references behind the null
        // guards above. JIT defers loading UnityEngine.UI until a Core method is
        // actually invoked — so null-arg unit tests (which never reach the Core
        // call) don't trigger FileNotFoundException for the Unity assembly.
        // ------------------------------------------------------------------ //

        private static NativeButtonHandle? FindButtonByTextCore(NativeCanvasHandle canvas, string text)
        {
            if (!(canvas.Inner is Canvas unityCanvas) || unityCanvas == null)
            {
                return null;
            }

            Button? found = NativeUiHelper.FindButtonByText(unityCanvas.transform, text);
            if (found == null) return null;

            return WrapButton(found);
        }

        private static NativeButtonHandle CloneCore(NativeButtonHandle source, string newLabel, Action onClick)
        {
            if (!(source.Inner is Button unityButton) || unityButton == null)
            {
                throw new InvalidOperationException(
                    "NativeButtonHandle.Inner is not a UnityEngine.UI.Button — handle was not produced by this adapter.");
            }

            Button clone = NativeUiHelper.CloneButton(unityButton, newLabel);
            // CloneButton resets onClick to a fresh ButtonClickedEvent; wire the supplied delegate.
            clone.onClick.AddListener(() => onClick());
            return WrapButton(clone);
        }

        private static void SetLabelCore(NativeButtonHandle button, string text)
        {
            if (!(button.Inner is Button unityButton) || unityButton == null)
            {
                throw new InvalidOperationException(
                    "NativeButtonHandle.Inner is not a UnityEngine.UI.Button — handle was not produced by this adapter.");
            }

            NativeUiHelper.SetButtonText(unityButton, text);
        }

        private static NativeButtonHandle WrapButton(Button button)
        {
            string label = NativeUiHelper.GetButtonText(button);
            return new NativeButtonHandle(button.name, label, button);
        }
    }
}
