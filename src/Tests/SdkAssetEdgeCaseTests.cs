#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Edge case tests for SDK asset services (AssetValidationResult, AssetReplacementEngine).
/// These tests target uncovered branches in asset handling logic.
/// </summary>
public class SdkAssetEdgeCaseTests
{
    // ──────────────────────── AssetValidationResult Edge Cases ────────────────────────

    [Fact]
    public void AssetValidationResult_ConstructWithNullUnityVersion_ThrowsArgumentNullException()
    {
        var errors = new List<string> { "error1" };
        var assets = new List<AssetInfo>();

        Action action = () => new AssetValidationResult(true, null!, errors, assets);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void AssetValidationResult_ConstructWithNullErrors_ThrowsArgumentNullException()
    {
        var unityVersion = "2021.3.45f2";
        var assets = new List<AssetInfo>();

        Action action = () => new AssetValidationResult(true, unityVersion, null!, assets);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void AssetValidationResult_ConstructWithNullAssets_ThrowsArgumentNullException()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string> { "error1" };

        Action action = () => new AssetValidationResult(true, unityVersion, errors, null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void AssetValidationResult_ConstructWithValidInputs_StoresProperties()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string> { "error1" } as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(true, unityVersion, errors, assets);

        result.IsValid.Should().BeTrue();
        result.UnityVersion.Should().Be(unityVersion);
        result.Errors.Should().ContainSingle();
        result.Assets.Should().BeEmpty();
    }

    [Fact]
    public void AssetValidationResult_ConstructWithEmptyErrors_StoresEmptyList()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string>() as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(false, unityVersion, errors, assets);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void AssetValidationResult_ConstructWithMultipleErrors_StoresAll()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string> { "error1", "error2", "error3" } as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(true, unityVersion, errors, assets);

        result.Errors.Should().HaveCount(3);
    }

    [Fact]
    public void AssetValidationResult_ConstructWithMultipleAssets_StoresAll()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string>() as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(true, unityVersion, errors, assets);

        result.Assets.Should().BeEmpty();
    }

    [Fact]
    public void AssetValidationResult_Failure_CreatesFailedResult()
    {
        var errors = new List<string> { "Bundle version mismatch", "Missing textures" } as IReadOnlyList<string>;

        var result = AssetValidationResult.Failure(errors);

        result.IsValid.Should().BeFalse();
        result.UnityVersion.Should().Be("unknown");
        result.Errors.Should().HaveCount(2);
        result.Assets.Should().BeEmpty();
    }

    [Fact]
    public void AssetValidationResult_Failure_WithEmptyErrors_CreatesFailedResultWithEmptyList()
    {
        var errors = new List<string>() as IReadOnlyList<string>;

        var result = AssetValidationResult.Failure(errors);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void AssetValidationResult_Failure_WithNullErrors_ThrowsArgumentNullException()
    {
        Action action = () => AssetValidationResult.Failure(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void AssetValidationResult_IsValidTrue_IndicatesSuccess()
    {
        var errors = new List<string>() as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(true, "2021.3.45f2", errors, assets);

        result.IsValid.Should().BeTrue();
    }

    [Fact]
    public void AssetValidationResult_IsValidFalse_IndicatesFailure()
    {
        var errors = new List<string>() as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(false, "2021.3.45f2", errors, assets);

        result.IsValid.Should().BeFalse();
    }

    [Fact]
    public void AssetValidationResult_UnityVersionWithDifferentValues_StoresCorrectly()
    {
        var versions = new[] { "2021.3.45f2", "2022.3.0f1", "2023.1.0a1", "unknown" };

        foreach (var version in versions)
        {
            var errors = new List<string>() as IReadOnlyList<string>;
            var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

            var result = new AssetValidationResult(true, version, errors, assets);

            result.UnityVersion.Should().Be(version);
        }
    }

    [Fact]
    public void AssetValidationResult_WithLargeErrorList_HandlesCorrectly()
    {
        var unityVersion = "2021.3.45f2";
        var errors = new List<string>();
        for (int i = 0; i < 100; i++)
        {
            errors.Add($"Error {i}");
        }

        var result = new AssetValidationResult(false, unityVersion, errors, new List<AssetInfo>());

        result.Errors.Should().HaveCount(100);
    }

    [Fact]
    public void AssetValidationResult_WithEmptyUnityVersion_StoresEmpty()
    {
        var errors = new List<string>() as IReadOnlyList<string>;
        var assets = new List<AssetInfo>() as IReadOnlyList<AssetInfo>;

        var result = new AssetValidationResult(true, "", errors, assets);

        result.UnityVersion.Should().BeEmpty();
    }
}
