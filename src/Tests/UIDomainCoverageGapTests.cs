using System;
using DINOForge.Domains.UI;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for HUDInjectionSystem and HUDElementDefinition edge cases and branch coverage gaps.
    /// Focuses on lifecycle management, null validation, case sensitivity, and error paths.
    /// </summary>
    public class UIDomainCoverageGapTests
    {
        [Fact]
        public void HUDInjectionSystem_Initialize_SetsInitializedFlag()
        {
            // Arrange
            var system = new HUDInjectionSystem();

            // Act
            system.Initialize();

            // Assert
            system.IsInitialized.Should().BeTrue("System should be marked as initialized");
            system.ElementCount.Should().Be(0, "Element count should be zero after initialization");
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_BeforeInitialize_ThrowsInvalidOperationException()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            var element = new HUDElementDefinition("test-id", "Test Element", "pack1");

            // Act & Assert
            system.Invoking(s => s.RegisterElement(element))
                .Should().Throw<InvalidOperationException>()
                .WithMessage("*has not been initialized*");
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_NullElement_ThrowsArgumentNullException()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();

            // Act & Assert
            system.Invoking(s => s.RegisterElement(null!))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("element");
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_AfterInitialize_IncrementsElementCount()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("test-id", "Test Element", "pack1");

            // Act
            system.RegisterElement(element);

            // Assert
            system.ElementCount.Should().Be(1, "Element count should increment after registration");
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_MultipleElements_AllRegistered()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element1 = new HUDElementDefinition("id-1", "Element 1", "pack1");
            var element2 = new HUDElementDefinition("id-2", "Element 2", "pack1");
            var element3 = new HUDElementDefinition("id-3", "Element 3", "pack2");

            // Act
            system.RegisterElement(element1);
            system.RegisterElement(element2);
            system.RegisterElement(element3);

            // Assert
            system.ElementCount.Should().Be(3, "All elements should be registered");
            var elements = system.GetElements();
            elements.Should().HaveCount(3, "GetElements should return all registered elements");
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_NotFound_ReturnsFalse()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("existing-id", "Element", "pack1");
            system.RegisterElement(element);

            // Act
            var result = system.UnregisterElement("non-existent-id");

            // Assert
            result.Should().BeFalse("Unregistering non-existent element should return false");
            system.ElementCount.Should().Be(1, "Element count should not change");
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_Found_RemovesAndReturnsTrue()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("target-id", "Element", "pack1");
            system.RegisterElement(element);

            // Act
            var result = system.UnregisterElement("target-id");

            // Assert
            result.Should().BeTrue("Unregistering existing element should return true");
            system.ElementCount.Should().Be(0, "Element should be removed from system");
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_CaseInsensitiveId_Removes()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("TestElement", "Element", "pack1");
            system.RegisterElement(element);

            // Act
            var result = system.UnregisterElement("testelement");

            // Assert
            result.Should().BeTrue("Should match ID case-insensitively");
            system.ElementCount.Should().Be(0, "Element should be removed despite case difference");
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_MixedCase_Removes()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("MyHealthBar", "Health Bar", "pack1");
            system.RegisterElement(element);

            // Act
            var result = system.UnregisterElement("myhealthbar");

            // Assert
            result.Should().BeTrue("Should match ID case-insensitively");
            system.ElementCount.Should().Be(0, "Element should be removed");
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_MultipleElements_RemovesCorrectOne()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element1 = new HUDElementDefinition("health-bar", "Health Bar", "pack1");
            var element2 = new HUDElementDefinition("mana-bar", "Mana Bar", "pack1");
            var element3 = new HUDElementDefinition("stamina-bar", "Stamina Bar", "pack1");
            system.RegisterElement(element1);
            system.RegisterElement(element2);
            system.RegisterElement(element3);

            // Act
            var result = system.UnregisterElement("mana-bar");

            // Assert
            result.Should().BeTrue("Should find and remove the element");
            system.ElementCount.Should().Be(2, "One element should be removed");
            var remaining = system.GetElements();
            remaining.Should().NotContain(e => e.Id == "mana-bar", "Mana bar should be removed");
            remaining.Should().Contain(e => e.Id == "health-bar", "Health bar should remain");
            remaining.Should().Contain(e => e.Id == "stamina-bar", "Stamina bar should remain");
        }

        [Fact]
        public void HUDInjectionSystem_Update_BeforeInitialize_DoesNotThrow()
        {
            // Arrange
            var system = new HUDInjectionSystem();

            // Act & Assert - should not throw
            system.Invoking(s => s.Update()).Should().NotThrow();
        }

        [Fact]
        public void HUDInjectionSystem_Update_AfterInitialize_DoesNotThrow()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("test", "Test", "pack1");
            system.RegisterElement(element);

            // Act & Assert - should not throw
            system.Invoking(s => s.Update()).Should().NotThrow();
        }

        [Fact]
        public void HUDInjectionSystem_Shutdown_ClearsElementsAndDisablesSystem()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element1 = new HUDElementDefinition("id-1", "Element 1", "pack1");
            var element2 = new HUDElementDefinition("id-2", "Element 2", "pack1");
            system.RegisterElement(element1);
            system.RegisterElement(element2);
            system.ElementCount.Should().Be(2, "Setup: Elements should be registered");

            // Act
            system.Shutdown();

            // Assert
            system.IsInitialized.Should().BeFalse("System should no longer be initialized");
            system.ElementCount.Should().Be(0, "All elements should be cleared");
            var elements = system.GetElements();
            elements.Should().BeEmpty("GetElements should return empty list after shutdown");
        }

        [Fact]
        public void HUDInjectionSystem_Shutdown_ThenRegisterElement_ThrowsInvalidOperationException()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            system.Shutdown();
            var element = new HUDElementDefinition("test", "Test", "pack1");

            // Act & Assert
            system.Invoking(s => s.RegisterElement(element))
                .Should().Throw<InvalidOperationException>()
                .WithMessage("*has not been initialized*");
        }

        [Fact]
        public void HUDInjectionSystem_InitializeMultipleTimes_ClearsExistingElements()
        {
            // Arrange
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("first", "First Element", "pack1");
            system.RegisterElement(element);
            system.ElementCount.Should().Be(1, "Setup: First element should be registered");

            // Act
            system.Initialize();

            // Assert
            system.ElementCount.Should().Be(0, "Initializing again should clear elements");
            system.IsInitialized.Should().BeTrue("System should still be initialized");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_NullId_ThrowsArgumentNullException()
        {
            // Act & Assert
            new Action(() => new HUDElementDefinition(null!, "Test", "pack1"))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("id");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_NullName_ThrowsArgumentNullException()
        {
            // Act & Assert
            new Action(() => new HUDElementDefinition("test-id", null!, "pack1"))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("name");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_NullSourcePackId_ThrowsArgumentNullException()
        {
            // Act & Assert
            new Action(() => new HUDElementDefinition("test-id", "Test", null!))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("sourcePackId");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_ValidParams_InitializesCorrectly()
        {
            // Act
            var element = new HUDElementDefinition("health-bar", "Health Bar", "pack1", "top-left", 100, true);

            // Assert
            element.Id.Should().Be("health-bar");
            element.Name.Should().Be("Health Bar");
            element.SourcePackId.Should().Be("pack1");
            element.Anchor.Should().Be("top-left");
            element.ZOrder.Should().Be(100);
            element.IsVisible.Should().BeTrue();
        }

        [Fact]
        public void HUDElementDefinition_Constructor_DefaultAnchor_IsTopLeft()
        {
            // Act
            var element = new HUDElementDefinition("test", "Test", "pack1");

            // Assert
            element.Anchor.Should().Be("top-left", "Default anchor should be top-left");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_DefaultZOrder_IsZero()
        {
            // Act
            var element = new HUDElementDefinition("test", "Test", "pack1");

            // Assert
            element.ZOrder.Should().Be(0, "Default z-order should be zero");
        }

        [Fact]
        public void HUDElementDefinition_Constructor_DefaultIsVisible_IsTrue()
        {
            // Act
            var element = new HUDElementDefinition("test", "Test", "pack1");

            // Assert
            element.IsVisible.Should().BeTrue("Default visibility should be true");
        }

        [Fact]
        public void HUDElementDefinition_IsVisible_CanBeToggled()
        {
            // Arrange
            var element = new HUDElementDefinition("test", "Test", "pack1", isVisible: true);
            element.IsVisible.Should().BeTrue("Setup: Should be visible initially");

            // Act
            element.IsVisible = false;

            // Assert
            element.IsVisible.Should().BeFalse("IsVisible should be toggleable");
        }

        [Fact]
        public void HUDElementDefinition_AnchorVariations_AllValid()
        {
            // Arrange
            var anchors = new[] { "top-left", "top-center", "top-right", "middle-left", "middle-center", "middle-right", "bottom-left", "bottom-center", "bottom-right" };

            // Act & Assert
            foreach (var anchor in anchors)
            {
                var element = new HUDElementDefinition("test", "Test", "pack1", anchor: anchor);
                element.Anchor.Should().Be(anchor, $"Should accept {anchor} as valid anchor");
            }
        }

        [Fact]
        public void HUDElementDefinition_NegativeZOrder_IsAccepted()
        {
            // Act
            var element = new HUDElementDefinition("test", "Test", "pack1", zOrder: -10);

            // Assert
            element.ZOrder.Should().Be(-10, "Negative z-order should be accepted");
        }

        [Fact]
        public void HUDElementDefinition_LargeZOrder_IsAccepted()
        {
            // Act
            var element = new HUDElementDefinition("test", "Test", "pack1", zOrder: 1000);

            // Assert
            element.ZOrder.Should().Be(1000, "Large z-order should be accepted");
        }
    }
}
