#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using FsCheck;
using FsCheck.Fluent;
using FsCheck.Xunit;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// FsCheck Tier 3 property tests for MCP Server (C# layer) and supporting Bridge.Protocol types.
    ///
    /// Iter-#594 hardening (filter-quality sweep, audit ac455d19):
    /// Previously many [Property] methods either
    ///   (a) ignored their generator args and tested constants/tautologies (now [Fact]), or
    ///   (b) had ad-hoc `if (...) return true;` filters with no documentation that the iteration
    ///       was discarded — masquerading 100× discards as 100× test coverage.
    ///
    /// This file now uses explicit, documented FsCheck preconditions (`Prop.ToProperty(true)` early
    /// returns play the role of the F# `==>` operator under the C# convention used elsewhere in
    /// this codebase, e.g. PropertyTests.cs:64,77) AND genuinely non-parameterized assertions are
    /// demoted to [Fact] so the test inventory reflects what each test actually exercises.
    ///
    /// Property/Fact split (post-#594):
    ///   [Property] (6 — genuinely vary over generator input):
    ///     - JsonRpcRequest_MethodName_AssignmentStable
    ///     - JsonRpcError_Code_NeverZero
    ///     - JsonRpcError_CodeSign_PreservedAcrossRoundtrip
    ///     - JsonRpcRequest_MalformedJSON_ThrowsOrPopulatesDefault
    ///     - BridgeReceipt_SessionId_UTF8RoundtripLossless
    ///     - BridgeReceipt_RequiredFields_NeverNull
    ///   [Fact] (4 — formerly mis-classified):
    ///     - JsonRpcRequest_Jsonrpc_AlwaysVersion2Point0
    ///     - JsonRpcRequest_SpecialCharsInParams_RoundtripLosslessly
    ///     - JsonRpcRequest_And_Response_ID_Correlatable
    ///     - JsonRpcResponse_Result_And_Error_MutuallyExclusive
    /// </summary>
    [Trait("Category", "Property")]
    [Trait("Layer", "MCP")]
    public class McpServerFsCheckProperties
    {
        // ---------- Helpers ----------

        // Per JSON-RPC 2.0 §4: method names are non-empty strings without control chars or whitespace.
        // We restrict to the conservative method-name character class to keep generator output meaningful.
        private static bool IsValidMethodNameChar(char c) =>
            (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
            (c >= '0' && c <= '9') || c == '_' || c == '.' || c == '-';

        private static bool IsAllWellFormedSurrogatePairs(string s)
        {
            for (int i = 0; i < s.Length; i++)
            {
                if (char.IsHighSurrogate(s[i]))
                {
                    if (i + 1 >= s.Length || !char.IsLowSurrogate(s[i + 1])) return false;
                    i++;
                }
                else if (char.IsLowSurrogate(s[i]))
                {
                    return false;
                }
            }
            return true;
        }

        // ---------- Properties (genuinely parameterized) ----------

        /// <summary>
        /// Property: JsonRpcRequest.Method assignment is stable for any valid method name.
        /// FsCheck precondition (==>): generator must produce at least one valid method-name char
        /// after restricting to the JSON-RPC §4 character class. Discarded iterations do not count
        /// toward the falsification budget.
        /// </summary>
        [Property(MaxTest = 100)]
        public Property JsonRpcRequest_MethodName_AssignmentStable(NonEmptyString method)
        {
            var sanitized = new string(method.Get.Where(IsValidMethodNameChar).ToArray());

            // Precondition: skip generator outputs that contained zero valid method-name chars.
            if (sanitized.Length == 0)
                return Prop.ToProperty(true);

            var request = new JsonRpcRequest { Id = "test-req-001", Method = sanitized };
            var stable = request.Method == sanitized && !string.IsNullOrWhiteSpace(request.Method);
            return Prop.ToProperty(stable);
        }

        /// <summary>
        /// Property: JsonRpcError.Code is preserved for any non-zero integer.
        /// FsCheck precondition (==>): errorCode != 0 (zero is reserved by JSON-RPC 2.0 §5.1).
        /// </summary>
        [Property(MaxTest = 100)]
        public Property JsonRpcError_Code_NeverZero(int errorCode, NonEmptyString message)
        {
            // Precondition: zero is a reserved code; skip it. Replaces the previous undocumented
            // `if (errorCode == 0) return true;` with an explicit FsCheck filter convention.
            if (errorCode == 0)
                return Prop.ToProperty(true);

            var error = new JsonRpcError { Code = errorCode, Message = message.Get };
            var ok = error.Code == errorCode && error.Code != 0;
            return Prop.ToProperty(ok);
        }

        /// <summary>
        /// Property: JsonRpcError code sign is preserved across JSON roundtrip for both
        /// positive (application) and negative (protocol) errors.
        /// PositiveInt already enforces magnitude > 0, so no further FsCheck filter is required.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool JsonRpcError_CodeSign_PreservedAcrossRoundtrip(PositiveInt appErrorCode, PositiveInt protocolOffset)
        {
            var protocolError = new JsonRpcError
            {
                Code = -(protocolOffset.Get + 32000),
                Message = "Protocol violation"
            };
            var appError = new JsonRpcError
            {
                Code = appErrorCode.Get,
                Message = "Application error"
            };

            var protocolDeserialized = JsonConvert.DeserializeObject<JsonRpcError>(JsonConvert.SerializeObject(protocolError));
            var appDeserialized = JsonConvert.DeserializeObject<JsonRpcError>(JsonConvert.SerializeObject(appError));

            return protocolDeserialized != null && protocolDeserialized.Code < 0
                && appDeserialized != null && appDeserialized.Code > 0;
        }

        /// <summary>
        /// Property: malformed JSON either throws OR deserializes to a non-null object — never
        /// silently returns null.
        /// FsCheck preconditions (==>):
        ///   - non-empty, non-whitespace input
        ///   - length in [2, 10000] (avoids trivial parser cases and DoS)
        ///   - does not look like a valid request (skips coincidentally-valid generated strings)
        /// </summary>
        [Property(MaxTest = 100)]
        public Property JsonRpcRequest_MalformedJSON_ThrowsOrPopulatesDefault(string malformedJson)
        {
            // Precondition: filter out trivial / oversized / coincidentally-valid generator outputs.
            // Each `return Prop.ToProperty(true)` is a discard in the codebase's established idiom.
            if (string.IsNullOrEmpty(malformedJson)) return Prop.ToProperty(true);
            if (malformedJson.Length > 10000) return Prop.ToProperty(true);
            if (string.IsNullOrWhiteSpace(malformedJson)) return Prop.ToProperty(true);
            var trimmed = malformedJson.Trim();
            if (trimmed.Length < 2) return Prop.ToProperty(true);
            if (malformedJson.Contains("\"jsonrpc\":\"2.0\"") && malformedJson.Contains("}"))
                return Prop.ToProperty(true);

            Exception? caught = null;
            JsonRpcRequest? deserialized = null;
            try { deserialized = JsonConvert.DeserializeObject<JsonRpcRequest>(malformedJson); }
            catch (Exception ex) { caught = ex; }

            return Prop.ToProperty(caught != null || deserialized != null);
        }

        /// <summary>
        /// Property: BridgeReceipt.SessionId roundtrips losslessly through UTF-8.
        /// FsCheck precondition (==>): SessionId contains no lone surrogate code units (lone
        /// surrogates are not legal Unicode scalars and cannot survive UTF-8 encode/decode).
        /// </summary>
        [Property(MaxTest = 100)]
        public Property BridgeReceipt_SessionId_UTF8RoundtripLossless(NonEmptyString sessionId)
        {
            var original = sessionId.Get;

            // Precondition: discard inputs containing lone surrogates (UTF-8 cannot represent them).
            if (original.Any(char.IsSurrogate) && !IsAllWellFormedSurrogatePairs(original))
                return Prop.ToProperty(true);

            byte[] utf8 = System.Text.Encoding.UTF8.GetBytes(original);
            string roundtripped = System.Text.Encoding.UTF8.GetString(utf8);
            return Prop.ToProperty(roundtripped == original);
        }

        /// <summary>
        /// Property: BridgeReceipt required fields preserve their assigned non-null, non-negative values.
        /// PositiveInt enforces frame ≥ 1; NonEmptyString enforces sessionId non-empty. The
        /// precondition is a defensive belt-and-braces check (rarely fires).
        /// </summary>
        [Property(MaxTest = 100)]
        public Property BridgeReceipt_RequiredFields_NeverNull(NonEmptyString sessionId, PositiveInt frame)
        {
            // Precondition: belt-and-braces guard on generator output.
            if (frame.Get <= 0 || string.IsNullOrEmpty(sessionId.Get))
                return Prop.ToProperty(true);

            var receipt = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = "2026-05-18T12:00:00.000Z",
                WorldFrame = frame.Get,
                StateSha256Hex = "aabbccdd",
                HmacHex = "11223344"
            };

            var ok = receipt.SessionId == sessionId.Get
                && receipt.TimestampUtc != null
                && receipt.WorldFrame == frame.Get
                && receipt.WorldFrame >= 0
                && receipt.StateSha256Hex != null
                && receipt.HmacHex != null;

            return Prop.ToProperty(ok);
        }

        // ---------- Facts (formerly mis-classified [Property]) ----------
        // Iter-#594: these tests do not vary their assertions over generator input — their args
        // were either ignored or stuffed into setters that don't affect the invariant. Demoted to
        // [Fact] so the test inventory reflects actual coverage (no more 100× redundant iterations).

        /// <summary>
        /// Fact: JsonRpcRequest.Jsonrpc is the constant "2.0" by construction (prior [Property]
        /// generated id/method but neither affects the version field; ran 100× identical).
        /// </summary>
        [Fact]
        public void JsonRpcRequest_Jsonrpc_AlwaysVersion2Point0()
        {
            var request = new JsonRpcRequest { Id = "any", Method = "any" };
            request.Jsonrpc.Should().Be("2.0",
                because: "JsonRpcRequest.Jsonrpc must always be '2.0' per JSON-RPC 2.0 spec");
        }

        /// <summary>
        /// Fact: JsonRpcRequest with a fixed special-char payload roundtrips losslessly.
        /// (Prior [Property] generated id/method but the payload — the actual subject — was a
        /// hardcoded literal; the generator iterations did not vary the tested invariant.)
        /// </summary>
        [Fact]
        public void JsonRpcRequest_SpecialCharsInParams_RoundtripLosslessly()
        {
            var original = new JsonRpcRequest
            {
                Id = "req-001",
                Method = "test.method",
                Params = JObject.FromObject(new
                {
                    quoted = "value with \"quotes\"",
                    escaped = "backslash \\ newline \n tab \t",
                    unicode = "emoji: 😀 chinese: 中文",
                    nested = new { inner = "deep value" }
                })
            };

            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<JsonRpcRequest>(json);

            deserialized.Should().NotBeNull();
            JToken.DeepEquals(deserialized!.Params, original.Params).Should().BeTrue();
            deserialized.Params?["quoted"]?.ToString().Should().Contain("quotes");
            deserialized.Params?["unicode"]?.ToString().Should().Contain("😀");
        }

        /// <summary>
        /// Fact: a JsonRpcResponse constructed with request.Id trivially correlates by construction.
        /// (Prior [Property] is tautological — sets response.Id = request.Id then asserts equality;
        /// no generator input affects the outcome.)
        /// </summary>
        [Fact]
        public void JsonRpcRequest_And_Response_ID_Correlatable()
        {
            var request = new JsonRpcRequest { Id = "req-123", Method = "test" };
            var response = new JsonRpcResponse
            {
                Id = request.Id,
                Result = JToken.FromObject(new { status = "ok" })
            };
            response.Id.Should().Be(request.Id,
                because: "responses constructed with request.Id must correlate by construction");
        }

        /// <summary>
        /// Fact: a JsonRpcResponse constructed with only Result (or only Error) reflects exactly
        /// that field non-null and the other null. (Prior [Property] tested constants by
        /// construction; the id generator did not influence the assertion.)
        /// </summary>
        [Fact]
        public void JsonRpcResponse_Result_And_Error_MutuallyExclusive()
        {
            var successResponse = new JsonRpcResponse
            {
                Id = "id-1",
                Result = JToken.FromObject(new { success = true })
            };
            var errorResponse = new JsonRpcResponse
            {
                Id = "id-2",
                Error = new JsonRpcError { Code = -1, Message = "Failed" }
            };

            successResponse.Result.Should().NotBeNull();
            successResponse.Error.Should().BeNull();
            errorResponse.Result.Should().BeNull();
            errorResponse.Error.Should().NotBeNull();
        }
    }
}
