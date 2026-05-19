// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 1 of #193 SDK split — adapter contract tests.

using System;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.UI.Native;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.UI
{
    public class NativeLabelGuardAdapterTests
    {
        // NativeButtonHandle ctor is internal but DINOForge.SDK has
        // [InternalsVisibleTo("DINOForge.Tests")] so we can construct it here.
        private static NativeButtonHandle MakeHandle(string name = "TestButton", string label = "Mods")
            => new NativeButtonHandle(name, label, new object());

        [Fact]
        public void Instance_IsSingleton()
        {
            NativeLabelGuardAdapter a = NativeLabelGuardAdapter.Instance;
            NativeLabelGuardAdapter b = NativeLabelGuardAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void Instance_ImplementsINativeLabelGuard()
        {
            NativeLabelGuardAdapter.Instance.Should().BeAssignableTo<INativeLabelGuard>();
        }

        [Fact]
        public void PinLabel_ThrowsOnNullButton()
        {
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            Action act = () => guard.PinLabel(null!, "Mods");
            act.Should().Throw<ArgumentNullException>().WithParameterName("button");
        }

        [Fact]
        public void PinLabel_ThrowsOnNullText()
        {
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            NativeButtonHandle handle = MakeHandle();
            Action act = () => guard.PinLabel(handle, null!);
            act.Should().Throw<ArgumentException>().WithParameterName("text");
        }

        [Fact]
        public void PinLabel_ThrowsOnEmptyText()
        {
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            NativeButtonHandle handle = MakeHandle();
            Action act = () => guard.PinLabel(handle, string.Empty);
            act.Should().Throw<ArgumentException>().WithParameterName("text");
        }

        [Fact]
        public void PinLabel_NoOpOnValidArgs_DoesNotThrow()
        {
            // Phase 2 Dispatch 1: adapter delegates to install-once Harmony patch already
            // applied at Plugin.cs startup. The PinLabel call is a contract-level
            // acknowledgement until Phase 2b refactors the patch for multi-label support.
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            NativeButtonHandle handle = MakeHandle();
            Action act = () => guard.PinLabel(handle, "Mods");
            act.Should().NotThrow();
        }

        [Fact]
        public void UnpinLabel_ThrowsOnNullButton()
        {
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            Action act = () => guard.UnpinLabel(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("button");
        }

        [Fact]
        public void UnpinLabel_NoOpOnValidArg_DoesNotThrow()
        {
            // Phase 2 Dispatch 1: ModsButtonTextPatch is install-once and single-label;
            // unpinning is a no-op until Phase 2b. This test locks the contract.
            INativeLabelGuard guard = NativeLabelGuardAdapter.Instance;
            NativeButtonHandle handle = MakeHandle();
            Action act = () => guard.UnpinLabel(handle);
            act.Should().NotThrow();
        }
    }
}
