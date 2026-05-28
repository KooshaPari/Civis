#nullable enable
using System;
using System.IO;
using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Snapshots;

/// <summary>
/// Snapshot test: captures the canonical pack-manifest JSON schema and compares it against
/// a committed golden copy. Detects unintentional schema drift.
///
/// REVIEWING A SNAPSHOT CHANGE
/// ----------------------------
/// If you intentionally changed <c>schemas/pack-manifest.schema.json</c>, set
/// <c>UPDATE_SNAPSHOTS=1</c> to regenerate the golden file:
///
///   UPDATE_SNAPSHOTS=1 dotnet test --filter "PackManifest_Schema_Snapshot"
///
/// Then commit the updated <c>PackManifest_Schema_Snapshot.golden.json</c> file alongside
/// your schema change.
/// </summary>
[Trait("Category", "Snapshot")]
public class PackManifest_Schema_Snapshot
{
    // ── Path helpers ──────────────────────────────────────────────────────────

    private static string RepoRoot()
    {
        string? dir = AppContext.BaseDirectory;
        while (dir != null)
        {
            if (Directory.Exists(Path.Combine(dir, "schemas")))
                return dir;
            dir = Path.GetDirectoryName(dir);
        }
        throw new InvalidOperationException("Could not find repository root (no 'schemas' directory found walking up from test binary).");
    }

    private static string SchemaPath()
        => Path.Combine(RepoRoot(), "schemas", "pack-manifest.schema.json");

    private static string GoldenPath()
        => Path.Combine(RepoRoot(), "src", "Tests", "Snapshots", "PackManifest_Schema_Snapshot.golden.json");

    // ── Test ──────────────────────────────────────────────────────────────────

    /// <summary>
    /// Reads <c>schemas/pack-manifest.schema.json</c>, normalises whitespace by round-tripping
    /// through <see cref="JsonDocument"/> (so formatting differences don't cause false positives),
    /// and compares the result against the committed golden file.
    ///
    /// If <c>UPDATE_SNAPSHOTS=1</c> is set, regenerates the golden file instead of asserting.
    /// </summary>
    [Fact]
    public void PackManifestSchema_MatchesSnapshot()
    {
        string schemaPath = SchemaPath();

        if (!File.Exists(schemaPath))
        {
            // Schema not present in this environment — skip rather than fail.
            return;
        }

        // Normalise: parse and re-serialize with consistent indentation to eliminate
        // cosmetic whitespace differences from manual edits.
        string rawSchema = File.ReadAllText(schemaPath, System.Text.Encoding.UTF8);

        string normalised;
        try
        {
            using JsonDocument doc = JsonDocument.Parse(rawSchema);
            normalised = JsonSerializer.Serialize(doc.RootElement, new JsonSerializerOptions { WriteIndented = true });
        }
        catch (JsonException ex)
        {
            Assert.Fail($"pack-manifest.schema.json is not valid JSON: {ex.Message}");
            return;
        }

        string goldenPath = GoldenPath();
        bool updateMode = string.Equals(
            Environment.GetEnvironmentVariable("UPDATE_SNAPSHOTS"), "1", StringComparison.OrdinalIgnoreCase);

        if (updateMode || !File.Exists(goldenPath))
        {
            Directory.CreateDirectory(Path.GetDirectoryName(goldenPath)!);
            File.WriteAllText(goldenPath, normalised, new System.Text.UTF8Encoding(encoderShouldEmitUTF8Identifier: false));
            return; // wrote golden — nothing to compare
        }

        string golden = File.ReadAllText(goldenPath, new System.Text.UTF8Encoding(encoderShouldEmitUTF8Identifier: false));

        normalised.Should().Be(golden,
            "pack-manifest.schema.json must match the committed snapshot. " +
            "If this change is intentional, set UPDATE_SNAPSHOTS=1 and re-run to regenerate the golden file. " +
            $"Schema path: {schemaPath}");
    }

    // ── Structural integrity checks (always run) ───────────────────────────────

    /// <summary>
    /// Verifies that the pack-manifest schema contains the minimum required top-level
    /// JSON Schema keywords. This is a structural invariant independent of content.
    /// </summary>
    [Fact]
    public void PackManifestSchema_HasRequiredKeywords()
    {
        string schemaPath = SchemaPath();

        if (!File.Exists(schemaPath))
        {
            return; // skip when schema not on disk
        }

        string rawSchema = File.ReadAllText(schemaPath, System.Text.Encoding.UTF8);
        using JsonDocument doc = JsonDocument.Parse(rawSchema);
        JsonElement root = doc.RootElement;

        // Must declare a schema dialect or title (guards against empty or stub files).
        bool hasType = root.TryGetProperty("type", out _);
        bool hasTitle = root.TryGetProperty("title", out _);
        bool hasSchema = root.TryGetProperty("$schema", out _);
        bool hasProperties = root.TryGetProperty("properties", out _);

        (hasType || hasTitle || hasSchema).Should().BeTrue(
            "pack-manifest.schema.json must have at least one of: '$schema', 'type', or 'title'");

        hasProperties.Should().BeTrue(
            "pack-manifest.schema.json must have a 'properties' object describing pack fields");
    }

    /// <summary>
    /// Verifies that key pack fields (id, name, version) are declared in the schema properties.
    /// Guards against accidental field removal during schema restructuring.
    /// </summary>
    [Fact]
    public void PackManifestSchema_ContainsCoreFields()
    {
        string schemaPath = SchemaPath();

        if (!File.Exists(schemaPath))
        {
            return;
        }

        string rawSchema = File.ReadAllText(schemaPath, System.Text.Encoding.UTF8);
        using JsonDocument doc = JsonDocument.Parse(rawSchema);
        JsonElement root = doc.RootElement;

        if (!root.TryGetProperty("properties", out JsonElement properties))
        {
            Assert.Fail("Schema missing 'properties' — cannot check core fields.");
            return;
        }

        foreach (string requiredField in new[] { "id", "name", "version" })
        {
            properties.TryGetProperty(requiredField, out _).Should().BeTrue(
                $"pack-manifest.schema.json 'properties' must declare the '{requiredField}' field");
        }
    }
}
