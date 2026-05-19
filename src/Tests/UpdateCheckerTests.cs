using FluentAssertions;
using Octokit;
using System;
using System.Diagnostics.CodeAnalysis;
using System.Net.Http;
using System.Threading.Tasks;
using Xunit;
using DINOForge.Tools.Installer;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for <see cref="UpdateChecker"/> using delegate injection
/// for test isolation.
///
/// NOTE: This test class is excluded from code coverage due to a known Coverlet limitation
/// when instrumenting net6.0 assemblies within an net8.0 test project. The tests execute
/// correctly; they are simply not instrumented for coverage metrics.
/// </summary>
[ExcludeFromCodeCoverage]
public class UpdateCheckerTests
{
    [Fact]
    public async Task CheckAsync_LatestGreaterThanCurrent_ReturnsHasUpdateTrue()
    {
        // Arrange: Mock release with tag 9.9.9
        var mockRelease = new Release(
            url: "https://api.github.com/repos/KooshaPari/Dino/releases/123",
            htmlUrl: "https://github.com/KooshaPari/Dino/releases/tag/v9.9.9",
            assetsUrl: "",
            uploadUrl: "",
            id: 123,
            nodeId: "MDEyOlJlbGVhc2UxMjM=",
            tagName: "v9.9.9",
            targetCommitish: "main",
            name: "v9.9.9",
            draft: false,
            author: null,
            prerelease: false,
            createdAt: DateTime.Now,
            publishedAt: DateTime.Now,
            assets: new System.Collections.Generic.List<ReleaseAsset>(),
            body: "Latest release",
            tarballUrl: "https://api.github.com/repos/KooshaPari/Dino/tarball/v9.9.9",
            zipballUrl: "https://api.github.com/repos/KooshaPari/Dino/zipball/v9.9.9"
        );

        // Delegate that returns our mock release
        UpdateChecker.GetLatestReleaseDelegate mockDelegate = async (owner, repo) =>
            await Task.FromResult(mockRelease);

        var checker = new UpdateChecker(mockDelegate);

        // Act
        var result = await checker.CheckAsync();

        // Assert
        result.HasUpdate.Should().BeTrue();
        result.LatestVersion.Should().Be("9.9.9");
        result.ReleaseUrl.Should().Be("https://github.com/KooshaPari/Dino/releases/tag/v9.9.9");
    }

    [Fact]
    public async Task CheckAsync_LatestEqualOrLessThanCurrent_ReturnsHasUpdateFalse()
    {
        // Arrange: Mock release with tag 0.0.0 (equal to current assembly version which is 0.0.0 by default)
        var mockRelease = new Release(
            url: "https://api.github.com/repos/KooshaPari/Dino/releases/456",
            htmlUrl: "https://github.com/KooshaPari/Dino/releases/tag/v0.0.0",
            assetsUrl: "",
            uploadUrl: "",
            id: 456,
            nodeId: "MDEyOlJlbGVhc2U0NTY=",
            tagName: "v0.0.0",
            targetCommitish: "main",
            name: "v0.0.0",
            draft: false,
            author: null,
            prerelease: false,
            createdAt: DateTime.Now,
            publishedAt: DateTime.Now,
            assets: new System.Collections.Generic.List<ReleaseAsset>(),
            body: "Current release",
            tarballUrl: "https://api.github.com/repos/KooshaPari/Dino/tarball/v0.0.0",
            zipballUrl: "https://api.github.com/repos/KooshaPari/Dino/zipball/v0.0.0"
        );

        // Delegate that returns our mock release
        UpdateChecker.GetLatestReleaseDelegate mockDelegate = async (owner, repo) =>
            await Task.FromResult(mockRelease);

        var checker = new UpdateChecker(mockDelegate);

        // Act
        var result = await checker.CheckAsync();

        // Assert
        result.HasUpdate.Should().BeFalse();
    }

    [Fact]
    public async Task CheckAsync_NetworkException_ReturnsHasUpdateFalseGracefully()
    {
        // Arrange: Delegate that throws HttpRequestException
        UpdateChecker.GetLatestReleaseDelegate mockDelegate = async (owner, repo) =>
        {
            throw new HttpRequestException("Network error");
        };

        var checker = new UpdateChecker(mockDelegate);

        // Act
        var result = await checker.CheckAsync();

        // Assert
        result.HasUpdate.Should().BeFalse();
        result.ReleaseUrl.Should().BeEmpty();
    }
}
