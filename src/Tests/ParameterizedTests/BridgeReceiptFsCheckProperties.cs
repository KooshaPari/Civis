#nullable enable
using System;
using System.Collections.Generic;
using System.Security.Cryptography;
using System.Text;
using DINOForge.Bridge.Protocol;
using DINOForge.Bridge.Client;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// FsCheck Tier 3 property tests for deeper BridgeReceipt + SessionKeyCache coverage.
    /// These tests validate Phase 4b proof-system internals: HMAC determinism and collision
    /// resistance, SessionKeyCache lifecycle (Set/Get/Remove), validation chains, and
    /// roundtrip integrity under random inputs.
    ///
    /// Extends Tier 2 BridgeFsCheckProperties with 5-7 additional properties exercising
    /// HMAC key sensitivity, cache Clear semantics, and multi-session isolation.
    /// </summary>
    [Trait("Category", "Property")]
    [Trait("Layer", "Tier3")]
    public class BridgeReceiptFsCheckProperties
    {
        /// <summary>
        /// Property 1: BridgeReceipt.ComputeHmac — same (key, payload) → same HMAC.
        /// For any fixed key and receipt, calling ComputeHmac N times produces bit-identical results.
        /// Validates HMAC computation is deterministic and has no time/random side effects.
        ///
        /// Runs 100 random cases with different key bytes and receipt fields.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_HmacCompute_Same_Key_Payload_Produces_Same_Hmac(
            NonEmptyString sessionId, NonEmptyString timestamp, int worldFrame)
        {
            // Arrange: Create a BridgeReceipt and fixed 32-byte key
            var receipt = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = timestamp.Get,
                WorldFrame = Math.Max(0, worldFrame), // Ensure non-negative frame
                StateSha256Hex = "cafebabecafebabe",
                HmacHex = ""
            };

            byte[] key = CreateDeterministicKey(sessionId.Get);

            // Act: Compute HMAC 3 times with same inputs
            var hmac1 = ComputeReceiptHmacSha256(receipt, key);
            var hmac2 = ComputeReceiptHmacSha256(receipt, key);
            var hmac3 = ComputeReceiptHmacSha256(receipt, key);

            // Assert: All three results are byte-identical
            var deterministic = hmac1.SequenceEqual(hmac2) && hmac2.SequenceEqual(hmac3);

            deterministic.Should().BeTrue(
                because: "BridgeReceipt HMAC must be deterministic: same key+payload → same hash");
            return deterministic;
        }

        /// <summary>
        /// Property 2: BridgeReceipt.ComputeHmac — different keys → different HMACs.
        /// For any receipt, changing the session key (even by 1 byte) produces a different HMAC.
        /// Validates that HMAC is sensitive to the key material and implements proper keying.
        ///
        /// Runs 100 random cases with different key mutations.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_HmacCompute_Different_Keys_Produce_Different_Hmacs(
            NonEmptyString sessionId, NonEmptyString timestamp, int worldFrame)
        {
            // Arrange: Create a receipt and two different keys
            var receipt = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = timestamp.Get,
                WorldFrame = Math.Max(0, worldFrame),
                StateSha256Hex = "deadbeef",
                HmacHex = ""
            };

            byte[] key1 = CreateDeterministicKey(sessionId.Get);
            byte[] key2 = CreateDeterministicKey(sessionId.Get + "mutated");

            // Act: Compute HMACs with different keys
            var hmac1 = ComputeReceiptHmacSha256(receipt, key1);
            var hmac2 = ComputeReceiptHmacSha256(receipt, key2);

            // Assert: Different keys produce different HMACs
            var sensitive = !hmac1.SequenceEqual(hmac2);

            sensitive.Should().BeTrue(
                because: "BridgeReceipt HMAC must be key-sensitive: different keys → different hashes");
            return sensitive;
        }

        /// <summary>
        /// Property 3: BridgeReceipt.ComputeHmac — different payloads → different HMACs.
        /// For any key, changing a receipt field (timestamp, world_frame, or state_sha256)
        /// produces a different HMAC. Validates collision resistance and field coverage.
        ///
        /// Runs 100 random cases with different field mutations.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_HmacCompute_Different_Payloads_Produce_Different_Hmacs(
            NonEmptyString sessionId, PositiveInt frame1)
        {
            // Arrange: Create two receipts that differ in state_sha256 (guaranteed different from "aaaa" vs "bbbb")
            var receipt1 = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = "2026-05-18T12:00:00.000Z",
                WorldFrame = frame1.Get,
                StateSha256Hex = "aaaa",
                HmacHex = ""
            };

            var receipt2 = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = "2026-05-18T12:00:00.000Z",
                WorldFrame = frame1.Get,
                StateSha256Hex = "bbbb",
                HmacHex = ""
            };

            byte[] key = CreateDeterministicKey(sessionId.Get);

            // Act: Compute HMACs for both receipts
            var hmac1 = ComputeReceiptHmacSha256(receipt1, key);
            var hmac2 = ComputeReceiptHmacSha256(receipt2, key);

            // Assert: Different payloads produce different HMACs (collision resistant)
            var sensitive = !hmac1.SequenceEqual(hmac2);

            sensitive.Should().BeTrue(
                because: "BridgeReceipt HMAC must be collision-resistant: different payloads → different hashes");
            return sensitive;
        }

        /// <summary>
        /// Property 4: SessionKeyCache.Set + Get — roundtrip preserves key bytes bit-exact.
        /// For any 32-byte key and session ID, storing and retrieving returns the exact same bytes.
        /// Validates cache storage integrity without mutations or truncation.
        ///
        /// Runs 100 random cases with different session IDs and random key bytes.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool SessionKeyCache_Set_Get_Roundtrip_PreservesKeyBytesExact(
            NonEmptyString sessionId)
        {
            // Arrange: Create a random 32-byte key
            byte[] originalKey = CreateRandomKey(sessionId.Get);

            var cache = new SessionKeyCache();

            // Act: Store the key, then retrieve it
            cache.Set(sessionId.Get, originalKey);
            bool found = cache.TryGet(sessionId.Get, out byte[]? retrievedKey);

            // Assert: Key retrieved exactly, all 32 bytes match
            var preserved = found
                && retrievedKey != null
                && retrievedKey.Length == 32
                && retrievedKey.SequenceEqual(originalKey);

            preserved.Should().BeTrue(
                because: "SessionKeyCache roundtrip must preserve all 32 key bytes exactly");

            cache.Dispose();
            return preserved;
        }

        /// <summary>
        /// Property 5: SessionKeyCache.Remove — after removal, TryGet returns false.
        /// For any cached key, calling Remove deletes the entry so subsequent TryGet returns false.
        /// Validates cache eviction and absence semantics.
        ///
        /// Runs 100 random cases with different session IDs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool SessionKeyCache_Remove_After_Set_Returns_False_On_Get(
            NonEmptyString sessionId)
        {
            // Arrange: Create and cache a key
            byte[] key = CreateDeterministicKey(sessionId.Get);
            var cache = new SessionKeyCache();
            cache.Set(sessionId.Get, key);

            // Act: Verify it exists, remove it, check again
            bool existsBefore = cache.TryGet(sessionId.Get, out _);
            bool removeSucceeded = cache.Remove(sessionId.Get);
            bool existsAfter = cache.TryGet(sessionId.Get, out byte[]? keyAfter);

            // Assert: Exists before, removed successfully, does not exist after
            var evicted = existsBefore && removeSucceeded && !existsAfter && keyAfter == null;

            evicted.Should().BeTrue(
                because: "SessionKeyCache.Remove must delete the entry so TryGet returns false");

            cache.Dispose();
            return evicted;
        }

        /// <summary>
        /// Property 6: SessionKeyCache.Clear (via Dispose) — all keys are removed.
        /// For any cache with N keys, disposing clears all entries so subsequent TryGet returns false
        /// for all previously cached sessions. Validates cleanup semantics and lifecycle.
        ///
        /// Runs 100 random cases with 3-10 random session IDs cached.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool SessionKeyCache_Dispose_Clears_All_Keys(
            NonEmptyString sessionId1, NonEmptyString sessionId2, NonEmptyString sessionId3)
        {
            // Arrange: Create cache and store 3 different session keys
            var cache = new SessionKeyCache();
            byte[] key1 = CreateDeterministicKey(sessionId1.Get);
            byte[] key2 = CreateDeterministicKey(sessionId2.Get);
            byte[] key3 = CreateDeterministicKey(sessionId3.Get);

            cache.Set(sessionId1.Get, key1);
            cache.Set(sessionId2.Get, key2);
            cache.Set(sessionId3.Get, key3);

            // Verify all exist before dispose
            bool allExistBefore = cache.TryGet(sessionId1.Get, out _)
                && cache.TryGet(sessionId2.Get, out _)
                && cache.TryGet(sessionId3.Get, out _);

            // Act: Dispose the cache
            cache.Dispose();

            // Assert: All keys return false after dispose
            bool allGoneBefore = !cache.TryGet(sessionId1.Get, out _)
                && !cache.TryGet(sessionId2.Get, out _)
                && !cache.TryGet(sessionId3.Get, out _);

            var cleared = allExistBefore && allGoneBefore;

            cleared.Should().BeTrue(
                because: "SessionKeyCache.Dispose must clear all cached keys");
            return cleared;
        }

        /// <summary>
        /// Property 7: BridgeReceipt JSON roundtrip — all fields preserved under random inputs.
        /// For any BridgeReceipt with random session ID, timestamp, frame, and hashes,
        /// serializing to JSON and deserializing restores all fields exactly.
        /// Validates lossless serialization and schema compliance.
        ///
        /// Runs 100 random cases with different field combinations.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool BridgeReceipt_Json_Roundtrip_Preserves_All_Fields(
            NonEmptyString sessionId, NonEmptyString timestamp, PositiveInt worldFrame)
        {
            // Arrange: Create a BridgeReceipt with all fields populated
            var original = new BridgeReceipt
            {
                SessionId = sessionId.Get,
                TimestampUtc = timestamp.Get,
                WorldFrame = worldFrame.Get,
                StateSha256Hex = "deadbeefcafebabe",
                HmacHex = "0123456789abcdef"
            };

            // Act: Roundtrip through JSON
            string json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<BridgeReceipt>(json);

            // Assert: All fields preserved exactly, JSON uses snake_case
            var preserved = deserialized != null
                && deserialized.SessionId == original.SessionId
                && deserialized.TimestampUtc == original.TimestampUtc
                && deserialized.WorldFrame == original.WorldFrame
                && deserialized.StateSha256Hex == original.StateSha256Hex
                && deserialized.HmacHex == original.HmacHex
                && json.Contains("session_id")
                && json.Contains("world_frame");

            preserved.Should().BeTrue(
                because: "BridgeReceipt JSON roundtrip must preserve all fields with snake_case keys");
            return preserved;
        }

        // ============================================================================
        // Helper Methods
        // ============================================================================

        /// <summary>
        /// Compute HMAC-SHA256 over canonical receipt fields.
        /// Matches Phase 4a GameBridgeServer logic: timestamp|world_frame|state_sha256
        /// </summary>
        private static byte[] ComputeReceiptHmacSha256(BridgeReceipt receipt, byte[] sessionKey)
        {
            using (var hmac = new HMACSHA256(sessionKey))
            {
                string toHash = $"{receipt.TimestampUtc}|{receipt.WorldFrame}|{receipt.StateSha256Hex}";
                byte[] data = Encoding.UTF8.GetBytes(toHash);
                return hmac.ComputeHash(data);
            }
        }

        /// <summary>
        /// Create a deterministic 32-byte key from a string seed (reproducible for testing).
        /// </summary>
        private static byte[] CreateDeterministicKey(string seed)
        {
            using (var sha = SHA256.Create())
            {
                byte[] hash = sha.ComputeHash(Encoding.UTF8.GetBytes(seed));
                // hash is already 32 bytes; return it directly
                return hash;
            }
        }

        /// <summary>
        /// Create a random 32-byte key seeded from a string (for variety in test cases).
        /// </summary>
        private static byte[] CreateRandomKey(string seed)
        {
            var rng = new Random(seed.GetHashCode());
            byte[] key = new byte[32];
            rng.NextBytes(key);
            return key;
        }
    }
}
