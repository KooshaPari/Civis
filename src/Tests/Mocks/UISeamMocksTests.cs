#nullable enable
// Pattern #125 drift-prevention: exercise each UI-seam mock at least once so
// signature changes break compile-time here before they leak silently into
// ad-hoc test handrolls across the suite. Task #773.

using DINOForge.SDK.UI.Bridge;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;
using DINOForge.SDK.UI.Native;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public sealed class UISeamMocksTests
{
    [Fact]
    public void MockModButtonInjector_RecordsInjectAndSetCanvasRoot()
    {
        var injector = new MockModButtonInjector();
        var menu = new MockModMenuHost();
        // NativeCanvasHandle has an internal ctor; InternalsVisibleTo("DINOForge.Tests") in SDK allows this.
        var canvas = new NativeCanvasHandle("vanilla-main-menu", isActive: true, inner: new object());

        IModButtonInjector seam = injector;
        var handle = seam.InjectModsButton(canvas, menu);
        seam.SetCanvasRoot(null);
        seam.SetCanvasRoot(new object());

        injector.InjectModsButtonCount.Should().Be(1);
        injector.InjectCalls[0].Target.Should().BeSameAs(menu);
        injector.InjectCalls[0].Canvas.Should().BeSameAs(canvas);
        injector.SetCanvasRootCount.Should().Be(2);
        injector.SetCanvasRootCalls[0].Should().BeNull();
        handle.Should().NotBeNull();
    }

    [Fact]
    public void MockHudElementRenderer_TracksRenderUnrenderLifecycle()
    {
        var renderer = new MockHudElementRenderer();
        var def = new HudElementDefinition { Id = "hp-bar", Type = "health_bar" };
        IHudElementRenderer seam = renderer;

        var handle = seam.Render(def);
        renderer.RenderCount.Should().Be(1);
        renderer.ActiveHandles.Should().ContainSingle().Which.Should().BeSameAs(handle);

        seam.Unrender(handle);
        renderer.UnrenderCount.Should().Be(1);
        renderer.ActiveHandles.Should().BeEmpty();

        seam.SetCanvasRoot(null);
        renderer.SetCanvasRootCalls.Should().ContainSingle().Which.Should().BeNull();
    }

    [Fact]
    public void MockModCanvas_RecordsCreatesAndDestroys()
    {
        var canvas = new MockModCanvas();
        IModCanvas seam = canvas;

        var panel = seam.CreatePanel(new PanelSpec("p1", RectAnchor.TopLeft, new Vector2(100, 50), new ColorRgba(0, 0, 0, 1)));
        var label = seam.CreateLabel(new LabelSpec("l1", "hello", new ColorRgba(1, 1, 1, 1), FontSize.Medium));
        var button = seam.CreateButton(new ButtonSpec("b1", "Click", () => { }));

        canvas.PanelsCreated.Should().ContainSingle();
        canvas.LabelsCreated.Should().ContainSingle();
        canvas.ButtonsCreated.Should().ContainSingle();
        panel.Id.Should().Be("p1");

        seam.Destroy(panel);
        seam.Destroy(label);
        seam.Destroy(button);
        canvas.Destroyed.Should().HaveCount(3);
    }

    [Fact]
    public void MockModMenuHost_ShowHideToggleTrackVisibility()
    {
        var host = new MockModMenuHost();
        IModMenuHost seam = host;

        seam.IsVisible.Should().BeFalse();
        seam.Toggle();
        seam.IsVisible.Should().BeTrue();
        host.ToggleCount.Should().Be(1);

        seam.Show();
        seam.IsVisible.Should().BeTrue();
        host.ShowCount.Should().Be(1);

        seam.Hide();
        seam.IsVisible.Should().BeFalse();
        host.HideCount.Should().Be(1);
    }

    [Fact]
    public void MockModSettingsHost_RecordsRegistrations()
    {
        var settings = new MockModSettingsHost();
        var panel = new StubSettingsPanel("Foo");
        IModSettingsHost seam = settings;

        seam.RegisterSettingsPanel("mod.foo", panel);
        seam.RegisterSettingsPanel("mod.bar", new StubSettingsPanel("Bar"));

        settings.RegisterCount.Should().Be(2);
        settings.Panels.Should().ContainKey("mod.foo");
        settings.Panels["mod.foo"].Should().BeSameAs(panel);
    }

    private sealed class StubSettingsPanel : IModSettingsPanel
    {
        public StubSettingsPanel(string title) { Title = title; }
        public string Title { get; }
        public void Build(IModCanvas canvas) { }
        public void Dispose() { }
    }
}
