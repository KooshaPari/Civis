#nullable enable
using System;
using System.IO;
using DINOForge.Tools.PackCompiler.Services;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tools.PackCompiler.Tests
{
    /// <summary>
    /// Tests for <see cref="SafePathResolver"/> path-containment behavior.
    /// Mirrors the #166 pattern (AssetctlPipeline + InstallLifecycle) — combine + GetFullPath + StartsWith check.
    /// Task #208 (Pattern #74 / #166 follow-up).
    /// </summary>
    public class SafePathResolverTests : IDisposable
    {
        private readonly string _tempRoot;

        public SafePathResolverTests()
        {
            _tempRoot = Path.Combine(Path.GetTempPath(), "safepath_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
            {
                try { Directory.Delete(_tempRoot, recursive: true); } catch { /* best effort */ }
            }
        }

        [Fact]
        public void TryResolveSafePath_ValidRelativePathInsideRoot_ReturnsAbsolutePath()
        {
            // Arrange
            string relative = Path.Combine("assets", "models", "infantry.glb");

            // Act
            string? resolved = SafePathResolver.TryResolveSafePath(_tempRoot, relative);

            // Assert
            resolved.Should().NotBeNull();
            resolved.Should().StartWith(Path.GetFullPath(_tempRoot));
            resolved.Should().EndWith("infantry.glb");
        }

        [Fact]
        public void TryResolveSafePath_DotDotTraversal_ReturnsNull()
        {
            // Arrange — try to escape the trusted root via ../../../etc/passwd-style traversal
            string traversal = Path.Combine("..", "..", "evil.txt");

            // Act
            string? resolved = SafePathResolver.TryResolveSafePath(_tempRoot, traversal);

            // Assert
            resolved.Should().BeNull("path-traversal segments must be rejected");
        }

        [Fact]
        public void TryResolveSafePath_AbsolutePathOutsideRoot_ReturnsNull()
        {
            // Arrange — absolute path Path.Combine semantics: an absolute segment overrides earlier segments
            string absoluteOutside = OperatingSystem.IsWindows()
                ? @"C:\Windows\System32\drivers\etc\hosts"
                : "/etc/passwd";

            // Act
            string? resolved = SafePathResolver.TryResolveSafePath(_tempRoot, absoluteOutside);

            // Assert
            resolved.Should().BeNull("absolute paths outside the trusted root must be rejected");
        }

        [Fact]
        public void TryResolveSafePath_EmptyTrustedRoot_Throws()
        {
            // Act
            Action act = () => SafePathResolver.TryResolveSafePath("", "anything.txt");

            // Assert
            act.Should().Throw<ArgumentException>().WithMessage("trustedRoot must be non-empty*");
        }
    }
}
