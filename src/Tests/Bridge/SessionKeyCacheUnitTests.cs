#nullable enable
using System;
using System.Linq;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="SessionKeyCache"/>.
/// Covers thread-safe caching, key lifecycle, argument validation.
/// </summary>
public class SessionKeyCacheUnitTests
{
    private static byte[] CreateKey(byte seed = 42)
    {
        var key = new byte[32];
        for (int i = 0; i < 32; i++) key[i] = (byte)((seed + i) % 256);
        return key;
    }

    [Fact]
    public void Set_WithValidKey_CachesSuccessfully()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();

        cache.Set("session-123", key);

        cache.TryGet("session-123", out var retrieved).Should().BeTrue();
        retrieved.Should().Equal(key);
    }

    [Fact]
    public void Set_WithNullSessionId_ThrowsArgumentNullException()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();

        Action act = () => cache.Set(null!, key);

        act.Should().Throw<ArgumentNullException>().WithParameterName("sessionId");
    }

    [Fact]
    public void Set_WithNullKey_ThrowsArgumentNullException()
    {
        var cache = new SessionKeyCache();

        Action act = () => cache.Set("session-123", null!);

        act.Should().Throw<ArgumentNullException>().WithParameterName("key");
    }

    [Fact]
    public void Set_WithWrongKeyLength_ThrowsArgumentException()
    {
        var cache = new SessionKeyCache();
        var wrongKey = new byte[16]; // Should be 32

        Action act = () => cache.Set("session-123", wrongKey);

        act.Should().Throw<ArgumentException>().WithParameterName("key");
    }

    [Fact]
    public void Set_Overwrites_ExistingEntry()
    {
        var cache = new SessionKeyCache();
        var key1 = CreateKey(seed: 1);
        var key2 = CreateKey(seed: 2);

        cache.Set("session-123", key1);
        cache.Set("session-123", key2);

        cache.TryGet("session-123", out var retrieved).Should().BeTrue();
        retrieved.Should().Equal(key2);
        retrieved.Should().NotEqual(key1);
    }

    [Fact]
    public void TryGet_WithExistingSession_ReturnsTrue()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();
        cache.Set("session-123", key);

        var result = cache.TryGet("session-123", out var retrieved);

        result.Should().BeTrue();
        retrieved.Should().Equal(key);
    }

    [Fact]
    public void TryGet_WithMissingSession_ReturnsFalse()
    {
        var cache = new SessionKeyCache();

        var result = cache.TryGet("missing-session", out var retrieved);

        result.Should().BeFalse();
        retrieved.Should().BeNull();
    }

    [Fact]
    public void TryGet_WithNullSessionId_ReturnsFalse()
    {
        var cache = new SessionKeyCache();

        var result = cache.TryGet(null!, out var retrieved);

        result.Should().BeFalse();
        retrieved.Should().BeNull();
    }

    [Fact]
    public void Remove_WithExistingSession_ReturnsTrue()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();
        cache.Set("session-123", key);

        var result = cache.Remove("session-123");

        result.Should().BeTrue();
        cache.TryGet("session-123", out _).Should().BeFalse();
    }

    [Fact]
    public void Remove_WithMissingSession_ReturnsFalse()
    {
        var cache = new SessionKeyCache();

        var result = cache.Remove("missing-session");

        result.Should().BeFalse();
    }

    [Fact]
    public void Remove_WithNullSessionId_ReturnsFalse()
    {
        var cache = new SessionKeyCache();

        var result = cache.Remove(null!);

        result.Should().BeFalse();
    }

    [Fact]
    public void Remove_ClearsKeyBytes()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();
        cache.Set("session-123", key);

        // Verify key has non-zero bytes before removal
        key.Any(b => b != 0).Should().BeTrue("key should have non-zero bytes initially");

        cache.Remove("session-123");

        // Verify key bytes were cleared (Array.Clear modifies the array in-place)
        for (int i = 0; i < key.Length; i++)
        {
            key[i].Should().Be(0, "Remove should zero all key bytes at index {0}", i);
        }
    }

    [Fact]
    public void Dispose_ClearsAllKeys()
    {
        var cache = new SessionKeyCache();
        var key1 = CreateKey(1);
        var key2 = CreateKey(2);
        cache.Set("session-1", key1);
        cache.Set("session-2", key2);

        cache.Dispose();

        cache.TryGet("session-1", out _).Should().BeFalse();
        cache.TryGet("session-2", out _).Should().BeFalse();
    }

    [Fact]
    public void Dispose_MultipleCallsAreIdempotent()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();
        cache.Set("session-123", key);

        Action act1 = () => cache.Dispose();
        Action act2 = () => cache.Dispose();

        act1.Should().NotThrow();
        act2.Should().NotThrow();
    }

    [Fact]
    public void MultipleKeys_StoredIndependently()
    {
        var cache = new SessionKeyCache();
        var key1 = CreateKey(1);
        var key2 = CreateKey(2);
        var key3 = CreateKey(3);

        cache.Set("session-1", key1);
        cache.Set("session-2", key2);
        cache.Set("session-3", key3);

        cache.TryGet("session-1", out var r1).Should().BeTrue();
        cache.TryGet("session-2", out var r2).Should().BeTrue();
        cache.TryGet("session-3", out var r3).Should().BeTrue();

        r1.Should().Equal(key1);
        r2.Should().Equal(key2);
        r3.Should().Equal(key3);
    }

    [Fact]
    public void SessionId_CaseSensitive()
    {
        var cache = new SessionKeyCache();
        var key = CreateKey();
        cache.Set("SESSION-123", key);

        cache.TryGet("session-123", out _).Should().BeFalse();
        cache.TryGet("SESSION-123", out _).Should().BeTrue();
    }

    [Fact]
    public void Remove_RemovesOnlySpecificSession()
    {
        var cache = new SessionKeyCache();
        var key1 = CreateKey(1);
        var key2 = CreateKey(2);
        cache.Set("session-1", key1);
        cache.Set("session-2", key2);

        cache.Remove("session-1");

        cache.TryGet("session-1", out _).Should().BeFalse();
        cache.TryGet("session-2", out _).Should().BeTrue();
    }

    [Fact]
    public void Set_32ByteKeyExactly_Succeeds()
    {
        var cache = new SessionKeyCache();
        var key = new byte[32];

        Action act = () => cache.Set("session-123", key);

        act.Should().NotThrow();
    }

    [Fact]
    public void Set_CorrectKeyLengthForHmacSha256()
    {
        var cache = new SessionKeyCache();
        var key = new byte[32]; // HMAC-SHA256 standard key length

        cache.Set("session-123", key);

        cache.TryGet("session-123", out var retrieved).Should().BeTrue();
        retrieved.Should().HaveCount(32);
    }
}
