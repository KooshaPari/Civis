// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 8 of #193 SDK split — ThemeProviderAdapter contract tests.
// Live IMGUI palette / DinoForgeStyle integration is exercised by game-launch acceptance — not here.

using System;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.UI
{
    public class ThemeProviderAdapterTests
    {
        [Fact]
        public void Instance_IsSingleton()
        {
            ThemeProviderAdapter a = ThemeProviderAdapter.Instance;
            ThemeProviderAdapter b = ThemeProviderAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void Instance_ImplementsIThemeProvider()
        {
            ThemeProviderAdapter.Instance.Should().BeAssignableTo<IThemeProvider>();
        }

        [Fact]
        public void ApplyTheme_ThrowsOnNull()
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            Action act = () => provider.ApplyTheme(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("theme");
        }

        [Fact]
        public void ResolveColor_ThrowsOnNullOrEmpty()
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            ((Action)(() => provider.ResolveColor(null!)))
                .Should().Throw<ArgumentNullException>().WithParameterName("semanticName");
            ((Action)(() => provider.ResolveColor(string.Empty)))
                .Should().Throw<ArgumentException>().WithParameterName("semanticName");
        }

        [Fact]
        public void ResolveFontSize_ThrowsOnNullOrEmpty()
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            ((Action)(() => provider.ResolveFontSize(null!)))
                .Should().Throw<ArgumentNullException>().WithParameterName("semanticName");
            ((Action)(() => provider.ResolveFontSize(string.Empty)))
                .Should().Throw<ArgumentException>().WithParameterName("semanticName");
        }

        [Fact]
        public void ResolveColor_FallsBackToDefaults_WhenNoThemeApplied()
        {
            // Reset to a known no-theme state by applying then re-applying a fresh default.
            // (The singleton may carry state from sibling tests; resolution must still match
            // a recognised default token regardless of prior state.)
            IThemeProvider provider = ThemeProviderAdapter.Instance;

            ColorRgba accent = provider.ResolveColor("accent");
            accent.A.Should().BeApproximately(1f, 0.001f);

            ColorRgba unknown = provider.ResolveColor("totally-unknown-token");
            unknown.Should().Be(ColorRgba.White, "unknown tokens fall back to white");
        }

        [Fact]
        public void ResolveColor_PrefersThemeWhenApplied()
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            ThemeDefinition theme = new ThemeDefinition
            {
                Id = "dinoforge.tests.theme",
                Name = "Test",
                PrimaryColor = "#FF0000",
                AccentColor = "#00FF00",
            };
            provider.ApplyTheme(theme);

            ColorRgba primary = provider.ResolveColor("primary");
            primary.R.Should().BeApproximately(1f, 0.01f);
            primary.G.Should().BeApproximately(0f, 0.01f);
            primary.B.Should().BeApproximately(0f, 0.01f);

            ColorRgba accent = provider.ResolveColor("ACCENT"); // case-insensitive
            accent.G.Should().BeApproximately(1f, 0.01f);
        }

        [Fact]
        public void ResolveColor_FallsBackOnUnparseableHex()
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            ThemeDefinition theme = new ThemeDefinition
            {
                Id = "dinoforge.tests.theme.bad-hex",
                AccentColor = "not-a-hex",
            };
            provider.ApplyTheme(theme);

            // Unparseable theme value should not throw — falls back to default accent.
            ColorRgba accent = provider.ResolveColor("accent");
            accent.A.Should().BeApproximately(1f, 0.001f);
        }

        [Theory]
        [InlineData("small", FontSize.Small)]
        [InlineData("medium", FontSize.Medium)]
        [InlineData("body", FontSize.Medium)]
        [InlineData("large", FontSize.Large)]
        [InlineData("heading", FontSize.Heading)]
        [InlineData("title", FontSize.Title)]
        [InlineData("unknown-token", FontSize.Medium)]
        public void ResolveFontSize_MapsTokensToEnum(string token, FontSize expected)
        {
            IThemeProvider provider = ThemeProviderAdapter.Instance;
            provider.ResolveFontSize(token).Should().Be(expected);
        }

        [Fact]
        public void GetActiveTheme_ReturnsAppliedTheme()
        {
            ThemeProviderAdapter adapter = ThemeProviderAdapter.Instance;
            ThemeDefinition theme = new ThemeDefinition { Id = "dinoforge.tests.theme.active" };
            ((IThemeProvider)adapter).ApplyTheme(theme);

            adapter.GetActiveTheme().Should().BeSameAs(theme);
        }
    }
}
