#nullable enable
using System;
using System.IO;
using Xunit;

namespace DINOForge.Tests.UiAutomation;

/// <summary>
/// Marks a UiAutomation test as runnable only when COMPANION_EXE points to an
/// existing DesktopCompanion binary.
/// </summary>
[AttributeUsage(AttributeTargets.Method, AllowMultiple = false)]
public sealed class CompanionFactAttribute : FactAttribute
{
    private const string SkipReason =
        "UiAutomation tests require COMPANION_EXE to point to a DesktopCompanion binary.";

    public CompanionFactAttribute()
    {
        string? exePath = Environment.GetEnvironmentVariable("COMPANION_EXE");
        if (string.IsNullOrWhiteSpace(exePath) || !File.Exists(exePath))
        {
            Skip = SkipReason;
        }
    }
}
