// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 6 of #193 SDK split — ModMenuHostAdapter contract tests.
// Unity-runtime behavior (DFCanvas mount, IMGUI window) lives behind ContextualModMenuHost
// and is exercised by game-launch acceptance — not here.

using System.Collections.Generic;
using DINOForge.Runtime.UI.Adapters;
using FluentAssertions;
using Xunit;
using ExtendedIModMenuHost = DINOForge.SDK.UI.Extended.IModMenuHost;

namespace DINOForge.Tests.SDK.UI
{
    public class ModMenuHostAdapterTests
    {
        [Fact]
        public void Instance_IsSingleton()
        {
            ModMenuHostAdapter a = ModMenuHostAdapter.Instance;
            ModMenuHostAdapter b = ModMenuHostAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void Instance_ImplementsIModMenuHost()
        {
            ModMenuHostAdapter.Instance.Should().BeAssignableTo<ExtendedIModMenuHost>();
        }

        [Fact]
        public void Show_WithoutTarget_RecordsIntendedVisible()
        {
            // Detach any prior target from sibling tests.
            ModMenuHostAdapter.Instance.SetTarget(null);

            ExtendedIModMenuHost host = ModMenuHostAdapter.Instance;
            host.Hide(); // reset
            host.Show();
            host.IsVisible.Should().BeTrue("Show() must record intent even when no target is wired");

            host.Hide();
            host.IsVisible.Should().BeFalse();
        }

        [Fact]
        public void Toggle_WithoutTarget_FlipsIntendedVisible()
        {
            ModMenuHostAdapter.Instance.SetTarget(null);

            ExtendedIModMenuHost host = ModMenuHostAdapter.Instance;
            host.Hide();
            bool before = host.IsVisible;

            host.Toggle();
            host.IsVisible.Should().Be(!before);

            host.Toggle();
            host.IsVisible.Should().Be(before);
        }

        [Fact]
        public void Show_DelegatesToTarget_WhenWired()
        {
            FakeRuntimeMenuHost fake = new FakeRuntimeMenuHost();
            ModMenuHostAdapter.Instance.SetTarget(fake);

            ExtendedIModMenuHost host = ModMenuHostAdapter.Instance;
            host.Show();

            fake.ShowCalls.Should().Be(1);
            host.IsVisible.Should().BeTrue();

            host.Hide();
            fake.HideCalls.Should().Be(1);
            host.IsVisible.Should().BeFalse();

            host.Toggle();
            fake.ToggleCalls.Should().Be(1);

            // Detach so other tests see a clean adapter.
            ModMenuHostAdapter.Instance.SetTarget(null);
        }

        // ------------------------------------------------------------------ //
        // Test fakes — implement only the runtime IModMenuHost surface the
        // adapter actually touches. Keeps tests Unity-free.
        // ------------------------------------------------------------------ //

        private sealed class FakeRuntimeMenuHost : DINOForge.Runtime.UI.IModMenuHost
        {
            public int ShowCalls { get; private set; }
            public int HideCalls { get; private set; }
            public int ToggleCalls { get; private set; }

            public System.Action? OnReloadRequested { get; set; }
            public System.Action<string, bool>? OnPackToggled { get; set; }

            public bool IsVisible { get; private set; }

            public void Show() { ShowCalls++; IsVisible = true; }
            public void Hide() { HideCalls++; IsVisible = false; }
            public void Toggle() { ToggleCalls++; IsVisible = !IsVisible; }
            public void SetPacks(IEnumerable<DINOForge.Runtime.UI.PackDisplayInfo> packs) { }
            public void SetStatus(string message, int errorCount = 0) { }
        }
    }
}
