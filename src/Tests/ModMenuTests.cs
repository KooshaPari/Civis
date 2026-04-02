using System;
using System.Collections.Generic;
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
    }
}
