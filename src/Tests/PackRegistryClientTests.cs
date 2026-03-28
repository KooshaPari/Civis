#nullable enable
using System;
using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public class PackRegistryClientTests
{
    [Fact]
    public async Task GetAllPacksAsync_CachesResults_UntilCacheIsInvalidated()
    {
        string registryJson = CreateRegistryJson(CreateTestEntries());
        RecordingHandler handler = new(registryJson);

        using HttpClient httpClient = new(handler);
        PackRegistryClient client = new("https://example.test/registry.json", httpClient);

        IReadOnlyList<RegistryPackEntry> firstLoad = await client.GetAllPacksAsync();
        IReadOnlyList<RegistryPackEntry> secondLoad = await client.GetAllPacksAsync();

        firstLoad.Should().HaveCount(3);
        firstLoad[0].Id.Should().Be("warfare-modern");
        secondLoad.Should().HaveCount(3);
        handler.RequestCount.Should().Be(1);

        client.InvalidateCache();

        IReadOnlyList<RegistryPackEntry> thirdLoad = await client.GetAllPacksAsync();

        thirdLoad.Should().HaveCount(3);
        handler.RequestCount.Should().Be(2);
    }

    [Fact]
    public async Task SearchAsync_AndConvenienceMethods_FilterCachedRegistryCorrectly()
    {
        string registryJson = CreateRegistryJson(CreateTestEntries());
        RecordingHandler handler = new(registryJson);

        using HttpClient httpClient = new(handler);
        PackRegistryClient client = new("https://example.test/registry.json", httpClient);

        IReadOnlyList<RegistryPackEntry> allPacks = await client.SearchAsync(null);
        allPacks.Should().HaveCount(3);

        IReadOnlyList<RegistryPackEntry> combined = await client.SearchAsync(
            new PackRegistryFilter
            {
                Tags = new[] { "WARFARE", "SCIFI" },
                Type = "CONTENT",
                Verified = false,
                Featured = false
            });

        combined.Should().ContainSingle(pack => pack.Id == "warfare-starwars");

        RegistryPackEntry? byId = await client.GetByIdAsync("BALANCE-ECONOMY");
        byId.Should().NotBeNull();
        byId!.Name.Should().Be("Economy Balance");

        IReadOnlyList<RegistryPackEntry> byTag = await client.GetByTagAsync("WARFARE");
        byTag.Should().HaveCount(2);
        byTag.Should().OnlyContain(
            pack => pack.Tags.Any(tag => string.Equals(tag, "warfare", StringComparison.OrdinalIgnoreCase)));

        IReadOnlyList<RegistryPackEntry> byType = await client.GetByTypeAsync("CONTENT");
        byType.Should().HaveCount(2);
        byType.Should().OnlyContain(pack => string.Equals(pack.Type, "content", StringComparison.OrdinalIgnoreCase));

        IReadOnlyList<RegistryPackEntry> verified = await client.GetVerifiedAsync();
        verified.Should().HaveCount(2);
        verified.Should().OnlyContain(pack => pack.Verified);

        IReadOnlyList<RegistryPackEntry> featured = await client.GetFeaturedAsync();
        featured.Should().ContainSingle(pack => pack.Id == "warfare-modern");

        handler.RequestCount.Should().Be(1);
    }

    [Fact]
    public async Task GetByIdAsync_ReturnsNullForUnknownPackId()
    {
        string registryJson = CreateRegistryJson(CreateTestEntries());
        RecordingHandler handler = new(registryJson);

        using HttpClient httpClient = new(handler);
        PackRegistryClient client = new("https://example.test/registry.json", httpClient);

        RegistryPackEntry? missing = await client.GetByIdAsync("does-not-exist");

        missing.Should().BeNull();
        handler.RequestCount.Should().Be(1);
    }

    private static List<RegistryPackEntry> CreateTestEntries()
    {
        return new List<RegistryPackEntry>
        {
            new()
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
            new()
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
            new()
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

    private static string CreateRegistryJson(List<RegistryPackEntry> packs)
    {
        object document = new
        {
            version = "1.0.0",
            updated = "2026-03-27",
            packs
        };

        JsonSerializerOptions options = new()
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase
        };

        return JsonSerializer.Serialize(document, options);
    }

    private sealed class RecordingHandler : HttpMessageHandler
    {
        private readonly string _responseJson;

        public RecordingHandler(string responseJson)
        {
            _responseJson = responseJson ?? throw new ArgumentNullException(nameof(responseJson));
        }

        public int RequestCount { get; private set; }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            RequestCount++;

            HttpResponseMessage response = new(HttpStatusCode.OK)
            {
                Content = new StringContent(_responseJson, Encoding.UTF8, "application/json")
            };

            return Task.FromResult(response);
        }
    }
}
