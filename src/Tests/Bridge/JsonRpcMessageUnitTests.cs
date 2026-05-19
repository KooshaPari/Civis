#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="JsonRpcRequest"/>, <see cref="JsonRpcResponse"/>,
/// <see cref="JsonRpcError"/>, and <see cref="BridgeReceipt"/> serialization
/// and construction.
/// </summary>
public class JsonRpcMessageUnitTests
{
    [Fact]
    public void JsonRpcRequest_CtorDefaults_InitializesCorrectly()
    {
        // Arrange & Act
        var request = new JsonRpcRequest();

        // Assert
        request.Jsonrpc.Should().Be("2.0");
        request.Id.Should().Be("");
        request.Method.Should().Be("");
        request.Params.Should().BeNull();
    }

    [Fact]
    public void JsonRpcRequest_WithId_PreservesValue()
    {
        // Arrange
        var request = new JsonRpcRequest
        {
            Id = "test-123",
            Method = "GetStatus"
        };

        // Act & Assert
        request.Id.Should().Be("test-123");
        request.Method.Should().Be("GetStatus");
    }

    [Fact]
    public void JsonRpcRequest_WithNullParams_SerializesCorrectly()
    {
        // Arrange
        var request = new JsonRpcRequest
        {
            Id = "req-1",
            Method = "Connect",
            Params = null
        };

        // Act
        string json = JsonConvert.SerializeObject(request);

        // Assert
        json.Should().NotContain("params"); // NullValueHandling.Ignore
        json.Should().Contain("\"jsonrpc\":\"2.0\"");
        json.Should().Contain("\"id\":\"req-1\"");
        json.Should().Contain("\"method\":\"Connect\"");
    }

    [Fact]
    public void JsonRpcRequest_WithParams_SerializesIncluded()
    {
        // Arrange
        var request = new JsonRpcRequest
        {
            Id = "req-2",
            Method = "Query",
            Params = JObject.FromObject(new { entity_id = 123 })
        };

        // Act
        string json = JsonConvert.SerializeObject(request);

        // Assert
        json.Should().Contain("\"params\"");
        json.Should().Contain("\"entity_id\":123");
    }

    [Fact]
    public void JsonRpcResponse_CtorDefaults_InitializesCorrectly()
    {
        // Arrange & Act
        var response = new JsonRpcResponse();

        // Assert
        response.Jsonrpc.Should().Be("2.0");
        response.Id.Should().BeNull();
        response.Result.Should().BeNull();
        response.Error.Should().BeNull();
        response.BridgeReceipt.Should().BeNull();
    }

    [Fact]
    public void JsonRpcResponse_WithResult_SerializesResult()
    {
        // Arrange
        var response = new JsonRpcResponse
        {
            Id = "resp-1",
            Result = JToken.FromObject(new { status = "ok" })
        };

        // Act
        string json = JsonConvert.SerializeObject(response);

        // Assert
        json.Should().Contain("\"result\"");
        json.Should().Contain("\"status\":\"ok\"");
        json.Should().NotContain("\"error\""); // NullValueHandling.Ignore
    }

    [Fact]
    public void JsonRpcResponse_WithError_SerializesError()
    {
        // Arrange
        var response = new JsonRpcResponse
        {
            Id = "resp-2",
            Error = new JsonRpcError { Code = -32600, Message = "Invalid Request" }
        };

        // Act
        string json = JsonConvert.SerializeObject(response);

        // Assert
        json.Should().Contain("\"error\"");
        json.Should().Contain("\"code\":-32600");
        json.Should().Contain("\"message\":\"Invalid Request\"");
        json.Should().NotContain("\"result\""); // NullValueHandling.Ignore
    }

    [Fact]
    public void JsonRpcResponse_WithBridgeReceipt_SerializesSnakeCase()
    {
        // Arrange
        var response = new JsonRpcResponse
        {
            Id = "resp-3",
            Result = JToken.FromObject(new { value = 42 }),
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = "sess-abc",
                TimestampUtc = "2026-04-25T12:34:56.789Z",
                WorldFrame = 100,
                StateSha256Hex = "abcd1234",
                HmacHex = "efgh5678"
            }
        };

        // Act
        string json = JsonConvert.SerializeObject(response);

