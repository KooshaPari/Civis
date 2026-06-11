using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class RegistryEntryCoverageTests
    {
        [Fact]
        public void Constructor_AssignsProperties_AndCalculatesPriority_FromExplicitLoadOrder()
        {
            RegistryEntry<string> entry = new RegistryEntry<string>(
                "packs.core:unit",
                "payload",
                RegistrySource.DomainPlugin,
                "core-pack",
                42);

            entry.Id.Should().Be("packs.core:unit");
            entry.Data.Should().Be("payload");
            entry.Source.Should().Be(RegistrySource.DomainPlugin);
            entry.SourcePackId.Should().Be("core-pack");
            entry.Priority.Should().Be(((int)RegistrySource.DomainPlugin * 1000) + 42);
        }

        [Fact]
        public void Constructor_UsesDefaultLoadOrder_WhenOmitted()
        {
            RegistryEntry<string> entry = new RegistryEntry<string>(
                "packs.base:unit",
                "payload",
                RegistrySource.BaseGame,
                "base-pack");

            entry.Priority.Should().Be(((int)RegistrySource.BaseGame * 1000) + 100);
        }
    }
}
