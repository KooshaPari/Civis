#nullable enable
using System.Reflection;
using NetArchTest.Rules;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Architecture enforcement tests using NetArchTest.
    /// Verifies layering rules: SDK has no dependency on Runtime or Domains.
    /// </summary>
    public class ArchitectureTests
    {
        private static readonly Assembly SdkAssembly =
            typeof(DINOForge.SDK.Registry.IRegistry<>).Assembly;

        [Fact]
        public void SDK_ShouldNot_DependOnRuntime()
        {
            TestResult result = Types.InAssembly(SdkAssembly)
                .ShouldNot()
                .HaveDependencyOn("DINOForge.Runtime")
                .GetResult();

            result.IsSuccessful.Should().BeTrue(
                because: "SDK must not depend on Runtime (hexagonal architecture)");
        }

        [Fact]
        public void SDK_ShouldNot_DependOnDomains()
        {
            TestResult result = Types.InAssembly(SdkAssembly)
                .ShouldNot()
                .HaveDependencyOnAny("DINOForge.Domains.Warfare",
                    "DINOForge.Domains.Economy",
                    "DINOForge.Domains.Scenario",
                    "DINOForge.Domains.UI")
                .GetResult();

            result.IsSuccessful.Should().BeTrue(
                because: "SDK must not depend on Domain plugins (hexagonal architecture)");
        }

        [Fact]
        public void SDK_PublicInterfaces_ShouldBeInSDKNamespace()
        {
            TestResult result = Types.InAssembly(SdkAssembly)
                .That()
                .AreInterfaces()
                .And()
                .ArePublic()
                .Should()
                .ResideInNamespace("DINOForge.SDK")
                .GetResult();

            result.IsSuccessful.Should().BeTrue(
                because: "All public SDK interfaces should be in the DINOForge.SDK namespace");
        }

        [Fact]
        public void SDK_Models_ShouldBeInModelsNamespace()
        {
            TestResult result = Types.InAssembly(SdkAssembly)
                .That()
                .ResideInNamespace("DINOForge.SDK.Models")
                .Should()
                .NotBeAbstract()
                .GetResult();

            result.IsSuccessful.Should().BeTrue(
                because: "SDK model classes should be concrete data types, not abstract");
        }
    }
}
