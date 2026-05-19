#nullable enable
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// A JSON-RPC 2.0 request message sent to the game bridge server.
    /// </summary>
    // pattern-226: public properties (netstandard2.0) instead of fields for encapsulation
    public sealed class JsonRpcRequest
    {
        /// <summary>JSON-RPC version, always "2.0".</summary>
        [JsonProperty("jsonrpc")]
        public string Jsonrpc { get; set; } = "2.0";

        /// <summary>Unique request identifier.</summary>
        [JsonProperty("id")]
        public string Id { get; set; } = "";

        /// <summary>The method name to invoke on the server.</summary>
        [JsonProperty("method")]
        public string Method { get; set; } = "";

        /// <summary>Optional method parameters as a JSON object.</summary>
        [JsonProperty("params", NullValueHandling = NullValueHandling.Ignore)]
        public JObject? Params { get; set; }
    }

    /// <summary>
    /// A JSON-RPC 2.0 response message returned by the game bridge server.
    /// </summary>
    // pattern-226: public properties (netstandard2.0) instead of fields for encapsulation
    public sealed class JsonRpcResponse
    {
        /// <summary>JSON-RPC version, always "2.0".</summary>
        [JsonProperty("jsonrpc")]
        public string Jsonrpc { get; set; } = "2.0";

        /// <summary>The request identifier this response corresponds to.</summary>
        [JsonProperty("id")]
        public string? Id { get; set; }

        /// <summary>The result payload, present on success.</summary>
        [JsonProperty("result", NullValueHandling = NullValueHandling.Ignore)]
        public JToken? Result { get; set; }

        /// <summary>The error payload, present on failure.</summary>
        [JsonProperty("error", NullValueHandling = NullValueHandling.Ignore)]
        public JsonRpcError? Error { get; set; }

        /// <summary>Optional response receipt for tamper detection (Phase 4a+).</summary>
        [JsonProperty("bridge_receipt", NullValueHandling = NullValueHandling.Ignore)]
        public BridgeReceipt? BridgeReceipt { get; set; }
    }

    /// <summary>
    /// A JSON-RPC 2.0 error object describing a failed request.
    /// </summary>
    // pattern-226: public properties (netstandard2.0) instead of fields for encapsulation
    public sealed class JsonRpcError
    {
        /// <summary>Numeric error code (negative for protocol errors, positive for application errors).</summary>
        [JsonProperty("code")]
        public int Code { get; set; }

        /// <summary>Human-readable error message.</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";

        /// <summary>Optional additional error data.</summary>
        [JsonProperty("data", NullValueHandling = NullValueHandling.Ignore)]
        public JToken? Data { get; set; }
    }
}
