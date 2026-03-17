#nullable enable
using System.Threading;
using FlaUI.Core.AutomationElements;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-DASH-001 → 005: Dashboard page — stat cards, status message, refresh button.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionDashboardTests(CompanionFixture fixture)
{
    // Navigate to dashboard before each test assertion — shared fixture so state may drift.
    private void GoToDashboard() => fixture.GoToDashboard();

    // ── COMP-DASH-001 ─────────────────────────────────────────────────────────

    [Fact]
    public void Dashboard_LoadedCount_IsDisplayed()
    {
        GoToDashboard();

        AutomationElement? el = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("DashLoadedCount"));

        el.Should().NotBeNull("the Packs Loaded count must be visible on the Dashboard");

        string text = el!.AsLabel().Text;
        int.TryParse(text, out int count).Should().BeTrue(
            $"LoadedCount '{text}' must be a parseable integer");
        count.Should().BeGreaterThanOrEqualTo(0);
    }

    // ── COMP-DASH-002 ─────────────────────────────────────────────────────────

    [Fact]
    public void Dashboard_ErrorCount_IsDisplayed()
    {
        GoToDashboard();

        AutomationElement? el = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("DashErrorCount"));

        el.Should().NotBeNull("the Load Errors count must be visible on the Dashboard");

        string text = el!.AsLabel().Text;
        int.TryParse(text, out int count).Should().BeTrue(
            $"ErrorCount '{text}' must be a parseable integer");
        count.Should().BeGreaterThanOrEqualTo(0);
    }

    // ── COMP-DASH-003 ─────────────────────────────────────────────────────────

    [Fact]
    public void Dashboard_StatusMessage_IsPresent()
    {
        GoToDashboard();

        AutomationElement? el = fixture.WaitForElement("DashStatusText");
        el.Should().NotBeNull("the status message text block must be present on the Dashboard");
    }

    // ── COMP-DASH-004 ─────────────────────────────────────────────────────────

    [Fact]
    public void Dashboard_RefreshButton_IsEnabled()
    {
        GoToDashboard();

        AutomationElement? btn = fixture.WaitForElement("DashRefreshButton");
        btn.Should().NotBeNull("Refresh Packs button must be present on the Dashboard");
        btn!.IsEnabled.Should().BeTrue("button must be enabled when not loading");
    }

    // ── COMP-DASH-005 ─────────────────────────────────────────────────────────

    [Fact]
    public void Dashboard_RefreshButton_Click_CompletesWithoutException()
    {
        GoToDashboard();

        AutomationElement? btn = fixture.WaitForElement("DashRefreshButton");
        btn.Should().NotBeNull();

        // Click and wait for async refresh to finish — no crash expected
        btn!.AsButton().Invoke();
        Thread.Sleep(1500);

        // Dashboard should still be navigable (window alive, stat still present)
        AutomationElement? el = fixture.WaitForElement("DashLoadedCount");
        el.Should().NotBeNull("Dashboard must remain functional after a Refresh click");
    }
}