        // Assert
        json.Should().Contain("\"bridge_receipt\""); // snake_case per JsonProperty
        json.Should().Contain("\"session_id\":\"sess-abc\"");
        json.Should().Contain("\"timestamp\":\"2026-04-25T12:34:56.789Z\"");
        json.Should().Contain("\"world_frame\":100");
        json.Should().Contain("\"state_sha256\":\"abcd1234\"");
        json.Should().Contain("\"hmac\":\"efgh5678\"");
    }

    [Fact]
    public void JsonRpcError_CtorDefaults_InitializesCorrectly()
    {
        // Arrange & Act
        var error = new JsonRpcError();

        // Assert
        error.Code.Should().Be(0);
        error.Message.Should().Be("");
        error.Data.Should().BeNull();
    }

    [Fact]
    public void JsonRpcError_WithCodeAndMessage_SerializesCorrectly()
    {
        // Arrange
        var error = new JsonRpcError
        {
            Code = -32700,
            Message = "Parse error",
            Data = JToken.FromObject(new { detail = "unexpected token" })
        };

        // Act
        string json = JsonConvert.SerializeObject(error);

        // Assert
        json.Should().Contain("\"code\":-32700");
        json.Should().Contain("\"message\":\"Parse error\"");
        json.Should().Contain("\"data\"");
        json.Should().Contain("\"detail\":\"unexpected token\"");
    }

    [Fact]
    public void JsonRpcRequest_RoundTrip_DeserializeEquality()
    {
        // Arrange
        var original = new JsonRpcRequest
        {
            Id = "round-trip-1",
            Method = "TestMethod",
            Params = JObject.FromObject(new { key = "value", number = 42 })
        };

        // Act
        string json = JsonConvert.SerializeObject(original);
        var deserialized = JsonConvert.DeserializeObject<JsonRpcRequest>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Jsonrpc.Should().Be(original.Jsonrpc);
        deserialized.Id.Should().Be(original.Id);
        deserialized.Method.Should().Be(original.Method);
        deserialized.Params.Should().NotBeNull();
        deserialized.Params!["key"]!.ToString().Should().Be("value");
        deserialized.Params["number"]!.Value<int>().Should().Be(42);
    }

    [Fact]
    public void JsonRpcResponse_RoundTrip_DeserializeEquality()
    {
        // Arrange
        var original = new JsonRpcResponse
        {
            Id = "resp-round-trip",
            Result = JToken.FromObject(new { success = true, data = "test" }),
            BridgeReceipt = new BridgeReceipt
            {
                SessionId = "sess-123",
                TimestampUtc = "2026-04-26T14:25:30.100Z",
                WorldFrame = 500,
                StateSha256Hex = "deadbeef",
                HmacHex = "cafebabe"
            }
        };

        // Act
        string json = JsonConvert.SerializeObject(original);
        var deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Jsonrpc.Should().Be(original.Jsonrpc);
        deserialized.Id.Should().Be(original.Id);
        deserialized.Result.Should().NotBeNull();
        deserialized.Result!["success"]!.Value<bool>().Should().BeTrue();
        deserialized.BridgeReceipt.Should().NotBeNull();
        deserialized.BridgeReceipt!.SessionId.Should().Be("sess-123");
        deserialized.BridgeReceipt.WorldFrame.Should().Be(500);
        deserialized.BridgeReceipt.HmacHex.Should().Be("cafebabe");
    }

    [Fact]
    public void JsonRpcResponse_ResultAndErrorMutualExclusivity_ErrorIgnoredIfBothSet()
    {
        // Arrange: Simulate wire data with both result and error (malformed but possible)
        string json = @"{
            ""jsonrpc"": ""2.0"",
            ""id"": ""test-both"",
            ""result"": { ""value"": 123 },
            ""error"": { ""code"": -32600, ""message"": ""Invalid"" }
        }";

        // Act
        var response = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

        // Assert: Newtonsoft.Json deserializes both properties; application layer must enforce mutual exclusivity
        response.Should().NotBeNull();
        response!.Result.Should().NotBeNull();
        response.Error.Should().NotBeNull();
        // This test documents that the JSON layer does NOT enforce the constraint —
        // it's a protocol-level invariant that MUST be validated by consumers (e.g., GameClient).
    }

    [Fact]
    public void BridgeReceipt_AllProperties_SerializeDeserialize()
    {
        // Arrange
        var receipt = new BridgeReceipt
        {
            SessionId = "f0e1d2c3b4a59687",
            TimestampUtc = "2026-05-18T08:15:30.250Z",
            WorldFrame = 5000,
            StateSha256Hex = "0123456789abcdef",
            HmacHex = "fedcba9876543210"
        };

        // Act
        string json = JsonConvert.SerializeObject(receipt);
        var deserialized = JsonConvert.DeserializeObject<BridgeReceipt>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.SessionId.Should().Be(receipt.SessionId);
        deserialized.TimestampUtc.Should().Be(receipt.TimestampUtc);
        deserialized.WorldFrame.Should().Be(receipt.WorldFrame);
        deserialized.StateSha256Hex.Should().Be(receipt.StateSha256Hex);
        deserialized.HmacHex.Should().Be(receipt.HmacHex);
    }
}
