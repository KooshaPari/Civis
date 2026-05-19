#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.Registry;
using DINOForge.SDK.UI.Extended;
using FluentAssertions;
using Xunit;

using DomainsHud = DINOForge.Domains.UI.Models.HudElementDefinition;
using SdkHud = DINOForge.SDK.UI.Models.HudElementDefinition;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Dispatch 2 of task #194 (UI registry wiring) — Step 10 of the plan.
///
/// Asserts that registered HudElements actually instantiate (in registry form) when
/// the real <c>packs/ui-hud-minimal</c> pack is loaded through
/// <see cref="UIContentLoader.LoadPack"/>. The DFCanvas render assertion is
/// Unity-only and cannot run in CI, so this integration test verifies the registry
/// side of the contract: count + per-element shape + theme application.
///
/// The Runtime/UI/DFCanvas.RenderHudElementsFromRegistry() path is exercised separately
/// by the in-game smoke test under <c>scripts/proof/</c>; here we only verify that the
/// data DFCanvas would consume is correctly populated end-to-end through the pack
/// loader.
/// </summary>
[Trait("Category", "Integration")]
[Trait("Bridge", "None")]
public class UIWireupIntegrationTests
{
    [Fact]
    public void UiHudMinimalPack_LoadPack_PopulatesAllFiveHudElements()
    {
        // Arrange
        string packDir = LocateRepoPack("ui-hud-minimal");
        File.Exists(Path.Combine(packDir, "pack.yaml")).Should().BeTrue(
            "ui-hud-minimal/pack.yaml is the fixture this integration test depends on");

        UIPlugin plugin = new UIPlugin(new RegistryManager());

        // Act — LoadPack scans hud_elements/, menus/, themes/ subdirs of packDir.
        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");

        // Assert — the 5 elements DFCanvas.RenderHudElementsFromRegistry() will iterate.
        plugin.HudElements.Count.Should().Be(5,
            "the pack declares player-health-bar, resource-counter, minimap, unit-portrait, alert-banner");

        IReadOnlyList<HudElementDefinition> all = plugin.HudElements.All;

        all.Select(e => e.Id).Should().BeEquivalentTo(new[]
        {
            "player-health-bar",
            "resource-counter",
            "minimap",
            "unit-portrait",
            "alert-banner",
        });

        // Every element must carry the shape DFCanvas expects (id, position, size).
        foreach (HudElementDefinition element in all)
        {
            element.Id.Should().NotBeNullOrWhiteSpace();
            element.Type.Should().NotBeNullOrWhiteSpace();
            element.Position.Should().NotBeNullOrWhiteSpace();
            element.Width.Should().BeGreaterThan(0,
                $"hud element '{element.Id}' must have positive width for DFCanvas to size its panel");
            element.Height.Should().BeGreaterThan(0,
                $"hud element '{element.Id}' must have positive height for DFCanvas to size its panel");
            element.Opacity.Should().BeInRange(0f, 1f,
                $"hud element '{element.Id}' opacity must be a valid CanvasGroup alpha");
        }
    }

    [Fact]
    public void UiHudMinimalPack_PositionAnchors_AreAllRecognizedByDFCanvas()
    {
        // Arrange — same pack load as above
        string packDir = LocateRepoPack("ui-hud-minimal");
        UIPlugin plugin = new UIPlugin(new RegistryManager());
        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");

        // Act + Assert — every position string must be one of the 5 anchors DFCanvas
        // understands. If a pack ever introduces a new value, DFCanvas will silently
        // fall back to top_left, which is a UX bug — guard against that here.
        string[] supportedPositions = { "top_left", "top_right", "bottom_left", "bottom_right", "center" };
        foreach (HudElementDefinition element in plugin.HudElements.All)
        {
            supportedPositions.Should().Contain(element.Position,
                $"DFCanvas.ApplyPositionAnchor only recognizes {string.Join(",", supportedPositions)}; " +
                $"hud element '{element.Id}' has position '{element.Position}'");
        }
    }

    [Fact]
    public void UiHudMinimalPack_LoadPack_PopulatesMenusAndThemes()
    {
        // Arrange
        string packDir = LocateRepoPack("ui-hud-minimal");
        UIPlugin plugin = new UIPlugin(new RegistryManager());

        // Act
        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");

        // Assert — all three registries Dispatch 2 cares about for theming + menu rendering.
        plugin.Menus.Count.Should().Be(7, "main-menu.yaml declares 7 menus");
        plugin.Themes.Count.Should().BeGreaterOrEqualTo(4,
            "pack adds 4 themes (2 overwriting registry defaults)");

        // ActiveTheme must be set so DFCanvas.ResolveBackgroundColor returns a valid color.
        plugin.Themes.ActiveTheme.Should().NotBeNull(
            "DFCanvas.RenderHudElementsFromRegistry uses ActiveTheme.BackgroundColor for panel tint");
    }

