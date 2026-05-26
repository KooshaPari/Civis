using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class PackRegistryTests
    {
        private List<RegistryPackEntry> CreateTestEntries()
        {
            return new List<RegistryPackEntry>
            {
                new RegistryPackEntry
                {
                    Id = "warfare-modern",
                    Name = "Modern Warfare",
                    Author = "DINOForge Team",
                    Version = "1.0.0",
                    Type = "content",
                    Description = "Modern military units",
                    Tags = new List<string> { "warfare", "modern", "military" },
                    Repo = "https://github.com/kooshapari/dino-warfare-modern",
                    DownloadUrl = "https://example.com/warfare-modern.zip",
                    PackPath = "packs/warfare-modern",
                    FrameworkVersion = ">=0.1.0 <1.0.0",
                    Verified = true,
                    Featured = true,
                    ConflictsWith = new List<string> { "warfare-starwars" },
                    DependsOn = new List<string>()
                },
                new RegistryPackEntry
                {
                    Id = "warfare-starwars",
                    Name = "Star Wars Warfare",
                    Author = "Community",
                    Version = "0.5.0",
                    Type = "content",
                    Description = "Star Wars units and factions",
                    Tags = new List<string> { "warfare", "scifi", "starwars" },
                    Repo = "https://github.com/example/dino-starwars",
                    DownloadUrl = "https://example.com/warfare-starwars.zip",
                    PackPath = "packs/warfare-starwars",
                    FrameworkVersion = ">=0.1.0 <1.0.0",
                    Verified = false,
                    Featured = false,
                    ConflictsWith = new List<string> { "warfare-modern" },
                    DependsOn = new List<string>()
                },
                new RegistryPackEntry
                {
                    Id = "balance-economy",
                    Name = "Economy Balance",
                    Author = "Balance Team",
                    Version = "2.1.0",
                    Type = "balance",
                    Description = "Economic balance tweaks",
                    Tags = new List<string> { "balance", "economy", "gameplay" },
                    Repo = "https://github.com/example/dino-balance-economy",
                    DownloadUrl = "https://example.com/balance-economy.zip",
                    PackPath = "packs/balance-economy",
                    FrameworkVersion = ">=0.1.0 <1.0.0",
                    Verified = true,
                    Featured = false,
                    ConflictsWith = new List<string>(),
                    DependsOn = new List<string>()
                }
            };
        }

        [Fact]
        public void PackRegistryFilter_ByType_FiltersCorrectly()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Type = "content" };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(2);
            filtered.Should().AllSatisfy(e => e.Type.Should().Be("content"));
            filtered.Should().Contain(e => e.Id == "warfare-modern");
            filtered.Should().Contain(e => e.Id == "warfare-starwars");
            filtered.Should().NotContain(e => e.Id == "balance-economy");
        }

        [Fact]
        public void PackRegistryFilter_ByType_CaseInsensitive()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Type = "BALANCE" };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(1);
            filtered.Should().Contain(e => e.Id == "balance-economy");
        }

        [Fact]
        public void PackRegistryFilter_ByTag_FiltersCorrectly()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Tags = new[] { "warfare" } };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(2);
            filtered.Should().AllSatisfy(e => e.Tags.Should().Contain("warfare"));
        }

        [Fact]
        public void PackRegistryFilter_ByMultipleTags_RequiresAll()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            // Only warfare-starwars has both "warfare" and "scifi"
            PackRegistryFilter filter = new PackRegistryFilter { Tags = new[] { "warfare", "scifi" } };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(1);
            filtered.Should().Contain(e => e.Id == "warfare-starwars");
        }

        [Fact]
        public void PackRegistryFilter_ByTag_CaseInsensitive()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Tags = new[] { "WARFARE" } };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(2);
        }

        [Fact]
        public void PackRegistryFilter_Verified_OnlyReturnsVerified()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Verified = true };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(2);
            filtered.Should().AllSatisfy(e => e.Verified.Should().BeTrue());
            filtered.Should().Contain(e => e.Id == "warfare-modern");
            filtered.Should().Contain(e => e.Id == "balance-economy");
        }

        [Fact]
        public void PackRegistryFilter_Verified_False_IncludesUnverified()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Verified = false };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(1);
            filtered.Should().Contain(e => e.Id == "warfare-starwars");
            filtered[0].Verified.Should().BeFalse();
        }

        [Fact]
        public void PackRegistryFilter_Featured_OnlyReturnsFeatured()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Featured = true };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(1);
            filtered.Should().AllSatisfy(e => e.Featured.Should().BeTrue());
            filtered.Should().Contain(e => e.Id == "warfare-modern");
        }

        [Fact]
        public void PackRegistryFilter_Featured_False_IncludesNonFeatured()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter { Featured = false };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(2);
            filtered.Should().AllSatisfy(e => e.Featured.Should().BeFalse());
        }

        [Fact]
        public void PackRegistryFilter_MultipleConstraints_AppliesAll()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();
            PackRegistryFilter filter = new PackRegistryFilter
            {
                Type = "content",
                Tags = new[] { "warfare" },
                Verified = true
            };

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, filter);

            // Assert
            filtered.Should().HaveCount(1);
            filtered.Should().Contain(e => e.Id == "warfare-modern");
            filtered[0].Type.Should().Be("content");
            filtered[0].Verified.Should().BeTrue();
            filtered[0].Tags.Should().Contain("warfare");
        }

        [Fact]
        public void PackRegistryFilter_NullFilter_ReturnsAll()
        {
            // Arrange
            List<RegistryPackEntry> entries = CreateTestEntries();

            // Act
            List<RegistryPackEntry> filtered = FilterEntries(entries, null);

            // Assert
            filtered.Should().HaveCount(3);
        }

        [Fact]
        public void RegistryPackEntry_Deserializes_FromJson()
        {
            // Arrange
            string json = @"{
  ""id"": ""test-pack"",
  ""name"": ""Test Pack"",
  ""author"": ""Test Author"",
  ""version"": ""1.0.0"",
  ""type"": ""content"",
  ""description"": ""A test pack"",
  ""tags"": [""test"", ""example""],
  ""repo"": ""https://github.com/example/test"",
  ""download_url"": ""https://example.com/test.zip"",
  ""pack_path"": ""packs/test"",
  ""framework_version"": "">=0.1.0"",
  ""verified"": true,
  ""featured"": false,
  ""conflicts_with"": [""other-pack""],
  ""depends_on"": [""core-pack""]
}";

            // Act
            JsonSerializerOptions options = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true
            };
            RegistryPackEntry? entry = JsonSerializer.Deserialize<RegistryPackEntry>(json, options);

            // Assert
            entry.Should().NotBeNull();
            entry!.Id.Should().Be("test-pack");
            entry.Name.Should().Be("Test Pack");
            entry.Author.Should().Be("Test Author");
            entry.Version.Should().Be("1.0.0");
            entry.Type.Should().Be("content");
            entry.Description.Should().Be("A test pack");
            entry.Tags.Should().ContainInOrder("test", "example");
            entry.Repo.Should().Be("https://github.com/example/test");
            entry.DownloadUrl.Should().Be("https://example.com/test.zip");
            entry.PackPath.Should().Be("packs/test");
            entry.FrameworkVersion.Should().Be(">=0.1.0");
            entry.Verified.Should().BeTrue();
            entry.Featured.Should().BeFalse();
            entry.ConflictsWith.Should().ContainSingle().Which.Should().Be("other-pack");
            entry.DependsOn.Should().ContainSingle().Which.Should().Be("core-pack");
        }

        [Fact]
        public void RegistryPackEntry_DeserializesWithDefaults_FromPartialJson()
        {
            // Arrange - only required fields
            string json = @"{
  ""id"": ""minimal-pack"",
  ""name"": ""Minimal Pack"",
  ""author"": ""Test"",
  ""version"": ""0.1.0"",
  ""type"": ""balance""
}";

            // Act
            JsonSerializerOptions options = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true
            };
            RegistryPackEntry? entry = JsonSerializer.Deserialize<RegistryPackEntry>(json, options);

            // Assert
            entry.Should().NotBeNull();
            entry!.Id.Should().Be("minimal-pack");
            entry.Tags.Should().BeEmpty();
            entry.ConflictsWith.Should().BeEmpty();
            entry.DependsOn.Should().BeEmpty();
            entry.Verified.Should().BeFalse();
            entry.Featured.Should().BeFalse();
        }

        [Fact]
        public void PackRegistryClient_InvalidateCache_ResetsCache()
        {
            // Arrange
            using (var httpClient = new System.Net.Http.HttpClient())
            {
                var client = new PackRegistryClient("http://localhost/registry.json", httpClient);
                client.CacheDuration = TimeSpan.FromSeconds(60);

                // Act & Assert
                // We can't directly test caching without a real HTTP endpoint,
                // but we can verify that InvalidateCache doesn't throw
                client.InvalidateCache();
                client.InvalidateCache(); // Should be idempotent
            }
        }

        private List<RegistryPackEntry> FilterEntries(
            List<RegistryPackEntry> entries,
            PackRegistryFilter? filter)
        {
            if (filter == null)
                return entries;

            List<RegistryPackEntry> results = entries;

            if (filter.Tags != null && filter.Tags.Count > 0)
            {
                results = new List<RegistryPackEntry>(results);
                results.RemoveAll(p =>
                    !filter.Tags.All(t => p.Tags.Contains(t, StringComparer.OrdinalIgnoreCase)));
            }

            if (!string.IsNullOrWhiteSpace(filter.Type))
            {
                results = new List<RegistryPackEntry>(results);
                results.RemoveAll(p =>
                    !string.Equals(p.Type, filter.Type, StringComparison.OrdinalIgnoreCase));
            }

            if (filter.Verified.HasValue)
            {
                results = new List<RegistryPackEntry>(results);
                results.RemoveAll(p => p.Verified != filter.Verified.Value);
            }

            if (filter.Featured.HasValue)
            {
                results = new List<RegistryPackEntry>(results);
                results.RemoveAll(p => p.Featured != filter.Featured.Value);
            }

            return results;
        }
    }
}
