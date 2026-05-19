#nullable enable
using System;
using System.Globalization;
using System.Linq;
using System.Numerics;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Bridge.Protocol;

/// <summary>
/// Deterministic JSON canonicalizer shared by both <c>GameBridgeServer</c>
/// (server-side state hash) and <c>BridgeReceiptVerifier</c> (client-side
/// receipt verification).
/// </summary>
/// <remarks>
/// <para>
/// CRITICAL: this is the single source of truth for canonical JSON across
/// the bridge wire. Both producers and consumers MUST hash the byte-identical
/// output of <see cref="Canonicalize(JToken)"/> — a single divergent byte
/// breaks every receipt across the wire.
/// </para>
/// <para>
/// Rules (mirror RFC 8785 JCS for the parts that matter to us):
/// <list type="bullet">
///   <item>UTF-8 output (caller is responsible for encoding the returned <c>string</c>)</item>
///   <item>Object keys sorted by ordinal Unicode code point (<see cref="StringComparer.Ordinal"/>)</item>
///   <item>No insignificant whitespace</item>
///   <item>Integers as decimal (invariant culture); BigInteger fallback for values that don't fit in <see cref="long"/></item>
///   <item>Floats via <c>"R"</c> round-trip format; <c>NaN</c> / <c>Infinity</c> are rejected (not valid JSON per RFC 8259)</item>
///   <item>Strings via <see cref="JsonConvert.ToString(string)"/></item>
///   <item>Null input → literal <c>"null"</c> so the SHA-256 of "no payload" is well-defined</item>
/// </list>
/// </para>
/// <para>
/// History: lifted from the per-side mirrors in <c>GameBridgeServer.CanonicalizeJson</c>
/// and <c>Bridge.Client.CanonicalJson</c> in v0.24.x. Drift between the two
/// copies was the canonical Phase-4b risk; this shared lib closes it.
/// </para>
/// </remarks>
public static class CanonicalJson
{
    /// <summary>
    /// Returns the canonical JSON serialization of <paramref name="token"/>.
    /// Returns the literal string <c>"null"</c> for a null input — matching
    /// the legacy server behavior so the SHA-256 hash of "no payload" is
    /// well-defined and compatible with existing receipts on the wire.
    /// </summary>
    /// <exception cref="InvalidOperationException">
    /// Thrown when a <see cref="JTokenType.Float"/> token contains <c>NaN</c>
    /// or <c>±Infinity</c>; canonical JSON does not have a representation
    /// for these values (RFC 8259 §6).
    /// </exception>
    public static string Canonicalize(JToken? token)
    {
        if (token == null) return "null";
        return CanonicalizeToken(token);
    }

    private static string CanonicalizeToken(JToken token)
    {
        switch (token.Type)
        {
            case JTokenType.Object:
                {
                    var obj = (JObject)token;
                    var sb = new StringBuilder("{");
                    bool first = true;
                    foreach (var prop in obj.Properties()
                                            .OrderBy(p => p.Name, StringComparer.Ordinal))
                    {
                        if (!first) sb.Append(',');
                        first = false;
                        sb.Append(JsonConvert.ToString(prop.Name));
                        sb.Append(':');
                        sb.Append(CanonicalizeToken(prop.Value));
                    }
                    sb.Append('}');
                    return sb.ToString();
                }
            case JTokenType.Array:
                {
                    var arr = (JArray)token;
                    var sb = new StringBuilder("[");
                    bool first = true;
                    foreach (var item in arr)
                    {
                        if (!first) sb.Append(',');
                        first = false;
                        sb.Append(CanonicalizeToken(item));
                    }
                    sb.Append(']');
                    return sb.ToString();
                }
            case JTokenType.Null:
                return "null";
            case JTokenType.Boolean:
                return token.Value<bool>() ? "true" : "false";
            case JTokenType.Integer:
                // Most integers fit in long. For values outside long range
                // (e.g. uint64 max+1, BigInteger literals), Newtonsoft surfaces
                // them as JTokenType.Integer backed by a BigInteger; Value<long>()
                // routes through Convert.ChangeType which throws
                // InvalidCastException ("Object must implement IConvertible")
                // for BigInteger. Some Newtonsoft paths surface OverflowException
                // instead. Fall back to BigInteger so canonicalization stays lossless.
                try
                {
                    return token.Value<long>().ToString(CultureInfo.InvariantCulture);
                }
                catch (OverflowException)
                {
                    return token.Value<BigInteger>().ToString(CultureInfo.InvariantCulture);
                }
                catch (InvalidCastException)
                {
                    // JValue.Value is BigInteger and ChangeType<long> can't bridge it.
                    var raw = ((JValue)token).Value;
                    if (raw is BigInteger big)
                    {
                        return big.ToString(CultureInfo.InvariantCulture);
                    }
                    // Last-resort: stringify the underlying value invariantly.
                    return Convert.ToString(raw, CultureInfo.InvariantCulture) ?? "0";
                }
            case JTokenType.Float:
                {
                    double d = token.Value<double>();
                    if (double.IsNaN(d) || double.IsInfinity(d))
                    {
                        throw new InvalidOperationException(
                            "Canonical JSON does not support NaN/Infinity");
                    }
                    return d.ToString("R", CultureInfo.InvariantCulture);
                }
            case JTokenType.String:
            case JTokenType.Date:
            case JTokenType.Guid:
            case JTokenType.Uri:
            case JTokenType.TimeSpan:
                return JsonConvert.ToString(token.Value<string>() ?? "");
            default:
                return JsonConvert.ToString(token.ToString(Formatting.None));
        }
    }
}
