// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 6 of #193 SDK split — ModSettingsHostAdapter contract tests.
// Real renderer wiring (BepInEx ConfigEntry surface, IModSettingsPanel.Build under DFCanvas)
// lands with the runtime renderer in M11.5 — not here.

using System;
using System.Collections.Generic;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.UI.Extended;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.UI
{
    public class ModSettingsHostAdapterTests
    {
        private sealed class FakePanel : IModSettingsPanel
        {
            public int DisposeCalls { get; private set; }
            public string Title => "FakePanel";
            public void Build(IModCanvas canvas) { }
            public void Dispose() { DisposeCalls++; }
        }

        [Fact]
        public void Instance_IsSingleton()
        {
            ModSettingsHostAdapter a = ModSettingsHostAdapter.Instance;
            ModSettingsHostAdapter b = ModSettingsHostAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void Instance_ImplementsIModSettingsHost()
        {
            ModSettingsHostAdapter.Instance.Should().BeAssignableTo<IModSettingsHost>();
        }

        [Fact]
        public void RegisterSettingsPanel_ThrowsOnNullModId()
        {
            IModSettingsHost host = ModSettingsHostAdapter.Instance;
            Action act = () => host.RegisterSettingsPanel(null!, new FakePanel());
            act.Should().Throw<ArgumentNullException>().WithParameterName("modId");
        }

        [Fact]
        public void RegisterSettingsPanel_ThrowsOnEmptyModId()
        {
            IModSettingsHost host = ModSettingsHostAdapter.Instance;
            Action act = () => host.RegisterSettingsPanel(string.Empty, new FakePanel());
            act.Should().Throw<ArgumentException>().WithParameterName("modId");
        }

        [Fact]
        public void RegisterSettingsPanel_ThrowsOnNullPanel()
        {
            IModSettingsHost host = ModSettingsHostAdapter.Instance;
            Action act = () => host.RegisterSettingsPanel("dinoforge.tests.settings.null", null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("panel");
        }

        [Fact]
        public void Register_ThenSnapshot_ContainsPanel()
        {
            ModSettingsHostAdapter adapter = ModSettingsHostAdapter.Instance;
            const string id = "dinoforge.tests.settings.register";
            adapter.UnregisterSettingsPanel(id); // defensive cleanup

            FakePanel panel = new FakePanel();
            ((IModSettingsHost)adapter).RegisterSettingsPanel(id, panel);

            IReadOnlyDictionary<string, IModSettingsPanel> snap = adapter.GetRegisteredPanels();
            snap.Should().ContainKey(id);
            snap[id].Should().BeSameAs(panel);

            adapter.UnregisterSettingsPanel(id);
            adapter.GetRegisteredPanels().Should().NotContainKey(id);
            panel.DisposeCalls.Should().Be(1, "unregister disposes the previously registered panel");
        }

        [Fact]
        public void Register_DuplicateId_LastWriterWins_AndDisposesPrevious()
        {
            ModSettingsHostAdapter adapter = ModSettingsHostAdapter.Instance;
            const string id = "dinoforge.tests.settings.duplicate";
            adapter.UnregisterSettingsPanel(id);

            FakePanel first = new FakePanel();
            FakePanel second = new FakePanel();

            IModSettingsHost host = adapter;
            host.RegisterSettingsPanel(id, first);
            host.RegisterSettingsPanel(id, second);

            first.DisposeCalls.Should().Be(1, "evicted panel must be disposed");
            second.DisposeCalls.Should().Be(0);
            adapter.GetRegisteredPanels()[id].Should().BeSameAs(second);

            adapter.UnregisterSettingsPanel(id);
        }

        [Fact]
        public void UnregisterSettingsPanel_UnknownId_NoOp()
        {
            ModSettingsHostAdapter adapter = ModSettingsHostAdapter.Instance;
            Action act = () => adapter.UnregisterSettingsPanel("dinoforge.tests.settings.never-registered");
            act.Should().NotThrow();
        }

        [Fact]
        public void UnregisterSettingsPanel_ThrowsOnNullOrEmptyModId()
        {
            ModSettingsHostAdapter adapter = ModSettingsHostAdapter.Instance;
            ((Action)(() => adapter.UnregisterSettingsPanel(null!)))
                .Should().Throw<ArgumentNullException>().WithParameterName("modId");
            ((Action)(() => adapter.UnregisterSettingsPanel(string.Empty)))
                .Should().Throw<ArgumentException>().WithParameterName("modId");
        }
    }
}
