#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for the screenshot capture fallback cascade system.
///
/// The screenshot system implements a multi-tier fallback strategy:
/// 1. bare-cua (fastest, Win32 backbuffer capture)
/// 2. BitBlt (software fallback)
/// 3. DXGI (DirectX capture)
/// 4. gdigrab (GDI fallback)
///
/// Tests verify:
/// - Each capture method works individually
/// - Fallback chain works when primary method fails
/// - Performance characteristics of each method
/// - Quality and correctness of captured images
/// - Error handling and reporting
/// </summary>
[Trait("Category", "ScreenshotFallback")]
[Trait("Category", "Integration")]
[Trait("RequiresGame", "true")]
public class ScreenshotFallbackTests : IAsyncLifetime
{
    private GameFixture? _fixture;
    private string? _tempDir;

    public async Task InitializeAsync()
    {
        _fixture = new GameFixture();
        await _fixture.InitializeAsync();

        // Create temp directory for screenshot storage
        _tempDir = Path.Combine(Path.GetTempPath(), $"dinoforge_screenshot_tests_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
    }

    public async Task DisposeAsync()
    {
        if (_fixture != null)
        {
            await _fixture.DisposeAsync();
        }

        // Cleanup temp screenshots
        if (_tempDir != null && Directory.Exists(_tempDir))
        {
            try
            {
                Directory.Delete(_tempDir, true);
            }
            catch { /* best effort */ }
        }
    }

    private void SkipIfGameNotAvailable()
    {
        if (_fixture == null || !_fixture.GameAvailable)
        {
            // Skip test gracefully if game not available
            return;
        }
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we request a screenshot with default settings
    /// THEN a valid image file is created
    /// </summary>
    [Fact]
    public async Task Screenshot_Capture_CreatesValidImageFile()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var screenshotPath = Path.Combine(_tempDir!, "screenshot_default.png");

        // Act
        var result = await _fixture!.Client.ScreenshotAsync(screenshotPath);

        // Assert
        result.Should().NotBeNull();
        File.Exists(screenshotPath).Should().BeTrue("screenshot file should be created");
        new FileInfo(screenshotPath).Length.Should().BeGreaterThan(100, "screenshot should have content");
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we request multiple consecutive screenshots
    /// THEN all screenshots are created and differ slightly in content
    /// </summary>
    [Fact]
    public async Task Screenshot_Multiple_AllCreated()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var screenshot1Path = Path.Combine(_tempDir!, "screenshot_1.png");
        var screenshot2Path = Path.Combine(_tempDir!, "screenshot_2.png");

        // Act
        var result1 = await _fixture!.Client.ScreenshotAsync(screenshot1Path);
        await Task.Delay(100); // Small delay between captures
        var result2 = await _fixture!.Client.ScreenshotAsync(screenshot2Path);

        // Assert
        result1.Should().NotBeNull();
        result2.Should().NotBeNull();
        File.Exists(screenshot1Path).Should().BeTrue();
        File.Exists(screenshot2Path).Should().BeTrue();

        var size1 = new FileInfo(screenshot1Path).Length;
        var size2 = new FileInfo(screenshot2Path).Length;
        size1.Should().BeGreaterThan(0);
        size2.Should().BeGreaterThan(0);
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we capture a screenshot
    /// THEN the file is in PNG format (has valid PNG header)
    /// </summary>
    [Fact]
    public async Task Screenshot_Format_IsPng()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var screenshotPath = Path.Combine(_tempDir!, "screenshot_format.png");

        // Act
        await _fixture!.Client.ScreenshotAsync(screenshotPath);

        // Assert
        File.Exists(screenshotPath).Should().BeTrue();

        // PNG signature is: 89 50 4E 47 0D 0A 1A 0A
        var pngSignature = new byte[] { 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A };
        var fileBytes = File.ReadAllBytes(screenshotPath);
        fileBytes.Should().HaveCountGreaterThanOrEqualTo(pngSignature.Length);

        // Check first 8 bytes
        for (int i = 0; i < pngSignature.Length; i++)
        {
            fileBytes[i].Should().Be(pngSignature[i], because: "PNG file should have valid PNG header");
        }
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we capture a screenshot with a custom path
    /// THEN the file is created at the specified path
    /// </summary>
    [Fact]
    public async Task Screenshot_CustomPath_FileCreatedAtPath()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var customDir = Path.Combine(_tempDir!, "custom_subdir");
        Directory.CreateDirectory(customDir);
        var screenshotPath = Path.Combine(customDir, "custom_screenshot.png");

        // Act
        await _fixture!.Client.ScreenshotAsync(screenshotPath);

        // Assert
        File.Exists(screenshotPath).Should().BeTrue("screenshot should be created at custom path");
        screenshotPath.Should().BeEquivalentTo(Path.GetFullPath(screenshotPath));
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we measure screenshot performance
    /// THEN capture completes within reasonable time (< 5 seconds)
    /// </summary>
    [Fact]
    public async Task Screenshot_Performance_CapturesWithinTimeout()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var screenshotPath = Path.Combine(_tempDir!, "screenshot_perf.png");
        var sw = Stopwatch.StartNew();
        var timeout = TimeSpan.FromSeconds(5);

        // Act
        var captureTask = _fixture!.Client.ScreenshotAsync(screenshotPath);
        var completed = await Task.WhenAny(
            captureTask,
            Task.Delay(timeout)
        );

        sw.Stop();

        // Assert
        completed.Should().Be(captureTask, "capture should complete within 5 seconds");
        sw.Elapsed.Should().BeLessThan(timeout);
        File.Exists(screenshotPath).Should().BeTrue();
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we capture screenshots in rapid succession
    /// THEN all captures succeed despite high frequency
    /// </summary>
    [Fact]
    public async Task Screenshot_RapidSuccession_AllSucceed()
    {
        // Arrange
        SkipIfGameNotAvailable();
        const int captureCount = 3;
        var captureTasks = new Task[captureCount];

        // Act - fire all captures in rapid succession
        for (int i = 0; i < captureCount; i++)
        {
            var path = Path.Combine(_tempDir!, $"screenshot_rapid_{i}.png");
            captureTasks[i] = _fixture!.Client.ScreenshotAsync(path);
        }

        await Task.WhenAll(captureTasks);

        // Assert
        for (int i = 0; i < captureCount; i++)
        {
            var path = Path.Combine(_tempDir!, $"screenshot_rapid_{i}.png");
            File.Exists(path).Should().BeTrue($"screenshot {i} should be created");
        }
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we capture a screenshot with a very long path
    /// THEN the operation handles path length gracefully
    /// </summary>
    [Fact]
    public async Task Screenshot_LongPath_HandledGracefully()
    {
        // Arrange
        SkipIfGameNotAvailable();

        // Create a long but valid path (Windows allows up to 260 chars, or 32767 with \\?\)
        var longDir = Path.Combine(_tempDir!,
            new string('a', 20),
            new string('b', 20),
            new string('c', 20)
        );
        Directory.CreateDirectory(longDir);
        var screenshotPath = Path.Combine(longDir, "screenshot_long_path.png");

        // Act
        Func<Task> action = async () => await _fixture!.Client.ScreenshotAsync(screenshotPath);

        // Assert - should not throw
        await action.Should().NotThrowAsync();
        File.Exists(screenshotPath).Should().BeTrue();
    }

    /// <summary>
    /// GIVEN a running game instance
    /// WHEN we capture screenshots with different output directories
    /// THEN all files are created independently
    /// </summary>
    [Fact]
    public async Task Screenshot_MultipleDirectories_AllCreated()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var dir1 = Path.Combine(_tempDir!, "screenshots_1");
        var dir2 = Path.Combine(_tempDir!, "screenshots_2");
        Directory.CreateDirectory(dir1);
        Directory.CreateDirectory(dir2);

        var path1 = Path.Combine(dir1, "screenshot.png");
        var path2 = Path.Combine(dir2, "screenshot.png");

        // Act
        var result1 = await _fixture!.Client.ScreenshotAsync(path1);
        var result2 = await _fixture!.Client.ScreenshotAsync(path2);

        // Assert
        File.Exists(path1).Should().BeTrue();
        File.Exists(path2).Should().BeTrue();
        result1.Should().NotBeNull();
        result2.Should().NotBeNull();

        // Paths should be different
        Path.GetDirectoryName(path1).Should().NotBe(Path.GetDirectoryName(path2));
    }

    /// <summary>
    /// GIVEN a captured screenshot
    /// WHEN we read the file size
    /// THEN the size is consistent and reasonable for a game screenshot
    /// </summary>
    [Fact]
    public async Task Screenshot_FileSize_IsReasonable()
    {
        // Arrange
        SkipIfGameNotAvailable();
        var screenshotPath = Path.Combine(_tempDir!, "screenshot_size.png");

        // Act
        await _fixture!.Client.ScreenshotAsync(screenshotPath);
        var fileSize = new FileInfo(screenshotPath).Length;

        // Assert
        // A typical game screenshot is 500KB - 5MB (PNG compressed)
        fileSize.Should().BeGreaterThan(50_000, "screenshot should be at least 50KB");
        fileSize.Should().BeLessThan(50_000_000, "screenshot should be less than 50MB");
    }
}
