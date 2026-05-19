// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #264 / Pattern #95 — IValidatable + JsonGuard at HIGH cross-FFI DTOs.
// Negative tests for SDK Rust asset pipeline DTOs (ImportedAsset, OptimizedAsset).
// Mirrors UniverseLoaderValidationTests / EconomyContentLoaderValidationTests
// negative-test pattern at the FFI boundary.

using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.SDK.NativeInterop;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the <see cref="JsonGuard.ValidateOrThrow{T}"/> wiring at the three
    /// <see cref="RustAssetPipeline"/> deserialize sites:
    ///   1. <c>ImportAssetViaRustAsync</c>  (MCP path)
    ///   2. <c>OptimizeAssetViaRustAsync</c> (MCP path)
    ///   3. <c>ImportAssetViaPInvoke</c>    (P/Invoke path)
    ///
    /// Each negative test asserts that <see cref="InvalidDataException"/> is thrown
    /// when the cross-FFI DTO violates its semantic invariants.
    /// </summary>
    public class RustAssetPipelineValidationTests
    {
        // ── ImportedAsset ───────────────────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_BlankAssetId_FailsValidation()
        {
            var asset = new ImportedAsset
            {
                AssetId = "",
                Mesh = new MeshData { Vertices = new float[] { 0f, 0f, 0f } }
            };

            ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "asset_id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_EmptyVertices_FailsValidation()
        {
            var asset = new ImportedAsset
            {
                AssetId = "sw-rep-clone-trooper",
                Mesh = new MeshData { Vertices = System.Array.Empty<float>() }
            };

            ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "mesh.vertices");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_JsonGuard_BlankAssetId_ThrowsInvalidDataException()
        {
            // Simulate the deserialize site: JSON arrives from Rust without an asset_id.
            string json = "{\"AssetId\":\"\",\"Mesh\":{\"Vertices\":[0.0,0.0,0.0],\"Indices\":[],\"TriangleCount\":0}}";
            var imported = JsonSerializer.Deserialize<ImportedAsset>(json);

            System.Action act = () => JsonGuard.ValidateOrThrow(imported!, "RustAssetPipelineValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*asset_id*");
        }

        // ── OptimizedAsset ──────────────────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void OptimizedAsset_BlankAssetId_FailsValidation()
        {
            var asset = new OptimizedAsset
            {
                AssetId = "",
                LOD0 = new MeshData(),
                LOD1 = new MeshData(),
                LOD2 = new MeshData()
            };

            ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "asset_id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void OptimizedAsset_JsonGuard_BlankAssetId_ThrowsInvalidDataException()
        {
            // Simulate the deserialize site: JSON arrives from Rust without an asset_id.
            string json = "{\"AssetId\":\"\",\"LOD0\":{},\"LOD1\":{},\"LOD2\":{}}";
            var optimized = JsonSerializer.Deserialize<OptimizedAsset>(json);

            System.Action act = () => JsonGuard.ValidateOrThrow(optimized!, "RustAssetPipelineValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*asset_id*");
        }
    }
}
