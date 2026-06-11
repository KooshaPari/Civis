#nullable enable
using System;
using DINOForge.Tests.Support;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Definitions;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-DEBUG-001 → 005: Debug Panel page — refresh button, expander sections,
/// section headers matching the 5 canonical debug categories.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionDebugPanelTests(CompanionFixture fixture)
{
    private void GoToDebugPanel() => fixture.GoToDebugPanel();

    // ── COMP-DEBUG-001 ─────────────────────────────────────────────────────────

    [CompanionFact]
    public void DebugPanel_RefreshButton_Exists()
    {
        GoToDebugPanel();

        AutomationElement? btn = fixture.WaitForElement("DebugRefreshButton");
        btn.Should().NotBeNull("DebugRefreshButton must be present on the Debug Panel page");
    }

    // ── COMP-DEBUG-002 ─────────────────────────────────────────────────────────

    [CompanionFact]
    public void DebugPanel_RefreshButton_IsEnabled()
    {
        GoToDebugPanel();

        AutomationElement? btn = fixture.WaitForElement("DebugRefreshButton");
        btn.Should().NotBeNull();
        btn!.IsEnabled.Should().BeTrue("Refresh button must be enabled on the Debug Panel");
    }

    // ── COMP-DEBUG-003 ─────────────────────────────────────────────────────────

    [CompanionFact]
    public void DebugPanel_AfterRefresh_SectionsControlExists()
    {
        GoToDebugPanel();

        AutomationElement? btn = fixture.WaitForElement("DebugRefreshButton");
        btn.Should().NotBeNull();
        btn!.AsButton().Invoke();

        // Allow async ViewModel.RefreshAsync to complete
        bool sectionsReady = TestWait.UntilAsync(
            () => fixture.MainWindow!.FindFirstDescendant(cf => cf.ByAutomationId("DebugSectionsControl")) != null,
            TimeSpan.FromSeconds(5),
            pollMs: 50).GetAwaiter().GetResult();
        sectionsReady.Should().BeTrue("debug sections must appear after refresh completes");

        AutomationElement? sections = fixture.WaitForElement("DebugSectionsControl");
        sections.Should().NotBeNull(
            "DebugSectionsControl must be present after refresh completes");
    }

    // ── COMP-DEBUG-004 ─────────────────────────────────────────────────────────

    [CompanionFact]
    public void DebugPanel_AfterRefresh_HasExpanderSections()
    {
        GoToDebugPanel();

        AutomationElement? btn = fixture.WaitForElement("DebugRefreshButton");
        btn!.AsButton().Invoke();
        bool expandersReady = TestWait.UntilAsync(
            () => fixture.MainWindow!.FindAllDescendants(cf => cf.ByControlType(ControlType.Group)).Length > 0,
            TimeSpan.FromSeconds(5),
            pollMs: 50).GetAwaiter().GetResult();
        expandersReady.Should().BeTrue("debug sections must render after refresh completes");

        // Expanders are rendered inside the DebugSectionsControl ItemsControl.
        // In UIA, WinUI Expanders appear as Group or custom control types.
        AutomationElement[] expanders = fixture.MainWindow!
            .FindAllDescendants(cf => cf.ByControlType(ControlType.Group));

        expanders.Length.Should().BeGreaterThan(0,
            "at least one Expander section must be rendered after refresh");
    }

    // ── COMP-DEBUG-005 ─────────────────────────────────────────────────────────

    [CompanionFact]
    public void DebugPanel_AfterRefresh_ContainsExpectedSectionNames()
    {
        GoToDebugPanel();

        AutomationElement? btn = fixture.WaitForElement("DebugRefreshButton");
        btn!.AsButton().Invoke();
        bool textReady = TestWait.UntilAsync(
            () => fixture.MainWindow!.FindAllDescendants(cf => cf.ByControlType(ControlType.Text)).Length > 0,
            TimeSpan.FromSeconds(5),
            pollMs: 50).GetAwaiter().GetResult();
        textReady.Should().BeTrue("debug panel text must be available after refresh completes");

        // Gather all visible TextBlock names — section headers show as TextBlock content
        AutomationElement[] texts = fixture.MainWindow!
            .FindAllDescendants(cf => cf.ByControlType(ControlType.Text));

        string allText = string.Join(" ", System.Array.ConvertAll(texts, t => t.Name));

        // Each of the 5 DebugSectionViewModel names must appear somewhere in the UI
        string[] expectedSections = { "Platform Status", "ECS Info", "Pack Info", "System State", "Errors" };
        foreach (string section in expectedSections)
        {
            allText.Should().Contain(section,
                $"Debug Panel must show the '{section}' section after refresh");
        }
    }
}
