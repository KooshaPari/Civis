#nullable enable
using System;
using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json.Linq;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Pure-C# helper that constructs a <see cref="BridgeReceipt"/> for a given
    /// JSON-RPC response payload (Wave 2 Phase 4a wiring — task #223). Extracted
    /// from <see cref="GameBridgeServer"/> so it can be unit-tested without the
    /// Unity / BepInEx runtime hosting.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The builder is stateless. Callers (the bridge server) own the
    /// <see cref="SessionHmac"/> instance and a frame-counter source; this class
    /// only assembles the deterministic <c>state_sha256</c> + signed receipt
    /// triple defined in <c>docs/design/2026-04-25-bridge-hmac-phase4.md</c>.
    /// </para>
    /// <para>
    /// CRITICAL: <see cref="BuildReceipt"/> uses
    /// <see cref="CanonicalJson.Canonicalize(JToken)"/> for the payload-hash
    /// input. Both producer (here) and consumer
    /// (<c>BridgeReceiptVerifier</c>) MUST canonicalize via the shared library
    /// or HMACs will fail to verify across the wire.
    /// </para>
    /// </remarks>
    public static class BridgeReceiptBuilder
    {
        /// <summary>
        /// Builds a <see cref="BridgeReceipt"/> covering <paramref name="result"/>
        /// signed under <paramref name="session"/>.
        /// </summary>
        /// <param name="session">The active per-session HMAC holder.</param>
        /// <param name="result">
        /// The JSON-RPC <c>result</c> payload to attest. May be null — in which
        /// case the canonical input is the literal string <c>"null"</c> per
        /// <see cref="CanonicalJson.Canonicalize(JToken)"/> contract.
        /// </param>
        /// <param name="worldFrame">
        /// Monotonic ECS world-frame counter at the time the response was prepared.
        /// Pass 0 for handshake responses or when the world is not yet ready
        /// (Phase 4a sentinel; Phase 4c will enforce strict monotonicity).
        /// </param>
        /// <param name="timestampUtc">
        /// Wall-clock timestamp source. Defaults to <see cref="DateTime.UtcNow"/>
        /// when null; tests may inject a fixed value for determinism.
        /// </param>
        /// <param name="timeProvider">
        /// Optional TimeProvider for testing. When <paramref name="timestampUtc"/> is null,
        /// defaults to <see cref="TimeProvider.System"/>.
        /// </param>
        /// <returns>
        /// A populated receipt; <see cref="BridgeReceipt.HmacHex"/> is signed
        /// over the canonical <c>(state_sha256, timestamp, world_frame)</c>
        /// triple via <see cref="SessionHmac.ComputeHmac"/>.
        /// </returns>
        /// <exception cref="ArgumentNullException">If <paramref name="session"/> is null.</exception>
        public static BridgeReceipt BuildReceipt(
            SessionHmac session,
            JToken? result,
            long worldFrame,
            DateTime? timestampUtc = null,
            TimeProvider? timeProvider = null)
        {
            if (session == null) throw new ArgumentNullException(nameof(session));

            string stateSha256 = ComputePayloadHash(result);
            var provider = timeProvider ?? TimeProvider.System;
            DateTime ts_val = timestampUtc ?? provider.GetUtcNow().UtcDateTime;
            string ts = FormatTimestamp(ts_val);
            string hmacHex = session.ComputeHmac(ts, worldFrame, stateSha256);

            return new BridgeReceipt
            {
                SessionId = session.SessionId,
                TimestampUtc = ts,
                WorldFrame = worldFrame,
                StateSha256Hex = stateSha256,
                HmacHex = hmacHex,
            };
        }

        /// <summary>
        /// Computes the lowercase-hex SHA-256 of the canonical UTF-8 JSON
        /// serialization of <paramref name="payload"/>. Public for symmetry
        /// with <c>BridgeReceiptVerifier.Verify</c>'s recompute step.
        /// </summary>
        public static string ComputePayloadHash(JToken? payload)
        {
            string canonical = CanonicalJson.Canonicalize(payload);
            byte[] bytes = Encoding.UTF8.GetBytes(canonical);
            using var sha = SHA256.Create();
            return ToHexLower(sha.ComputeHash(bytes));
        }

        /// <summary>
        /// Formats <paramref name="utc"/> in the canonical ISO 8601 UTC shape
        /// required by the receipt spec: millisecond precision and a trailing
        /// 'Z'. Always uses <see cref="CultureInfo.InvariantCulture"/>.
        /// </summary>
        public static string FormatTimestamp(DateTime utc)
        {
            // Force UTC: callers may pass DateTimeKind.Unspecified (e.g. fixed
            // test values constructed via `new DateTime(...)`); the spec is
            // unambiguous about the trailing 'Z' meaning UTC.
            DateTime asUtc = utc.Kind == DateTimeKind.Utc
                ? utc
                : DateTime.SpecifyKind(utc, DateTimeKind.Utc);
            return asUtc.ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture);
        }

        // netstandard2.0 has no Convert.ToHexString — implement locally.
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
}
