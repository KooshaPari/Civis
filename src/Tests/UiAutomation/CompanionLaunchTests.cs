#nullable enable
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Conditions;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-UI-001: Main window launches and is visible.
/// COMP-UI-002: Pack list ListView is present and has items.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionLaunchTests(CompanionFixture fixture)
{
    [Fact]
    public void MainWindow_IsVisible()
    {
        fixture.MainWindow.Should().NotBeNull();
        fixture.MainWindow!.IsOffscreen.Should().BeFalse(
            "the main window should be on screen after launch");
    }

    [Fact]
    public void PackList_ListView_ExistsAndHasItems()
    {
        // The pack list is rendered as a ListView with AutomationId "PackListView"
        AutomationElement? listView = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("PackListView"));

        listView.Should().NotBeNull("PackListView must be present in the main window");

        ListBox lb = listView!.AsListBox();
        lb.Items.Should().NotBeEmpty("at least the example-balance pack should be present");
    }
}
