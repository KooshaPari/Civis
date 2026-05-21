#nullable enable
using System;
using System.IO;
using DINOForge.Tools.Cli.Assetctl;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// #775 P0: Path-injection guards for AssetctlPipeline.
///
/// Verifies that <see cref="PathSafety"/> blocks traversal-bearing path segments
/// (e.g. crafted Sketchfab <c>externalId=../../etc/passwd</c> or
/// <c>OriginalFormat=../../../tmp/x</c>) from escaping the pipeline root when
/// they flow into <see cref="Path.Combine(string, string)"/>.
/// </summary>
public class PathSafetySecurityTests
{
    [Theory]
    [InlineData("../escape")]
    [InlineData("..\\escape")]
    [InlineData("../../etc/passwd")]
    [InlineData("..\\..\\Windows\\System32\\config\\SAM")]
    [InlineData("subdir/../../../escape")]
    public void EnsureWithin_TraversalCandidate_Throws(string traversal)
    {
        string root = Path.Combine(Path.GetTempPath(), "dinoforge_pathsafety_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        try
        {
            Action act = () => PathSafety.EnsureWithin(root, traversal);
            act.Should().Throw<UnauthorizedAccessException>()
                .WithMessage("*Path traversal blocked*");
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    [Theory]
    [InlineData("..")]
    [InlineData("../escape")]
    [InlineData("..\\escape")]
    [InlineData("foo/bar")]
    [InlineData("foo\\bar")]
    [InlineData("C:foo")]
    public void EnsureSafeSegment_UnsafeSegment_Throws(string bad)
    {
        Action act = () => PathSafety.EnsureSafeSegment(bad, "candidate");
        act.Should().Throw<UnauthorizedAccessException>()
            .WithMessage("*Unsafe path segment*");
    }

    [Fact]
    public void EnsureSafeSegment_EmptySegment_ThrowsArgumentException()
    {
        Action act = () => PathSafety.EnsureSafeSegment("", "candidate");
        act.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void EnsureWithin_SafeRelativePath_Returns_FullPathUnderRoot()
    {
        string root = Path.Combine(Path.GetTempPath(), "dinoforge_pathsafety_ok_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        try
        {
            string resolved = PathSafety.EnsureWithin(root, Path.Combine("raw", "sw_sketchfab_model123"));
            resolved.Should().StartWith(Path.GetFullPath(root));
            resolved.Should().EndWith("sw_sketchfab_model123");
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    [Theory]
    [InlineData("glb")]
    [InlineData("fbx")]
    [InlineData("blend")]
    [InlineData("sw_sketchfab_model_123")]
    public void EnsureSafeSegment_SafeSegment_ReturnsInput(string ok)
    {
        PathSafety.EnsureSafeSegment(ok, "candidate").Should().Be(ok);
    }
}
