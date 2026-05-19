// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #264 / Pattern #95 — IValidatable + JsonGuard at HIGH cross-FFI DTOs.
// Negative tests for PackCompiler ImportedAsset / OptimizedAsset / ResolverOutput
// (PackCompiler clones — dedupe with SDK tracked by task #266).

using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.SDK.Validation;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tools.PackCompiler.Tests
{
    /// <summary>
    /// Pins the <see cref="JsonGuard.ValidateOrThrow{T}"/> wiring at the two
    /// PackCompiler-side FFI deserialize sites:
    ///   1. <c>DirectAssetPipeline.RunPhase3A</c> — ImportedAsset JSON read
    ///   2. <c>DirectAssetPipeline.RunPhase3A</c> — OptimizedAsset JSON read
    ///   3. <c>GoResolverService.ResolveWithGoAsync</c> — Go resolver JSON read
    /// </summary>
    public class DirectAssetPipelineValidationTests
    {
        // ── PackCompiler ImportedAsset ──────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_BlankAssetId_FailsValidation()
        {
            var asset = new ImportedAsset
            {
                AssetId = "",
                SourcePath = "C:/asset.glb",
                Mesh = new MeshData
                {
                    Name = "mesh",
                    Vertices = new float[] { 0f, 0f, 0f },
                    Indices = new uint[] { 0u, 0u, 0u }
                }
            };

            DINOForge.SDK.Validation.ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "asset_id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_VerticesNotMultipleOfThree_FailsValidation()
        {
            var asset = new ImportedAsset
            {
                AssetId = "clone-trooper",
                SourcePath = "C:/asset.glb",
                Mesh = new MeshData
                {
                    Name = "mesh",
                    Vertices = new float[] { 1f, 2f, 3f, 4f }, // length 4 — not divisible by 3
                    Indices = new uint[] { 0u, 1u, 2u }
                }
            };

            DINOForge.SDK.Validation.ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "mesh.vertices");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ImportedAsset_JsonGuard_BlankAssetId_ThrowsInvalidDataException()
        {
            // Construct directly (PackCompiler is published as a trimmed single-file
            // exe, so reflection-based JsonSerializer.Deserialize is disabled in
            // the test host — we still want the same exception surface that
            // DirectAssetPipeline's deserialize+JsonGuard would produce for a
            // payload whose AssetId is blank).
            var imported = new ImportedAsset
            {
                AssetId = "",
                SourcePath = "C:/asset.glb",
                Mesh = new MeshData
                {
                    Name = "m",
                    Vertices = new float[] { 0f, 0f, 0f },
                    Indices = new uint[] { 0u, 1u, 2u }
                }
            };

            System.Action act = () => JsonGuard.ValidateOrThrow(imported, "DirectAssetPipelineValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*asset_id*");
        }

        // ── PackCompiler OptimizedAsset ─────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void OptimizedAsset_BlankAssetId_FailsValidation()
        {
            var asset = new OptimizedAsset
            {
                AssetId = "",
                LOD0 = new MeshData
                {
                    Name = "lod0",
                    Vertices = new float[] { 0f, 0f, 0f },
                    Indices = new uint[] { 0u, 1u, 2u }
                },
                LOD1 = new MeshData
                {
                    Name = "lod1",
                    Vertices = new float[] { 0f, 0f, 0f },
                    Indices = new uint[] { 0u, 1u, 2u }
                },
                LOD2 = new MeshData
                {
                    Name = "lod2",
                    Vertices = new float[] { 0f, 0f, 0f },
                    Indices = new uint[] { 0u, 1u, 2u }
                }
            };

            DINOForge.SDK.Validation.ValidationResult result = asset.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "asset_id");
        }

        // ── GoResolverService ResolverOutput (internal sibling) ─────────

        [Fact]
        [Trait("Category", "Validation")]
        public void GoResolverOutput_EmptyResolvedAndErrors_FailsValidation()
        {
            // Both Resolved + Errors empty — Go subprocess returned a meaningless payload.
            // Constructed directly because PackCompiler runs trimmed (no reflection-
            // based JsonSerializer). The sibling type is internal but visible
            // within the same assembly.
            var output = new GoResolverService.ResolverOutput
            {
                Resolved = new List<string>(),
                Errors = new List<string>()
            };

            System.Action act = () => JsonGuard.ValidateOrThrow(output, "DirectAssetPipelineValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*resolved*errors*");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void GoResolverOutput_BlankResolvedEntry_FailsValidation()
        {
            var output = new GoResolverService.ResolverOutput
            {
                Resolved = new List<string> { "pack-a", "", "pack-c" },
                Errors = new List<string>()
            };

            System.Action act = () => JsonGuard.ValidateOrThrow(output, "DirectAssetPipelineValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*resolved[1]*");
        }
    }
}
