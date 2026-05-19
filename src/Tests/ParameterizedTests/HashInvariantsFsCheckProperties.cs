using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

[Trait("Category", "Property")]
[Trait("Layer", "Crypto")]
public class HashInvariantsFsCheckProperties
{
    private static byte[] Sha256(byte[] data) => SHA256.HashData(data);

    private static byte[] Hmac(byte[] key, byte[] data)
    {
        using var h = new HMACSHA256(key);
        return h.ComputeHash(data);
    }

    private static string ToHex(byte[] b) => string.Concat(b.Select(x => x.ToString("x2")));

    private static byte[] FromHex(string s)
    {
        var r = new byte[s.Length / 2];
        for (int i = 0; i < r.Length; i++)
        {
            r[i] = byte.Parse(s.Substring(i * 2, 2), System.Globalization.NumberStyles.HexNumber);
        }
        return r;
    }

    /// <summary>
    /// SHA256 is deterministic: same input always produces same hash.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool Sha256_Deterministic(NonNull<byte[]> data)
    {
        var h1 = Sha256(data.Get);
        var h2 = Sha256(data.Get);
        return h1.SequenceEqual(h2);
    }

    /// <summary>
    /// SHA256 exhibits collision resistance: different inputs produce different hashes.
    /// Tested over 100 random input pairs.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool Sha256_Different_Inputs_Produce_Different_Hashes(NonEmptyArray<byte> a, NonEmptyArray<byte> b)
    {
        // Only test when inputs are genuinely different
        if (a.Get.SequenceEqual(b.Get))
            return true;

        return !Sha256(a.Get).SequenceEqual(Sha256(b.Get));
    }

    /// <summary>
    /// HMAC-SHA256 with different keys produces different outputs for the same payload.
    /// Tests key material separation property.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool Hmac_Different_Keys_Same_Payload_Differ(NonEmptyArray<byte> k1, NonEmptyArray<byte> k2, NonNull<byte[]> payload)
    {
        // Only test when keys are genuinely different
        if (k1.Get.SequenceEqual(k2.Get))
            return true;

        return !Hmac(k1.Get, payload.Get).SequenceEqual(Hmac(k2.Get, payload.Get));
    }

    /// <summary>
    /// HMAC-SHA256 with different payloads produces different outputs for the same key.
    /// Tests payload integrity property.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool Hmac_Different_Payloads_Same_Key_Differ(NonEmptyArray<byte> key, NonEmptyArray<byte> p1, NonEmptyArray<byte> p2)
    {
        // Only test when payloads are genuinely different
        if (p1.Get.SequenceEqual(p2.Get))
            return true;

        return !Hmac(key.Get, p1.Get).SequenceEqual(Hmac(key.Get, p2.Get));
    }

    /// <summary>
    /// Hex encoding roundtrips: bytes → hex → bytes preserves original data.
    /// Tests serialization/deserialization symmetry.
    /// </summary>
    [Property(MaxTest = 100, Verbose = true)]
    public bool Hex_Roundtrip_Preserves_Bytes(NonNull<byte[]> bytes)
    {
        var hex = ToHex(bytes.Get);
        var back = FromHex(hex);
        return back.SequenceEqual(bytes.Get);
    }
}
