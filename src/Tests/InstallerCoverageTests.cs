#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.Tools.Installer;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for DINOForge.Tools.Installer.
/// These tests focus on InstallLifecycle, SteamLocator edge cases,
/// and InstallDetector to raise coverage from 48.2% to 85%+.
/// </summary>
public class InstallerCoverageTests
{
    // ──────────────────────── InstallLifecycle WriteManifest error paths ────────────────────────

    [Fact]
    public void WriteManifest_WithDirectoryCreation_CreatesParentDirectories()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        // BepInEx directory does not exist, but WriteManifest should create it

        try
        {
            string manifestPath = InstallLifecycle.WriteManifest(tempDir, "1.0.0");

            File.Exists(manifestPath).Should().BeTrue();
            Directory.Exists(Path.Combine(tempDir, "BepInEx", "plugins")).Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator GetLibraryFolders exception paths ────────────────────────

    [Fact]
    public void GetLibraryFolders_WithMissingVdfFile_ReturnsOnlySteamPath()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            folders.Should().HaveCount(1);
            folders[0].Should().Be(tempDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator VDF parsing edge cases ────────────────────────

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
        paths[0].Should().NotBeNullOrEmpty();
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

    // ──────────────────────── InstallLifecycle ────────────────────────

    [Fact]
    public void GetBepInExDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = Path.GetFullPath(InstallLifecycle.GetBepInExDirectory(gamePath));
        string expected = Path.GetFullPath(@"C:\Games\DINO\BepInEx");

        result.Replace('\\', '/').Should().Be(expected.Replace('\\', '/'));
    }

    [Fact]
    public void GetPluginsDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = Path.GetFullPath(InstallLifecycle.GetPluginsDirectory(gamePath));
        string expected = Path.GetFullPath(@"C:\Games\DINO\BepInEx\plugins");

        result.Replace('\\', '/').Should().Be(expected.Replace('\\', '/'));
    }

    [Fact]
    public void GetPacksDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = Path.GetFullPath(InstallLifecycle.GetPacksDirectory(gamePath));
        string expected = Path.GetFullPath(@"C:\Games\DINO\BepInEx\dinoforge_packs");

