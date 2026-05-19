using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System;
using System.IO;
using System.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

/// <summary>
/// Pure-math property tests for path normalization invariants.
/// No SUT (System Under Test) references — all helpers embedded.
/// Uses FsCheck [Property] shrinking to discover minimal counterexamples.
/// </summary>
[Trait("Category", "Property")]
[Trait("Layer", "PathSafety")]
public class PathSafetyFsCheckProperties
{
    private static readonly char[] InvalidChars = Path.GetInvalidFileNameChars();

    /// <summary>
    /// Sanitize random strings to valid path segment characters.
    /// Removes unprintable, control chars, separator chars, and invalid filename chars.
    /// Returns "x" if empty after cleaning to ensure non-empty segments.
    /// </summary>
    private static string Clean(string s)
    {
        if (string.IsNullOrEmpty(s)) return "x";
        var chars = s
            .Where(c => c >= 0x20 && c <= 0x7E && !InvalidChars.Contains(c) && c != '/' && c != '\\' && c != '.')
            .ToArray();
        if (chars.Length == 0) return "x";
        return new string(chars);
    }

    /// <summary>
    /// Property 1: Path normalization is idempotent.
    /// Norm(Norm(p)) == Norm(p) for all paths p.
    /// Tests via Path.GetFullPath with a consistent base directory.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool PathNormalize_IsIdempotent(NonEmptyString segA, NonEmptyString segB)
    {
        var a = Clean(segA.Get);
        var b = Clean(segB.Get);
        var p = Path.Combine(a, b);

        var n1 = Path.GetFullPath(p);
        var n2 = Path.GetFullPath(n1);

        return n1.Equals(n2, StringComparison.OrdinalIgnoreCase);
    }

    /// <summary>
    /// Property 2: Path.GetFileName preserves the last segment exactly.
    /// For p = a/b/filename.txt, GetFileName(p) == "filename.txt".
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool GetFileName_PreservesLastSegment(NonEmptyString seg)
    {
        var s = Clean(seg.Get) + ".txt";
        var p = Path.Combine("dir", s);

        var result = Path.GetFileName(p);

        return result == s;
    }

    /// <summary>
    /// Property 3: GetDirectoryName + GetFileName = original path (modulo separators).
    /// Combine(GetDirectoryName(p), GetFileName(p)) ≈ p for all paths p.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool GetDirectoryName_Plus_GetFileName_Reconstructs_Path(NonEmptyString dirName, NonEmptyString fileBase)
    {
        var d = Clean(dirName.Get);
        var f = Clean(fileBase.Get) + ".dat";
        var p = Path.Combine(d, f);

        var dir = Path.GetDirectoryName(p);
        var name = Path.GetFileName(p);
        var recombined = string.IsNullOrEmpty(dir) ? name : Path.Combine(dir, name);

        return string.Equals(recombined, p, StringComparison.OrdinalIgnoreCase);
    }

    /// <summary>
    /// Property 4: Path.Combine with empty string does not throw.
    /// Combining any valid path with "" should succeed (no exception).
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool PathCombine_EmptySegment_DoesNotThrow(NonEmptyString a)
    {
        var cleaned = Clean(a.Get);

        try
        {
            _ = Path.Combine(cleaned, "");
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Property 5: Path traversal with ".." is resolved consistently by GetFullPath.
    /// Path.GetFullPath(path, base) is deterministic even with .. sequences.
    /// Tests that resolution is stable across multiple calls.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool PathTraversal_GetFullPath_IsDeterministic(NonEmptyString seg)
    {
        var clean = Clean(seg.Get);
        var pathWithDotDot = "base/" + clean + "/../escape.txt";
        var baseDir = Path.GetTempPath();

        var resolved1 = Path.GetFullPath(pathWithDotDot, baseDir);
        var resolved2 = Path.GetFullPath(pathWithDotDot, baseDir);

        // Determinism confirmed if both resolutions are identical
        return resolved1 == resolved2;
    }
}
