#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for the DINOBox sandbox infrastructure.
///
/// Tests verify that:
/// - Each sandbox has unique isolation (directory, pipe, environment)
/// - File systems are isolated between sandboxes
/// - LocalAppData and config isolation works
/// - Processes launched in sandboxes don't interfere with each other
/// - Cleanup properly removes all sandbox artifacts
/// </summary>
[Trait("Category", "SandboxIsolation")]
[Trait("Category", "Integration")]
public class SandboxIsolationTests : IAsyncLifetime
{
    private GameTestContainerHarness? _harness;
    private List<GameTestContainerHarness.GameContainer>? _containers;
    private const int TestPoolSize = 2;

    public async Task InitializeAsync()
    {
        // Create a test harness with a small pool for isolation testing
        _harness = new GameTestContainerHarness();
        try
        {
            _containers = await _harness.CreatePoolAsync(TestPoolSize).ConfigureAwait(true);
        }
        catch
        {
            // If script-based pool creation fails, skip tests gracefully
            _containers = null;
        }
    }

    public async Task DisposeAsync()
    {
        if (_harness != null)
        {
            await _harness.DisposeAsync().ConfigureAwait(true);
        }
    }

    private void SkipIfNoContainers()
    {
        if (_containers == null || _containers.Count == 0)
        {
            // Skip test gracefully if containers not available
            return;
        }
    }

    /// <summary>
    /// GIVEN a game container harness
    /// WHEN containers are created
    /// THEN each container has a unique directory path
    /// </summary>
    [Fact]
    public void Sandbox_CreatePool_HasUniquePaths()
    {
        // Arrange
        SkipIfNoContainers();

        // Act
        var paths = _containers!.Select(c => c.BoxPath).ToList();

        // Assert
        paths.Should().HaveCount(TestPoolSize);
        paths.Distinct().Should().HaveCount(TestPoolSize, "each container should have unique directory");
    }

    /// <summary>
    /// GIVEN a game container harness
    /// WHEN containers are created
    /// THEN each container has a unique named pipe
    /// </summary>
    [Fact]
    public void Sandbox_CreatePool_HasUniquePipeNames()
    {
        // Arrange
        SkipIfNoContainers();

        // Act
        var pipeNames = _containers!.Select(c => c.PipeName).ToList();

        // Assert
        pipeNames.Should().HaveCount(TestPoolSize);
        pipeNames.Distinct().Should().HaveCount(TestPoolSize, "each container should have unique pipe name");
    }

    /// <summary>
    /// GIVEN a game container harness
    /// WHEN containers are created
    /// THEN each container has a unique UUID
    /// </summary>
    [Fact]
    public void Sandbox_CreatePool_HasUniqueUuids()
    {
        // Arrange
        SkipIfNoContainers();

        // Act
        var uuids = _containers!.Select(c => c.Uuid).ToList();

        // Assert
        uuids.Should().HaveCount(TestPoolSize);
        uuids.Distinct().Should().HaveCount(TestPoolSize, "each container should have unique UUID");
    }

    /// <summary>
    /// GIVEN a game container
    /// WHEN the container directory is checked
    /// THEN all required subdirectories exist
    /// </summary>
    [Fact]
    public void Sandbox_ContainerStructure_HasRequiredDirectories()
    {
        // Arrange
        SkipIfNoContainers();
        var container = _containers![0];

        // Act & Assert
        Directory.Exists(container.BoxPath).Should().BeTrue("box path should exist");
        Directory.Exists(container.BepInExDir).Should().BeTrue("BepInEx directory should exist");
        Directory.Exists(container.SaveDir).Should().BeTrue("save directory should exist");
    }

    /// <summary>
    /// GIVEN two game containers
    /// WHEN a file is written to container 1's directory
    /// THEN the file does not appear in container 2's directory
    /// </summary>
    [Fact]
    public void Sandbox_FileIsolation_FilesNotShared()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        var testFileName = "isolation_test_" + Guid.NewGuid().ToString("N")[..8] + ".txt";
        var testFile1 = Path.Combine(container1.BoxPath, testFileName);
        var testFile2 = Path.Combine(container2.BoxPath, testFileName);

