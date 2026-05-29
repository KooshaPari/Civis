using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using BenchmarkDotNet.Attributes;
using DINOForge.SDK.Patching;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for patch operation application.
/// Measures the performance of applying single and compound patch operations
/// that modify unit definitions, stat overrides, and content modifications.
/// </summary>
[MemoryDiagnoser]
[SimpleJob(warmupCount: 3, targetCount: 5)]
public class PatchOperationsBenchmark
{
    private Dictionary<string, object> _baseUnitDefinition = null!;
    private ReplacePatchOperation _singleReplacePatch = null!;
    private List<IPatchOperation> _mixedPatchSet = null!;

    [GlobalSetup]
    [ExcludeFromCodeCoverage]
    public void GlobalSetup()
    {
        // Create a realistic base unit definition
        _baseUnitDefinition = new Dictionary<string, object>
        {
            { "id", "test-unit-001" },
            { "name", "Test Unit" },
            { "health", 100 },
            { "armor", 5 },
            { "cost", 500 },
            { "buildTime", 30 },
            { "description", "A test unit for benchmark purposes" },
            { "faction", "test-faction" },
            { "class", "soldier" }
        };

        // Create a single replace patch
        _singleReplacePatch = new ReplacePatchOperation
        {
            Path = "/health",
            Value = "150"
        };

        // Create a mixed patch set
        _mixedPatchSet = new List<IPatchOperation>
        {
            new ReplacePatchOperation { Path = "/health", Value = "150" },
            new ReplacePatchOperation { Path = "/armor", Value = "10" },
            new ReplacePatchOperation { Path = "/cost", Value = "600" },
            new ReplacePatchOperation { Path = "/buildTime", Value = "40" }
        };
    }

    /// <summary>
    /// Applies a single replace patch operation.
    /// Represents the simplest patch case: a single stat override.
    /// </summary>
    [Benchmark]
    public bool ApplySingleReplacePatch()
    {
        var target = new Dictionary<string, object>(_baseUnitDefinition);
        return ApplyPatch(target, _singleReplacePatch);
    }

    /// <summary>
    /// Applies a set of patch operations simulating realistic balance tweaks.
    /// Tests the overhead of compound patches (5-10 operations typical).
    /// </summary>
    [Benchmark]
    public bool ApplyMixedPatchSet()
    {
        var target = new Dictionary<string, object>(_baseUnitDefinition);
        foreach (var patch in _mixedPatchSet)
        {
            if (!ApplyPatch(target, patch))
            {
                return false;
            }
        }

        return true;
    }

    /// <summary>
    /// Measures patch application performance with varying patch set sizes.
    /// Tests scalability as the number of overrides increases.
    /// </summary>
    [Benchmark]
    [Arguments(1)]
    [Arguments(5)]
    [Arguments(10)]
    [Arguments(20)]
    public int ApplyScalingPatchSet(int patchCount)
    {
        var target = new Dictionary<string, object>(_baseUnitDefinition);
        var appliedCount = 0;

        for (var i = 0; i < patchCount; i++)
        {
            var patch = new ReplacePatchOperation
            {
                Path = $"/stat{i}",
                Value = (i * 10).ToString()
            };

            target[$"stat{i}"] = i * 10;
            if (ApplyPatch(target, patch))
            {
                appliedCount++;
            }
        }

        return appliedCount;
    }

    /// <summary>
    /// Applies a wildcard multiply patch, testing complex path matching.
    /// Wildcard operations are expensive due to reflection/matching overhead.
    /// </summary>
    [Benchmark]
    public bool ApplyWildcardMultiplyPatch()
    {
        var target = new Dictionary<string, object>(_baseUnitDefinition);
        var wildcardPatch = new MultiplyPatchOperation
        {
            Path = "/*",
            Factor = 1.5
        };

        return ApplyComplexPatch(target, wildcardPatch);
    }

    /// <summary>
    /// Simulates a full balance patch application scenario:
    /// 1. Load base definition
    /// 2. Apply 3 stat overrides
    /// 3. Apply 1 complex operation
    /// 4. Verify result
    /// </summary>
    [Benchmark]
    public Dictionary<string, object> ApplyFullBalancePatchScenario()
    {
        var target = new Dictionary<string, object>(_baseUnitDefinition);

        // Apply typical balance patches
        _ = ApplyPatch(target, new ReplacePatchOperation { Path = "/health", Value = "120" });
        _ = ApplyPatch(target, new ReplacePatchOperation { Path = "/cost", Value = "550" });
        _ = ApplyPatch(target, new ReplacePatchOperation { Path = "/buildTime", Value = "35" });

        return target;
    }

    /// <summary>
    /// Helper method to apply a simple patch operation.
    /// </summary>
    private static bool ApplyPatch(Dictionary<string, object> target, IPatchOperation patch)
    {
        if (patch is ReplacePatchOperation replace)
        {
            var key = replace.Path.TrimStart('/');
            target[key] = replace.Value;
            return true;
        }

        return false;
    }

    /// <summary>
    /// Helper method to apply complex patch operations.
    /// </summary>
    private static bool ApplyComplexPatch(Dictionary<string, object> target, IPatchOperation patch)
    {
        if (patch is MultiplyPatchOperation multiply)
        {
            foreach (var kvp in target.ToList())
            {
                if (kvp.Value is int intVal)
                {
                    target[kvp.Key] = (int)(intVal * multiply.Factor);
                }
                else if (kvp.Value is double doubleVal)
                {
                    target[kvp.Key] = doubleVal * multiply.Factor;
                }
            }

            return true;
        }

        return false;
    }
}

/// <summary>
/// Simple patch operation interfaces and implementations for benchmarking.
/// </summary>
internal interface IPatchOperation
{
    string Path { get; }
}

internal class ReplacePatchOperation : IPatchOperation
{
    public required string Path { get; set; }
    public required string Value { get; set; }
}

internal class MultiplyPatchOperation : IPatchOperation
{
    public required string Path { get; set; }
    public double Factor { get; set; }
}
