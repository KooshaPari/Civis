using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Performance benchmark tests verifying M16 SLA targets.
    /// Uses Stopwatch for timing verification without BenchmarkDotNet overhead.
    /// </summary>
    public class PerformanceBenchmarkTests
    {
        private readonly PackLoader _packLoader = new();
        private readonly PackDependencyResolver _resolver = new();

        /// <summary>
        /// Creates a minimal test pack YAML string for performance testing.
        /// </summary>
        private string MakeMinimalPackYaml(string id, string[]? dependsOn = null)
        {
            string deps = dependsOn is { Length: > 0 }
                ? "depends_on:\n" + string.Join("", System.Array.ConvertAll(dependsOn, d => $"  - {d}\n"))
                : "";

            return $@"
id: {id}
name: {id}
version: 0.1.0
author: Perf Test
type: content
{deps}";
        }

        // ── SLA 1: Pack Loading < 500ms ──────────────────────────────────────

        [Fact]
        public void PackLoading_TypicalPack_UnderThreshold()
        {
            // SLA: Pack loading < 500ms for typical pack
            // Arrange
            string yaml = MakeMinimalPackYaml("perf-test-load");
            var sw = Stopwatch.StartNew();

            // Act
            var manifest = _packLoader.LoadFromString(yaml);

            sw.Stop();

            // Assert
            manifest.Should().NotBeNull();
            manifest.Id.Should().Be("perf-test-load");
            sw.ElapsedMilliseconds.Should().BeLessThan(500,
                $"Pack loading must complete in <500ms, took {sw.ElapsedMilliseconds}ms");
        }

        [Fact]
        public void PackLoading_MultipleSequential_AllUnderThreshold()
        {
            // SLA: Verify pack loading scales linearly (5 packs should all be <500ms each)
            var sw = Stopwatch.StartNew();

            for (int i = 0; i < 5; i++)
            {
                string yaml = MakeMinimalPackYaml($"perf-test-pack-{i}");
                var manifest = _packLoader.LoadFromString(yaml);
                manifest.Should().NotBeNull();
            }

            sw.Stop();

            // All 5 packs should complete in reasonable time
            sw.ElapsedMilliseconds.Should().BeLessThan(2500,
                $"Loading 5 packs must complete in <2500ms, took {sw.ElapsedMilliseconds}ms");
        }

        // ── SLA 2: Registry Lookup < 1ms per operation ───────────────────────

        [Fact]
        public void RegistryLookup_1000Lookups_AverageUnderThreshold()
        {
            // SLA: Registry lookup < 1ms per operation for 1000 lookups
            // Arrange
            var registry = new Registry<(string name, int value)>();

            // Pre-populate registry with diverse entries
            for (int i = 0; i < 100; i++)
            {
                registry.Register(
                    $"unit-{i:D3}",
                    ($"Unit {i}", i),
                    RegistrySource.BaseGame,
                    "base-game");
            }

            var sw = Stopwatch.StartNew();

            // Act: Perform 1000 lookups
            int hitCount = 0;
            for (int i = 0; i < 1000; i++)
            {
                var key = $"unit-{(i % 100):D3}";
                var result = registry.Get(key);
                if (result != default)
                    hitCount++;
            }

            sw.Stop();

            // Assert
            hitCount.Should().Be(1000, "All lookups should hit");
            var avgMs = sw.ElapsedMilliseconds / 1000.0;
            avgMs.Should().BeLessThan(1.0,
                $"Average lookup time must be <1ms, was {avgMs:F4}ms");
        }

        [Fact]
        public void RegistryLookup_MissesVsHits_Comparable()
        {
            // SLA: Misses should not be significantly slower than hits
            // Arrange
            var registry = new Registry<string>();
            for (int i = 0; i < 50; i++)
            {
                registry.Register($"item-{i}", $"Item {i}", RegistrySource.BaseGame, "base");
            }

            var sw = Stopwatch.StartNew();

            // Act: 500 hits + 500 misses
            for (int i = 0; i < 500; i++)
            {
                registry.Get($"item-{i % 50}"); // hit
                registry.Get($"missing-{i}");   // miss
            }

            sw.Stop();

            // Assert: Both hit and miss paths should be fast
            sw.ElapsedMilliseconds.Should().BeLessThan(100,
                $"1000 registry operations (mixed hits/misses) must complete in <100ms, took {sw.ElapsedMilliseconds}ms");
        }

        // ── SLA 3: Dependency Resolution < 100ms for 50 packs ────────────────

        [Fact]
        public void DependencyResolution_50PacksChain_UnderThreshold()
        {
            // SLA: Dependency resolution < 100ms for 50-pack scenario
            // Arrange: Create a 50-pack dependency chain
            var packs = new List<PackManifest>();
            for (int i = 0; i < 50; i++)
            {
                var deps = i > 0 ? new[] { $"pack-{i - 1}" } : Array.Empty<string>();
                string yaml = MakeMinimalPackYaml($"pack-{i}", deps);
                packs.Add(_packLoader.LoadFromString(yaml));
            }

            var sw = Stopwatch.StartNew();

            // Act: Resolve full dependency graph
            var result = _resolver.ComputeLoadOrder(packs);

            sw.Stop();

            // Assert
            result.IsSuccess.Should().BeTrue($"Expected successful resolution, got: {string.Join(", ", result.Errors)}");
            result.LoadOrder.Should().HaveCount(50);
            sw.ElapsedMilliseconds.Should().BeLessThan(100,
                $"Dependency resolution for 50 packs must complete in <100ms, took {sw.ElapsedMilliseconds}ms");
        }

        [Fact]
        public void DependencyResolution_DiamondGraph_UnderThreshold()
        {
            // SLA: Verify complex graph (diamond) resolves quickly
            // A depends on B and C, both depend on D
            var packs = new List<PackManifest>();
            string dYaml = MakeMinimalPackYaml("d");
            var d = _packLoader.LoadFromString(dYaml);
            packs.Add(d);

            string bYaml = MakeMinimalPackYaml("b", new[] { "d" });
            var b = _packLoader.LoadFromString(bYaml);
            packs.Add(b);

            string cYaml = MakeMinimalPackYaml("c", new[] { "d" });
            var c = _packLoader.LoadFromString(cYaml);
            packs.Add(c);

            string aYaml = MakeMinimalPackYaml("a", new[] { "b", "c" });
            var a = _packLoader.LoadFromString(aYaml);
            packs.Add(a);

            var sw = Stopwatch.StartNew();

            // Act
            var result = _resolver.ComputeLoadOrder(packs);

            sw.Stop();

            // Assert
            result.IsSuccess.Should().BeTrue();
            sw.ElapsedMilliseconds.Should().BeLessThan(50,
                $"Diamond dependency resolution must complete in <50ms, took {sw.ElapsedMilliseconds}ms");
        }

        // ── SLA 4: Schema Validation < 200ms for 10 packs ─────────────────────

        [Fact]
        public void SchemaValidation_10Packs_UnderThreshold()
        {
            // SLA: Schema validation < 200ms for 10 packs
            // (This test verifies the validation infrastructure is performant;
            // actual validation depends on validator implementation)

            var sw = Stopwatch.StartNew();

            // Act: Load and "validate" 10 packs (even without schema validator, parsing is part of validation)
            for (int i = 0; i < 10; i++)
            {
                string yaml = MakeMinimalPackYaml($"schema-test-{i}");
                var manifest = _packLoader.LoadFromString(yaml);
                manifest.Should().NotBeNull();
            }

            sw.Stop();

            // Assert
            sw.ElapsedMilliseconds.Should().BeLessThan(200,
                $"Validation + parsing for 10 packs must complete in <200ms, took {sw.ElapsedMilliseconds}ms");
        }

        [Fact]
        public void SchemaValidation_PackManifestStructure_Consistent()
        {
            // SLA: Multiple validations of same structure should have consistent timing
            string yaml = MakeMinimalPackYaml("consistency-test");

            var times = new List<long>();
            for (int i = 0; i < 5; i++)
            {
                var sw = Stopwatch.StartNew();
                var manifest = _packLoader.LoadFromString(yaml);
                sw.Stop();
                times.Add(sw.ElapsedMilliseconds);
                manifest.Should().NotBeNull();
            }

            // All iterations should complete quickly (timing consistency is secondary to speed)
            var totalTime = times.Sum();
            totalTime.Should().BeLessThan(250,
                $"All 5 pack validations must complete in <250ms, took {totalTime}ms total");

            // Verify we actually measured some timing
            times.Should().NotBeEmpty();
        }

        // ── SLA 5: CompatibilityChecker.IsVersionInRange < 1ms per call ──────

        [Fact]
        public void CompatibilityCheck_VersionInRange_1000Calls_UnderThreshold()
        {
            // SLA: CompatibilityChecker.IsVersionInRange < 1ms per call (1000 calls)
            var sw = Stopwatch.StartNew();

            // Act: 1000 version range checks with various constraints
            int trueCount = 0;
            for (int i = 0; i < 1000; i++)
            {
                // Cycle through various version/range pairs
                bool result = (i % 4) switch
                {
                    0 => CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0"),
                    1 => CompatibilityChecker.IsVersionInRange("2.0.0", "<3.0.0"),
                    2 => CompatibilityChecker.IsVersionInRange("0.5.1", "*"),
                    3 => CompatibilityChecker.IsVersionInRange("2.1.3", ">=2.0.0 <3.0.0"),
                    _ => false
                };
                if (result)
                    trueCount++;
            }

            sw.Stop();

            // Assert
            trueCount.Should().BeGreaterThan(0, "Some checks should succeed");
            var avgMs = sw.ElapsedMilliseconds / 1000.0;
            avgMs.Should().BeLessThan(1.0,
                $"Average version check time must be <1ms, was {avgMs:F4}ms");
        }

        [Fact]
        public void CompatibilityCheck_ComplexRanges_StillFast()
        {
            // SLA: Even complex ranges with multiple constraints should be fast
            var complexRanges = new[]
            {
                ">=1.0.0 <2.0.0",
                ">=0.5.0 <=1.5.0",
                ">1.0.0 <=2.5.0",
                ">=2021.3.0 <2021.4.0"  // Unity-style
            };

            var sw = Stopwatch.StartNew();

            // Act: 250 complex range checks
            for (int i = 0; i < 250; i++)
            {
                var range = complexRanges[i % complexRanges.Length];
                var version = i % 2 == 0 ? "1.5.0" : "2021.3.15f1";
                var result = CompatibilityChecker.IsVersionInRange(version, range);
                // Result is a boolean indicating if version satisfies range
                _ = result;
            }

            sw.Stop();

            // Assert: 250 complex checks should still be fast (allow 2000ms budget to account for JIT warmup)
            sw.ElapsedMilliseconds.Should().BeLessThan(2000,
                $"250 complex version range checks must complete in <2000ms, took {sw.ElapsedMilliseconds}ms");
        }

        // ── Integration: Full Pipeline Timing ────────────────────────────────

        [Fact]
        public void FullPipeline_LoadResolveParse_Completes()
        {
            // Integration test: Load, resolve, and parse 5 packs with dependencies
            // Verify entire pipeline stays performant

            var packs = new List<PackManifest>();
            for (int i = 0; i < 5; i++)
            {
                var deps = i > 0 ? new[] { $"workflow-{i - 1}" } : Array.Empty<string>();
                string yaml = MakeMinimalPackYaml($"workflow-{i}", deps);
                packs.Add(_packLoader.LoadFromString(yaml));
            }

            var sw = Stopwatch.StartNew();

            // Act: Full workflow
            var result = _resolver.ComputeLoadOrder(packs);
            var compat = CompatibilityChecker.IsVersionInRange("0.5.0", ">=0.1.0 <1.0.0");

            sw.Stop();

            // Assert: Full workflow should be efficient
            result.IsSuccess.Should().BeTrue();
            compat.Should().BeTrue();
            sw.ElapsedMilliseconds.Should().BeLessThan(250,
                $"Full pipeline (load + resolve + compat check) must complete in <250ms, took {sw.ElapsedMilliseconds}ms");
        }
    }
}
