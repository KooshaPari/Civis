#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// xUnit collection fixture: launches the DesktopCompanion process and finds its main window
/// via Windows UI Automation (FlaUI + UIA3).
///
/// Required environment variables:
///   COMPANION_EXE  - path to DINOForge.DesktopCompanion.exe (built Release artifact)
///
/// Gracefully leaves the fixture uninitialized when COMPANION_EXE is not set or invalid.
/// All UI automation tests are tagged [Trait("Category","UiAutomation")] and run via
/// ui-automation.yml on a windows-latest GitHub Actions runner.
/// </summary>
public sealed class CompanionFixture : IAsyncLifetime
{
    private const int WindowTimeoutMs = 15_000;
    private const int NavWaitMs = 400;

    private Application? _app;
    private UIA3Automation? _automation;

    public bool IsInitialized { get; private set; }

    public Window? MainWindow { get; private set; }

    public Task InitializeAsync()
    {
        string? exePath = Environment.GetEnvironmentVariable("COMPANION_EXE");

        if (string.IsNullOrWhiteSpace(exePath) || !File.Exists(exePath))
        {
            IsInitialized = false;
            return Task.CompletedTask;
        }

        _automation = new UIA3Automation();
        _app = Application.Launch(exePath);

        MainWindow = _app.GetMainWindow(_automation, TimeSpan.FromMilliseconds(WindowTimeoutMs));
        MainWindow.Should().NotBeNull("the companion main window must appear within the timeout");

        Thread.Sleep(600);
        IsInitialized = true;
        return Task.CompletedTask;
    }

    public Task DisposeAsync()
    {
        try { _app?.Close(); } catch { /* best-effort */ }
        _app?.Dispose();
        _automation?.Dispose();
        return Task.CompletedTask;
    }

    public void NavigateTo(string navAutomationId)
    {
        AutomationElement? item = RequireWindow()
            .FindFirstDescendant(cf => cf.ByAutomationId(navAutomationId));

        item.Should().NotBeNull($"navigation item '{navAutomationId}' must be present");
        item!.Click();
        Thread.Sleep(NavWaitMs);
    }

    public void GoToDashboard() => NavigateTo("NavDashboard");
    public void GoToPackList() => NavigateTo("NavPackList");
    public void GoToDebugPanel() => NavigateTo("NavDebugPanel");
    public void GoToSettings() => NavigateTo("NavSettings");

    public AutomationElement? WaitForElement(string automationId, int timeoutMs = 3000)
    {
        Window window = RequireWindow();

        Stopwatch sw = Stopwatch.StartNew();
        while (sw.ElapsedMilliseconds < timeoutMs)
        {
            AutomationElement? el = window.FindFirstDescendant(cf => cf.ByAutomationId(automationId));
            if (el != null)
            {
                return el;
            }

            Thread.Sleep(100);
        }

        return null;
    }

    private Window RequireWindow()
    {
        if (!IsInitialized || MainWindow == null)
        {
            throw new InvalidOperationException("CompanionFixture was not initialized.");
        }

        return MainWindow;
    }
}

[CollectionDefinition(UiAutomationCollection.Name)]
public sealed class UiAutomationCollection : ICollectionFixture<CompanionFixture>
{
    public const string Name = "UiAutomation";
}
