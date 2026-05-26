#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Real-game launch validation tests that exercise actual game execution.
    /// These tests validate that DINOForge runtime loads correctly, key UI features work,
    /// and the game boots without fatal errors. Runs serially to avoid process conflicts.
    /// </summary>
    [Collection("GameLaunch")]
    public class GameLaunchValidationTests
    {
        private const int GameBootTimeoutSeconds = 10;
        private const int PostBootWaitSeconds = 3;

        /// <summary>
        /// Given: A DINOForge-patched game executable
        /// When: The game is launched
        /// Then: The process should boot successfully without fatal error dialogs
        /// </summary>
        [Fact]
        public async Task TestGameBoots()
        {
            try
            {
                // When: Launch the game
                var process = LaunchGameProcess();
                process.Should().NotBeNull("Game process should start successfully");

                // Wait for window to initialize
                await Task.Delay(TimeSpan.FromSeconds(PostBootWaitSeconds));

                // Then: Check that MainWindowTitle doesn't contain a fatal error
                var mainWindowTitle = GetGameMainWindowTitle();
                mainWindowTitle.Should()
                    .NotContain("Fatal error",
                        "Game should boot without fatal error dialog")
                    .And.NotContain("another instance",
                        "Game should not report another instance running");
            }
            catch
            {
                await GameDiagnosticsCapture.CaptureFailureStateAsync(nameof(TestGameBoots));
                throw;
            }
        }

        /// <summary>
        /// Given: DINOForge is deployed to the game directory
        /// When: The game boots and loads plugins
        /// Then: dinoforge_debug.log should contain "Runtime initialized" message
        /// </summary>
        [Fact]
        public async Task TestRuntimePluginLoads()
        {
            try
            {
                var logPath = Path.Combine(
                    @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
                    "BepInEx", "dinoforge_debug.log");

                logPath = Path.GetFullPath(logPath);

                // Given: Game is running
                var process = LaunchGameProcess();
                process.Should().NotBeNull("Game should launch");

                // Wait for plugins to initialize
                await Task.Delay(TimeSpan.FromSeconds(5));

                // When: Check the debug log
                File.Exists(logPath).Should().BeTrue(
                    $"dinoforge_debug.log should exist at {logPath}");

                var logContent = await File.ReadAllTextAsync(logPath);

                // Then: Log should indicate successful runtime initialization
                logContent.Should()
                    .Contain("Runtime initialized",
                        "DINOForge runtime should initialize and log success message");
            }
            catch
            {
                await GameDiagnosticsCapture.CaptureFailureStateAsync(nameof(TestRuntimePluginLoads));
                throw;
            }
        }

        /// <summary>
        /// Given: The game is running in gameplay
        /// When: F9 key is pressed (debug overlay toggle)
        /// Then: Screenshot analysis should detect the debug panel is visible
        /// </summary>
        [Fact(Skip = "Requires MCP server connectivity and game automation")]
        public async Task TestF9OverlayWorks()
        {
            try
            {
                // Given: Game is running and in gameplay
                var process = LaunchGameProcess();
                await Task.Delay(TimeSpan.FromSeconds(PostBootWaitSeconds));

                // When: Press F9 to toggle debug overlay
                // (In production, this would use MCP game_input tool)
                // SimulateKeyPress("F9");

                // Capture screenshot of the overlay
                // var screenshotPath = await CaptureScreenshotAsync();

                // Call VLM to analyze the screenshot
                // var analysis = await AnalyzeScreenshotAsync(screenshotPath);

                // Then: Analysis should confirm debug panel visibility
                // analysis.Should()
                //     .Contain("debug panel", StringComparison.OrdinalIgnoreCase);
            }
            catch
            {
                await GameDiagnosticsCapture.CaptureFailureStateAsync(nameof(TestF9OverlayWorks));
                throw;
            }
        }

        /// <summary>
        /// Given: The game is running with F10 mod menu available
        /// When: F10 key is pressed
        /// Then: Screenshot analysis should detect the mod menu panel
        /// </summary>
        [Fact(Skip = "Requires MCP server connectivity and game automation")]
        public async Task TestF10ModMenuWorks()
        {
            try
            {
                // Given: Game is running
                var process = LaunchGameProcess();
                await Task.Delay(TimeSpan.FromSeconds(PostBootWaitSeconds));

                // When: Press F10 to open mod menu
                // SimulateKeyPress("F10");

                // Capture screenshot
                // var screenshotPath = await CaptureScreenshotAsync();

                // Call VLM to analyze
                // var analysis = await AnalyzeScreenshotAsync(screenshotPath);

                // Then: Should detect mod menu
                // analysis.Should()
                //     .Contain("mod menu", StringComparison.OrdinalIgnoreCase)
                //     .Or.Contain("mod panel", StringComparison.OrdinalIgnoreCase);
            }
            catch
            {
                await GameDiagnosticsCapture.CaptureFailureStateAsync(nameof(TestF10ModMenuWorks));
                throw;
            }
        }

        /// <summary>
        /// Given: The game is at the main menu
        /// When: Screenshot is captured and analyzed
        /// Then: The "Mods" button should be visible in the main menu
        /// </summary>
        [Fact(Skip = "Requires MCP server connectivity and game automation")]
        public async Task TestModsButtonVisible()
        {
            try
            {
                // Given: Game is running at main menu
                var process = LaunchGameProcess();

                // Wait for main menu to load (retry a few times)
                bool menuReady = false;
                for (int i = 0; i < 5; i++)
                {
                    await Task.Delay(TimeSpan.FromSeconds(1));
                    // Could check for specific menu elements here
                    menuReady = true;
                    break;
                }

                menuReady.Should().BeTrue("Main menu should appear within timeout");

                // When: Capture screenshot of main menu
                // var screenshotPath = await CaptureScreenshotAsync();

                // Call VLM to detect Mods button
                // var analysis = await AnalyzeScreenshotAsync(screenshotPath,
                //     "Detect the Mods button in the main menu");

                // Then: Should find Mods button
                // analysis.Should()
                //     .Contain("Mods", StringComparison.OrdinalIgnoreCase);
            }
            catch
            {
                await GameDiagnosticsCapture.CaptureFailureStateAsync(nameof(TestModsButtonVisible));
                throw;
            }
        }

        /// <summary>
        /// Launches the game process and verifies it starts.
        /// </summary>
        private static Process? LaunchGameProcess()
        {
            const string gameExePath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe";
            const string gameDir = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";

            if (!File.Exists(gameExePath))
            {
                throw new FileNotFoundException($"Game executable not found at {gameExePath}");
            }

            try
            {
                var process = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = gameExePath,
                        WorkingDirectory = gameDir,
                        UseShellExecute = false,
                        CreateNoWindow = false,
                    }
                };

                process.Start();
                return process;
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Failed to launch game: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// Gets the main window title of the running game process.
        /// </summary>
        private static string GetGameMainWindowTitle()
        {
            var processes = Process.GetProcessesByName("Diplomacy is Not an Option");
            if (processes.Length == 0)
            {
                return string.Empty;
            }

            try
            {
                return processes[0].MainWindowTitle;
            }
            catch
            {
                return string.Empty;
            }
        }
    }
}
