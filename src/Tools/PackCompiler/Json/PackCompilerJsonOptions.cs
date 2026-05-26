#nullable enable
using System.Text.Json;

namespace DINOForge.Tools.PackCompiler.Json;

/// <summary>
/// Project-static <see cref="JsonSerializerOptions"/> holders for the PackCompiler tool.
/// Splits FFI-targeted output (snake_case for Go/Rust/Thunderstore parity) from the
/// in-process pipeline I/O (default casing for symmetric write→read of intermediate
/// artifacts). Pattern #109: inline-JsonSerializerOptions consolidation.
/// </summary>
public static class PackCompilerJsonOptions
{
    /// <summary>
    /// Snake-case lower options for cross-language FFI artifacts (Thunderstore manifest, Go/Rust
    /// inter-op payloads). Use only for files consumed by non-.NET tooling that expects
    /// snake_case keys.
    /// </summary>
    public static JsonSerializerOptions GoFfi { get; } = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };

    /// <summary>
    /// Indented default-casing options for intermediate pipeline artifacts that are written and
    /// later re-read by PackCompiler itself (e.g. <c>imported/*.json</c>, <c>optimized/*.json</c>).
    /// These types (<c>ImportedAsset</c>, <c>OptimizedAsset</c>) use plain PascalCase properties,
    /// so the writer must NOT apply a naming policy or downstream <c>JsonSerializer.Deserialize</c>
    /// calls will silently produce defaulted instances.
    /// </summary>
    public static JsonSerializerOptions Indented { get; } = new()
    {
        WriteIndented = true,
    };
}
