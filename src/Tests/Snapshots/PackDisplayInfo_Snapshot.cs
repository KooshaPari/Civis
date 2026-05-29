#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Snapshots;

/// <summary>
/// Snapshot test: captures the "display info" surface of the warfare-starwars pack
/// (all fields visible in the F10 mod menu detail pane) and compares it against a
/// committed golden file.
///
/// REVIEWING A SNAPSHOT CHANGE
/// ----------------------------
/// If you intentionally changed pack metadata (display name, description, tags, etc.)
/// run the test with UPDATE_SNAPSHOTS=1 to regenerate the golden file:
///
///   UPDATE_SNAPSHOTS=1 dotnet test --filter "PackDisplayInfo_Snapshot"
///
/// Then commit the updated <c>PackDisplayInfo_Snapshot.golden.json</c> file alongside
/// your pack.yaml changes.
/// </summary>
[Trait("Category", "Snapshot")]
public class PackDisplayInfo_Snapshot
{
    // ── Snapshot options ──────────────────────────────────────────────────────

    private static readonly JsonSerializerOptions JsonOpts = new JsonSerializerOptions
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    // ── Golden-file path (relative to solution root) ──────────────────────────

    private static string GoldenFilePath()
    {
        string? dir = AppContext.BaseDirectory;
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir, "packs")))
                return Path.Combine(dir, "src", "Tests", "Snapshots", "PackDisplayInfo_Snapshot.golden.json");
            dir = Path.GetDirectoryName(dir);
        }
        throw new InvalidOperationException("Could not find repository root.");
    }

    private static string PacksRoot()
    {
        string? dir = AppContext.BaseDirectory;
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir, "packs")))
                return Path.Combine(dir, "packs");
            dir = Path.GetDirectoryName(dir);
        }
        throw new InvalidOperationException("Could not find repository root.");
    }

    // ── Snapshot shape ────────────────────────────────────────────────────────

    /// <summary>
    /// The captured display-info fields for a pack. Only includes fields shown in the
    /// F10 detail pane; internal fields (load_order, schema versions, etc.) are excluded
    /// to avoid snapshot churn from infrastructure changes.
    /// </summary>
    private sealed class PackDisplaySnapshot
    {
        public string Id { get; set; } = "";
        public string Name { get; set; } = "";
        public string Version { get; set; } = "";
        public string Author { get; set; } = "";
        public string Type { get; set; } = "";
        public string? Description { get; set; }
        public List<string>? Tags { get; set; }
        public string? Classification { get; set; }
        public string? License { get; set; }
        public string? HomepageUrl { get; set; }
        public string? GithubUrl { get; set; }
        public List<string> DependsOn { get; set; } = new();
        public List<string> ConflictsWith { get; set; } = new();
    }

    private static PackDisplaySnapshot ToSnapshot(PackManifest m) => new PackDisplaySnapshot
    {
        Id = m.Id,
        Name = m.Name,
        Version = m.Version,
        Author = m.Author,
        Type = m.Type,
        Description = m.Description,
        Tags = m.Tags,
        Classification = m.Classification,
        License = m.License,
        HomepageUrl = m.HomepageUrl,
        GithubUrl = m.GithubUrl,
        DependsOn = m.DependsOn,
        ConflictsWith = m.ConflictsWith,
    };

    // ── Test ──────────────────────────────────────────────────────────────────

    /// <summary>
    /// Loads the warfare-starwars pack manifest, captures its display-info fields as JSON,
    /// and compares them against the committed golden file.
    ///
    /// If <c>UPDATE_SNAPSHOTS=1</c> is set, regenerates the golden file instead of asserting.
    /// </summary>
    [Fact]
    public void StarWarsPack_DisplayInfo_MatchesSnapshot()
    {
        string packsRoot = PacksRoot();
        string packDir = Path.Combine(packsRoot, "warfare-starwars");
        string manifestPath = Path.Combine(packDir, "pack.yaml");

        if (!File.Exists(manifestPath))
        {
            // Pack not present in this environment — skip rather than fail.
            return;
        }

        var packLoader = new PackLoader();
        PackManifest manifest = packLoader.LoadFromFile(manifestPath);
        PackDisplaySnapshot snapshot = ToSnapshot(manifest);

        string actualJson = JsonSerializer.Serialize(snapshot, JsonOpts);
        string goldenPath = GoldenFilePath();

        bool updateMode = string.Equals(
            Environment.GetEnvironmentVariable("UPDATE_SNAPSHOTS"), "1", StringComparison.OrdinalIgnoreCase);

        if (updateMode || !File.Exists(goldenPath))
        {
            Directory.CreateDirectory(Path.GetDirectoryName(goldenPath)!);
            File.WriteAllText(goldenPath, actualJson, new System.Text.UTF8Encoding(encoderShouldEmitUTF8Identifier: false));
            return; // wrote golden — nothing to compare
        }

        string goldenJson = File.ReadAllText(goldenPath);

        actualJson.Should().Be(goldenJson,
            "warfare-starwars pack display info must match the committed snapshot. " +
            "If this change is intentional, set UPDATE_SNAPSHOTS=1 and re-run the test to regenerate the golden file.");
    }
}

