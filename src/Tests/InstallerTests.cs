using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Tools.Installer;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for the DINOForge installer library (SteamLocator, InstallVerifier).
    /// </summary>
    public class InstallerTests
    {
        // ──────────────────────── SteamLocator VDF parsing ────────────────────────

        [Fact]
        public void ParseLibraryFoldersVdf_SingleLibrary_ExtractsPath()
        {
            string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\Program Files (x86)\\Steam""
        ""label""		""""
    }
}";
            IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

            paths.Should().HaveCount(1);
            paths[0].Should().Be(@"C:\Program Files (x86)\Steam");
        }

        [Fact]
        public void ParseLibraryFoldersVdf_MultipleLibraries_ExtractsAllPaths()
        {
            string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\Program Files (x86)\\Steam""
    }
    ""1""
    {
        ""path""		""D:\\SteamLibrary""
    }
    ""2""
    {
        ""path""		""E:\\Games\\Steam""
    }
}";
            IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

            paths.Should().HaveCount(3);
            paths[0].Should().Be(@"C:\Program Files (x86)\Steam");
            paths[1].Should().Be(@"D:\SteamLibrary");
            paths[2].Should().Be(@"E:\Games\Steam");
        }

        [Fact]
        public void ParseLibraryFoldersVdf_EmptyContent_ReturnsEmptyList()
        {
            IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf("");

            paths.Should().BeEmpty();
        }

        [Fact]
        public void ParseLibraryFoldersVdf_NoPathKeys_ReturnsEmptyList()
        {
            string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""label""		""main""
    }
}";
            IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

            paths.Should().BeEmpty();
        }

        [Fact]
        public void FindGameInLibrary_NonexistentLibrary_ReturnsNull()
        {
            string result = SteamLocator.FindGameInLibrary(
                Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString()),
                SteamLocator.DinoAppId)!;

            result.Should().BeNull();
        }

        // ──────────────────────── InstallVerifier ────────────────────────

        [Fact]
        public void Verify_NullPath_ReturnsIssues()
        {
            InstallStatus status = InstallVerifier.Verify(null!);

            status.IsFullyInstalled.Should().BeFalse();
            status.Issues.Should().Contain(i => i.Contains("null or empty"));
        }

        [Fact]
        public void Verify_NonexistentPath_ReturnsIssues()
        {
            string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

            InstallStatus status = InstallVerifier.Verify(fakePath);

            status.IsFullyInstalled.Should().BeFalse();
            status.Issues.Should().Contain(i => i.Contains("does not exist"));
        }

        [Fact]
        public void Verify_EmptyDirectory_ReportsAllMissing()
        {
            string tempDir = Path.Combine(Path.GetTempPath(), $"dinoforge_test_{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                InstallStatus status = InstallVerifier.Verify(tempDir);

                status.IsFullyInstalled.Should().BeFalse();
                status.GameExists.Should().BeFalse();
                status.BepInExInstalled.Should().BeFalse();
                status.RuntimeInstalled.Should().BeFalse();
                status.PacksReady.Should().BeFalse();
                status.Issues.Should().HaveCountGreaterThan(0);
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Verify_FullInstall_ReportsAllGood()
        {
            string tempDir = Path.Combine(Path.GetTempPath(), $"dinoforge_test_{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                // Create all expected files/directories
                File.WriteAllText(Path.Combine(tempDir, "Diplomacy is Not an Option.exe"), "");
                File.WriteAllText(Path.Combine(tempDir, "winhttp.dll"), "");
                File.WriteAllText(Path.Combine(tempDir, "doorstop_config.ini"), "");
                Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "core"));
                Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "plugins"));
                File.WriteAllText(Path.Combine(tempDir, "BepInEx", "plugins", "DINOForge.Runtime.dll"), "");
                Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "dinoforge_packs"));

                InstallStatus status = InstallVerifier.Verify(tempDir);

                status.IsFullyInstalled.Should().BeTrue();
                status.GameExists.Should().BeTrue();
                status.BepInExInstalled.Should().BeTrue();
                status.RuntimeInstalled.Should().BeTrue();
                status.PacksReady.Should().BeTrue();
                status.Issues.Should().BeEmpty();
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Verify_MissingBepInExOnly_ReportsPartialInstall()
        {
            string tempDir = Path.Combine(Path.GetTempPath(), $"dinoforge_test_{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                File.WriteAllText(Path.Combine(tempDir, "Diplomacy is Not an Option.exe"), "");
                Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "dinoforge_packs"));

                InstallStatus status = InstallVerifier.Verify(tempDir);

                status.GameExists.Should().BeTrue();
                status.BepInExInstalled.Should().BeFalse();
                status.PacksReady.Should().BeTrue();
                status.IsFullyInstalled.Should().BeFalse();
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void DinoAppId_IsCorrect()
        {
            SteamLocator.DinoAppId.Should().Be(1272320);
        }
    }
}
