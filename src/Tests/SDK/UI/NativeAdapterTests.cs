// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatches 2-3 of #193 SDK split — adapter contract tests.
// Unity-runtime behavior (FindCanvas, Clone, SelectOne with live Transforms) is exercised
// in #193 Phase 2 acceptance via game-launch — not here.

using System;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.UI.Native;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.UI
{
    public class NativeAdapterTests
    {
        // ------------------------------------------------------------------ //
        // Handle factory helpers — ctors are internal but DINOForge.SDK has
        // [InternalsVisibleTo("DINOForge.Tests")] so we can construct here.
        // ------------------------------------------------------------------ //

        private static NativeButtonHandle MakeButtonHandle(string name = "TestButton", string label = "Mods")
            => new NativeButtonHandle(name, label, new object());

        private static NativeCanvasHandle MakeCanvasHandle(string name = "TestCanvas", bool active = true)
            => new NativeCanvasHandle(name, active, new object());

        // ================================================================== //
        // NativeCanvasLocatorAdapter
        // ================================================================== //

        [Fact]
        public void CanvasLocator_Instance_IsSingleton()
        {
            NativeCanvasLocatorAdapter a = NativeCanvasLocatorAdapter.Instance;
            NativeCanvasLocatorAdapter b = NativeCanvasLocatorAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void CanvasLocator_Instance_ImplementsINativeCanvasLocator()
        {
            NativeCanvasLocatorAdapter.Instance.Should().BeAssignableTo<INativeCanvasLocator>();
        }

        [Fact]
        public void CanvasLocator_FindCanvas_ThrowsOnNullHint()
        {
            INativeCanvasLocator locator = NativeCanvasLocatorAdapter.Instance;
            Action act = () => locator.FindCanvas(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("nameHint");
        }

        // ================================================================== //
        // NativeButtonAdapter
        // ================================================================== //

        [Fact]
        public void ButtonAdapter_Instance_IsSingleton()
        {
            NativeButtonAdapter a = NativeButtonAdapter.Instance;
            NativeButtonAdapter b = NativeButtonAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void ButtonAdapter_Instance_ImplementsINativeButtonAdapter()
        {
            NativeButtonAdapter.Instance.Should().BeAssignableTo<INativeButtonAdapter>();
        }

        [Fact]
        public void ButtonAdapter_FindButtonByText_ThrowsOnNullCanvas()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            Action act = () => btn.FindButtonByText(null!, "Settings");
            act.Should().Throw<ArgumentNullException>().WithParameterName("canvas");
        }

        [Fact]
        public void ButtonAdapter_FindButtonByText_ThrowsOnNullText()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            NativeCanvasHandle canvas = MakeCanvasHandle();
            Action act = () => btn.FindButtonByText(canvas, null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("text");
        }

        [Fact]
        public void ButtonAdapter_Clone_ThrowsOnNullSource()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            Action act = () => btn.Clone(null!, "Mods", () => { });
            act.Should().Throw<ArgumentNullException>().WithParameterName("source");
        }

        [Fact]
        public void ButtonAdapter_Clone_ThrowsOnNullLabel()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            NativeButtonHandle src = MakeButtonHandle();
            Action act = () => btn.Clone(src, null!, () => { });
            act.Should().Throw<ArgumentNullException>().WithParameterName("newLabel");
        }

        [Fact]
        public void ButtonAdapter_Clone_ThrowsOnNullOnClick()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            NativeButtonHandle src = MakeButtonHandle();
            Action act = () => btn.Clone(src, "Mods", null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("onClick");
        }

        [Fact]
        public void ButtonAdapter_SetLabel_ThrowsOnNullButton()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            Action act = () => btn.SetLabel(null!, "Mods");
            act.Should().Throw<ArgumentNullException>().WithParameterName("button");
        }

        [Fact]
        public void ButtonAdapter_SetLabel_ThrowsOnNullText()
        {
            INativeButtonAdapter btn = NativeButtonAdapter.Instance;
            NativeButtonHandle src = MakeButtonHandle();
            Action act = () => btn.SetLabel(src, null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("text");
        }

        // ================================================================== //
        // NativeUiSelectorAdapter
        // ================================================================== //

        [Fact]
        public void UiSelector_Instance_IsSingleton()
        {
            NativeUiSelectorAdapter a = NativeUiSelectorAdapter.Instance;
            NativeUiSelectorAdapter b = NativeUiSelectorAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void UiSelector_Instance_ImplementsINativeUiSelector()
        {
            NativeUiSelectorAdapter.Instance.Should().BeAssignableTo<INativeUiSelector>();
        }

        [Fact]
        public void UiSelector_SelectOne_ThrowsOnNullSelector()
        {
            INativeUiSelector sel = NativeUiSelectorAdapter.Instance;
            Action act = () => sel.SelectOne(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("selector");
        }

        [Fact]
        public void UiSelector_SelectMany_ThrowsOnNullSelector()
        {
            INativeUiSelector sel = NativeUiSelectorAdapter.Instance;
            Action act = () => sel.SelectMany(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("selector");
        }
    }
}
