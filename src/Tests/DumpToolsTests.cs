using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Integration tests for DumpTools command-line tool.
    /// Exercises the parse path for dump files by running against a synthetic fixture.
    /// </summary>
    public class DumpToolsTests
    {
        private string GetFixturePath()
        {
            string repoRoot = GetRepoRoot();
            return Path.Combine(repoRoot, "src", "Tests", "Fixtures", "sample-dump");
        }

        private static string GetRepoRoot()
        {
            // global.json is at the true repo root (C:\Users\koosh\Dino), not src/
            string? current = Path.GetDirectoryName(typeof(DumpToolsTests).Assembly.Location);
            while (current != null)
            {
                if (File.Exists(Path.Combine(current, "global.json")))
                {
                    return current;
                }
                current = Path.GetDirectoryName(current);
            }

            // Fallback to current directory
            current = Directory.GetCurrentDirectory();
            while (current != null)
            {
                if (File.Exists(Path.Combine(current, "global.json")))
                {
                    return current;
                }
                current = Path.GetDirectoryName(current);
            }

            throw new InvalidOperationException("Cannot find repo root (global.json)");
        }

        private (int exitCode, string stdout) RunDumpToolsCommand(string command, string? dumpDir = null)
        {
            string repoRoot = GetRepoRoot();
            string args = string.IsNullOrEmpty(dumpDir)
                ? command
                : $"{command} \"{dumpDir}\"";

            var psi = new ProcessStartInfo
            {
                FileName = "dotnet",
                Arguments = $"run --project src/Tools/DumpTools -- {args}",
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                WorkingDirectory = repoRoot,
                CreateNoWindow = true
            };

            using var proc = Process.Start(psi);
            if (proc == null)
                throw new InvalidOperationException("Failed to start DumpTools process");

            // Read streams while the process runs to avoid pipe buffer deadlock.
            var stdoutTask = proc.StandardOutput.ReadToEndAsync();
            var stderrTask = proc.StandardError.ReadToEndAsync();

            bool exited = proc.WaitForExit(60_000);
            exited.Should().BeTrue("DumpTools process should exit within 60 seconds");

            Task.WaitAll(stdoutTask, stderrTask);
            string stdout = stdoutTask.Result;
            _ = stderrTask.Result;

            return (proc.ExitCode, stdout);
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void AnalyzeDump_WithValidFixture_ReturnsSuccessAndParsesWorlds()
        {
            // Arrange
            string fixturePath = GetFixturePath();
            Directory.Exists(fixturePath).Should().BeTrue("Fixture directory should exist");
            File.Exists(Path.Combine(fixturePath, "worlds.json")).Should().BeTrue();

            // Act
            var (exitCode, stdout) = RunDumpToolsCommand("analyze", fixturePath);

            // Assert
            exitCode.Should().Be(0, "DumpTools analyze should exit successfully");
            stdout.Should().NotBeNullOrEmpty("DumpTools should produce output");
            stdout.Should().Contain("Default World", "Output should contain parsed world name");
            stdout.Should().Contain("entities", "Output should reference entity counts");
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void ShowComponents_WithValidFixture_ReturnsSuccessAndListsComponentCounts()
        {
            // Arrange
            string fixturePath = GetFixturePath();
            File.Exists(Path.Combine(fixturePath, "ecs_types.json")).Should().BeTrue();

            // Act
            var (exitCode, stdout) = RunDumpToolsCommand("components", fixturePath);

            // Assert
            exitCode.Should().Be(0, "DumpTools components should exit successfully");
            stdout.Should().NotBeNullOrEmpty("DumpTools should produce output");
            stdout.Should().Contain("Components", "Output should reference component types");
            stdout.Should().Match("*5*", "Output should show component count (fixture has 5 components)");
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void ShowSystems_WithValidFixture_ReturnsSuccessAndListsSystems()
        {
            // Arrange
            string fixturePath = GetFixturePath();
            File.Exists(Path.Combine(fixturePath, "systems_DefaultWorld.json")).Should().BeTrue();

            // Act
            var (exitCode, stdout) = RunDumpToolsCommand("systems", fixturePath);

            // Assert
            exitCode.Should().Be(0, "DumpTools systems should exit successfully");
            stdout.Should().NotBeNullOrEmpty("DumpTools should produce output");
            stdout.Should().Contain("DefaultWorld", "Output should reference world name");
            stdout.Should().Contain("System", "Output should reference systems");
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void ShowNamespaces_WithValidFixture_ReturnsSuccessAndDisplaysNamespaceTree()
        {
            // Arrange
            string fixturePath = GetFixturePath();
            File.Exists(Path.Combine(fixturePath, "game_namespaces.json")).Should().BeTrue();

            // Act
            var (exitCode, stdout) = RunDumpToolsCommand("namespaces", fixturePath);

            // Assert
            exitCode.Should().Be(0, "DumpTools namespaces should exit successfully");
            stdout.Should().NotBeNullOrEmpty("DumpTools should produce output");
            stdout.Should().Contain("DNO.Main", "Output should display parsed assemblies");
            stdout.Should().Contain("Components", "Output should display parsed namespaces");
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void AnalyzeDump_WithMissingFile_ReturnsErrorCode()
        {
            // Arrange
            string nonexistentPath = Path.Combine(Path.GetTempPath(), "nonexistent-dump-" + Guid.NewGuid());

            // Act
            var (exitCode, _) = RunDumpToolsCommand("analyze", nonexistentPath);

            // Assert
            exitCode.Should().NotBe(0, "DumpTools should fail when dump directory doesn't exist");
        }

        [Fact]
        [Trait("Category", "Tool")]
        public void ShowComponents_WithMissingEcsTypes_ReturnsErrorCode()
        {
            // Arrange - create temp dir without ecs_types.json
            string tempDir = Path.Combine(Path.GetTempPath(), "bad-dump-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);
            try
            {
                // Act
                var (exitCode, _) = RunDumpToolsCommand("components", tempDir);

                // Assert
                exitCode.Should().NotBe(0, "DumpTools should fail when ecs_types.json is missing");
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }
    }
}