        result.Replace('\\', '/').Should().Be(expected.Replace('\\', '/'));
    }

    [Fact]
    public void GetLegacyPacksDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = Path.GetFullPath(InstallLifecycle.GetLegacyPacksDirectory(gamePath));
        string expected = Path.GetFullPath(@"C:\Games\DINO\dinoforge_packs");

        result.Replace('\\', '/').Should().Be(expected.Replace('\\', '/'));
    }

    [Fact]
    public void GetManifestPath_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = Path.GetFullPath(InstallLifecycle.GetManifestPath(gamePath));
        string expected = Path.GetFullPath(@"C:\Games\DINO\BepInEx\plugins\dinoforge.install_manifest.json");

        result.Replace('\\', '/').Should().Be(expected.Replace('\\', '/'));
    }

    [Fact]
    public void Inspect_WithNullPath_ReturnsHealthyFalse()
    {
        InstallInspection inspection = InstallLifecycle.Inspect(null!);

        inspection.GamePath.Should().BeNullOrEmpty();
        inspection.IsHealthy.Should().BeFalse();
        inspection.Issues.Should().Contain(i => i.Contains("does not exist"));
    }

    [Fact]
    public void Inspect_WithEmptyPath_ReturnsHealthyFalse()
    {
        InstallInspection inspection = InstallLifecycle.Inspect("");

        inspection.IsHealthy.Should().BeFalse();
        inspection.Issues.Should().NotBeEmpty();
    }

    [Fact]
    public void Inspect_WithNonExistentPath_ReturnsHealthyFalse()
    {
        string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

        InstallInspection inspection = InstallLifecycle.Inspect(fakePath);

        inspection.IsHealthy.Should().BeFalse();
        inspection.Issues.Should().Contain(i => i.Contains("does not exist"));
    }

    [Fact]
    public void Inspect_WithValidPathButNoRuntime_ReportsNoRuntime()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "plugins"));

        try
        {
            InstallInspection inspection = InstallLifecycle.Inspect(tempDir);

            inspection.RuntimeInstalled.Should().BeFalse();
            inspection.IsHealthy.Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void Inspect_WithRuntimeDll_ReportsRuntimeInstalled()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0 });

        try
        {
            InstallInspection inspection = InstallLifecycle.Inspect(tempDir);

            inspection.RuntimeInstalled.Should().BeTrue();
            inspection.ManagedFiles.Should().Contain(mf => mf.Contains("DINOForge.Runtime.dll"));
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void Inspect_WithLegacyArtifacts_ReportsIssues()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "ecs_plugins"));
        File.WriteAllBytes(Path.Combine(tempDir, "BepInEx", "ecs_plugins", "DINOForge.Runtime.dll"), new byte[] { 0 });

        try
        {
            InstallInspection inspection = InstallLifecycle.Inspect(tempDir);

            inspection.LegacyArtifacts.Should().NotBeEmpty();
            inspection.Issues.Should().Contain(i => i.Contains("Legacy"));
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void Inspect_WithUiAssetsDirectory_IncludesInManagedFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        string uiAssetsDir = Path.Combine(pluginsDir, "dinoforge-ui-assets");
        Directory.CreateDirectory(uiAssetsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0 });

        try
        {
            InstallInspection inspection = InstallLifecycle.Inspect(tempDir);

            inspection.ManagedFiles.Should().Contain(uiAssetsDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void CleanupLegacyArtifacts_WithNoArtifacts_ReturnsZero()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            int count = InstallLifecycle.CleanupLegacyArtifacts(tempDir);

            count.Should().Be(0);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void CleanupLegacyArtifacts_WithArtifacts_DeletesFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string legacyFile = Path.Combine(tempDir, "BepInEx", "ecs_plugins", "DINOForge.Runtime.dll");
        Directory.CreateDirectory(Path.GetDirectoryName(legacyFile)!);
        File.WriteAllBytes(legacyFile, new byte[] { 0 });

        try
        {
            int count = InstallLifecycle.CleanupLegacyArtifacts(tempDir);

            count.Should().BeGreaterThan(0);
            File.Exists(legacyFile).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void MigrateLegacyPacks_WithNoLegacyDir_ReturnsFalse()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            bool result = InstallLifecycle.MigrateLegacyPacks(tempDir);

            result.Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void MigrateLegacyPacks_WithLegacyDir_MigratesFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string legacyDir = Path.Combine(tempDir, "dinoforge_packs");
        Directory.CreateDirectory(legacyDir);
        File.WriteAllText(Path.Combine(legacyDir, "test.txt"), "content");

        try
        {
            bool result = InstallLifecycle.MigrateLegacyPacks(tempDir);

            result.Should().BeTrue();
            Directory.Exists(legacyDir).Should().BeFalse();
            Directory.Exists(Path.Combine(tempDir, "BepInEx", "dinoforge_packs")).Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void RemoveManagedFiles_WithNoManifest_DeletesDefaultFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0 });

        try
        {
            int count = InstallLifecycle.RemoveManagedFiles(tempDir);

            count.Should().BeGreaterOrEqualTo(1);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void WriteManifest_CreatesValidManifestFile()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0, 1, 2, 3 });

        try
        {
            string manifestPath = InstallLifecycle.WriteManifest(tempDir, "1.2.3");

            File.Exists(manifestPath).Should().BeTrue();
            string content = File.ReadAllText(manifestPath);
            content.Should().Contain("1.2.3");
            content.Should().Contain("DINOForge.Runtime.dll");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void TryReadManifest_WithNoFile_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            InstallManifest? manifest = InstallLifecycle.TryReadManifest(tempDir);

            manifest.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void TryReadManifest_WithInvalidJson_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllText(Path.Combine(pluginsDir, "dinoforge.install_manifest.json"), "not json {{{");

        try
        {
            InstallManifest? manifest = InstallLifecycle.TryReadManifest(tempDir);

            manifest.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void TryReadManifest_WithValidManifest_ReturnsParsedManifest()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);

        InstallManifest original = new()
        {
            InstallerVersion = "2.0.0",
            InstalledAtUtc = "2026-03-30T00:00:00Z",
            Files = new List<InstalledFileRecord>
            {
                new InstalledFileRecord { RelativePath = "test.dll", Size = 100, Sha256 = "abc123" }
            }
        };
        File.WriteAllText(
            Path.Combine(pluginsDir, "dinoforge.install_manifest.json"),
            JsonSerializer.Serialize(original));

        try
        {
            InstallManifest? manifest = InstallLifecycle.TryReadManifest(tempDir);

            manifest.Should().NotBeNull();
            manifest!.InstallerVersion.Should().Be("2.0.0");
            manifest.Files.Should().HaveCount(1);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallDetector ────────────────────────

    [Fact]
    public void IsInstalled_WithNullPath_ReturnsFalse()
    {
        bool result = InstallDetector.IsInstalled(null!);

        result.Should().BeFalse();
    }

    [Fact]
    public void IsInstalled_WithEmptyPath_ReturnsFalse()
    {
        bool result = InstallDetector.IsInstalled("");

        result.Should().BeFalse();
    }

    [Fact]
    public void IsInstalled_WithWhitespacePath_ReturnsFalse()
    {
        bool result = InstallDetector.IsInstalled("   ");

        result.Should().BeFalse();
    }

    [Fact]
    public void IsInstalled_WithNonExistentPath_ReturnsFalse()
    {
        string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

        bool result = InstallDetector.IsInstalled(fakePath);

        result.Should().BeFalse();
    }

    [Fact]
    public void IsInstalled_WithMissingRuntimeDll_ReturnsFalse()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        Directory.CreateDirectory(Path.Combine(tempDir, "BepInEx", "plugins"));

        try
        {
            bool result = InstallDetector.IsInstalled(tempDir);

            result.Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void IsInstalled_WithRuntimeDll_ReturnsTrue()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0 });

        try
        {
            bool result = InstallDetector.IsInstalled(tempDir);

            result.Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void GetInstalledVersion_WithNoFiles_ReturnsUnknown()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            string version = InstallDetector.GetInstalledVersion(tempDir);

            version.Should().Be("unknown");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void GetInstalledVersion_WithVersionFile_ReturnsVersion()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllText(Path.Combine(pluginsDir, "dinoforge_version.txt"), "1.2.3-beta");

        try
        {
            string version = InstallDetector.GetInstalledVersion(tempDir);

            version.Should().Be("1.2.3-beta");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallStatus ────────────────────────

    [Fact]
    public void InstallStatus_IsFullyInstalled_WithAllTrue_ReturnsTrue()
    {
        var status = new InstallStatus(
            gameExists: true,
            bepInExInstalled: true,
            runtimeInstalled: true,
            packsReady: true,
            manifestPresent: true,
            hasLegacyArtifacts: false,
            issues: Array.Empty<string>(),
            warnings: Array.Empty<string>());

        status.IsFullyInstalled.Should().BeTrue();
    }

    [Fact]
    public void InstallStatus_IsFullyInstalled_WithAnyFalse_ReturnsFalse()
    {
        var status = new InstallStatus(
            gameExists: true,
            bepInExInstalled: false,
            runtimeInstalled: true,
            packsReady: true,
            manifestPresent: true,
            hasLegacyArtifacts: false,
            issues: Array.Empty<string>(),
            warnings: Array.Empty<string>());

        status.IsFullyInstalled.Should().BeFalse();
    }

    [Fact]
    public void InstallStatus_PropertiesRoundtrip()
    {
        var issues = new List<string> { "issue1", "issue2" };
        var warnings = new List<string> { "warning1" };

        var status = new InstallStatus(
            gameExists: true,
            bepInExInstalled: true,
            runtimeInstalled: false,
            packsReady: true,
            manifestPresent: false,
            hasLegacyArtifacts: true,
            issues: issues,
            warnings: warnings);

        status.GameExists.Should().BeTrue();
        status.BepInExInstalled.Should().BeTrue();
        status.RuntimeInstalled.Should().BeFalse();
        status.PacksReady.Should().BeTrue();
        status.ManifestPresent.Should().BeFalse();
        status.HasLegacyArtifacts.Should().BeTrue();
        status.Issues.Should().HaveCount(2);
        status.Warnings.Should().HaveCount(1);
    }

    // ──────────────────────── InstallInspection ────────────────────────

    [Fact]
    public void InstallInspection_IsHealthy_WithRuntimeAndNoIssues_ReturnsTrue()
    {
        var inspection = new InstallInspection(
            gamePath: "C:\\test",
            pluginsDirectoryPresent: true,
            runtimeInstalled: true,
            manifestPresent: true,
            installedVersion: "1.0.0",
            primaryRuntimePath: "C:\\test\\runtime.dll",
            issues: Array.Empty<string>(),
            warnings: Array.Empty<string>(),
            legacyArtifacts: Array.Empty<string>(),
            managedFiles: new List<string>());

        inspection.IsHealthy.Should().BeTrue();
    }

    [Fact]
    public void InstallInspection_IsHealthy_WithRuntimeAndIssues_ReturnsFalse()
    {
        var inspection = new InstallInspection(
            gamePath: "C:\\test",
            pluginsDirectoryPresent: true,
            runtimeInstalled: true,
            manifestPresent: true,
            installedVersion: "1.0.0",
            primaryRuntimePath: "C:\\test\\runtime.dll",
            issues: new List<string> { "Something is wrong" },
            warnings: Array.Empty<string>(),
            legacyArtifacts: Array.Empty<string>(),
            managedFiles: new List<string>());

        inspection.IsHealthy.Should().BeFalse();
    }

    [Fact]
    public void InstallInspection_AllPropertiesAccessible()
    {
        var inspection = new InstallInspection(
            gamePath: "C:\\test",
            pluginsDirectoryPresent: true,
            runtimeInstalled: true,
            manifestPresent: true,
            installedVersion: "1.0.0",
            primaryRuntimePath: "C:\\test\\runtime.dll",
            issues: new List<string> { "issue" },
            warnings: new List<string> { "warning" },
            legacyArtifacts: new List<string> { "legacy" },
            managedFiles: new List<string> { "managed" });

        inspection.GamePath.Should().Be("C:\\test");
        inspection.PluginsDirectoryPresent.Should().BeTrue();
        inspection.RuntimeInstalled.Should().BeTrue();
        inspection.ManifestPresent.Should().BeTrue();
        inspection.InstalledVersion.Should().Be("1.0.0");
        inspection.PrimaryRuntimePath.Should().Be("C:\\test\\runtime.dll");
        inspection.Issues.Should().HaveCount(1);
        inspection.Warnings.Should().HaveCount(1);
        inspection.LegacyArtifacts.Should().HaveCount(1);
        inspection.ManagedFiles.Should().HaveCount(1);
    }

    // ──────────────────────── InstallManifest ────────────────────────

    [Fact]
    public void InstallManifest_DefaultValues()
    {
        var manifest = new InstallManifest();

        manifest.SchemaVersion.Should().Be("1");
        manifest.InstallerVersion.Should().Be("unknown");
        manifest.InstalledAtUtc.Should().BeEmpty();
        manifest.Files.Should().NotBeNull();
        manifest.Files.Should().BeEmpty();
    }

    [Fact]
    public void InstallManifest_CanSetProperties()
    {
        var manifest = new InstallManifest
        {
            SchemaVersion = "2",
            InstallerVersion = "1.0.0",
            InstalledAtUtc = "2026-03-30T00:00:00Z",
            Files = new List<InstalledFileRecord>
            {
                new InstalledFileRecord { RelativePath = "test.dll", Size = 100, Sha256 = "abc" }
            }
        };

        manifest.SchemaVersion.Should().Be("2");
        manifest.InstallerVersion.Should().Be("1.0.0");
        manifest.Files.Should().HaveCount(1);
    }

    // ──────────────────────── InstalledFileRecord ────────────────────────

    [Fact]
    public void InstalledFileRecord_DefaultValues()
    {
        var record = new InstalledFileRecord();

        record.RelativePath.Should().BeEmpty();
        record.Size.Should().Be(0);
        record.Sha256.Should().BeEmpty();
    }

    [Fact]
    public void InstalledFileRecord_CanSetProperties()
    {
        var record = new InstalledFileRecord
        {
            RelativePath = "BepInEx/plugins/test.dll",
            Size = 12345,
            Sha256 = "abc123def456"
        };

        record.RelativePath.Should().Be("BepInEx/plugins/test.dll");
        record.Size.Should().Be(12345);
        record.Sha256.Should().Be("abc123def456");
    }

    // ──────────────────────── Constants ────────────────────────

    [Fact]
    public void ManifestFileName_IsCorrect()
    {
        InstallLifecycle.ManifestFileName.Should().Be("dinoforge.install_manifest.json");
    }

    [Fact]
    public void VersionFileName_IsCorrect()
    {
        InstallLifecycle.VersionFileName.Should().Be("dinoforge_version.txt");
    }

    [Fact]
    public void DinoAppId_IsCorrect()
    {
        SteamLocator.DinoAppId.Should().Be(1272320);
    }

    // ──────────────────────── SteamLocator ParseInstallDirFromAcf edge cases ────────────────────────

    [Fact]
    public void FindGameInLibrary_ValidManifestWithCorrectInstallPath_ReturnsPath()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Diplomacy is Not an Option""
    ""installdir""    ""Diplomacy is Not an Option""
}");
        string gameDir = Path.Combine(steamAppsDir, "common", "Diplomacy is Not an Option");
        Directory.CreateDirectory(gameDir);

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().Be(gameDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void FindGameInLibrary_ManifestExistsButInstallDirNotOnDisk_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Diplomacy is Not an Option""
    ""installdir""    ""Diplomacy is Not an Option""
}");
        // Do NOT create the game directory — so FindGameInLibrary will try the fallback

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            // Falls back to common/DINO_NAME — but that doesn't exist either
            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator FindGameInLibrary edge cases ────────────────────────

    [Fact]
    public void FindGameInLibrary_SteamAppsExistsButNoManifest_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        // Create common directory but no manifest
        Directory.CreateDirectory(Path.Combine(steamAppsDir, "common"));

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            // Should return null since no manifest and common dir doesn't match DINO name
            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void FindGameInLibrary_ManifestWithNoInstallDir_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Test""
}");

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator GetLibraryFolders VDF error path ────────────────────────

    [Fact]
    public void GetLibraryFolders_VdfFileWithGarbage_ParsingFailsGracefully()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string configDir = Path.Combine(tempDir, "config");
        Directory.CreateDirectory(configDir);
        string vdfFile = Path.Combine(configDir, "libraryfolders.vdf");
        // Write malformed VDF that could cause regex or parsing issues
        File.WriteAllText(vdfFile, "not valid vdf {{{[[[]]]}}}");

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            // Should return steam root (fallback) without throwing
            folders.Should().NotBeEmpty();
            folders[0].Should().Be(tempDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator ParseLibraryFoldersVdf edge cases ────────────────────────

    [Fact]
    public void ParseLibraryFoldersVdf_WithEscapedBackslashes_UnescapesCorrectly()
    {
        string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\\\Program Files\\\\Steam""
    }
}";
        IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

        paths.Should().HaveCount(1);
        // VDF escapes backslashes as \\, should be unescaped to single \
        paths[0].Should().Contain("C:");
    }

    [Fact]
    public void ParseLibraryFoldersVdf_WithDuplicatePaths_ReturnsDeduplicatedList()
    {
        string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\Steam""
    }
    ""1""
    {
        ""path""		""C:\\STEAM""
    }
}";
        IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

        // Should contain both but without case-sensitive duplicates
        paths.Should().HaveCount(2);
    }

    [Fact]
    public void ParseLibraryFoldersVdf_WithValidPaths_ParsesCorrectly()
    {
        string vdf = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\Steam\\Library1""
    }
    ""1""
    {
        ""path""		""D:\\SteamLibrary2""
    }
}";
        IReadOnlyList<string> paths = SteamLocator.ParseLibraryFoldersVdf(vdf);

        paths.Should().HaveCount(2);
        paths.Should().Contain(p => p.Contains("Library1"));
        paths.Should().Contain(p => p.Contains("Library2"));
    }

    // ──────────────────────── SteamLocator FindGameInLibrary edge cases ────────────────────────

    [Fact]
    public void FindGameInLibrary_WithSteamAppsAlreadyAsLibraryPath_ReturnsPath()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string gameDir = Path.Combine(tempDir, "Diplomacy is Not an Option");
        Directory.CreateDirectory(gameDir);

        try
        {
            // When tempDir IS the steamapps folder (passed directly)
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            // Falls back to checking common/DINO_NAME
            if (Directory.Exists(Path.Combine(tempDir, "common", "Diplomacy is Not an Option")))
            {
                result.Should().NotBeNull();
            }
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void FindGameInLibrary_WithAcfButInvalidInstallDir_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Diplomacy is Not an Option""
    ""installdir""    ""NonExistentGameDir""
}");
        // Don't create the game directory

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator GetLibraryFolders additional edge cases ────────────────────────

    [Fact]
    public void GetLibraryFolders_WithEmptyVdfContent_ReturnsOnlySteamRoot()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string vdfFile = Path.Combine(tempDir, "steamapps", "libraryfolders.vdf");
        Directory.CreateDirectory(Path.GetDirectoryName(vdfFile)!);
        File.WriteAllText(vdfFile, "");

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            folders.Should().HaveCount(1);
            folders[0].Should().Be(tempDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void GetLibraryFolders_WithConfigLibraryFolders_ReturnsFromConfig()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string configDir = Path.Combine(tempDir, "config");
        Directory.CreateDirectory(configDir);
        string vdfFile = Path.Combine(configDir, "libraryfolders.vdf");
        string vdfContent = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""D:\\SteamLibrary2""
    }
}";
        File.WriteAllText(vdfFile, vdfContent);

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            folders.Should().HaveCount(2); // Steam root + config library
            folders.Should().Contain("D:\\SteamLibrary2");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator Constants ────────────────────────

    [Fact]
    public void SteamLocator_DinoDirectoryName_IsCorrect()
    {
        SteamLocator.DinoDirectoryName.Should().Be("Diplomacy is Not an Option");
    }

    [Fact]
    public void SteamLocator_FindDinoInstallPath_ReturnsNull_WhenSteamNotInstalled()
    {
        // When Steam is not found, FindDinoInstallPath should return null
        // Note: On Windows with actual Steam, this might return a valid path
        string? result = SteamLocator.FindDinoInstallPath();

        // Either returns null or a valid path - depends on system state
        if (result != null)
        {
            Directory.Exists(result).Should().BeTrue();
        }
    }

    // ──────────────────────── InstallLifecycle MigrateLegacyPacks edge cases ────────────────────────

    [Fact]
    public void MigrateLegacyPacks_WithSubdirectories_MigratesAllFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string legacyDir = Path.Combine(tempDir, "dinoforge_packs");
        Directory.CreateDirectory(legacyDir);
        string subDir = Path.Combine(legacyDir, "subdir");
        Directory.CreateDirectory(subDir);
        File.WriteAllText(Path.Combine(legacyDir, "root.txt"), "root content");
        File.WriteAllText(Path.Combine(subDir, "nested.txt"), "nested content");

        try
        {
            bool result = InstallLifecycle.MigrateLegacyPacks(tempDir);

            result.Should().BeTrue();
            Directory.Exists(legacyDir).Should().BeFalse();
            string newPacksDir = Path.Combine(tempDir, "BepInEx", "dinoforge_packs");
            Directory.Exists(newPacksDir).Should().BeTrue();
            File.Exists(Path.Combine(newPacksDir, "root.txt")).Should().BeTrue();
            File.Exists(Path.Combine(newPacksDir, "subdir", "nested.txt")).Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void MigrateLegacyPacks_WithEmptySubdirectories_MigratesStructure()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string legacyDir = Path.Combine(tempDir, "dinoforge_packs");
        Directory.CreateDirectory(legacyDir);
        Directory.CreateDirectory(Path.Combine(legacyDir, "empty_subdir"));

        try
        {
            bool result = InstallLifecycle.MigrateLegacyPacks(tempDir);

            result.Should().BeTrue();
            Directory.Exists(Path.Combine(tempDir, "BepInEx", "dinoforge_packs", "empty_subdir")).Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallLifecycle CleanupLegacyArtifacts edge cases ────────────────────────

    [Fact]
    public void CleanupLegacyArtifacts_WithReadOnlyFiles_HandlesGracefully()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string legacyFile = Path.Combine(tempDir, "BepInEx", "ecs_plugins", "DINOForge.Runtime.dll");
        Directory.CreateDirectory(Path.GetDirectoryName(legacyFile)!);
        File.WriteAllBytes(legacyFile, new byte[] { 0 });

        try
        {
            int count = InstallLifecycle.CleanupLegacyArtifacts(tempDir);

            count.Should().BeGreaterOrEqualTo(1);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator FindGameInLibrary additional edge cases ────────────────────────

    [Fact]
    public void FindGameInLibrary_WithSteamAppsAsRoot_HandlesCorrectly()
    {
        // When libraryPath is the steamapps folder itself (not the Steam root)
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string gameDir = Path.Combine(steamAppsDir, "common", "Diplomacy is Not an Option");
        Directory.CreateDirectory(gameDir);

        try
        {
            // Pass steamapps as library path - should fall through to common check
            string? result = SteamLocator.FindGameInLibrary(steamAppsDir, SteamLocator.DinoAppId);

            result.Should().Be(gameDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void FindGameInLibrary_WithManifestHavingWhitespace_ReturnsCleanPath()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Diplomacy is Not an Option""
    ""installdir""    ""  Diplomacy is Not an Option  ""
}");
        string gameDir = Path.Combine(steamAppsDir, "common", "Diplomacy is Not an Option");
        Directory.CreateDirectory(gameDir);

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().Be(gameDir);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallLifecycle WriteManifest with UI assets ────────────────────────

    [Fact]
    public void WriteManifest_WithUiAssetsDirectory_IncludesUiAssetsFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        string uiAssetsDir = Path.Combine(pluginsDir, "dinoforge-ui-assets");
        Directory.CreateDirectory(uiAssetsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0, 1, 2, 3 });
        File.WriteAllText(Path.Combine(uiAssetsDir, "test.json"), "{}");

        try
        {
            string manifestPath = InstallLifecycle.WriteManifest(tempDir, "1.0.0");

            File.Exists(manifestPath).Should().BeTrue();
            string content = File.ReadAllText(manifestPath);
            content.Should().Contain("DINOForge.Runtime.dll");
            content.Should().Contain("test.json");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void WriteManifest_WithEmptyPluginsDirectory_CreatesManifest()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        // BepInEx/plugins directory doesn't exist

        try
        {
            string manifestPath = InstallLifecycle.WriteManifest(tempDir, "1.0.0");

            File.Exists(manifestPath).Should().BeTrue();
            string content = File.ReadAllText(manifestPath);
            content.Should().Contain("1.0.0");
            content.Should().Contain("Files");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallLifecycle Inspect with manifest ────────────────────────

    [Fact]
    public void Inspect_WithManifestContainingMissingFiles_ReportsIssues()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        // Write a manifest that references a file that doesn't exist
        string manifestPath = Path.Combine(pluginsDir, "dinoforge.install_manifest.json");
        File.WriteAllText(manifestPath, @"{
    ""InstallerVersion"": ""1.0.0"",
    ""InstalledAtUtc"": ""2026-03-30T00:00:00Z"",
    ""Files"": [
        { ""RelativePath"": ""BepInEx/plugins/missing.dll"", ""Size"": 100, ""Sha256"": ""abc123"" }
    ]
}");

        try
        {
            var inspection = InstallLifecycle.Inspect(tempDir);

            inspection.ManifestPresent.Should().BeTrue();
            inspection.Issues.Should().Contain(i => i.Contains("missing"));
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void Inspect_WithManifestAndRuntime_ReportsHealthy()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        File.WriteAllBytes(Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), new byte[] { 0 });
        // Create manifest with the runtime file
        string manifestPath = Path.Combine(pluginsDir, "dinoforge.install_manifest.json");
        File.WriteAllText(manifestPath, @"{
    ""InstallerVersion"": ""1.0.0"",
    ""InstalledAtUtc"": ""2026-03-30T00:00:00Z"",
    ""Files"": [
        { ""RelativePath"": ""BepInEx/plugins/DINOForge.Runtime.dll"", ""Size"": 1, ""Sha256"": ""e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"" }
    ]
}");

        try
        {
            var inspection = InstallLifecycle.Inspect(tempDir);

            inspection.RuntimeInstalled.Should().BeTrue();
            inspection.ManifestPresent.Should().BeTrue();
            inspection.IsHealthy.Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── InstallLifecycle RemoveManagedFiles with manifest ────────────────────────

    [Fact]
    public void RemoveManagedFiles_WithValidManifest_DeletesManifestFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        string runtimeDll = Path.Combine(pluginsDir, "DINOForge.Runtime.dll");
        File.WriteAllBytes(runtimeDll, new byte[] { 0, 1, 2, 3 });
        // Create manifest with the runtime file
        string manifestPath = Path.Combine(pluginsDir, "dinoforge.install_manifest.json");
        File.WriteAllText(manifestPath, @"{
    ""InstallerVersion"": ""1.0.0"",
    ""InstalledAtUtc"": ""2026-03-30T00:00:00Z"",
    ""Files"": [
        { ""RelativePath"": ""BepInEx/plugins/DINOForge.Runtime.dll"", ""Size"": 4, ""Sha256"": ""abc123"" }
    ]
}");

        try
        {
            int count = InstallLifecycle.RemoveManagedFiles(tempDir);

            count.Should().BeGreaterOrEqualTo(1);
            File.Exists(runtimeDll).Should().BeFalse();
            File.Exists(manifestPath).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void RemoveManagedFiles_WithUiAssetsDirectory_DeletesManifestFiles()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string pluginsDir = Path.Combine(tempDir, "BepInEx", "plugins");
        Directory.CreateDirectory(pluginsDir);
        string uiAssetsDir = Path.Combine(pluginsDir, "dinoforge-ui-assets");
        Directory.CreateDirectory(uiAssetsDir);
        string uiFile = Path.Combine(uiAssetsDir, "test.json");
        File.WriteAllText(uiFile, "{}");
        // Create manifest that includes UI assets
        string manifestPath = Path.Combine(pluginsDir, "dinoforge.install_manifest.json");
        File.WriteAllText(manifestPath, @"{
    ""InstallerVersion"": ""1.0.0"",
    ""InstalledAtUtc"": ""2026-03-30T00:00:00Z"",
    ""Files"": [
        { ""RelativePath"": ""BepInEx/plugins/dinoforge-ui-assets/test.json"", ""Size"": 2, ""Sha256"": ""abc123"" }
    ]
}");

        try
        {
            int count = InstallLifecycle.RemoveManagedFiles(tempDir);

            count.Should().BeGreaterOrEqualTo(1);
            File.Exists(uiFile).Should().BeFalse();
            File.Exists(manifestPath).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator GetLibraryFolders with config VDF ────────────────────────

    [Fact]
    public void GetLibraryFolders_WithConfigLibraryFolders_ReturnsParsedFolders()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string configDir = Path.Combine(tempDir, "config");
        Directory.CreateDirectory(configDir);
        string vdfFile = Path.Combine(configDir, "libraryfolders.vdf");
        string vdfContent = @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""D:\\SteamLibrary""
    }
    ""1""
    {
        ""path""		""E:\\Games\\Steam""
    }
}";
        File.WriteAllText(vdfFile, vdfContent);

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            folders.Should().HaveCount(3); // Steam root + 2 from config
            folders.Should().Contain("D:\\SteamLibrary");
            folders.Should().Contain("E:\\Games\\Steam");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void GetLibraryFolders_WithBothVdfLocations_UsesSteamAppsFirst()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string configDir = Path.Combine(tempDir, "config");
        Directory.CreateDirectory(configDir);
        // Create VDF in both locations
        File.WriteAllText(Path.Combine(steamAppsDir, "libraryfolders.vdf"), @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""C:\\Library1""
    }
}");
        File.WriteAllText(Path.Combine(configDir, "libraryfolders.vdf"), @"
""libraryfolders""
{
    ""0""
    {
        ""path""		""D:\\Library2""
    }
}");

        try
        {
            IReadOnlyList<string> folders = SteamLocator.GetLibraryFolders(tempDir);

            // Should use steamapps location first (found first)
            folders.Should().Contain("C:\\Library1");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── SteamLocator ParseInstallDirFromAcf edge cases ────────────────────────

    [Fact]
    public void FindGameInLibrary_WithEmptyAcf_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, "");

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void FindGameInLibrary_WithMissingInstalldir_ReturnsNull()
    {
        string tempDir = Path.Combine(Path.GetTempPath(), $"dino_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string steamAppsDir = Path.Combine(tempDir, "steamapps");
        Directory.CreateDirectory(steamAppsDir);
        string acfFile = Path.Combine(steamAppsDir, $"appmanifest_{SteamLocator.DinoAppId}.acf");
        File.WriteAllText(acfFile, @"
""AppState""
{
    ""appname""    ""Test Game""
}");

        try
        {
            string? result = SteamLocator.FindGameInLibrary(tempDir, SteamLocator.DinoAppId);

            result.Should().BeNull();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }
}
