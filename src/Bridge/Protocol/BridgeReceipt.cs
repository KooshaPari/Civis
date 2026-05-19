#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Per-response provenance receipt. Emitted by the GameBridgeServer alongside every
    /// JSON-RPC reply (Phase 4a). Carries timestamp + frame + state hash + HMAC so
    /// downstream verifiers (Phase 4b) can detect tampering, replay, or fabrication.
    /// </summary>
    /// <remarks>
    /// The <c>HmacHex</c> field is computed by the server using the per-session
    /// 32-byte ephemeral key issued during the <c>Connect</c> handshake. The client
    /// stores the key and recomputes the HMAC on every response (see Phase 4b).
    /// </remarks>
    public sealed class BridgeReceipt
    {
        /// <summary>
        /// The session identifier (UUID hex, no dashes) the receipt was signed under.
        /// Used by the client to look up the cached session key.
        /// </summary>
        [JsonProperty("session_id")]
        public string SessionId { get; set; } = "";

        /// <summary>
        /// ISO 8601 UTC timestamp with millisecond precision and trailing 'Z'.
        /// Example: "2026-04-25T12:34:56.789Z".
        /// </summary>
        [JsonProperty("timestamp")]
        public string TimestampUtc { get; set; } = "";

        /// <summary>
        /// Monotonic ECS world frame number at the time the response was prepared.
        /// Frame=0 is reserved for sentinel responses (Connect/Status pre-world-ready).
        /// </summary>
        [JsonProperty("world_frame")]
        public long WorldFrame { get; set; }

        /// <summary>
        /// Lowercase hex SHA-256 of the canonical JSON of the response payload
        /// (excluding the receipt itself). Used to detect post-signing tampering.
        /// </summary>
        [JsonProperty("state_sha256")]
        public string StateSha256Hex { get; set; } = "";

        /// <summary>
        /// Lowercase hex HMAC-SHA256 over the canonical receipt fields
        /// (timestamp + world_frame + state_sha256), keyed with the session key.
        /// </summary>
        [JsonProperty("hmac")]
        public string HmacHex { get; set; } = "";
    }
}
