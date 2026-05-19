#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="BridgeReceipt"/>.
/// Covers construction, serialization, field defaults.
/// </summary>
public class BridgeReceiptUnitTests
{
    [Fact]
    public void Construction_InitializesWithDefaults()
    {
        var receipt = new BridgeReceipt();

        receipt.SessionId.Should().Be("");
        receipt.TimestampUtc.Should().Be("");
        receipt.StateSha256Hex.Should().Be("");
        receipt.HmacHex.Should().Be("");
        receipt.WorldFrame.Should().Be(0);
    }

    [Fact]
    public void SessionId_CanBeSet()
    {
        var receipt = new BridgeReceipt
        {
            SessionId = "abc123def456"
        };

        receipt.SessionId.Should().Be("abc123def456");
    }

    [Fact]
    public void TimestampUtc_CanBeSet()
    {
        var receipt = new BridgeReceipt
        {
            TimestampUtc = "2026-04-25T12:34:56.789Z"
        };

        receipt.TimestampUtc.Should().Be("2026-04-25T12:34:56.789Z");
    }

    [Fact]
    public void WorldFrame_CanBeSet()
    {
        var receipt = new BridgeReceipt
        {
            WorldFrame = 12345
        };

        receipt.WorldFrame.Should().Be(12345);
    }

    [Fact]
    public void StateSha256Hex_CanBeSet()
    {
        var receipt = new BridgeReceipt
        {
            StateSha256Hex = "abcd1234efgh5678"
        };

        receipt.StateSha256Hex.Should().Be("abcd1234efgh5678");
    }

    [Fact]
    public void HmacHex_CanBeSet()
    {
        var receipt = new BridgeReceipt
        {
            HmacHex = "fedcba9876543210"
        };

        receipt.HmacHex.Should().Be("fedcba9876543210");
    }

    [Fact]
    public void Serialization_JsonProperty_SessionId()
    {
        var json = @"{""session_id"":""test-session""}";
        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.SessionId.Should().Be("test-session");
    }

    [Fact]
    public void Serialization_JsonProperty_Timestamp()
    {
        var json = @"{""timestamp"":""2026-04-25T12:34:56.789Z""}";
        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.TimestampUtc.Should().Be("2026-04-25T12:34:56.789Z");
    }

    [Fact]
    public void Serialization_JsonProperty_WorldFrame()
    {
        var json = @"{""world_frame"":999}";
        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.WorldFrame.Should().Be(999);
    }

    [Fact]
    public void Serialization_JsonProperty_StateSha256()
    {
        var json = @"{""state_sha256"":""abc123""}";
        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.StateSha256Hex.Should().Be("abc123");
    }

    [Fact]
    public void Serialization_JsonProperty_Hmac()
    {
        var json = @"{""hmac"":""def456""}";
        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.HmacHex.Should().Be("def456");
    }

    [Fact]
    public void Deserialization_AllFields()
    {
        var json = @"{
            ""session_id"": ""sess-abc123"",
            ""timestamp"": ""2026-04-25T12:34:56.789Z"",
            ""world_frame"": 42,
            ""state_sha256"": ""abcd1234"",
            ""hmac"": ""fedcba98""
        }";

        var receipt = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        receipt.Should().NotBeNull();
        receipt!.SessionId.Should().Be("sess-abc123");
        receipt.TimestampUtc.Should().Be("2026-04-25T12:34:56.789Z");
        receipt.WorldFrame.Should().Be(42);
        receipt.StateSha256Hex.Should().Be("abcd1234");
        receipt.HmacHex.Should().Be("fedcba98");
    }

    [Fact]
    public void Serialization_RoundTrip()
    {
        var original = new BridgeReceipt
        {
            SessionId = "test-session",
            TimestampUtc = "2026-04-25T12:34:56.789Z",
            WorldFrame = 100,
            StateSha256Hex = "abc123",
            HmacHex = "def456"
        };

        var json = JsonConvert.SerializeObject(original);
        var deserialized = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        deserialized.Should().NotBeNull();
        deserialized!.SessionId.Should().Be(original.SessionId);
        deserialized.TimestampUtc.Should().Be(original.TimestampUtc);
        deserialized.WorldFrame.Should().Be(original.WorldFrame);
        deserialized.StateSha256Hex.Should().Be(original.StateSha256Hex);
        deserialized.HmacHex.Should().Be(original.HmacHex);
    }

    [Fact]
    public void Serialization_OutputUsesSnakeCase()
    {
        var receipt = new BridgeReceipt
        {
            SessionId = "test",
            TimestampUtc = "2026-04-25T12:34:56.789Z",
            WorldFrame = 1,
            StateSha256Hex = "abc",
            HmacHex = "def"
        };

        var json = JsonConvert.SerializeObject(receipt);

        json.Should().Contain("session_id");
        json.Should().Contain("world_frame");
        json.Should().Contain("state_sha256");
    }

    [Fact]
    public void NullFields_Tolerated()
    {
        var receipt = new BridgeReceipt();

        // All fields default to empty string, not null
        receipt.SessionId.Should().NotBeNull();
        receipt.TimestampUtc.Should().NotBeNull();
        receipt.StateSha256Hex.Should().NotBeNull();
        receipt.HmacHex.Should().NotBeNull();
    }

    [Fact]
    public void WorldFrame_SupportsLargeValues()
    {
        var receipt = new BridgeReceipt
        {
            WorldFrame = long.MaxValue
        };

        receipt.WorldFrame.Should().Be(long.MaxValue);
    }

    [Fact]
    public void WorldFrame_SupportsZero()
    {
        var receipt = new BridgeReceipt
        {
            WorldFrame = 0
        };

        receipt.WorldFrame.Should().Be(0);
    }

    [Fact]
    public void WorldFrame_SupportsNegativeValues()
    {
        var receipt = new BridgeReceipt
        {
            WorldFrame = -1
        };

        receipt.WorldFrame.Should().Be(-1);
    }

    [Fact]
    public void IsSealed()
    {
        var receipt = new BridgeReceipt();

        receipt.GetType().IsSealed.Should().BeTrue();
    }
}
