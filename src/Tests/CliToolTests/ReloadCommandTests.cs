#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for ReloadResult protocol type.
/// </summary>
public class ReloadCommandTests
{
    [Fact]
    public void ReloadCommand_ReloadSuccess()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateSuccessfulReloadResult();

        // Assert
        reloadResult.Success.Should().BeTrue();
    }

    [Fact]
    public void ReloadCommand_ReloadSpecificPack()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateSuccessfulReloadResult(
            ["example-balance"]);

        // Assert
        reloadResult.Success.Should().BeTrue();
        reloadResult.LoadedPacks.Should().Contain("example-balance");
    }

    [Fact]
    public void ReloadCommand_ReturnsFalseOnFailure()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateFailedReloadResult(
            ["Pack not found"]);

        // Assert
        reloadResult.Success.Should().BeFalse();
        reloadResult.Errors.Should().NotBeEmpty();
    }

    [Fact]
    public void ReloadCommand_ReturnsLoadedPacksList()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateSuccessfulReloadResult(
            ["pack1", "pack2", "pack3"]);

        // Assert
        reloadResult.LoadedPacks.Should().HaveCount(3);
        reloadResult.LoadedPacks.Should().Contain(["pack1", "pack2", "pack3"]);
    }

    [Fact]
    public void ReloadCommand_ReturnsErrorsOnFailure()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateFailedReloadResult(
            ["File not found: pack.yaml", "Validation failed"]);

        // Assert
        reloadResult.Errors.Should().HaveCount(2);
        reloadResult.Errors.Should().Contain("File not found: pack.yaml");
    }

    [Fact]
    public void ReloadCommand_SuccessHasNoErrors()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateSuccessfulReloadResult();

        // Assert
        reloadResult.Success.Should().BeTrue();
        reloadResult.Errors.Should().BeEmpty();
    }

    [Fact]
    public void ReloadCommand_FailureHasEmptyPacksList()
    {
        // Arrange & Act
        var reloadResult = CliTestFixtures.CreateFailedReloadResult();

        // Assert
        reloadResult.Success.Should().BeFalse();
        reloadResult.LoadedPacks.Should().BeEmpty();
    }

    [Fact]
    public void ReloadCommand_MultipleErrors()
    {
        // Arrange & Act
        var reloadResult = new ReloadResult
        {
            Success = false,
            LoadedPacks = new List<string>(),
            Errors = new List<string>
            {
                "Error 1",
                "Error 2",
                "Error 3"
            }
        };

        // Assert
        reloadResult.Errors.Should().HaveCount(3);
    }

    [Fact]
    public void ReloadCommand_ModifyingProperties()
    {
        // Arrange
        var reloadResult = CliTestFixtures.CreateSuccessfulReloadResult();

        // Act
        reloadResult.LoadedPacks.Add("new-pack");
        reloadResult.Success = false;

        // Assert
        reloadResult.LoadedPacks.Should().Contain("new-pack");
        reloadResult.Success.Should().BeFalse();
    }

    [Fact]
    public void ReloadCommand_EmptyResults()
    {
        // Arrange & Act
        var reloadResult = new ReloadResult
        {
            Success = true,
            LoadedPacks = new List<string>(),
            Errors = new List<string>()
        };

        // Assert
        reloadResult.LoadedPacks.Should().BeEmpty();
        reloadResult.Errors.Should().BeEmpty();
    }
}
