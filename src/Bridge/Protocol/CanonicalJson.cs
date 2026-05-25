#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Numerics;
using System.Linq;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Org.Webpki.JsonCanonicalizer;

namespace DINOForge.Bridge.Protocol;

/// <summary>
/// Deterministic JSON canonicalizer shared by both <c>GameBridgeServer</c>
/// (server-side state hash) and <c>BridgeReceiptVerifier</c> (client-side
/// receipt verification).
/// </summary>
/// <remarks>
/// <para>
/// This implementation delegates structured JSON canonicalization to the
/// upstream cyberphone <see cref="JsonCanonicalizer"/> package so object-key
/// ordering, float formatting, and string escaping stay aligned with the RFC
/// 8785 reference behavior.
/// </para>
/// <para>
/// The legacy null-input contract remains intact because the bridge protocol
/// hashes <c>"null"</c> for empty payloads.
/// </para>
/// </remarks>
public static class CanonicalJson
{
    /// <summary>
    /// Returns the canonical JSON serialization of <paramref name="token"/>.
    /// Returns the literal string <c>"null"</c> for a null input.
    /// </summary>
    public static string Canonicalize(JToken? token)
    {
        if (token == null)
        {
            return "null";
        }

        if (token is JValue value && IsNonFiniteValue(value))
        {
            throw new InvalidOperationException("NaN/Infinity values cannot be canonicalized.");
        }

        switch (token.Type)
        {
            case JTokenType.Object:
            case JTokenType.Array:
                return ContainsUnsafeNumericValue(token)
                    ? WriteCanonicalJson(token)
                    : new JsonCanonicalizer(token.ToString(Formatting.None)).GetEncodedString();
            case JTokenType.Float:
                return CanonicalizeFloat((JValue)token);
            default:
                return token.ToString(Formatting.None);
        }
    }

    private static bool IsNonFiniteValue(JValue value)
    {
        object? raw = value.Value;

        if (raw is double doubleValue)
        {
            return double.IsNaN(doubleValue) || double.IsInfinity(doubleValue);
        }

        if (raw is float floatValue)
        {
            return float.IsNaN(floatValue) || float.IsInfinity(floatValue);
        }

        string? rendered = raw?.ToString();
        return rendered == "NaN" || rendered == "Infinity" || rendered == "-Infinity";
    }

    private static string CanonicalizeFloat(JValue value)
    {
        object? raw = value.Value;

        if (raw is double doubleValue)
        {
            return IsNegativeZero(doubleValue) ? "0" : value.ToString(Formatting.None);
        }

        if (raw is float floatValue)
        {
            return IsNegativeZero(floatValue) ? "0" : value.ToString(Formatting.None);
        }

        return value.ToString(Formatting.None);
    }

    private static string WriteCanonicalJson(JToken token)
    {
        using StringWriter writer = new StringWriter(CultureInfo.InvariantCulture);
        WriteCanonicalJson(token, writer);
        return writer.ToString();
    }

    private static void WriteCanonicalJson(JToken token, TextWriter writer)
    {
        switch (token.Type)
        {
            case JTokenType.Object:
                writer.Write('{');
                bool firstProperty = true;
                IList<JProperty> properties = ((JObject)token).Properties().OrderBy(property => property.Name, StringComparer.Ordinal).ToList();
                foreach (JProperty property in properties)
                {
                    if (!firstProperty)
                    {
                        writer.Write(',');
                    }

                    firstProperty = false;
                    writer.Write(JsonConvert.ToString(property.Name));
                    writer.Write(':');
                    WriteCanonicalJson(property.Value, writer);
                }

                writer.Write('}');
                return;
            case JTokenType.Array:
                writer.Write('[');
                bool firstItem = true;
                foreach (JToken item in (JArray)token)
                {
                    if (!firstItem)
                    {
                        writer.Write(',');
                    }

                    firstItem = false;
                    WriteCanonicalJson(item, writer);
                }

                writer.Write(']');
                return;
            case JTokenType.Float:
                writer.Write(WriteCanonicalFloat((JValue)token));
                return;
            default:
                writer.Write(token.ToString(Formatting.None));
                return;
        }
    }

    private static string WriteCanonicalFloat(JValue value)
    {
        object? raw = value.Value;

        if (raw is double doubleValue)
        {
            ValidateFinite(doubleValue);
            return IsNegativeZero(doubleValue) ? "0" : doubleValue.ToString("R", CultureInfo.InvariantCulture);
        }

        if (raw is float floatValue)
        {
            ValidateFinite(floatValue);
            return IsNegativeZero(floatValue) ? "0" : floatValue.ToString("R", CultureInfo.InvariantCulture);
        }

        if (raw is decimal decimalValue)
        {
            return decimalValue.ToString(CultureInfo.InvariantCulture);
        }

        return value.ToString(Formatting.None);
    }

    private static void ValidateFinite(double value)
    {
        if (double.IsNaN(value) || double.IsInfinity(value))
        {
            throw new InvalidOperationException("NaN/Infinity values cannot be canonicalized.");
        }
    }

    private static void ValidateFinite(float value)
    {
        if (float.IsNaN(value) || float.IsInfinity(value))
        {
            throw new InvalidOperationException("NaN/Infinity values cannot be canonicalized.");
        }
    }

    private static bool ContainsUnsafeNumericValue(JToken token)
    {
        if (token is JValue value)
        {
            object? raw = value.Value;

            if (raw is BigInteger)
            {
                return true;
            }

            if (raw is double doubleValue)
            {
                return double.IsNaN(doubleValue) || double.IsInfinity(doubleValue);
            }

            if (raw is float floatValue)
            {
                return float.IsNaN(floatValue) || float.IsInfinity(floatValue);
            }
        }

        foreach (JToken child in token.Children())
        {
            if (ContainsUnsafeNumericValue(child))
            {
                return true;
            }
        }

        return false;
    }

    private static readonly long NegativeZeroDoubleBits = BitConverter.DoubleToInt64Bits(-0.0);

    private static readonly int NegativeZeroFloatBits =
        BitConverter.ToInt32(BitConverter.GetBytes(-0.0f), 0);

    private static bool IsNegativeZero(double value)
    {
        return BitConverter.DoubleToInt64Bits(value) == NegativeZeroDoubleBits;
    }

    private static bool IsNegativeZero(float value)
    {
        return BitConverter.ToInt32(BitConverter.GetBytes(value), 0) == NegativeZeroFloatBits;
    }
}
