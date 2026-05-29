using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;
using DINOForge.Domains.UI;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for in-game mod menu (F10) and debug overlay (F9).
    /// Maps to: US-F2.1 Debug Overlay, US-F3.1 Mod Menu Toggle
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F2.1")]
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-RuntimeUI")]
    public class ModMenuTests
    {
        #region MenuManager Toggle Tests

        [Fact]
        [Trait("UserStory", "US-F3.1")]
        public void MenuManager_InitialState_IsClosed()
        {
            var manager = new MenuManager();

            manager.IsMenuOpen.Should().BeFalse();
            manager.ActivePanel.Should().Be(MenuManager.PanelModList);
        }

        [Fact]
        public void MenuManager_Toggle_OpensAndCloses()
        {
            var manager = new MenuManager();

            manager.Toggle();
            manager.IsMenuOpen.Should().BeTrue();

            manager.Toggle();
            manager.IsMenuOpen.Should().BeFalse();
        }

        [Fact]
        public void MenuManager_Open_SetsIsMenuOpen()
        {
            var manager = new MenuManager();

            manager.Open();
            manager.IsMenuOpen.Should().BeTrue();

            // Opening again is idempotent
            manager.Open();
            manager.IsMenuOpen.Should().BeTrue();
        }

        [Fact]
        public void MenuManager_Close_ClearsIsMenuOpen()
        {
            var manager = new MenuManager();
            manager.Open();

            manager.Close();
            manager.IsMenuOpen.Should().BeFalse();
        }

        [Fact]
        public void MenuManager_SetActivePanel_ChangesPanel()
        {
            var manager = new MenuManager();

            manager.SetActivePanel(MenuManager.PanelSettings);
            manager.ActivePanel.Should().Be(MenuManager.PanelSettings);

            manager.SetActivePanel(MenuManager.PanelPackDetails);
            manager.ActivePanel.Should().Be(MenuManager.PanelPackDetails);
        }

        [Fact]
        public void MenuManager_SetActivePanel_NullOrEmpty_Throws()
        {
            var manager = new MenuManager();

            Action act1 = () => manager.SetActivePanel(null!);
            act1.Should().Throw<ArgumentException>();

            Action act2 = () => manager.SetActivePanel("");
            act2.Should().Throw<ArgumentException>();
        }

        #endregion

        #region Panel Registration Tests

        [Fact]
        public void MenuManager_RegisterPanel_CanQueryVisibility()
        {
            var manager = new MenuManager();

            manager.RegisterPanel("custom_panel", true);
            manager.IsPanelVisible("custom_panel").Should().BeTrue();

            manager.RegisterPanel("hidden_panel", false);
            manager.IsPanelVisible("hidden_panel").Should().BeFalse();
        }

        [Fact]
        public void MenuManager_SetPanelVisible_UpdatesState()
        {
            var manager = new MenuManager();
            manager.RegisterPanel("test_panel", false);

            manager.SetPanelVisible("test_panel", true);
            manager.IsPanelVisible("test_panel").Should().BeTrue();

            manager.SetPanelVisible("test_panel", false);
            manager.IsPanelVisible("test_panel").Should().BeFalse();
        }

        [Fact]
        public void MenuManager_IsPanelVisible_UnregisteredPanel_ReturnsFalse()
        {
            var manager = new MenuManager();

            manager.IsPanelVisible("nonexistent").Should().BeFalse();
        }

        [Fact]
        public void MenuManager_GetPanelStates_ReturnsAllRegistered()
        {
            var manager = new MenuManager();
            manager.RegisterPanel("panel_a", true);
            manager.RegisterPanel("panel_b", false);

            IReadOnlyDictionary<string, bool> states = manager.GetPanelStates();

            states.Should().HaveCount(2);
            states["panel_a"].Should().BeTrue();
            states["panel_b"].Should().BeFalse();
        }

        #endregion

        #region UIPlugin Tests

        [Fact]
        public void UIPlugin_ContentTypes_ContainsExpectedTypes()
        {
            UIPlugin.ContentTypes.Should().Contain("ui_panels");
            UIPlugin.ContentTypes.Should().Contain("hud_elements");
            UIPlugin.ContentTypes.Count.Should().BeGreaterOrEqualTo(4);
        }

        [Fact]
        public void UIPlugin_ValidatePack_EmptyPackId_Throws()
        {
            var registries = new DINOForge.SDK.Registry.RegistryManager();
            var plugin = new UIPlugin(registries);

            Action act = () => plugin.ValidatePack("");
            act.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void UIPlugin_ValidatePack_ValidPackId_ReturnsEmpty()
        {
            var registries = new DINOForge.SDK.Registry.RegistryManager();
            var plugin = new UIPlugin(registries);

            IReadOnlyList<string> errors = plugin.ValidatePack("test-pack");
            errors.Should().BeEmpty();
        }

        #endregion

        #region HUDInjectionSystem Tests

        [Fact]
        public void HUDInjectionSystem_Initialize_SetsFlag()
        {
            var system = new HUDInjectionSystem();

            system.IsInitialized.Should().BeFalse();
            system.Initialize();
            system.IsInitialized.Should().BeTrue();
        }

        [Fact]
        public void HUDInjectionSystem_RegisterBeforeInit_Throws()
        {
            var system = new HUDInjectionSystem();
            var element = new HUDElementDefinition("test", "Test", "pack-1");

            Action act = () => system.RegisterElement(element);
            act.Should().Throw<InvalidOperationException>();
        }

        [Fact]
        public void HUDInjectionSystem_RegisterAndGet_Works()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            var element = new HUDElementDefinition("hp-bar", "HP Bar", "ui-pack", "top-left", 10);
            system.RegisterElement(element);

            system.ElementCount.Should().Be(1);
            system.GetElements()[0].Id.Should().Be("hp-bar");
        }

        [Fact]
        public void HUDInjectionSystem_Unregister_RemovesElement()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            system.RegisterElement(new HUDElementDefinition("e1", "E1", "pack-1"));
            system.RegisterElement(new HUDElementDefinition("e2", "E2", "pack-1"));

            bool removed = system.UnregisterElement("e1");

            removed.Should().BeTrue();
            system.ElementCount.Should().Be(1);
            system.GetElements()[0].Id.Should().Be("e2");
        }

        [Fact]
        public void HUDInjectionSystem_Shutdown_ClearsAll()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            system.RegisterElement(new HUDElementDefinition("e1", "E1", "pack-1"));

            system.Shutdown();

            system.IsInitialized.Should().BeFalse();
            system.ElementCount.Should().Be(0);
        }

        #endregion

        #region SPEC-007 Feature 2: Overlays hidden by default (source characterization)

        /// <summary>
        /// SPEC-007 Feature 2: UGUI overlays must start with CanvasGroup alpha 0 in Build().
        /// Pins <see cref="ModMenuPanel"/> without a Unity test host.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void ModMenuPanel_Build_StartsHiddenWithZeroAlpha()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("ModMenuPanel.cs"), "Build");

            buildBody.Should().MatchRegex(
                @"_canvasGroup\.alpha\s*=\s*0f",
                "ModMenuPanel.Build() must initialize CanvasGroup alpha to 0 (hidden by default)");
            buildBody.Should().MatchRegex(
                @"_canvasGroup\.interactable\s*=\s*false",
                "ModMenuPanel.Build() must not be interactable while hidden");
        }

        /// <summary>
        /// SPEC-007 Feature 2: Mod menu rebuilds its pack list after constructing the UI,
        /// so packs loaded before the panel is ready still appear in the left pane.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F3.1")]
        public void ModMenuPanel_Build_RehydratesLoadedPacks()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("ModMenuPanel.cs"), "Build");

            buildBody.Should().Contain("RebuildPackList();",
                "ModMenuPanel.Build() must populate the list from the current presenter state");
            buildBody.Should().Contain("RefreshDetail();",
                "ModMenuPanel.Build() must refresh the detail pane after the list is rendered");
        }

        /// <summary>
        /// SPEC-007 Feature 2: The list pane uses explicit RectTransform sizing (not
        /// VerticalLayoutGroup/ScrollRect) so pack rows stay visible when layout rebuild is broken.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F3.1")]
        public void ModMenuPanel_ListPane_UsesExplicitLayoutControls()
        {
            string body = ExtractMethodBody(ReadRuntimeUiSource("ModMenuPanel.cs"), "BuildListPane");

            body.Should().Contain("paneLe.preferredWidth = ListWidth;",
                "the list pane should have an explicit fixed width");
            body.Should().Contain("paneLe.minWidth = ListWidth;",
                "the list pane minimum width should match ListWidth");
            body.Should().Contain("_listContent = content;",
                "the list pane should wire the pack list content container");
            body.Should().Contain("scrollLe.flexibleHeight = 1f;",
                "scroll rect LayoutElement should use flexibleHeight to fill below the filter bar (not a fixed offsetMax override)");
        }

        /// <summary>
        /// SPEC-007 Feature 2: Debug UGUI panel mirrors ModMenuPanel hidden-by-default contract.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void DebugPanel_Build_StartsHiddenWithZeroAlpha()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("DebugPanel.cs"), "Build");

            buildBody.Should().MatchRegex(
                @"_canvasGroup\.alpha\s*=\s*0f",
                "DebugPanel.Build() must initialize CanvasGroup alpha to 0 (hidden by default)");
            buildBody.Should().MatchRegex(
                @"_canvasGroup\.interactable\s*=\s*false",
                "DebugPanel.Build() must not be interactable while hidden");
        }

        /// <summary>
        /// SPEC-007 Feature 2: Legacy IMGUI debug overlay must not render until toggled (F9).
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void DebugOverlayBehaviour_OnGui_SkipsRenderWhenNotVisible()
        {
            string source = ReadRuntimeSource("DebugOverlay.cs");

            source.Should().Contain(
                "if (!_visible) return",
                "DebugOverlayBehaviour must not draw IMGUI until Toggle() sets _visible");
        }

        private static string ReadRuntimeUiSource(string fileName) =>
            ReadRuntimeSource(Path.Combine("UI", fileName));

        private static string ReadRuntimeSource(string relativePath)
        {
            string path = LocateRuntimeFile(relativePath);
            return File.ReadAllText(path, System.Text.Encoding.UTF8);
        }

        private static string LocateRuntimeFile(string relativePath)
        {
            var dir = new DirectoryInfo(AppContext.BaseDirectory);
            for (int i = 0; i < 20 && dir != null; i++, dir = dir.Parent)
            {
                if (!File.Exists(Path.Combine(dir.FullName, "global.json")))
                {
                    continue;
                }

                string path = Path.Combine(dir.FullName, "src", "Runtime", relativePath.Replace('/', Path.DirectorySeparatorChar));
                if (File.Exists(path))
                {
                    return path;
                }
            }

            throw new InvalidOperationException(
                $"Runtime source '{relativePath}' not located from {AppContext.BaseDirectory}; " +
                "overlay characterization tests require repository source access.");
        }

        #endregion

        #region SPEC-007 HUD Strip alpha/visibility (source characterization)

        /// <summary>
        /// Pins <see cref="DINOForge.Runtime.UI.HudStrip"/> Build() contract: hidden at rest until hover fade.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void HudStrip_Build_StartsHiddenWithZeroAlpha()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("HudStrip.cs"), "Build");

            buildBody.Should().MatchRegex(
                @"_stripGroup\.alpha\s*=\s*0f",
                "HudStrip.Build() must initialize strip CanvasGroup alpha to 0 (hidden until hover)");
        }

        /// <summary>
        /// Hover fade targets full opacity on hover, not SPEC-007 historical 0.6f baseline.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void HudStrip_HoverFade_UsesZeroBaseAndFullOpacityOnHover()
        {
            string source = ReadRuntimeUiSource("HudStrip.cs");

            source.Should().Contain("private const float AlphaBase = 0f");
            source.Should().Contain("private const float AlphaHover = 1.0f");

            string animateBody = ExtractMethodBody(source, "AnimateHover");
            animateBody.Should().Contain("_hovered ? AlphaHover : AlphaBase");
        }

        /// <summary>
        /// DFCanvas drives hover state; strip does not self-detect pointer without SetHovered.
        /// </summary>
        [Fact]
        [Trait("UserStory", "US-F2.1")]
        public void HudStrip_SetHovered_IsCalledFromDFCanvas()
        {
            string source = ReadRuntimeUiSource("DFCanvas.cs");

            source.Should().Contain(
                "HudStrip.SetHovered",
                "DFCanvas must forward pointer-over state to HudStrip for hover fade");
        }

        /// <summary>
        /// SPEC-007 Feature 2 table still documents 0.6f always-visible HUD strip (bottom-right).
        /// Implementation: top-right, alpha 0 idle, fades to 1.0 on hover — see HudStrip_Build_* tests.
        /// </summary>
        [Fact(Skip = "SPEC-007 drift: doc says 0.6f always-visible bottom-right; HudStrip.cs uses AlphaBase=0 hover fade to AlphaHover=1.0 (top-right). Re-enable if product restores 0.6f baseline.")]
        [Trait("UserStory", "US-F2.1")]
        public void HudStrip_SPEC007_AlwaysVisibleAtPointSixAlpha()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("HudStrip.cs"), "Build");

            buildBody.Should().MatchRegex(
                @"_stripGroup\.alpha\s*=\s*0\.6f",
                "Historical SPEC-007 baseline: always-visible strip at 60% opacity");
        }

        private static string ExtractMethodBody(string source, string methodName)
        {
            var match = Regex.Match(
                source,
                $@"\bvoid\s+{Regex.Escape(methodName)}\s*\([^)]*\)\s*\{{",
                RegexOptions.Singleline);
            match.Success.Should().BeTrue($"expected method '{methodName}' in overlay source");

            int braceDepth = 0;
            int bodyStart = -1;
            for (int i = match.Index; i < source.Length; i++)
            {
                if (source[i] == '{')
                {
                    if (braceDepth == 0)
                    {
                        bodyStart = i + 1;
                    }

                    braceDepth++;
                }
                else if (source[i] == '}')
                {
                    braceDepth--;
                    if (braceDepth == 0 && bodyStart >= 0)
                    {
                        return source.Substring(bodyStart, i - bodyStart);
                    }
                }
            }

            throw new InvalidOperationException($"Could not extract body for method '{methodName}'.");
        }

        #endregion

        #region iter-145 click routing (EventSystem reconcile characterization)

        /// <summary>
        /// iter-145 H1: BuildCanvas must call the shared reconcile path, not only create when current is null.
        /// </summary>
        [Fact]
        public void DFCanvas_BuildCanvas_ReconcilesEventSystemViaPlugin()
        {
            string buildBody = ExtractMethodBody(ReadRuntimeUiSource("DFCanvas.cs"), "BuildCanvas");

            buildBody.Should().Contain(
                "Plugin.EnsureEventSystemAlive()",
                "DFCanvas.BuildCanvas must reconcile dual EventSystems before GraphicRaycaster");
            buildBody.Should().NotContain(
                "EventSystem.current == null",
                "null-only EventSystem guard misses dual-system click routing (iter-145 H1)");
        }

        /// <summary>
        /// iter-145 H1: PlayerLoop update is the live tick path in DINO when MonoBehaviour.Update does not run.
        /// </summary>
        [Fact]
        public void Plugin_PlayerLoopUpdate_ReconcilesEventSystemPeriodically()
        {
            string source = ReadRuntimeSource("Plugin.cs");
            string updateBody = ExtractMethodBody(source, "DINOForgePlayerLoopUpdate");

            updateBody.Should().Contain(
                "EnsureEventSystemAlive()",
                "DINOForgePlayerLoopUpdate must periodically reconcile EventSystem.current");
        }

        #endregion
    }
}
