#nullable enable
using System;
using System.IO;
using DINOForge.SDK.HotReload;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Behavior coverage for <see cref="FileSystemPackRootResolver"/> — walks parent directories from a
/// changed file up to (but not past) the packs-root, returning the first directory containing a
/// <c>pack.yaml</c>. Reachable from tests via <c>InternalsVisibleTo("DINOForge.Tests")</c>.
///
/// Uses a real temp directory tree (the resolver calls <see cref="File.Exists"/>); each test builds
/// an isolated tree under the system temp dir and removes it afterwards. Guards: manifest at the
/// immediate parent, several levels up, at the packs-root boundary itself, and the not-found → null
/// path.
/// </summary>
public sealed class FileSystemPackRootResolverTests : IDisposable
{
    private readonly string _root;
    private readonly FileSystemPackRootResolver _sut = new();

    public FileSystemPackRootResolverTests()
    {
        // Unique per-test root. Guid-free (Guid.NewGuid is fine in tests, but keep it deterministic-ish
        // via the temp dir API which already guarantees uniqueness).
        _root = Directory.CreateTempSubdirectory("dino-packroot-").FullName;
    }

    public void Dispose()
    {
        try { Directory.Delete(_root, recursive: true); } catch { /* best-effort temp cleanup */ }
    }

    private string Dir(params string[] parts)
    {
        string p = Path.Combine(_root, Path.Combine(parts));
        Directory.CreateDirectory(p);
        return p;
    }

    private static void WriteManifest(string dir) =>
        File.WriteAllText(Path.Combine(dir, "pack.yaml"), "id: test\n");

    [Fact]
    public void ResolvePackRoot_ManifestInImmediateParent_ReturnsThatDirectory()
    {
        string packDir = Dir("packs", "my-pack");
        WriteManifest(packDir);
        string changed = Path.Combine(packDir, "units", "trooper.yaml");

        string? resolved = _sut.ResolvePackRoot(changed, Path.Combine(_root, "packs"));

        resolved.Should().Be(packDir);
    }

    [Fact]
    public void ResolvePackRoot_ManifestSeveralLevelsUp_WalksToIt()
    {
        string packDir = Dir("packs", "deep-pack");
        WriteManifest(packDir);
        // Changed file is nested 3 levels below the pack root.
        string changed = Path.Combine(packDir, "assets", "models", "lod0", "mesh.glb");

        string? resolved = _sut.ResolvePackRoot(changed, Path.Combine(_root, "packs"));

        resolved.Should().Be(packDir);
    }

    [Fact]
    public void ResolvePackRoot_ManifestAtPacksRootBoundary_IsFound()
    {
        // The packs-root directory itself carries the manifest (the loop stops AT packsRoot, then the
        // post-loop check inspects packsRoot) — guards the boundary branch.
        string packsRoot = Dir("packs");
        WriteManifest(packsRoot);
        string changed = Path.Combine(packsRoot, "config.yaml");

        string? resolved = _sut.ResolvePackRoot(changed, packsRoot);

        resolved.Should().Be(packsRoot);
    }

    [Fact]
    public void ResolvePackRoot_NoManifestAnywhere_ReturnsNull()
    {
        string someDir = Dir("packs", "not-a-pack", "sub");
        string changed = Path.Combine(someDir, "stray.txt");

        string? resolved = _sut.ResolvePackRoot(changed, Path.Combine(_root, "packs"));

        resolved.Should().BeNull();
    }

    [Fact]
    public void ResolvePackRoot_StopsAtPacksRoot_DoesNotEscapeAbove()
    {
        // A manifest exists ABOVE the packs-root; the resolver must NOT walk past packsRoot to find it.
        WriteManifest(_root); // pack.yaml above the packs-root — must be ignored
        string packsRoot = Dir("packs");
        string changed = Path.Combine(packsRoot, "orphan", "file.yaml");
        Directory.CreateDirectory(Path.GetDirectoryName(changed)!);

        string? resolved = _sut.ResolvePackRoot(changed, packsRoot);

        resolved.Should().BeNull("the manifest above packsRoot is out of scope");
    }
}