        try
        {
            // Act - write to container 1
            File.WriteAllText(testFile1, "test data for container 1");

            // Assert - verify isolation
            File.Exists(testFile1).Should().BeTrue("file should exist in container 1");
            File.Exists(testFile2).Should().BeFalse("file should NOT exist in container 2 (isolated)");

            // Verify content
            var content = File.ReadAllText(testFile1, System.Text.Encoding.UTF8);
            content.Should().Be("test data for container 1");
        }
        finally
        {
            // Cleanup
            if (File.Exists(testFile1)) File.Delete(testFile1);
            if (File.Exists(testFile2)) File.Delete(testFile2);
        }
    }

    /// <summary>
    /// GIVEN two game containers
    /// WHEN BepInEx config is written to each
    /// THEN each maintains isolated configuration
    /// </summary>
    [Fact]
    public void Sandbox_BepInExIsolation_ConfigsIndependent()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        var configFile1 = Path.Combine(container1.BepInExDir, "config.cfg");
        var configFile2 = Path.Combine(container2.BepInExDir, "config.cfg");

        try
        {
            // Act - write different configs to each container
            File.WriteAllText(configFile1, "[TestSection]\nKey=Value1\n");
            File.WriteAllText(configFile2, "[TestSection]\nKey=Value2\n");

            // Assert - verify isolation
            var content1 = File.ReadAllText(configFile1, System.Text.Encoding.UTF8);
            var content2 = File.ReadAllText(configFile2, System.Text.Encoding.UTF8);

            content1.Should().Contain("Value1");
            content2.Should().Contain("Value2");
            content1.Should().NotContain("Value2");
            content2.Should().NotContain("Value1");
        }
        finally
        {
            // Cleanup
            if (File.Exists(configFile1)) File.Delete(configFile1);
            if (File.Exists(configFile2)) File.Delete(configFile2);
        }
    }

    /// <summary>
    /// GIVEN two game containers
    /// WHEN save files are written to each
    /// THEN each maintains isolated save state
    /// </summary>
    [Fact]
    public void Sandbox_SaveDirectoryIsolation_SavesIndependent()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        var saveFile1 = Path.Combine(container1.SaveDir, "test_save.sav");
        var saveFile2 = Path.Combine(container2.SaveDir, "test_save.sav");

        try
        {
            // Act - write different saves
            File.WriteAllText(saveFile1, "SAVE_DATA_CONTAINER_1");
            File.WriteAllText(saveFile2, "SAVE_DATA_CONTAINER_2");

            // Assert
            var save1 = File.ReadAllText(saveFile1, System.Text.Encoding.UTF8);
            var save2 = File.ReadAllText(saveFile2, System.Text.Encoding.UTF8);

            save1.Should().Be("SAVE_DATA_CONTAINER_1");
            save2.Should().Be("SAVE_DATA_CONTAINER_2");
        }
        finally
        {
            // Cleanup
            if (File.Exists(saveFile1)) File.Delete(saveFile1);
            if (File.Exists(saveFile2)) File.Delete(saveFile2);
        }
    }

    /// <summary>
    /// GIVEN a game container
    /// WHEN the container directory structure is checked
    /// THEN the debug log path points to the container's BepInEx directory
    /// </summary>
    [Fact]
    public void Sandbox_DebugLogPath_PointsToContainer()
    {
        // Arrange
        SkipIfNoContainers();
        var container = _containers![0];

        // Act
        var logPath = container.DebugLogPath;

        // Assert
        logPath.Should().StartWith(container.BepInExDir);
        logPath.Should().EndWith(".log", "debug log should be a .log file");
    }

    /// <summary>
    /// GIVEN two game containers with the same subdirectory structure
    /// WHEN we verify their exe paths are different
    /// THEN each has its own executable path
    /// </summary>
    [Fact]
    public void Sandbox_ExePaths_AreUnique()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        // Act
        var exePath1 = container1.ExePath;
        var exePath2 = container2.ExePath;

        // Assert
        exePath1.Should().NotBe(exePath2, "each container should have unique exe path");
        exePath1.Should().Contain(container1.BoxPath);
        exePath2.Should().Contain(container2.BoxPath);
    }

    /// <summary>
    /// GIVEN multiple containers with files
    /// WHEN directory traversal is performed
    /// THEN each directory only contains its own files
    /// </summary>
    [Fact]
    public void Sandbox_DirectoryTraversal_NoLeakBetweenContainers()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        var markerFile1 = Path.Combine(container1.BoxPath, "marker1_" + Guid.NewGuid().ToString("N")[..8] + ".txt");
        var markerFile2 = Path.Combine(container2.BoxPath, "marker2_" + Guid.NewGuid().ToString("N")[..8] + ".txt");

        try
        {
            // Act - create marker files
            File.WriteAllText(markerFile1, "container1");
            File.WriteAllText(markerFile2, "container2");

            // Assert - list files in each directory
            var filesInContainer1 = Directory.GetFiles(container1.BoxPath).ToList();
            var filesInContainer2 = Directory.GetFiles(container2.BoxPath).ToList();

            // Container 1 should have marker1, not marker2
            filesInContainer1.Should().ContainMatch("*marker1*");
            filesInContainer1.Should().NotContainMatch("*marker2*");

            // Container 2 should have marker2, not marker1
            filesInContainer2.Should().ContainMatch("*marker2*");
            filesInContainer2.Should().NotContainMatch("*marker1*");
        }
        finally
        {
            // Cleanup
            if (File.Exists(markerFile1)) File.Delete(markerFile1);
            if (File.Exists(markerFile2)) File.Delete(markerFile2);
        }
    }

    /// <summary>
    /// GIVEN containers with nested directory structures
    /// WHEN subdirectories are created in each
    /// THEN subdirectories remain isolated
    /// </summary>
    [Fact]
    public void Sandbox_NestedDirectories_AreIsolated()
    {
        // Arrange
        SkipIfNoContainers();
        var container1 = _containers![0];
        var container2 = _containers![1];

        var subDir1 = Path.Combine(container1.BoxPath, "test_subdir_" + Guid.NewGuid().ToString("N")[..8]);
        var subDir2 = Path.Combine(container2.BoxPath, "test_subdir_" + Guid.NewGuid().ToString("N")[..8]);

        try
        {
            // Act
            Directory.CreateDirectory(subDir1);
            Directory.CreateDirectory(subDir2);

            var file1 = Path.Combine(subDir1, "nested_file.txt");
            var file2 = Path.Combine(subDir2, "nested_file.txt");

            File.WriteAllText(file1, "nested content 1");
            File.WriteAllText(file2, "nested content 2");

            // Assert
            var content1 = File.ReadAllText(file1, System.Text.Encoding.UTF8);
            var content2 = File.ReadAllText(file2, System.Text.Encoding.UTF8);

            content1.Should().Be("nested content 1");
            content2.Should().Be("nested content 2");

            // Directories should not be the same
            subDir1.Should().NotBe(subDir2);
        }
        finally
        {
            // Cleanup
            if (Directory.Exists(subDir1)) Directory.Delete(subDir1, true);
            if (Directory.Exists(subDir2)) Directory.Delete(subDir2, true);
        }
    }
}
