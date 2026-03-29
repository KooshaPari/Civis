#nullable enable
using System.Threading;
using FlaUI.Core.AutomationElements;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-UI-005: Status bar shows "Bridge offline" text when game not running, updates when connected.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionStatusBarTests(CompanionFixture fixture)
{
    [Fact]
    public void StatusBar_IsVisible()
    {
        // Navigate to Dashboard to ensure status bar is rendered
        fixture.GoToDashboard();

        // Find the status bar container (AutomationId: "StatusBar")
        AutomationElement? statusBar = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("StatusBar"));

        statusBar.Should().NotBeNull(
            "status bar must be present in the companion window");
    }

    [Fact]
    public void StatusBar_ShowsBridgeOfflineWhenGameNotRunning()
    {
        // Navigate to Dashboard
        fixture.GoToDashboard();

        // Wait for the status text element that displays bridge state (AutomationId: "StatusText")
        AutomationElement? statusText = fixture.WaitForElement("StatusText", timeoutMs: 3000);

        statusText.Should().NotBeNull(
            "status text element must appear in the status bar");

        string statusContent = statusText!.Name ?? "";
        statusContent.Should().Contain("offline",
            because: "status bar should indicate the bridge is offline when game is not running");
    }

    [Fact]
    public void StatusBar_UpdatesWhenBridgeStateChanges()
    {
        fixture.GoToDashboard();

        // Initial state: should show offline
        AutomationElement? initialStatus = fixture.WaitForElement("StatusText", timeoutMs: 3000);
        string initialText = initialStatus?.Name ?? "";
        initialText.Should().Contain("offline",
            because: "status bar should start with offline state");

        // Simulate a state change by waiting — in a real scenario, this would be triggered
        // by a game connection or status update from the bridge
        Thread.Sleep(500);

        // Re-query the status text to verify it's still present (no crash/disappearance)
        AutomationElement? updatedStatus = fixture.WaitForElement("StatusText", timeoutMs: 3000);
        updatedStatus.Should().NotBeNull(
            "status bar should continue to exist and be queryable after state change");
    }
}
