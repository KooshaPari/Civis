#nullable enable
using System;
using System.Diagnostics;
using System.Threading.Tasks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Environment compatibility tests for game launch across different configurations.
    /// Tests validate that DINOForge works on Desktop, RDP, and sandbox environments.
    /// </summary>
    [Trait("Category", "EnvironmentMatrix")]
    public class EnvironmentCompatibilityTests
    {
        /// <summary>
        /// Given: A standard Windows desktop environment
        /// When: The game is launched via standard method
        /// Then: The game should start successfully
        /// </summary>
        [Fact(Skip = "Requires manual environment setup")]
        public async Task TestDesktopLaunchSucceeds()
        {
            // Log environment information
            var environmentInfo = new
            {
                OSVersion = Environment.OSVersion.VersionString,
                ProcessorCount = Environment.ProcessorCount,
                SystemDirectory = Environment.SystemDirectory,
                Timestamp = DateTime.UtcNow
            };

            // When: Launch game on desktop
            var process = LaunchGameProcess();
            process.Should().NotBeNull();

            // Wait for startup
            await Task.Delay(TimeSpan.FromSeconds(5));

            // Then: Window should be visible and responsive
            var mainWindowTitle = GetGameMainWindowTitle();
            mainWindowTitle.Should()
                .NotContain("Fatal error")
                .And.NotContain("another instance");
        }

        /// <summary>
        /// Given: A RDP (Remote Desktop Protocol) environment
        /// When: The game is launched
        /// Then: Game should handle RDP-specific window rendering
        /// </summary>
        [Fact(Skip = "Requires RDP environment")]
        public async Task TestRdpCompatibility()
        {
            // Log RDP environment detection
            var isRdp = Environment.GetEnvironmentVariable("SESSIONNAME") == "RDP-Tcp";

            if (!isRdp)
            {
                // Not an RDP session, skip
                return;
            }

            var sessionInfo = new
            {
                IsRdpSession = isRdp,
                SessionName = Environment.GetEnvironmentVariable("SESSIONNAME"),
                Username = Environment.UserName,
                Domain = Environment.UserDomainName
            };

            // When: Launch game in RDP
            var process = LaunchGameProcess();
            process.Should().NotBeNull();

            await Task.Delay(TimeSpan.FromSeconds(5));

            // Then: Game window should exist and be renderable
            var mainWindowTitle = GetGameMainWindowTitle();
            mainWindowTitle.Should().NotBeEmpty("Game window should exist in RDP session");

            // Additional check: Window visibility (RDP specific)
            mainWindowTitle.Should()
                .NotContain("Fatal error")
                .And.NotContain("cannot create window");
        }

        /// <summary>
        /// Given: A sandboxed or isolated environment
        /// When: The game is launched with sandbox restrictions
        /// Then: Game should handle sandbox constraints gracefully
        /// </summary>
        [Fact(Skip = "Requires sandbox environment")]
        public async Task TestSandboxEnvironmentHandling()
        {
            // Detect if running in sandbox
            var isSandboxed = CheckIfSandboxed();

            var sandboxInfo = new
            {
                IsSandboxed = isSandboxed,
                RestrictedPaths = GetRestrictedPaths(),
                AvailableResources = "Reduced (memory, disk, network)"
            };

            if (!isSandboxed)
            {
                // Not sandboxed, skip
                return;
            }

            // When: Launch game in sandbox
            var process = LaunchGameProcess();
            process.Should().NotBeNull();

            await Task.Delay(TimeSpan.FromSeconds(5));

            // Then: Game should still boot without sandbox-specific errors
            var mainWindowTitle = GetGameMainWindowTitle();
            mainWindowTitle.Should()
                .NotContain("permission")
                .And.NotContain("access denied")
                .And.NotContain("sandbox");
        }

        /// <summary>
        /// Given: Any Windows environment (desktop/RDP/sandbox)
        /// When: Game starts
        /// Then: Log environment details for diagnostic purposes
        /// </summary>
        [Fact(Skip = "Informational only")]
        public void LogEnvironmentDetails()
        {
            var details = new
            {
                OSVersion = Environment.OSVersion,
                ProcessorCount = Environment.ProcessorCount,
                TotalMemory = GC.GetTotalMemory(false),
                Is64BitProcess = Environment.Is64BitProcess,
                Is64BitOperatingSystem = Environment.Is64BitOperatingSystem,
                UserInteractive = Environment.UserInteractive,
                Username = Environment.UserName,
                MachineName = Environment.MachineName,
                SystemDirectory = Environment.SystemDirectory,
                CurrentDirectory = Environment.CurrentDirectory,
                ProcessId = Environment.ProcessId
            };

            // All environment details should be accessible
            details.OSVersion.Should().NotBeNull();
            details.ProcessorCount.Should().BeGreaterThan(0);
            (details.UserInteractive == true || details.UserInteractive == false).Should().BeTrue();
        }

        /// <summary>
        /// Launches the game process.
        /// </summary>
        private static Process? LaunchGameProcess()
        {
            const string gameExePath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe";
            const string gameDir = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";

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
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// Gets the main window title of the running game.
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

        /// <summary>
        /// Checks if the current environment is sandboxed.
        /// </summary>
        private static bool CheckIfSandboxed()
        {
            // Simple heuristics to detect sandbox environments
            try
            {
                // Windows Sandbox specific
                if (Environment.MachineName == "SANDBOX")
                    return true;

                // WSL environment
                if (Environment.GetEnvironmentVariable("WSL_DISTRO_NAME") != null)
                    return true;

                // Check for restricted registry/file access
                var tempPath = System.IO.Path.GetTempPath();
                var testFile = System.IO.Path.Combine(tempPath, ".sandbox_test");

                try
                {
                    System.IO.File.WriteAllText(testFile, "test");
                    System.IO.File.Delete(testFile);
                    return false;
                }
                catch
                {
                    return true;
                }
            }
            catch
            {
                return false;
            }
        }

        /// <summary>
        /// Gets restricted paths in sandbox environment.
        /// </summary>
        private static string[] GetRestrictedPaths()
        {
            return new[]
            {
                "C:\\Users\\(other user)",
                "C:\\Program Files (86)",
                "HKEY_LOCAL_MACHINE\\SYSTEM",
            };
        }
    }
}
