#nullable enable
using System.Linq;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.Contract;

/// <summary>
/// Consumer-driven contract test for the bridge handshake response.
/// Pins the public JSON shape that bridge clients consume without binding the
/// test to internal transport details.
/// </summary>
public sealed class BridgeRpcHandshakeContractTests
{
    [Fact]
    public void ConnectHandshake_ResponseShape_MatchesConsumerContract()
    {
        JsonRpcResponse response = new()
        {
            Jsonrpc = "2.0",
            Id = "connect-1",
            Result = JObject.FromObject(new
            {
                session_id = "0123456789abcdef0123456789abcdef",
                session_key_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            })
        };

        JObject document = JObject.Parse(JsonConvert.SerializeObject(response, Formatting.None));

        document.Properties().Select(property => property.Name)
            .Should().BeEquivalentTo(new[] { "jsonrpc", "id", "result" });
        document["jsonrpc"].Should().BeOfType<JValue>().Which.Value.Should().Be("2.0");
        document["id"].Should().BeOfType<JValue>().Which.Value.Should().Be("connect-1");
        document["error"].Should().BeNull("the handshake is a success response");
        document["bridge_receipt"].Should().BeNull("the handshake response is the public JSON-RPC envelope only");

        JObject result = document["result"].Should().BeOfType<JObject>().Which;
        result.Properties().Select(property => property.Name)
            .Should().BeEquivalentTo(new[] { "session_id", "session_key_b64" });
        result["session_id"].Should().BeOfType<JValue>().Which.Value.Should().Be("0123456789abcdef0123456789abcdef");
        result["session_key_b64"].Should().BeOfType<JValue>().Which.Value.Should().Be("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    }
}
