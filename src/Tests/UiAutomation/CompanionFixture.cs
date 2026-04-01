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
///   COMPANION_EXE  — path to DINOForge.DesktopCompanion.exe (built Release artifact)
///
/// Gracefully skips (not throws) when COMPANION_EXE is not set.
/// All UI automation tests are tagged [Trait("Category","UiAutomation")] and run via
/// ui-automation.yml on a windows-latest GitHub Actions runner.
/// </summary>
public sealed class CompanionFixture : IAsyncLifetime
{
    private const int WindowTimeoutMs  = 15_000;
    private const int NavWaitMs        = 400;
    
    private Application?   _app;
    private UIA3Automation? _automation;
    
    /// <summary>
    /// Indicates whether the fixture was initialized successfully.
    /// Tests should check this and skip if false.
    /// </summary>
    public bool IsInitialized { get; private set; }
    
    public Window? MainWindow { get; private set; }
    
    public Task InitializeAsync()
    {
        string? exePath = Environment.GetEnvironmentVariable("COMPANION_EXE");
        
        // Gracefully skip if COMPANION_EXE is not set
        if (string.IsNullOrEmpty(exePath))
        {
            IsInitialized = false;
            return Task.CompletedTask;
        }
        
        if (!File.Exists(exePath))
        {
            IsInitialized = false;
            return Task.CompletedTask;
        }
        
        _automation = new UIA3Automation();
        _app = Application.Launch(exePath);
        
        MainWindow = _app.GetMainWindow(_automation, TimeSpan.FromMilliseconds(WindowTimeoutMs));
        MainWindow.Should().NotBeNull("the companion main window must appear within the timeout");
        
        // Allow the default page (Dashboard) to fully load
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
    
    // ── Navigation helpers ────────────────────────────────────────────────────
    
    /// <summary>
    /// Clicks a NavigationView item by its AutomationId and waits for the page to render.
    /// </summary>
    public void NavigateTo(string navAutomationId)
    {
        if (!IsInitialized || MainWindow == null)
        {
            throw new InvalidOperationException("CompanionFixture was not initialized. COMPANION_EXE may not be set.");
        }
        
        AutomationElement? item = MainWindow!
            .FindFirstDescendant(cf => cf.ByAutomationId(navAutomationId));
            
        item.Should().NotBeNull($"navigation item '{navAutomationId}' must be present");
        item!.Click();
        Thread.Sleep(NavWaitMs);
    }
    
    /// <summary>Navigates to the Dashboard page.</summary>
    public void GoToDashboard()  => NavigateTo("NavDashboard");
    
    /// <summary>Navigates to the Pack List page.</summary>
    public void GoToPackList()   => NavigateTo("NavPackList");
    
    /// <summary>Navigates to the Debug Panel page.</summary>
    public void GoToDebugPanel() => NavigateTo("NavDebugPanel");
    
    /// <summary>Navigates to the Settings page.</summary>
    public void GoToSettings()   => NavigateTo("NavSettings");
    
    /// <summary>
    /// Finds an element by AutomationId, retrying for up to <paramref name="timeoutMs"/> ms.
    /// Useful for elements that appear after async data loads.
    /// </summary>
    public AutomationElement? WaitForElement(string automationId, int timeoutMs = 3000)
    {
        if (!IsInitialized || MainWindow == null)
        {
            throw new InvalidOperationException("CompanionFixture was not initialized. COMPANION_EXE may not be set.");
        }
        
        Stopwatch sw = Stopwatch.StartNew();
        while (sw.ElapsedMilliseconds < timeoutMs)
        {
            AutomationElement? el = MainWindow!
                .FindFirstDescendant(cf => cf.ByAutomationId(automationId));
            if (el != null) return el;
            Thread.Sleep(100);
        }
        return null;
    }
}

[CollectionDefinition(UiAutomationCollection.Name)]
public sealed class UiAutomationCollection : ICollectionFixture<CompanionFixture>
{
    public const string Name = "UiAutomation";
}
