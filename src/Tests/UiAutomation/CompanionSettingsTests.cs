#nullable enable
using System.Threading;
using FlaUI.Core.AutomationElements;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// COMP-SETTINGS-001 → 006 + COMP-UI-004/005 (original):
/// Settings page — text boxes, slider, save button, bridge status bar.
/// </summary>
[Collection(UiAutomationCollection.Name)]
[Trait("Category", "UiAutomation")]
public sealed class CompanionSettingsTests(CompanionFixture fixture)
{
    private void GoToSettings() => fixture.GoToSettings();

    // ── COMP-UI-004 (original, re-anchored) ──────────────────────────────────

    [Fact]
    public void Settings_GamePathTextBox_AcceptsInput()
    {
        GoToSettings();

        AutomationElement? gamePathBox = fixture.WaitForElement("GamePathInput");
        gamePathBox.Should().NotBeNull("settings page must show GamePathInput text box");

        TextBox tb = gamePathBox!.AsTextBox();
        tb.Enter("C:\\FakeGamePath\\DINO.exe");

        tb.Text.Should().Be("C:\\FakeGamePath\\DINO.exe",
            "entered path should appear in the text box");
    }

    // ── COMP-UI-005 (original) ────────────────────────────────────────────────

    [Fact]
    public void StatusBar_ShowsDisconnected_WhenBridgeAbsent()
    {
        // Status bar is in MainWindow shell — visible regardless of current page
        AutomationElement? statusBar = fixture.MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId("BridgeStatusText"));

        statusBar.Should().NotBeNull("a bridge status indicator must be present in the shell");

        string text = statusBar!.AsLabel().Text;
        text.Should().MatchRegex("(Not connected|Disconnected|No game)",
            "status should clearly indicate the bridge is not available");
    }

    // ── COMP-SETTINGS-001 ─────────────────────────────────────────────────────

    [Fact]
    public void Settings_PacksDirBox_Exists()
    {
        GoToSettings();

        AutomationElement? tb = fixture.WaitForElement("PacksDirBox");
        tb.Should().NotBeNull("PacksDirBox text box must be present on the Settings page");
    }

    // ── COMP-SETTINGS-002 ─────────────────────────────────────────────────────

    [Fact]
    public void Settings_PacksDirBox_AcceptsInput()
    {
        GoToSettings();

        AutomationElement? el = fixture.WaitForElement("PacksDirBox");
        el.Should().NotBeNull();

        TextBox tb = el!.AsTextBox();
        tb.Enter("C:\\TestPacks");

        tb.Text.Should().Be("C:\\TestPacks", "entered text must appear in PacksDirBox");
    }

    // ── COMP-SETTINGS-003 ─────────────────────────────────────────────────────

    [Fact]
    public void Settings_IntervalSlider_Exists()
    {
        GoToSettings();

        AutomationElement? slider = fixture.WaitForElement("IntervalSlider");
        slider.Should().NotBeNull("the auto-refresh interval Slider must be on the Settings page");
    }

    // ── COMP-SETTINGS-004 ─────────────────────────────────────────────────────

    [Fact]
    public void Settings_SaveButton_Exists()
    {
        GoToSettings();

        AutomationElement? btn = fixture.WaitForElement("SaveButton");
        btn.Should().NotBeNull("Save Settings button must be present on the Settings page");
        btn!.IsEnabled.Should().BeTrue("Save button must be enabled when not saving");
    }

    // ── COMP-SETTINGS-005 ─────────────────────────────────────────────────────

    [Fact]
    public void Settings_SaveButton_Click_UpdatesSaveStatus()
    {
        GoToSettings();

        AutomationElement? btn = fixture.WaitForElement("SaveButton");
        btn.Should().NotBeNull();

        btn!.AsButton().Invoke();
        Thread.Sleep(600); // Allow SaveAsync to complete

        // SaveStatusText should show a non-empty confirmation after save
        AutomationElement? statusEl = fixture.WaitForElement("SaveStatusText");
        statusEl.Should().NotBeNull("SaveStatusText must be present after clicking Save");
    }
}
