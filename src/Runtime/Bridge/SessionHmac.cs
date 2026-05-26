#nullable enable
using System;
using System.Security.Cryptography;
using System.Text;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Per-session HMAC key holder + canonical-JSON receipt signing (Wave 2 Phase 4a).
    /// One instance is generated at <see cref="GameBridgeServer"/> startup. The
    /// 32-byte key is transmitted to the client exactly once via the initial
    /// <c>Connect</c> handshake response, then cached client-side and never
    /// re-emitted. Subsequent responses carry an HMAC-SHA256 receipt computed
    /// with this key over the canonical (timestamp, world_frame, state_sha256)
    /// triple — see <see cref="ComputeHmac"/> for the exact wire shape.
    /// </summary>
    /// <remarks>
    /// The key never persists to disk, never enters logs, and is rotated on
    /// every reconnect. Compromising one session does not compromise others.
    /// See <c>docs/design/2026-04-25-bridge-hmac-phase4.md</c> section 6 for
    /// the full threat model.
    /// </remarks>
    public sealed class SessionHmac : IDisposable
    {
        /// <summary>
        /// Stable identifier (UUID hex, no dashes) for this session. Echoed in
        /// every <see cref="DINOForge.Bridge.Protocol.BridgeReceipt"/>.
        /// </summary>
        public string SessionId { get; }

        /// <summary>
        /// 256-bit key material used to derive HMACs. Exposed so the bridge
        /// server can base64-encode it once into the Connect handshake reply.
        /// Do not log, persist, or transmit beyond that single handshake.
        /// </summary>
        public byte[] KeyMaterial { get; }

        private readonly object _stateLock = new object();
        private bool _disposed;

        /// <summary>
        /// Creates a new session with a fresh random session id and 256-bit key.
        /// </summary>
        public SessionHmac()
        {
            SessionId = Guid.NewGuid().ToString("N");
            KeyMaterial = new byte[32];
            using (var rng = RandomNumberGenerator.Create())
            {
                rng.GetBytes(KeyMaterial);
            }
        }

        /// <summary>
        /// Computes an HMAC-SHA256 over the canonical JSON of the receipt
        /// triple <c>(state_sha256, timestamp, world_frame)</c>. Returns the
        /// digest as lowercase hex.
        /// </summary>
        /// <remarks>
        /// Per spec section 6, the canonical input is UTF-8 JSON with keys
        /// sorted lexicographically (alphabetical: state_sha256, timestamp,
        /// world_frame), no insignificant whitespace, ISO 8601 UTC timestamp
        /// with millisecond precision and trailing 'Z'. The <c>hmac</c> field
        /// itself is excluded from the canonical payload.
        /// </remarks>
        public string ComputeHmac(string timestampUtc, long worldFrame, string stateSha256Hex)
        {
            if (timestampUtc == null) throw new ArgumentNullException(nameof(timestampUtc));
            if (stateSha256Hex == null) throw new ArgumentNullException(nameof(stateSha256Hex));

            // Canonical input: keys sorted alphabetically, no whitespace.
            // Order: state_sha256, timestamp, world_frame (lexicographic).
            string canonical = "{\"state_sha256\":\"" + stateSha256Hex
                + "\",\"timestamp\":\"" + timestampUtc
                + "\",\"world_frame\":" + worldFrame.ToString(System.Globalization.CultureInfo.InvariantCulture)
                + "}";
            byte[] bytes = Encoding.UTF8.GetBytes(canonical);
            // HMACSHA256 instances are NOT thread-safe (per MS docs). Construct
            // per-call against the immutable key material so concurrent receipt
            // signing from multiple threads cannot corrupt internal state.
            // ctor cost is negligible (~allocation + key copy); avoids the
            // serialization bottleneck of a shared-instance + lock.
            lock (_stateLock)
            {
                if (_disposed) throw new ObjectDisposedException(nameof(SessionHmac));
            }
            using var hmac = new HMACSHA256(KeyMaterial);
            return ToHexLower(hmac.ComputeHash(bytes));
        }

        /// <summary>
        /// Returns the session key encoded as standard base64. Call exactly once,
        /// during the Connect handshake. After that the key must never leave
        /// the server process.
        /// </summary>
        public string KeyMaterialB64() => Convert.ToBase64String(KeyMaterial);

        /// <inheritdoc />
        public void Dispose()
        {
            lock (_stateLock)
            {
                if (_disposed) return;
                _disposed = true;
                // Best-effort scrub of key material; .NET strings/arrays are not
                // guaranteed-zeroable but this minimises the resident-memory window.
                // Note: scrubbing here races with any in-flight ComputeHmac that has
                // already passed the disposed check; per-call HMACSHA256 ctors copy
                // the key, so an in-flight call completes correctly. Subsequent
                // calls fault on the disposed check.
                Array.Clear(KeyMaterial, 0, KeyMaterial.Length);
            }
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
