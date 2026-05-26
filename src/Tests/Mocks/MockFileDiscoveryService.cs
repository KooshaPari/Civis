#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock implementation of <see cref="IFileDiscoveryService"/> for unit testing.
/// Provides configurable file and directory structure with call tracking.
/// NOT thread-safe; intended for single-threaded test fixtures only.
/// </summary>
public class MockFileDiscoveryService : IFileDiscoveryService
{
    private readonly Dictionary<string, string> _files;
    private readonly HashSet<string> _directories;
    private readonly List<string> _customExclusions;
    private readonly List<string> _defaultExclusions = new()
    {
        "archived/", "export/", "generated/", "bin/", "obj/", "node_modules/"
    };

    /// <summary>
    /// Number of times <see cref="GetFiles(string, string, SearchOption)"/> has been called.
    /// </summary>
    public int GetFilesCount { get; set; }

    /// <summary>
    /// Number of times <see cref="GetDirectories"/> has been called.
    /// </summary>
    public int GetDirectoriesCount { get; set; }

    /// <summary>
    /// Number of times <see cref="DiscoverPackDirectories"/> has been called.
    /// </summary>
    public int DiscoverPackDirectoriesCount { get; set; }

    /// <summary>
    /// Number of times <see cref="AddExclusion"/> has been called.
    /// </summary>
    public int AddExclusionCount { get; set; }

    /// <summary>
    /// Creates a new mock file discovery service with an empty file system.
    /// </summary>
    public MockFileDiscoveryService()
    {
        _files = new Dictionary<string, string>(StringComparer.Ordinal);
        _directories = new HashSet<string>(StringComparer.Ordinal);
        _customExclusions = new List<string>();
    }

    /// <summary>
    /// Adds a file to the mock file system.
    /// </summary>
    /// <param name="path">The file path.</param>
    /// <param name="content">The file content (not used by GetFiles but tracked).</param>
    public void AddFile(string path, string content)
    {
        _files[path] = content;
        var directory = Path.GetDirectoryName(path);
        if (!string.IsNullOrEmpty(directory))
            _directories.Add(directory);
    }

    /// <summary>
    /// Adds a directory to the mock file system.
    /// </summary>
    /// <param name="path">The directory path.</param>
    public void AddDirectory(string path)
    {
        _directories.Add(path);
    }

    public string[] GetFiles(string directory, string searchPattern, SearchOption searchOption = SearchOption.TopDirectoryOnly)
    {
        GetFilesCount++;
        var result = _files.Keys
            .Where(path => path.StartsWith(directory, StringComparison.Ordinal))
            .Where(path => MatchesPattern(Path.GetFileName(path), searchPattern))
            .ToArray();

        return result;
    }

    public string[] GetDirectories(string directory, SearchOption searchOption = SearchOption.TopDirectoryOnly)
    {
        GetDirectoriesCount++;
        return _directories
            .Where(dir => dir.StartsWith(directory, StringComparison.Ordinal))
            .ToArray();
    }

    public string[] GetFiles(string directory, string[] searchPatterns, SearchOption searchOption = SearchOption.TopDirectoryOnly)
    {
        GetFilesCount++;
        var result = _files.Keys
            .Where(path => path.StartsWith(directory, StringComparison.Ordinal))
            .Where(path => searchPatterns.Any(pattern => MatchesPattern(Path.GetFileName(path), pattern)))
            .ToArray();

        return result;
    }

    public string[] DiscoverPackDirectories(string rootDirectory)
    {
        DiscoverPackDirectoriesCount++;
        return _directories
            .Where(dir => dir.StartsWith(rootDirectory, StringComparison.Ordinal))
            .Where(dir => _files.Keys.Any(f => f.StartsWith(dir, StringComparison.Ordinal) && f.EndsWith("pack.yaml")))
            .ToArray();
    }

    public IReadOnlyList<string> DefaultExclusions => _defaultExclusions.AsReadOnly();

    public void AddExclusion(string pattern)
    {
        AddExclusionCount++;
        _customExclusions.Add(pattern);
    }

    public void RemoveExclusion(string pattern)
    {
        _customExclusions.Remove(pattern);
    }

    public void ClearExclusions()
    {
        _customExclusions.Clear();
    }

    public void ResetToDefaults()
    {
        _customExclusions.Clear();
    }

    private static bool MatchesPattern(string filename, string pattern)
    {
        if (pattern == "*")
            return true;
        if (pattern.StartsWith("*."))
            return filename.EndsWith(pattern.Substring(1), StringComparison.Ordinal);
        return filename.Equals(pattern, StringComparison.Ordinal);
    }
}
