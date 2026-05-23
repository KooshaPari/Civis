#nullable enable
using System;
using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json.Linq;

namespace DINOForge.Bridge.Client;

/// <summary>
/// Outcome of a single bridge_receipt verification (Wave 2 Phase 4b).
/// </summary>
/// <remarks>
/// In <see cref="VerificationMode.WarnOnly"/> mode the verifier always returns
/// <c>Valid=true</c> for backward compatibility with fixtures predating Phase
/// 4a, but <see cref="Reason"/> still records the underlying mismatch so the
/// caller can log it. In <see cref="VerificationMode.Strict"/> mode the same
/// mismatch becomes a thrown <see cref="GameClientException"/>.
/// </remarks>
public readonly struct VerificationResult
{
    /// <summary>Whether the receipt is considered acceptable under the current mode.</summary>
    public bool Valid { get; }

    /// <summary>The mode the verification was performed under.</summary>
    public VerificationMode Mode { get; }

    /// <summary>Diagnostic reason — empty on success, populated on mismatch.</summary>
    public string Reason { get; }

    /// <summary>Initializes a new <see cref="VerificationResult"/>.</summary>
    public VerificationResult(bool valid, VerificationMode mode, string reason)
    {
        Valid = valid;
        Mode = mode;
        Reason = reason ?? "";
    }
}

/// <summary>
/// Recomputes the HMAC over a JSON-RPC response's <c>bridge_receipt</c> using
/// the cached per-session key, and enforces world_frame monotonicity. Mirrors
/// the server signing logic in <c>SessionHmac.ComputeHmac</c>.
/// </summary>
/// <remarks>
/// <para>
/// CRITICAL: the canonical receipt format below MUST stay byte-identical to
/// the server's input in <c>SessionHmac.ComputeHmac</c>:
/// <c>{"state_sha256":"...","timestamp":"...","world_frame":N}</c> with keys
/// sorted alphabetically and no whitespace. A future refactor should hoist
/// this builder into a shared netstandard project so client and server cannot
/// drift.
/// </para>
/// </remarks>
public static class BridgeReceiptVerifier
{
    /// <summary>
    /// Verifies a response's <see cref="JsonRpcResponse.BridgeReceipt"/>.
    /// </summary>
    /// <param name="response">The deserialized JSON-RPC response.</param>
    /// <param name="cache">Session key cache populated by Connect handshake.</param>
    /// <param name="mode">Verification mode (Off / WarnOnly / Strict).</param>
    /// <param name="lastFrame">
    /// Last seen world frame for monotonicity enforcement. Updated to the
    /// current receipt's frame when verification succeeds.
    /// </param>
    /// <param name="isHandshake">
    /// True when verifying a Connect/handshake response (frame=0 sentinel is
    /// permitted only in this case).
    /// </param>
    public static VerificationResult Verify(
        JsonRpcResponse response,
        SessionKeyCache cache,
        VerificationMode mode,
        ref long lastFrame,
        bool isHandshake)
    {
        if (response == null) throw new ArgumentNullException(nameof(response));
        if (cache == null) throw new ArgumentNullException(nameof(cache));

        if (mode == VerificationMode.Off)
        {
            return new VerificationResult(true, mode, "");
        }

        BridgeReceipt? receipt = response.BridgeReceipt;
        if (receipt == null)
        {
            // Phase 4b: missing receipt is tolerated under WarnOnly (existing
            // fixtures and the Connect response itself); Strict rejects.
            string reason = "missing bridge_receipt";
            return new VerificationResult(mode != VerificationMode.Strict, mode, reason);
        }

        if (!cache.TryGet(receipt.SessionId, out var key) || key == null)
        {
            // Handshake responses may arrive before the session key has been cached
            // by the client (race between verification and PerformHandshakeAsync).
            // In this case, tolerate the missing key even in Strict mode.
            if (isHandshake)
            {
                return new VerificationResult(true, mode, "");
            }
            return new VerificationResult(mode != VerificationMode.Strict, mode,
                $"unknown session_id {receipt.SessionId}");
        }

        // 1) Recompute the payload hash and compare against state_sha256.
        // sync-over-async-unavoidable: JsonRpcResponse.Result is a JToken property (Newtonsoft.Json.Linq), not Task.Result. Analyzer false positive.
        string canonicalPayload = CanonicalJson.Canonicalize(response.Result);
        string recomputedPayloadHash = Sha256Hex(Encoding.UTF8.GetBytes(canonicalPayload));
        if (!StringComparer.Ordinal.Equals(recomputedPayloadHash, receipt.StateSha256Hex))
        {
            return new VerificationResult(mode != VerificationMode.Strict, mode,
                $"state_sha256 mismatch (recomputed={recomputedPayloadHash} receipt={receipt.StateSha256Hex})");
        }

        // 2) Recompute the receipt HMAC over the canonical (state, timestamp, frame) triple.
        // SECURITY (#614): use constant-time comparison on the decoded HMAC bytes to
        // avoid a hex-prefix timing side-channel. StringComparer.Ordinal.Equals short-
        // circuits on the first differing character and would leak the byte index of
        // the first mismatch to a remote attacker who can measure response latency.
        string expectedHmac = ComputeReceiptHmac(key, receipt.TimestampUtc, receipt.WorldFrame, receipt.StateSha256Hex);
        if (!ConstantTimeHexEquals(expectedHmac, receipt.HmacHex))
        {
            return new VerificationResult(mode != VerificationMode.Strict, mode, "hmac mismatch");
        }

        // 3) Frame monotonicity: world_frame must strictly advance, except for
        // the handshake (Connect) where frame=0 is the documented sentinel.
        if (!isHandshake)
        {
            if (receipt.WorldFrame == 0)
            {
                return new VerificationResult(mode != VerificationMode.Strict, mode,
                    "world_frame=0 outside Connect handshake");
            }
            if (receipt.WorldFrame <= lastFrame)
            {
                return new VerificationResult(mode != VerificationMode.Strict, mode,
                    $"world_frame regressed ({receipt.WorldFrame} <= last {lastFrame})");
            }
            lastFrame = receipt.WorldFrame;
        }

        return new VerificationResult(true, mode, "");
    }

