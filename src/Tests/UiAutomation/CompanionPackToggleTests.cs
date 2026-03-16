#nullable enable
using System.Threading;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Definitions;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-UI-003: Toggling a pack in the UI changes its enabled state in the data service.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionPackToggleTests(CompanionFixture fixture)
{
    [Fact]
    public void TogglePack_FlipsEnabledState()
    {
        // Navigate to Pack List first so the toggle is visible
        fixture.GoToPackList();

        // Find the first pack toggle button (AutomationId pattern: "PackToggle_{packId}")
        AutomationElement? firstToggle = fixture.WaitForElement("PackToggle_example-balance");

        firstToggle.Should().NotBeNull(
            "example-balance pack should have a toggle button in the UI");

        ToggleButton toggle = firstToggle!.AsToggleButton();
        ToggleState stateBefore = toggle.ToggleState;

        toggle.Toggle();
        Thread.Sleep(300);

        ToggleState stateAfter = toggle.ToggleState;
        stateAfter.Should().NotBe(stateBefore, "toggling should flip the pack's enabled state");
    }
}
