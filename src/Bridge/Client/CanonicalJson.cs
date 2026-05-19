#nullable enable
namespace DINOForge.Bridge.Client;

/// <summary>
/// HMAC verification mode for client-side bridge_receipt checking (Wave 2 Phase 4b).
/// </summary>
/// <remarks>
/// <para>
/// Phase 4b ships in <see cref="WarnOnly"/> by default to avoid breaking existing
/// fixtures that don't yet route through the receipt-emitting server build. Phase
/// 4c flips the default to <see cref="Strict"/>.
/// </para>
/// </remarks>
public enum VerificationMode
{
    /// <summary>Don't verify, don't log. Use only for legacy fixtures.</summary>
    Off = 0,

    /// <summary>Verify, log warning on mismatch, return success anyway. Default in Phase 4b.</summary>
    WarnOnly = 1,

    /// <summary>Verify, throw <see cref="GameClientException"/>("hmac_invalid") on mismatch. Default in Phase 4c.</summary>
    Strict = 2,
}

// NOTE: The deterministic JSON canonicalizer formerly defined in this file
// (DINOForge.Bridge.Client.CanonicalJson) was lifted into the shared
// netstandard2.0 protocol library at src/Bridge/Protocol/CanonicalJson.cs
// (#217 Phase A). Both server (GameBridgeServer) and client
// (BridgeReceiptVerifier) now reference the single implementation in
// DINOForge.Bridge.Protocol so the two sides cannot drift byte-for-byte.
//
// VerificationMode stays here because it is a client-side concept (the server
// has no notion of WarnOnly / Strict — it always emits receipts).
