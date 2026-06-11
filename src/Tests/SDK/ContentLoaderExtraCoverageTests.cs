// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Extra coverage for ContentLoader public API null-guard behavior.

using System;
using System.Runtime.Serialization;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public class ContentLoaderExtraCoverageTests
    {
        [Fact]
        public void Constructor_ThrowsOnNullRegistryManager()
        {
            Action act = () => new ContentLoader(null!, null, null);

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("registryManager");
        }

        [Fact]
        public void LoadPack_ThrowsOnNullPackDirectory()
        {
            ContentLoader loader = CreateLoader();

            Action act = () => loader.LoadPack(null!);

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("packDirectory");
        }

        [Fact]
        public void LoadPacks_ReturnsFailureOnNullPacksRootDirectory()
        {
            ContentLoader loader = CreateLoader();

            ContentLoadResult result = loader.LoadPacks(null!);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        private static ContentLoader CreateLoader()
        {
            RegistryManager registryManager =
                (RegistryManager)FormatterServices.GetUninitializedObject(typeof(RegistryManager));

            return new ContentLoader(registryManager);
        }
    }
}
