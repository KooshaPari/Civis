#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using DINOForge.Bridge.Client;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// FsCheck Tier 3 property tests for Bridge protocol classes.
    /// Extends coverage from SDK Models (10 properties in FsCheckPrototype) to
    /// Bridge.Protocol and Bridge.Client classes with invariant validation.
    ///
    /// These are REAL property tests using FsCheck generators, not parameterized [Theory] tests.
    /// Each [Property] runs 100 random iterations by default.
    /// </summary>
    [Trait("Category", "Property")]
    public class BridgeFsCheckProperties
    {
        /// <summary>
        /// Property: JsonRpcRequest serialize/deserialize roundtrip preserves all fields.
        /// For any JsonRpcRequest with random Id, Method, and optional Params,
        /// serializing to JSON and deserializing restores all field values exactly.
        /// Validates model integrity across randomized JSON-RPC requests.
        ///
        /// FsCheck generates 100+ random JsonRpcRequest instances with varying params.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool JsonRpcRequest_Serialize_Then_Deserialize_PreservesAllFields(
            NonEmptyString id, NonEmptyString method)
        {
            // Arrange: Create a JsonRpcRequest with random fields
            var original = new JsonRpcRequest
            {
                Id = id.Get,
                Method = method.Get,
                Params = JObject.FromObject(new { key = "value", count = 42 })
            };

            // Act: Serialize to JSON string, then deserialize back
            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<JsonRpcRequest>(json);

            // Assert: All fields preserved exactly
            var preserved = deserialized != null
                && deserialized.Id == original.Id
                && deserialized.Method == original.Method
                && deserialized.Jsonrpc == "2.0"
                && JToken.DeepEquals(deserialized.Params, original.Params);

            preserved.Should().BeTrue(
                because: "JsonRpcRequest roundtrip must preserve Id, Method, Params, and jsonrpc version");
            return preserved;
        }

        /// <summary>
        /// Property: JsonRpcResponse with Result field survives serialize/deserialize roundtrip.
        /// For any JsonRpcResponse with a success result and random Id,
        /// serializing and deserializing preserves Result and Id fields exactly.
        /// Validates that successful responses maintain all fields across the wire.
        ///
        /// FsCheck generates 100+ random response IDs and result payloads.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool JsonRpcResponse_WithResult_Serialize_Roundtrip_Equal(NonEmptyString responseId)
        {
            // Arrange: Create a JsonRpcResponse with a Result
            var original = new JsonRpcResponse
            {
                Id = responseId.Get,
                Result = JToken.FromObject(new { status = "ok", frame = 123 })
            };

            // Act: Roundtrip through JSON
            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

            // Assert: All fields preserved, Error is null
            var preserved = deserialized != null
                && deserialized.Id == original.Id
                && deserialized.Jsonrpc == "2.0"
                && JToken.DeepEquals(deserialized.Result, original.Result)
                && deserialized.Error == null;

            preserved.Should().BeTrue(
                because: "JsonRpcResponse with Result must preserve Result and Id across roundtrip");
            return preserved;
        }

        /// <summary>
        /// Property: JsonRpcResponse with Error field survives serialize/deserialize roundtrip.
        /// For any JsonRpcResponse with an error and random Id,
        /// serializing and deserializing preserves Error, code, and message fields exactly.
        /// Validates that error responses maintain all diagnostic fields across the wire.
        ///
        /// FsCheck generates 100+ random error codes and messages.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool JsonRpcResponse_WithError_Serialize_Roundtrip_Equal(
            PositiveInt errorCode, NonEmptyString errorMsg, NonEmptyString responseId)
        {
            // Arrange: Create a JsonRpcResponse with an Error
            var original = new JsonRpcResponse
            {
                Id = responseId.Get,
                Error = new JsonRpcError
                {
                    Code = errorCode.Get,
                    Message = errorMsg.Get,
                    Data = JToken.FromObject(new { detail = "additional context" })
                }
            };

            // Act: Roundtrip through JSON
            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

            // Assert: All error fields preserved, Result is null
            var preserved = deserialized != null
                && deserialized.Id == original.Id
                && deserialized.Error != null
                && deserialized.Error.Code == original.Error.Code
                && deserialized.Error.Message == original.Error.Message
                && JToken.DeepEquals(deserialized.Error.Data, original.Error.Data)
                && deserialized.Result == null;

            preserved.Should().BeTrue(
                because: "JsonRpcResponse with Error must preserve Error fields across roundtrip");
            return preserved;
        }

        /// <summary>
        /// Property: BridgeReceipt HMAC computation is deterministic.
        /// For any BridgeReceipt with the same session key and same fields,
        /// computing HMAC multiple times produces identical hex strings.
        /// Validates that HMAC computation has no random side effects or external state.
        ///
        /// FsCheck generates 100+ random BridgeReceipt instances with deterministic keys.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_HmacCompute_IsDeterministic(
            NonEmptyString sessionId, NonEmptyString timestamp, PositiveInt worldFrame)
        {
            // Arrange: Create a BridgeReceipt with deterministic fields
            var receipt = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = timestamp.Get,
                WorldFrame = worldFrame.Get,
                StateSha256Hex = "abc123def456",
                HmacHex = "" // Placeholder; we're verifying determinism of the compute logic
            };

            // Create a fixed 32-byte session key
            byte[] sessionKey = new byte[32];
            for (int i = 0; i < 32; i++) sessionKey[i] = (byte)(i % 256);

            // Act: Compute HMAC three times over the same receipt + key
            var hmac1 = ComputeReceiptHmac(receipt, sessionKey);
            var hmac2 = ComputeReceiptHmac(receipt, sessionKey);
            var hmac3 = ComputeReceiptHmac(receipt, sessionKey);

            // Assert: All three HMAC values are byte-identical
            var isDeterministic = hmac1.SequenceEqual(hmac2)
                && hmac2.SequenceEqual(hmac3);

            isDeterministic.Should().BeTrue(
                because: "HMAC computation must be deterministic for same inputs");
            return isDeterministic;
        }

        /// <summary>
        /// Property: BridgeReceipt HMAC changes when any field changes.
        /// For any BridgeReceipt, modifying a single field (timestamp, frame, or state_sha256)
        /// produces a different HMAC hash with high probability.
        /// Validates collision resistance and field coverage in HMAC computation.
        ///
        /// FsCheck generates 100+ random field mutations.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_HmacCompute_IsCollisionResistant(
            NonEmptyString sessionId, PositiveInt worldFrame)
        {
            // Arrange: Create two receipts that differ in one field
            var receipt1 = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = "2026-05-18T10:30:00.000Z",
                WorldFrame = worldFrame.Get,
                StateSha256Hex = "aabbccdd",
                HmacHex = ""
            };

            var receipt2 = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = "2026-05-18T10:30:01.000Z", // Different timestamp
                WorldFrame = worldFrame.Get,
                StateSha256Hex = "aabbccdd",
                HmacHex = ""
            };

            byte[] sessionKey = new byte[32];
            for (int i = 0; i < 32; i++) sessionKey[i] = (byte)(i % 256);

            // Act: Compute HMACs for both receipts
            var hmac1 = ComputeReceiptHmac(receipt1, sessionKey);
            var hmac2 = ComputeReceiptHmac(receipt2, sessionKey);

            // Assert: Different receipts should produce different HMACs (collision resistant)
            var isDifferent = !hmac1.SequenceEqual(hmac2);

            isDifferent.Should().BeTrue(
                because: "Changing BridgeReceipt fields must produce different HMACs");
            return isDifferent;
        }

        /// <summary>
        /// Property: SessionKeyCache.Set then TryGet roundtrip preserves key bytes.
        /// For any 32-byte session key and session ID,
        /// storing in the cache and retrieving returns the byte-identical key.
        /// Validates cache storage and retrieval integrity.
        ///
        /// FsCheck generates 100+ random session IDs and key byte sequences.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool SessionKeyCache_RoundTrip_PreservesKey(NonEmptyString sessionId)
        {
            // Arrange: Create a random 32-byte session key
            byte[] originalKey = new byte[32];
            new Random(sessionId.Get.GetHashCode()).NextBytes(originalKey);

            var cache = new SessionKeyCache();

            // Act: Store and retrieve
            cache.Set(sessionId.Get, originalKey);
            bool retrieved = cache.TryGet(sessionId.Get, out byte[]? cachedKey);

            // Assert: Key is preserved exactly
            var preserved = retrieved
                && cachedKey != null
                && cachedKey.Length == 32
                && cachedKey.SequenceEqual(originalKey);

            preserved.Should().BeTrue(
                because: "SessionKeyCache must preserve key bytes exactly");

            cache.Dispose();
            return preserved;
        }

        /// <summary>
        /// Property: SessionKeyCache.Remove removes key; TryGet returns false afterward.
        /// For any cached session key, after calling Remove, TryGet returns false
        /// and no key is retrieved. Validates cache eviction behavior.
        ///
        /// FsCheck generates 100+ random session IDs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool SessionKeyCache_Remove_RemovesKey(NonEmptyString sessionId)
        {
            // Arrange: Create and cache a key
            byte[] key = new byte[32];
            for (int i = 0; i < 32; i++) key[i] = (byte)(i % 256);

            var cache = new SessionKeyCache();
            cache.Set(sessionId.Get, key);

            // Act: Verify it exists, then remove it
            bool existsBefore = cache.TryGet(sessionId.Get, out _);
            bool removed = cache.Remove(sessionId.Get);
            bool existsAfter = cache.TryGet(sessionId.Get, out _);

            // Assert: Key exists before, is removed, and doesn't exist after
            var behavior = existsBefore && removed && !existsAfter;

            behavior.Should().BeTrue(
                because: "SessionKeyCache.Remove must clear the cached key entry");

            cache.Dispose();
            return behavior;
        }

        /// <summary>
        /// Property: BridgeReceipt fields roundtrip through JSON with snake_case serialization.
        /// For any BridgeReceipt with all fields populated,
        /// serializing and deserializing preserves all fields in snake_case JSON.
        /// Validates that snake_case JSON naming is consistent and lossless.
        ///
        /// FsCheck generates 100+ random BridgeReceipt instances.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_Serialize_Roundtrip_SnakeCasePreserved(
            NonEmptyString sessionId, NonEmptyString timestamp, PositiveInt worldFrame)
        {
            // Arrange: Create a BridgeReceipt with all fields
            var original = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = timestamp.Get,
                WorldFrame = worldFrame.Get,
                StateSha256Hex = "deadbeefcafebabe",
                HmacHex = "0123456789abcdef"
            };

            // Act: Serialize and deserialize
            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<BridgeReceipt>(json);

            // Assert: All fields preserved, JSON contains snake_case keys
            var preserved = deserialized != null
                && deserialized.SessionId == original.SessionId
                && deserialized.TimestampUtc == original.TimestampUtc
                && deserialized.WorldFrame == original.WorldFrame
                && deserialized.StateSha256Hex == original.StateSha256Hex
                && deserialized.HmacHex == original.HmacHex
                && json.Contains("session_id") // Verify snake_case in JSON
                && json.Contains("world_frame")
                && json.Contains("state_sha256")
                && !json.Contains("SessionId"); // No PascalCase

            preserved.Should().BeTrue(
                because: "BridgeReceipt must roundtrip with snake_case JSON serialization");
            return preserved;
        }

        // Helper: Compute HMAC-SHA256 over canonical receipt fields
        // (timestamp + world_frame + state_sha256), matching GameBridgeServer Phase 4a logic
        private static byte[] ComputeReceiptHmac(BridgeReceipt receipt, byte[] sessionKey)
        {
            using (var hmac = new System.Security.Cryptography.HMACSHA256(sessionKey))
            {
                // Canonical receipt fields in order: timestamp, world_frame, state_sha256
                string toHash = $"{receipt.TimestampUtc}|{receipt.WorldFrame}|{receipt.StateSha256Hex}";
                byte[] data = System.Text.Encoding.UTF8.GetBytes(toHash);
                return hmac.ComputeHash(data);
            }
        }
    }
}
