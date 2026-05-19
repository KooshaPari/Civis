using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

/// <summary>
/// FsCheck property-based tests for System.Text.Json roundtrip invariants.
/// Verifies that serialization→deserialization preserves value identity and null semantics.
/// </summary>
[Trait("Category", "Property")]
[Trait("Layer", "Serialization")]
public class SerializationFsCheckProperties
{
    /// <summary>
    /// Property 1: Roundtrip preserves int.
    /// Any int x serializes and deserializes to the same value.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Roundtrip_Int(int x)
    {
        var json = JsonSerializer.Serialize(x);
        var deserialized = JsonSerializer.Deserialize<int>(json);
        return deserialized == x;
    }

    /// <summary>
    /// Property 2: Roundtrip preserves bool.
    /// Any bool b serializes and deserializes to the same value.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Roundtrip_Bool(bool b)
    {
        var json = JsonSerializer.Serialize(b);
        var deserialized = JsonSerializer.Deserialize<bool>(json);
        return deserialized == b;
    }

    /// <summary>
    /// Property 3: Roundtrip preserves string (printable ASCII only).
    /// Any printable ASCII string serializes and deserializes to itself.
    /// Filters out control chars (per iter-127 JSON fuzz lesson) to prevent invalid UTF-8 edge cases.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Roundtrip_String_PrintableAscii(NonNull<string> s)
    {
        // Filter to printable ASCII: 0x20 (space) through 0x7E (~)
        var filtered = new string(s.Get.Where(c => c >= 0x20 && c <= 0x7E).ToArray());
        if (filtered.Length == 0) return true; // Empty string is valid

        var json = JsonSerializer.Serialize(filtered);
        var deserialized = JsonSerializer.Deserialize<string>(json);
        return deserialized == filtered;
    }

    /// <summary>
    /// Property 4: Roundtrip preserves dictionary.
    /// Any Dictionary&lt;string, int&gt; with distinct keys roundtrips with equivalent contents.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Roundtrip_Dictionary(NonNull<int[]> keys)
    {
        var distinctKeys = keys.Get.Distinct().Take(10).ToArray();
        if (distinctKeys.Length == 0) return true; // Empty dict is valid

        var dict = distinctKeys.ToDictionary(k => k.ToString(), k => k * 2);
        var json = JsonSerializer.Serialize(dict);
        var deserialized = JsonSerializer.Deserialize<Dictionary<string, int>>(json);

        return deserialized is not null
            && deserialized.Count == dict.Count
            && dict.All(kv => deserialized.ContainsKey(kv.Key) && deserialized[kv.Key] == kv.Value);
    }

    /// <summary>
    /// Property 5: Roundtrip null safety.
    /// Serializing null returns "null" literal; deserializing "null" returns null.
    /// Verifies bidirectional null handling.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Roundtrip_Null_String()
    {
        var nullString = (string?)null;
        var json = JsonSerializer.Serialize(nullString);
        var isNullJsonLiteral = json == "null";
        var deserialized = JsonSerializer.Deserialize<string?>(json);
        var isDeserializedNull = deserialized is null;

        return isNullJsonLiteral && isDeserializedNull;
    }
}
