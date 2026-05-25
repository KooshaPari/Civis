using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for the disabled-pack-list JSON data contract.
    /// These tests validate the JSON shape used to persist which packs are disabled
    /// across sessions, using <see cref="System.Text.Json.JsonSerializer"/> directly
    /// to verify the round-trip contract independently of any particular service.
    /// </summary>
    public class DisabledPacksPersistenceTests
    {
        // Represents the contract: a JSON object with a "disabled" array of pack IDs.
        private sealed class DisabledPacksDto
        {
            public List<string> Disabled { get; set; } = new List<string>();
        }

        // ─── serialisation ────────────────────────────────────────────────────

        [Fact]
        public void DisabledPackIds_SerializedAsJsonArray_CanBeDeserialized()
        {
            // Arrange
            string json = @"{""Disabled"":[""pack-a"",""pack-b"",""pack-c""]}";

            // Act
            DisabledPacksDto? dto = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert
            dto.Should().NotBeNull();
            dto!.Disabled.Should().HaveCount(3);
            dto.Disabled.Should().Contain("pack-a");
            dto.Disabled.Should().Contain("pack-b");
            dto.Disabled.Should().Contain("pack-c");
        }

        [Fact]
        public void EmptyDisabledList_SerializesToEmptyArray()
        {
            // Arrange
            DisabledPacksDto dto = new DisabledPacksDto { Disabled = new List<string>() };

            // Act
            string json = JsonSerializer.Serialize(dto);
            DisabledPacksDto? roundTripped = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert
            json.Should().Contain("[]");
            roundTripped.Should().NotBeNull();
            roundTripped!.Disabled.Should().BeEmpty();
        }

        [Fact]
        public void RoundTrip_DisabledPackIds_PreservesAllIds()
        {
            // Arrange
            List<string> originalIds = new List<string>
            {
                "warfare-modern",
                "warfare-starwars",
                "economy-balanced",
                "scenario-tutorial"
            };
            DisabledPacksDto original = new DisabledPacksDto { Disabled = originalIds };

            // Act — serialize then deserialize
            string json = JsonSerializer.Serialize(original);
            DisabledPacksDto? restored = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert
            restored.Should().NotBeNull();
            restored!.Disabled.Should().BeEquivalentTo(originalIds);
            restored.Disabled.Should().HaveCount(originalIds.Count);
        }

        // ─── ordering ─────────────────────────────────────────────────────────

        [Fact]
        public void RoundTrip_PreservesInsertionOrder()
        {
            // Arrange
            List<string> ids = new List<string> { "z-pack", "a-pack", "m-pack" };
            DisabledPacksDto dto = new DisabledPacksDto { Disabled = ids };

            // Act
            string json = JsonSerializer.Serialize(dto);
            DisabledPacksDto? restored = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert — order preserved
            restored!.Disabled.Should().ContainInOrder("z-pack", "a-pack", "m-pack");
        }

        // ─── edge cases ───────────────────────────────────────────────────────

        [Fact]
        public void SingleDisabledPack_SerializesAndDeserializesCorrectly()
        {
            // Arrange
            DisabledPacksDto dto = new DisabledPacksDto { Disabled = new List<string> { "only-pack" } };

            // Act
            string json = JsonSerializer.Serialize(dto);
            DisabledPacksDto? restored = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert
            restored!.Disabled.Should().ContainSingle()
                .Which.Should().Be("only-pack");
        }

        [Fact]
        public void DisabledListWithDuplicateIds_PreservesDuplicates()
        {
            // Arrange — contracts should preserve whatever the caller provides,
            // deduplication is the caller's responsibility
            List<string> ids = new List<string> { "pack-a", "pack-a", "pack-b" };
            DisabledPacksDto dto = new DisabledPacksDto { Disabled = ids };

            // Act
            string json = JsonSerializer.Serialize(dto);
            DisabledPacksDto? restored = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert
            restored!.Disabled.Should().HaveCount(3);
            restored.Disabled.Count(id => id == "pack-a").Should().Be(2);
        }

        [Fact]
        public void JsonWithExtraFields_DeserializesGracefully()
        {
            // Arrange — future-proof: extra fields should be ignored
            string json = @"{""Disabled"":[""pack-x""],""Version"":""1.0"",""LastSaved"":""2026-01-01""}";

            // Act
            JsonSerializerOptions options = new JsonSerializerOptions
            {
                // Allow unknown properties (default for System.Text.Json)
            };
            DisabledPacksDto? dto = JsonSerializer.Deserialize<DisabledPacksDto>(json, options);

            // Assert
            dto.Should().NotBeNull();
            dto!.Disabled.Should().ContainSingle().Which.Should().Be("pack-x");
        }

        [Fact]
        public void NullDisabledArray_DeserializedAsNull_HandledSafely()
        {
            // Arrange
            string json = @"{""Disabled"":null}";

            // Act
            DisabledPacksDto? dto = JsonSerializer.Deserialize<DisabledPacksDto>(json);

            // Assert — null array should not throw; caller checks for null
            dto.Should().NotBeNull();
            // Disabled may be null when explicitly set to null in JSON
            // The contract is that callers must guard against null
        }
    }
}
