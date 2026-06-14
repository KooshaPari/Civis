#nullable enable
using System.Reflection;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Reflection-based field-integrity tests for LoadingScreenController.LoadingTheme.
    /// Guards against field renames that break the theme-resolution / build pipeline
    /// (e.g. the TrackColor->ProgressTrackColor rename that regressed this session).
    /// </summary>
    public class LoadingScreenControllerThemeTests
    {
        private static readonly string ControllerTypeName = "DINOForge.Runtime.UI.LoadingScreenController, DINOForge.Runtime";

        [Fact]
        public void LoadingTheme_DeclaresProgressTrackColorField()
        {
            // Arrange
            Type? controllerType = Type.GetType(ControllerTypeName, throwOnError: false);
            controllerType.Should().NotBeNull(
                "LoadingScreenController must be present in the Runtime assembly");

            Type? themeType = controllerType!.GetNestedType(
                "LoadingTheme",
                BindingFlags.NonPublic);
            themeType.Should().NotBeNull(
                "LoadingTheme must be declared as a nested type on LoadingScreenController");

            // Act
            FieldInfo? trackField = themeType!.GetField(
                "ProgressTrackColor",
                BindingFlags.Public | BindingFlags.Instance);

            // Assert
            trackField.Should().NotBeNull(
                "LoadingTheme must declare a public instance field named ProgressTrackColor");
        }

        [Fact]
        public void LoadingTheme_DeclaresProgressShimmerColorField()
        {
            // Arrange
            Type? controllerType = Type.GetType(ControllerTypeName, throwOnError: false);
            controllerType.Should().NotBeNull(
                "LoadingScreenController must be present in the Runtime assembly");

            Type? themeType = controllerType!.GetNestedType(
                "LoadingTheme",
                BindingFlags.NonPublic);
            themeType.Should().NotBeNull(
                "LoadingTheme must be declared as a nested type on LoadingScreenController");

            // Act
            FieldInfo? shimmerField = themeType!.GetField(
                "ProgressShimmerColor",
                BindingFlags.Public | BindingFlags.Instance);

            // Assert
            shimmerField.Should().NotBeNull(
                "LoadingTheme must declare a public instance field named ProgressShimmerColor");
        }
    }
}
