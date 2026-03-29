#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using Assimp;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tools.PackCompiler.Tests
{
    public class AssetImportServiceTests
    {
        // ── Multi-mesh combination unit tests ──────────────────────────────

        [Fact]
        public void CombineMultipleMeshes_SingleMesh_ReturnsSingleMeshData()
        {
            // Arrange
            var mesh = CreateTestMesh("TestMesh", 8, 12); // Cube: 8 vertices, 12 triangles
            var meshes = new[] { mesh };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.Should().NotBeNull();
            result.VertexCount.Should().Be(8);
            result.TriangleCount.Should().Be(12);
            result.Name.Should().Be("TestMesh");
            result.Vertices.Length.Should().Be(24); // 8 vertices * 3 floats
            result.Indices.Length.Should().Be(36); // 12 triangles * 3 indices
        }

        [Fact]
        public void CombineMultipleMeshes_TwoMeshes_CombinesVerticesAndIndices()
        {
            // Arrange
            var mesh1 = CreateTestMesh("Mesh1", 8, 12);
            var mesh2 = CreateTestMesh("Mesh2", 8, 12);
            var meshes = new[] { mesh1, mesh2 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.Should().NotBeNull();
            result.VertexCount.Should().Be(16); // 8 + 8 vertices
            result.TriangleCount.Should().Be(24); // 12 + 12 triangles
            result.Name.Should().Contain("combined_2");
            result.Vertices.Length.Should().Be(48); // 16 vertices * 3 floats
            result.Indices.Length.Should().Be(72); // 24 triangles * 3 indices
        }

        [Fact]
        public void CombineMultipleMeshes_ThreeMeshes_PreservesAllGeometry()
        {
            // Arrange
            var mesh1 = CreateTestMesh("Mesh1", 8, 12);
            var mesh2 = CreateTestMesh("Mesh2", 6, 8);
            var mesh3 = CreateTestMesh("Mesh3", 4, 4);
            var meshes = new[] { mesh1, mesh2, mesh3 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.Should().NotBeNull();
            result.VertexCount.Should().Be(18); // 8 + 6 + 4
            result.TriangleCount.Should().Be(24); // 12 + 8 + 4
            result.Name.Should().Contain("combined_3");
        }

        [Fact]
        public void CombineMultipleMeshes_VerifiesIndexOffsets()
        {
            // Arrange - Create two simple meshes
            var mesh1 = CreateTestMesh("Mesh1", 4, 2);
            var mesh2 = CreateTestMesh("Mesh2", 4, 2);
            var meshes = new[] { mesh1, mesh2 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            // First mesh indices should be 0-3, second mesh indices should be 4-7 (offset by first mesh's vertex count)
            var firstMeshIndices = result.Indices.Take(6).ToArray(); // First mesh: 2 triangles * 3
            var secondMeshIndices = result.Indices.Skip(6).Take(6).ToArray(); // Second mesh indices

            // All indices in first mesh should be < 4
            firstMeshIndices.All(i => i < 4).Should().BeTrue();
            // All indices in second mesh should be >= 4 and < 8
            secondMeshIndices.All(i => i >= 4 && i < 8).Should().BeTrue();
        }

        [Fact]
        public void CombineMultipleMeshes_WithNormals_CombinesNormalsCorrectly()
        {
            // Arrange
            var mesh1 = CreateTestMeshWithNormals("Mesh1", 4, 2);
            var mesh2 = CreateTestMeshWithNormals("Mesh2", 4, 2);
            var meshes = new[] { mesh1, mesh2 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.Normals.Should().NotBeNull();
            result.Normals!.Length.Should().Be(24); // 8 vertices * 3 floats
        }

        [Fact]
        public void CombineMultipleMeshes_WithUVs_CombinesUVsCorrectly()
        {
            // Arrange
            var mesh1 = CreateTestMeshWithUVs("Mesh1", 4, 2);
            var mesh2 = CreateTestMeshWithUVs("Mesh2", 4, 2);
            var meshes = new[] { mesh1, mesh2 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.UVs.Should().NotBeNull();
            result.UVs!.Length.Should().Be(16); // 8 vertices * 2 floats
        }

        [Fact]
        public void CombineMultipleMeshes_EmptyMeshList_Throws()
        {
            // Arrange
            var meshes = Array.Empty<Mesh>();

            // Act
            var act = () => AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            // Reflection wraps the exception in TargetInvocationException
            var ex = Record.Exception(act);
            ex.Should().NotBeNull();
            (ex is InvalidOperationException ||
             (ex is System.Reflection.TargetInvocationException tie && tie.InnerException is InvalidOperationException))
                .Should().BeTrue("Expected InvalidOperationException or wrapped TargetInvocationException");
        }

        [Fact]
        public void CombineMultipleMeshes_CalculatesBoundsCorrectly()
        {
            // Arrange
            var mesh1 = CreateTestMesh("Mesh1", 8, 12);
            var meshes = new[] { mesh1 };

            // Act
            var result = AssetImportServiceTestHelper.CombineMultipleMeshes(meshes);

            // Assert
            result.Bounds.Should().NotBeNull();
            var (min, max) = result.Bounds.Value;
            min.Should().HaveCount(3);
            max.Should().HaveCount(3);
            min[0].Should().BeLessThanOrEqualTo(max[0]);
            min[1].Should().BeLessThanOrEqualTo(max[1]);
            min[2].Should().BeLessThanOrEqualTo(max[2]);
        }

        // ── Helper methods ─────────────────────────────────────────────────

        private static Mesh CreateTestMesh(string name, int vertexCount, int triangleCount)
        {
            var mesh = new Mesh(PrimitiveType.Triangle);
            mesh.Name = name;

            // Create dummy vertices
            for (int i = 0; i < vertexCount; i++)
            {
                mesh.Vertices.Add(new Vector3D(i * 0.1f, i * 0.2f, i * 0.3f));
            }

            // Create dummy faces
            for (int i = 0; i < triangleCount; i++)
            {
                int v0 = i % vertexCount;
                int v1 = (i + 1) % vertexCount;
                int v2 = (i + 2) % vertexCount;
                mesh.Faces.Add(new Face(new[] { v0, v1, v2 }));
            }

            return mesh;
        }

        private static Mesh CreateTestMeshWithNormals(string name, int vertexCount, int triangleCount)
        {
            var mesh = CreateTestMesh(name, vertexCount, triangleCount);

            // Add normals
            for (int i = 0; i < vertexCount; i++)
            {
                mesh.Normals.Add(new Vector3D(0.0f, 0.0f, 1.0f));
            }

            return mesh;
        }

        private static Mesh CreateTestMeshWithUVs(string name, int vertexCount, int triangleCount)
        {
            var mesh = CreateTestMesh(name, vertexCount, triangleCount);

            // Add UVs
            for (int i = 0; i < vertexCount; i++)
            {
                mesh.TextureCoordinateChannels[0].Add(new Vector3D(0.5f, 0.5f, 0.0f));
            }

            return mesh;
        }
    }

    /// <summary>
    /// Test helper to access internal AssetImportService methods.
    /// This class uses reflection to test the private CombineMultipleMeshes method.
    /// </summary>
    internal static class AssetImportServiceTestHelper
    {
        public static MeshData CombineMultipleMeshes(IList<Mesh> meshes)
        {
            var service = new AssetImportService();
            var method = typeof(AssetImportService).GetMethod(
                "CombineMultipleMeshes",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);

            if (method == null)
            {
                throw new InvalidOperationException("CombineMultipleMeshes method not found on AssetImportService");
            }

            var result = method.Invoke(service, new object[] { meshes });
            return (MeshData)result!;
        }
    }
}