    [Fact]
    public void Task238_RegistryDrain_PackHudElements_FlowToHudElementRendererAdapter()
    {
        // Task #238: end-to-end drain proof for #194. The pack→UIPlugin→adapter
        // pipeline must be functional even without a live DFCanvas Unity Transform.
        //   1. Load pack → UIPlugin.HudElements populated.
        //   2. Translate Domains-side defs to SDK-side mirrors (the same conversion
        //      DFCanvas.RenderRegistryHudElements performs in-game).
        //   3. Push each through HudElementRendererAdapter.Render.
        //   4. Adapter records the call (DeferredHudMount sentinel when no canvas
        //      root is attached) and returns a non-null handle keyed by id.
        //
        // If any step is broken, the pack→registry→renderer pipeline is dead end-to-end.
        string packDir = LocateRepoPack("ui-hud-minimal");
        UIPlugin plugin = new UIPlugin(new RegistryManager());
        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");

        // Detach so test starts from a clean adapter state (no leftover mounts from
        // sibling tests in the SDK.UI test class).
        HudElementRendererAdapter.Instance.SetCanvasRoot(null);

        List<ExtendedHandle> handles = new List<ExtendedHandle>();
        try
        {
            foreach (DomainsHud src in plugin.HudElements.All)
            {
                SdkHud sdkDef = new SdkHud
                {
                    Id = src.Id,
                    Type = src.Type,
                    Position = src.Position,
                    Width = src.Width,
                    Height = src.Height,
                    Opacity = src.Opacity,
                    Description = src.Description ?? string.Empty,
                };
                if (src.VisibleIn != null)
                    sdkDef.VisibleIn = new List<string>(src.VisibleIn);
                if (src.ColorOverrides != null)
                    sdkDef.ColorOverrides = new Dictionary<string, string>(src.ColorOverrides);

                ExtendedHandle handle = HudElementRendererAdapter.Instance.Render(sdkDef);
                handle.Should().NotBeNull(
                    $"adapter must accept registry-sourced HUD element '{src.Id}' (Task #238)");
                handle.Id.Should().Be(src.Id,
                    "handle id must round-trip the definition id so DFCanvas.Unrender can locate the mount");
                handles.Add(handle);
            }

            handles.Should().HaveCount(5,
                "all 5 ui-hud-minimal HUD elements must drain through the renderer adapter — " +
                "this is the closure proof for #194 registry-drain");
        }
        finally
        {
            // Clean up so siblings see a clean adapter.
            foreach (ExtendedHandle h in handles)
            {
                try { HudElementRendererAdapter.Instance.Unrender(h); } catch { /* best-effort cleanup */ }
            }
        }
    }

    [Fact]
    public void HudElementRegistry_HotReloadCycle_ReplacesRegisteredElementsByIdempotentId()
    {
        // Verifies the pre-condition for DFCanvas.ClearRegistryHudElements() + re-render:
        // when the same pack is loaded twice (HMR), Register() overwrites by ID and the
        // count stays stable instead of doubling.
        string packDir = LocateRepoPack("ui-hud-minimal");
        UIPlugin plugin = new UIPlugin(new RegistryManager());

        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");
        int firstCount = plugin.HudElements.Count;

        // Simulate a hot reload — load the same pack again
        plugin.ContentLoader.LoadPack(packDir, "ui-hud-minimal");
        int secondCount = plugin.HudElements.Count;

        secondCount.Should().Be(firstCount,
            "HudElementRegistry.Register replaces by ID; a re-load must not duplicate elements. " +
            "DFCanvas.ClearRegistryHudElements() relies on this invariant.");
    }

    /// <summary>
    /// Walk up from the test assembly's base directory to find the repo root containing
    /// <c>packs/&lt;packId&gt;/pack.yaml</c>. Returns the absolute pack directory.
    /// </summary>
    private static string LocateRepoPack(string packId)
    {
        string dir = AppContext.BaseDirectory;
        for (int i = 0; i < 12; i++)
        {
            string candidate = Path.Combine(dir, "packs", packId, "pack.yaml");
            if (File.Exists(candidate))
                return Path.Combine(dir, "packs", packId);

            DirectoryInfo? parent = Directory.GetParent(dir);
            if (parent == null)
                break;
            dir = parent.FullName;
        }
        throw new DirectoryNotFoundException(
            $"Could not locate packs/{packId}/pack.yaml by walking up from {AppContext.BaseDirectory}.");
    }
}
