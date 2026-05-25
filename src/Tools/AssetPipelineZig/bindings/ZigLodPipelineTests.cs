using Xunit;
using DINOForge.NativeInterop;

namespace DINOForge.Tests.NativeInterop;

/// <summary>
/// P/Invoke integration tests for Zig LOD pipeline.
/// Tests that the native Zig functions are accessible and behave correctly.
/// </summary>
public class ZigLodPipelineTests
{
    [Fact]
    public void ComputeLodLevel_WithValidInput_ReturnsCorrectTarget()
    {
        // Arrange
        uint vertexCount = 10000;
        float targetRatio = 0.5f; // 50% reduction

        // Act
        uint result = ZigLodPipeline.ComputeLodLevel(vertexCount, targetRatio);

        // Assert
        // Should compute roughly 50% of input (with minimum of 4)
        Assert.True(result >= 4);
        Assert.True(result <= vertexCount);
    }

    [Fact]
    public void ComputeLodLevel_With100Percent_ReturnsSameCount()
    {
        // Arrange
        uint vertexCount = 5000;
        float targetRatio = 1.0f; // No reduction

        // Act
        uint result = ZigLodPipeline.ComputeLodLevel(vertexCount, targetRatio);

        // Assert
        Assert.Equal(5000u, result);
    }

    [Fact]
    public void ComputeLodLevel_WithVerySmallRatio_ReturnsMinimum4()
    {
        // Arrange
        uint vertexCount = 10000;
        float targetRatio = 0.0001f; // Very small reduction

        // Act
        uint result = ZigLodPipeline.ComputeLodLevel(vertexCount, targetRatio);

        // Assert
        // Should enforce minimum of 4 vertices
        Assert.Equal(4u, result);
    }

    [Fact]
    public void ValidateMesh_WithValidMesh_ReturnsTrue()
    {
        // Arrange
        uint vertexCount = 100;
        uint triangleCount = 50;

        // Act
        bool result = ZigLodPipeline.ValidateMesh(vertexCount, triangleCount);

        // Assert
        Assert.True(result);
    }

    [Fact]
    public void ValidateMesh_WithTwoVertices_ReturnsFalse()
    {
        // Arrange
        uint vertexCount = 2; // Too few
        uint triangleCount = 1;

        // Act
        bool result = ZigLodPipeline.ValidateMesh(vertexCount, triangleCount);

        // Assert
        Assert.False(result);
    }

    [Fact]
    public void ValidateMesh_WithZeroTriangles_ReturnsFalse()
    {
        // Arrange
        uint vertexCount = 100;
        uint triangleCount = 0; // Too few

        // Act
        bool result = ZigLodPipeline.ValidateMesh(vertexCount, triangleCount);

        // Assert
        Assert.False(result);
    }

    [Fact]
    public void DecimateToTarget_WithValidInput_ReturnsCorrectValue()
    {
        // Arrange
        uint currentPolycount = 5000;
        float targetRatio = 0.5f; // 50% reduction

        // Act
        uint result = ZigLodPipeline.DecimateToTarget(currentPolycount, targetRatio);

        // Assert
        Assert.True(result >= 1);
        Assert.True(result <= currentPolycount);
    }

    [Fact]
    public void DecimateToTarget_WithVerySmallRatio_ReturnsMinimum1()
    {
        // Arrange
        uint currentPolycount = 10000;
        float targetRatio = 0.00001f; // Extremely small

        // Act
        uint result = ZigLodPipeline.DecimateToTarget(currentPolycount, targetRatio);

        // Assert
        // Should enforce minimum of 1 triangle
        Assert.Equal(1u, result);
    }
}
