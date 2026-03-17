#nullable enable
using System.Threading;
using FlaUI.Core.AutomationElements;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-NAV-001 → 006: Navigation shell — NavigationView routing, page transitions,
/// window title.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionNavigationTests(CompanionFixture fixture)
{
    // ── COMP-NAV-001 ─────────────────────────────────────────────────────────

    [Fact]
    public void Window_Title_IsDINOForgeCompanion()
    {
        fixture.MainWindow!.Title.Should().Contain("DINOForge",
            "window title must identify the application");
    }

    // ── COMP-NAV-002 ─────────────────────────────────────────────────────────

    [Fact]
    public void NavDashboard_Click_ShowsDashboardContent()
    {
        fixture.GoToDashboard();

        // Dashboard-specific element: the loaded count stat block
        AutomationElement? el = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("DashLoadedCount"));

        el.Should().NotBeNull("DashLoadedCount stat block must be present on the Dashboard page");
    }

    // ── COMP-NAV-003 ─────────────────────────────────────────────────────────

    [Fact]
    public void NavPackList_Click_ShowsPackListContent()
    {
        fixture.GoToPackList();

        AutomationElement? listView = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("PackListView"));

        listView.Should().NotBeNull("PackListView must appear after navigating to Pack List");
    }

    // ── COMP-NAV-004 ─────────────────────────────────────────────────────────

    [Fact]
    public void NavDebugPanel_Click_ShowsDebugContent()
    {
        fixture.GoToDebugPanel();

        AutomationElement? btn = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("DebugRefreshButton"));

        btn.Should().NotBeNull("DebugRefreshButton must appear after navigating to Debug Panel");
    }

    // ── COMP-NAV-005 ─────────────────────────────────────────────────────────

    [Fact]
    public void NavSettings_Click_ShowsSettingsContent()
    {
        fixture.GoToSettings();

        AutomationElement? tb = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("GamePathInput"));

        tb.Should().NotBeNull("GamePathInput must appear after navigating to Settings");
    }

    // ── COMP-NAV-006 ─────────────────────────────────────────────────────────

    [Fact]
    public void ReturningToDashboard_AfterPackList_ShowsDashboard()
    {
        fixture.GoToPackList();
        fixture.GoToDashboard();

        AutomationElement? refreshBtn = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("DashRefreshButton"));

        refreshBtn.Should().NotBeNull(
            "DashRefreshButton must be visible after navigating back to Dashboard");
    }
}
