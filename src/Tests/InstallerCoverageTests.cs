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

    // ──────────────────────── InstallLifecycle ────────────────────────

    [Fact]
    public void GetBepInExDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = InstallLifecycle.GetBepInExDirectory(gamePath);

        result.Should().Be(@"C:\Games\DINO\BepInEx");
    }

    [Fact]
    public void GetPluginsDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = InstallLifecycle.GetPluginsDirectory(gamePath);

        result.Should().Be(@"C:\Games\DINO\BepInEx\plugins");
    }

    [Fact]
    public void GetPacksDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = InstallLifecycle.GetPacksDirectory(gamePath);

        result.Should().Be(@"C:\Games\DINO\BepInEx\dinoforge_packs");
    }

    [Fact]
    public void GetLegacyPacksDirectory_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = InstallLifecycle.GetLegacyPacksDirectory(gamePath);

        result.Should().Be(@"C:\Games\DINO\dinoforge_packs");
    }

    [Fact]
    public void GetManifestPath_CombinesPathsCorrectly()
    {
        string gamePath = @"C:\Games\DINO";
        string result = InstallLifecycle.GetManifestPath(gamePath);

        result.Should().Be(@"C:\Games\DINO\BepInEx\plugins\dinoforge.install_manifest.json");
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
        string legacyFile = Path.Combine(tempDir, @"BepInEx\ecs_plugins\DINOForge.Runtime.dll");
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
}
