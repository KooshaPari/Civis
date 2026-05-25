using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class LoadingServicesTests : IDisposable
    {
        private readonly string _tempRoot;

        public LoadingServicesTests()
        {
            _tempRoot = Path.Combine(Path.GetTempPath(), "dinoforge_loading_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
            {
                Directory.Delete(_tempRoot, true);
            }
        }

        [Fact]
        public void ContentDiscoveryService_DiscoversPackDirectories_WithPackManifestOnly()
        {
            string packsRoot = Path.Combine(_tempRoot, "packs");
            Directory.CreateDirectory(packsRoot);

            string validPack = Path.Combine(packsRoot, "valid-pack");
            Directory.CreateDirectory(validPack);
            File.WriteAllText(Path.Combine(validPack, "pack.yaml"), "id: valid-pack");

            string invalidPack = Path.Combine(packsRoot, "not-a-pack");
            Directory.CreateDirectory(invalidPack);

            ContentDiscoveryService discoveryService = new ContentDiscoveryService();

            IReadOnlyList<string> packDirectories = discoveryService.DiscoverPackDirectories(packsRoot);

            packDirectories.Should().Contain(validPack);
            packDirectories.Should().NotContain(invalidPack);
        }

        [Fact]
        public void ContentDiscoveryService_DeclaredDirectory_ReturnsSortedYamlFiles()
        {
            string packRoot = Path.Combine(_tempRoot, "pack");
            string unitsDir = Path.Combine(packRoot, "units");
            Directory.CreateDirectory(unitsDir);
            File.WriteAllText(Path.Combine(unitsDir, "b.yaml"), "id: b");
            File.WriteAllText(Path.Combine(unitsDir, "a.yml"), "id: a");

            ContentDiscoveryService discoveryService = new ContentDiscoveryService();

            IReadOnlyList<string> files = discoveryService.DiscoverYamlFiles(
                packRoot,
                "units",
                new List<string> { "units" });

            files.Should().HaveCount(2);
            Path.GetFileName(files[0]).Should().Be("a.yml");
            Path.GetFileName(files[1]).Should().Be("b.yaml");
        }

        [Fact]
        public void SchemaResolverService_MapsKnownContentTypes()
        {
            SchemaResolverService resolver = new SchemaResolverService();

            resolver.TryResolveSchemaName("units", out string schemaName).Should().BeTrue();
            schemaName.Should().Be("unit");
            resolver.ContentTypes.Should().Contain("stats");
        }
    }
}
