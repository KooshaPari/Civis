#nullable enable
using System;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for LOD (Level of Detail) calculation logic.
/// These tests verify the pure logic without requiring Unity runtime.
/// </summary>
public class LODCalculationTests
{
    /// <summary>
    /// Test LOD tier calculation based on distance and screen size.
    /// </summary>
    [Theory]
    [InlineData(0, 100, LODLevel.Full)]
    [InlineData(50, 100, LODLevel.Full)]
    [InlineData(99, 100, LODLevel.Full)]
    [InlineData(100, 100, LODLevel.Medium)]
    [InlineData(150, 100, LODLevel.Medium)]
    [InlineData(199, 100, LODLevel.Medium)]
    [InlineData(200, 100, LODLevel.Culled)]
    [InlineData(300, 100, LODLevel.Culled)]
    public void CalculateLODTier_ReturnsExpected(float distance, float screenSize, LODLevel expected)
    {
        // Act
        var result = LODCalculator.GetTier(distance, screenSize);

        // Assert
        Assert.Equal(expected, result);
    }

    /// <summary>
    /// Test distance calculation between two points.
    /// </summary>
    [Theory]
    [InlineData(0, 0, 10, 0, 10)]
    [InlineData(0, 0, 0, 10, 10)]
    [InlineData(3, 4, 0, 0, 5)]  // 3-4-5 triangle
    public void CalculateDistance_ReturnsExpected(float x1, float z1, float x2, float z2, float expected)
    {
        // Act
        var result = LODCalculator.Distance(x1, z1, x2, z2);

        // Assert
        Assert.Equal(expected, result, 0.001f);
    }

    /// <summary>
    /// Test screen size calculation returns positive value.
    /// </summary>
    [Theory]
    [InlineData(100)]
    [InlineData(200)]
    [InlineData(400)]
    public void CalculateScreenSize_IsPositive(float distance)
    {
        // Act
        var result = LODCalculator.ScreenSize(distance, 60);

        // Assert - should be positive
        Assert.True(result > 0, $"Screen size should be positive at distance {distance}");
    }
}

/// <summary>
/// LOD tier levels based on distance thresholds.
/// </summary>
public enum LODLevel
{
    Full = 0,
    Medium = 1,
    Low = 2,
    Culled = 3
}

/// <summary>
/// Pure logic for LOD calculations without Unity dependencies.
/// </summary>
public static class LODCalculator
{
    private const float FullThreshold = 100f;
    private const float MediumThreshold = 200f;

    /// <summary>
    /// Calculate LOD tier based on distance and screen size.
    /// </summary>
    public static LODLevel GetTier(float distance, float screenSize)
    {
        // Use the larger of actual distance or screen percentage
        float effective = Math.Max(distance, 100f - screenSize);

        if (effective < FullThreshold)
            return LODLevel.Full;
        if (effective < MediumThreshold)
            return LODLevel.Medium;
        return LODLevel.Culled;
    }

    /// <summary>
    /// Calculate 2D distance on XZ plane.
    /// </summary>
    public static float Distance(float x1, float z1, float x2, float z2)
    {
        float dx = x2 - x1;
        float dz = z2 - z1;
        return MathF.Sqrt(dx * dx + dz * dz);
    }

    /// <summary>
    /// Calculate screen size percentage from camera distance and FOV.
    /// </summary>
    public static float ScreenSize(float distance, float fov)
    {
        if (distance <= 0)
            return 100f;

        // Simplified: screen size is inversely proportional to distance
        return (fov * 100f) / distance;
    }
}
