using FluentAssertions;
using FsCheck;
using FsCheck.Fluent;
using FsCheck.Xunit;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

/// <summary>
/// FsCheck property-based tests for System.Text.Json roundtrip invariants.
/// Verifies that serialization→deserialization preserves value identity and null semantics.
///
/// Iter-#594 hardening (filter-quality sweep, audit ac455d19):
/// Previously the printable-ASCII string property used a post-filter
/// `s.Get.Where(c => c >= 0x20 &amp;&amp; c &lt;= 0x7E)` that silently discarded any
/// generator output containing control chars — masquerading 100× discards as 100×
/// coverage. Now uses an upfront <c>Gen.Choose(0x20, 0x7E)</c> char generator wrapped
/// via <c>Prop.ForAll</c> so EVERY iteration exercises the property over a genuinely
/// printable-ASCII string. Same convention as <c>PropertyTests.cs:35-40</c>.
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
    ///
    /// Filter-quality (iter-#594): instead of generating arbitrary strings and
    /// filtering out non-printable chars (which discards ~100% of FsCheck's
    /// natural string output and degenerates to testing only empty/short strings),
    /// we compose the generator from <c>Gen.Choose(0x20, 0x7E)</c> directly so
    /// every iteration produces a non-degenerate printable-ASCII string of
    /// length 1..50.
    /// </summary>
    [Property(MaxTest = 100)]
    public Property Roundtrip_String_PrintableAscii()
    {
        // Upfront valid-domain generator: printable ASCII string of length 1..50.
        Gen<char> printableAsciiCharGen = Gen.Choose(0x20, 0x7E).Select(i => (char)i);
        Gen<string> printableAsciiStringGen =
            from len in Gen.Choose(1, 50)
            from arr in Gen.ArrayOf<char>(printableAsciiCharGen, len)
            select new string(arr);

        return Prop.ForAll(printableAsciiStringGen.ToArbitrary(), s =>
        {
            var json = JsonSerializer.Serialize(s);
            var deserialized = JsonSerializer.Deserialize<string>(json);
            return deserialized == s;
        });
    }

    /// <summary>
    /// Property 4: Roundtrip preserves dictionary.
    /// Any Dictionary&lt;string, int&gt; with distinct keys roundtrips with equivalent contents.
    ///
    /// Filter-quality (iter-#594): instead of <c>NonNull&lt;int[]&gt;</c> + a runtime
    /// <c>.Distinct().Take(10)</c> filter (which silently collapses many iterations
    /// to small dicts), we generate the dictionary size directly via
    /// <c>Gen.Choose(1, 10)</c> and a key-pool to guarantee distinctness upfront.
    /// </summary>
    [Property(MaxTest = 100)]
    public Property Roundtrip_Dictionary()
    {
        // Upfront valid-domain generator: dict of 1..10 distinct int keys.
        Gen<Dictionary<int, int>> dictGen =
            Gen.Choose(1, 10).SelectMany(count =>
                Gen.Choose(0, 1_000_000).Four().SelectMany(_ =>
                    // Build a unique key set: i * 7919 + offset gives distinct keys.
                    Gen.Choose(0, 1000).Select(offset =>
                        Enumerable.Range(0, count)
                                  .Select(i => i * 7919 + offset)
                                  .ToDictionary(k => k, k => k * 2))));

        return Prop.ForAll(dictGen.ToArbitrary(), intDict =>
        {
            var dict = intDict.ToDictionary(kv => kv.Key.ToString(), kv => kv.Value);
            var json = JsonSerializer.Serialize(dict);
            var deserialized = JsonSerializer.Deserialize<Dictionary<string, int>>(json);

            return deserialized is not null
                && deserialized.Count == dict.Count
                && dict.All(kv => deserialized.ContainsKey(kv.Key) && deserialized[kv.Key] == kv.Value);
        });
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
