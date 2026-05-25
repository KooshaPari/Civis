#nullable enable
using System;
using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public class PackRegistryClientCoverageTests
{
    private static readonly string RegistryJson = """
    {
      "version": "1",
      "updated": "2026-03-27T00:00:00Z",
      "packs": [
        {
          "id": "warfare-modern",
          "name": "Modern Warfare",
          "author": "DINOForge Team",
          "version": "1.0.0",
          "type": "content",
          "description": "Modern military units",
          "tags": ["warfare", "modern", "military"],
          "repo": "https://example.invalid/modern",
          "download_url": "https://example.invalid/modern.zip",
          "pack_path": "packs/warfare-modern",
          "framework_version": ">=0.1.0",
          "verified": true,
          "featured": true,
          "conflicts_with": ["warfare-starwars"],
          "depends_on": []
        },
        {
          "id": "warfare-starwars",
          "name": "Star Wars Warfare",
          "author": "Community",
          "version": "0.5.0",
          "type": "content",
          "description": "Star Wars units and factions",
          "tags": ["warfare", "scifi", "starwars"],
          "repo": "https://example.invalid/starwars",
          "download_url": "https://example.invalid/starwars.zip",
          "pack_path": "packs/warfare-starwars",
          "framework_version": ">=0.1.0",
          "verified": false,
          "featured": false,
          "conflicts_with": ["warfare-modern"],
          "depends_on": []
        },
        {
          "id": "balance-economy",
          "name": "Economy Balance",
          "author": "Balance Team",
          "version": "2.1.0",
          "type": "balance",
          "description": "Economic balance tweaks",
          "tags": ["balance", "economy"],
          "repo": "https://example.invalid/balance",
          "download_url": "https://example.invalid/balance.zip",
          "pack_path": "packs/balance-economy",
          "framework_version": ">=0.1.0",
          "verified": true,
          "featured": false,
          "conflicts_with": [],
          "depends_on": []
        }
      ]
    }
    """;

    [Fact]
    public async Task GetAllPacksAsync_DeserializesRegistryAndCachesResults()
    {
        using RecordingHandler handler = new(RegistryJson);
        using HttpClient httpClient = new(handler);
        PackRegistryClient client = new("https://example.invalid/registry.json", httpClient)
        {
            CacheDuration = TimeSpan.FromMinutes(5)
        };

        IReadOnlyList<RegistryPackEntry> packs = await client.GetAllPacksAsync();
        packs.Should().HaveCount(3);
        packs[0].Id.Should().Be("warfare-modern");
        packs[1].Verified.Should().BeFalse();
        packs[2].Type.Should().Be("balance");

        IReadOnlyList<RegistryPackEntry> cached = await client.GetAllPacksAsync();
        cached.Should().HaveCount(3);
        handler.RequestCount.Should().Be(1, "second call should reuse the cache");

        client.InvalidateCache();
        IReadOnlyList<RegistryPackEntry> refreshed = await client.GetAllPacksAsync();
        refreshed.Should().HaveCount(3);
        handler.RequestCount.Should().Be(2, "invalidate should force a refetch");
    }

    [Fact]
    public async Task SearchAsync_AppliesFilters_AndConvenienceWrappersDelegate()
    {
        using RecordingHandler handler = new(RegistryJson);
        using HttpClient httpClient = new(handler);
        PackRegistryClient client = new("https://example.invalid/registry.json", httpClient);

        IReadOnlyList<RegistryPackEntry> all = await client.SearchAsync(null);
        all.Should().HaveCount(3);

        IReadOnlyList<RegistryPackEntry> warfare = await client.GetByTagAsync("warfare");
        warfare.Should().HaveCount(2);

        IReadOnlyList<RegistryPackEntry> content = await client.GetByTypeAsync("content");
        content.Should().HaveCount(2);

        IReadOnlyList<RegistryPackEntry> verified = await client.GetVerifiedAsync();
        verified.Should().HaveCount(2);

        IReadOnlyList<RegistryPackEntry> featured = await client.GetFeaturedAsync();
        featured.Should().ContainSingle();
        featured[0].Id.Should().Be("warfare-modern");

        PackRegistryFilter filter = new()
        {
            Tags = new[] { "warfare", "modern" },
            Type = "content",
            Verified = true,
            Featured = true
        };

        IReadOnlyList<RegistryPackEntry> filtered = await client.SearchAsync(filter);
        filtered.Should().ContainSingle();
        filtered[0].Id.Should().Be("warfare-modern");

        RegistryPackEntry? found = await client.GetByIdAsync("WARFARE-MODERN");
        found.Should().NotBeNull();
        found!.Name.Should().Be("Modern Warfare");

        RegistryPackEntry? missing = await client.GetByIdAsync("does-not-exist");
        missing.Should().BeNull();
    }

    private sealed class RecordingHandler(string responseJson) : HttpMessageHandler
    {
        public int RequestCount { get; private set; }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            RequestCount++;
            HttpResponseMessage response = new(HttpStatusCode.OK)
            {
                Content = new StringContent(responseJson, Encoding.UTF8, "application/json")
            };

            return Task.FromResult(response);
        }
    }
}
