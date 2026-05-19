#nullable enable
using System;
using System.Collections.Concurrent;

namespace DINOForge.Bridge.Client;

/// <summary>
/// Thread-safe in-memory cache of per-session HMAC keys (Wave 2 Phase 4b).
/// Populated by <see cref="GameClient.ConnectAsync(System.Threading.CancellationToken)"/>
/// after the server emits a <c>session_key_b64</c> on the Connect handshake;
/// consulted on every subsequent response when verifying the receipt HMAC.
/// </summary>
/// <remarks>
/// <para>
/// Keys are 32 bytes (HMAC-SHA256 input). The cache is process-local and never
/// persists to disk — sessions rotate on reconnect (server-side
/// <see cref="DINOForge.Runtime.Bridge.SessionHmac"/> is freshly constructed
/// per server start) so there is no useful cross-process replay.
/// </para>
/// <para>
/// On <see cref="Dispose"/>, key bytes are best-effort zeroed before being
/// dropped. CLR strings/arrays are not guaranteed-zeroable (the GC may have
/// copied them), but this still minimises the resident-memory window.
/// </para>
/// </remarks>
public sealed class SessionKeyCache : IDisposable
{
    private readonly ConcurrentDictionary<string, byte[]> _keys = new(StringComparer.Ordinal);
    private bool _disposed;

    /// <summary>
    /// Caches <paramref name="key"/> against <paramref name="sessionId"/>.
    /// Overwrites any existing entry (used on reconnect with a new key).
    /// </summary>
    /// <param name="sessionId">Session identifier echoed by the server in every receipt.</param>
    /// <param name="key">Raw 32-byte HMAC-SHA256 key. Caller must not mutate after this call.</param>
    public void Set(string sessionId, byte[] key)
    {
        if (sessionId == null) throw new ArgumentNullException(nameof(sessionId));
        if (key == null) throw new ArgumentNullException(nameof(key));
        if (key.Length != 32) throw new ArgumentException("Session key must be 32 bytes (HMAC-SHA256 spec)", nameof(key));
        _keys[sessionId] = key;
    }

    /// <summary>
    /// Retrieves the cached key for <paramref name="sessionId"/>. Returns false
    /// if the session was never registered (e.g. receipt for an unknown session).
    /// </summary>
    public bool TryGet(string sessionId, out byte[]? key)
    {
        if (sessionId == null) { key = null; return false; }
        return _keys.TryGetValue(sessionId, out key);
    }

    /// <summary>
    /// Removes a session entry — typically called on Disconnect so the key bytes
    /// can be zeroed promptly without waiting for the cache to be disposed.
    /// </summary>
    public bool Remove(string sessionId)
    {
        if (sessionId == null) return false;
        if (_keys.TryRemove(sessionId, out var key))
        {
            Array.Clear(key, 0, key.Length);
            return true;
        }
        return false;
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        foreach (var kv in _keys)
        {
            Array.Clear(kv.Value, 0, kv.Value.Length);
        }
        _keys.Clear();
    }
}