    /// <summary>
    /// Mirrors the server-side `SessionHmac.ComputeHmac` implementation:
    /// canonical receipt = <c>{"state_sha256":"...","timestamp":"...","world_frame":N}</c>
    /// (keys sorted alphabetically, no whitespace), HMAC-SHA256 with the
    /// session key, lowercase hex output.
    /// </summary>
    public static string ComputeReceiptHmac(byte[] sessionKey, string timestampUtc, long worldFrame, string stateSha256Hex)
    {
        if (sessionKey == null) throw new ArgumentNullException(nameof(sessionKey));
        if (timestampUtc == null) throw new ArgumentNullException(nameof(timestampUtc));
        if (stateSha256Hex == null) throw new ArgumentNullException(nameof(stateSha256Hex));

        // CRITICAL: keep this concatenation byte-identical to SessionHmac.ComputeHmac
        // server-side. Order: state_sha256, timestamp, world_frame (alphabetical).
        string canonical = "{\"state_sha256\":\"" + stateSha256Hex
            + "\",\"timestamp\":\"" + timestampUtc
            + "\",\"world_frame\":" + worldFrame.ToString(CultureInfo.InvariantCulture)
            + "}";

        byte[] bytes = Encoding.UTF8.GetBytes(canonical);
        using var hmac = new HMACSHA256(sessionKey);
        return ToHexLower(hmac.ComputeHash(bytes));
    }

    private static string Sha256Hex(byte[] input)
    {
        using var sha = SHA256.Create();
        return ToHexLower(sha.ComputeHash(input));
    }

    /// <summary>
    /// Constant-time comparison of two lowercase hex strings representing
    /// equal-length byte digests (e.g. HMAC-SHA256 outputs). Decodes both
    /// strings to bytes and XOR-accumulates differences to avoid the early-
    /// exit timing side-channel inherent in <see cref="string.Equals(string, string)"/>.
    /// </summary>
    /// <remarks>
    /// netstandard2.0 polyfill — <c>Convert.FromHexString</c> and
    /// <c>CryptographicOperations.FixedTimeEquals</c> are .NET 5+ only.
    /// The length-mismatch branch is acceptable per RFC 6234 / dotnet runtime
    /// reference impl: HMAC outputs have a fixed, public length, so the length
    /// itself is not secret.
    /// </remarks>
    internal static bool ConstantTimeHexEquals(string? a, string? b)
    {
        if (a == null || b == null) return false;
        if (a.Length != b.Length) return false;
        if ((a.Length & 1) != 0) return false; // malformed hex — reject

        int diff = 0;
        for (int i = 0; i < a.Length; i++)
        {
            int da = HexNibble(a[i]);
            int db = HexNibble(b[i]);
            // OR malformed-nibble flag into diff so a non-hex char forces inequality
            // without an early return that would leak position.
            diff |= (da | db) >> 8;
            diff |= (da ^ db) & 0xFF;
        }
        return diff == 0;
    }

    /// <summary>
    /// Returns 0..15 for valid hex chars, or a value with bit 8 set (>=256)
    /// for invalid input. Branchless so the caller can fold the validity flag
    /// into its accumulator without leaking position.
    /// </summary>
    private static int HexNibble(char c)
    {
        if (c >= '0' && c <= '9') return c - '0';
        if (c >= 'a' && c <= 'f') return c - 'a' + 10;
        if (c >= 'A' && c <= 'F') return c - 'A' + 10;
        return 0x100; // sentinel: invalid
    }

    private static string ToHexLower(byte[] bytes)
    {
        const string hex = "0123456789abcdef";
        var sb = new StringBuilder(bytes.Length * 2);
        for (int i = 0; i < bytes.Length; i++)
        {
            byte b = bytes[i];
            sb.Append(hex[b >> 4]);
            sb.Append(hex[b & 0xF]);
        }
        return sb.ToString();
    }
}
