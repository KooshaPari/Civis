#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.Patching;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

/// <summary>
/// FsCheck property-based tests for <see cref="PatchApplicator"/>.
///
/// Each property runs 100+ random iterations to verify algebraic invariants of patch
/// semantics. These supplement the deterministic smoke tests in SmokeTests.cs with
/// exhaustive edge-case coverage derived from generated inputs.
/// </summary>
[Trait("Category", "Property")]
public class PatchApplicatorPropertyTests
{
    // ── Helpers ───────────────────────────────────────────────────────────────

    /// <summary>
    /// Builds a minimal pack-contents dictionary with a single item for use in property tests.
    /// The item always has keys "hp" (int) and "name" (string) to keep tests predictable.
    /// </summary>
    private static Dictionary<string, Dictionary<string, Dictionary<string, object>>> BuildContent(
        string packId,
        string sectionName,
        string itemId,
        int hp,
        string name)
    {
        return new Dictionary<string, Dictionary<string, Dictionary<string, object>>>(StringComparer.Ordinal)
        {
            [packId] = new Dictionary<string, Dictionary<string, object>>(StringComparer.Ordinal)
            {
                [sectionName] = new Dictionary<string, object>(StringComparer.Ordinal)
                {
                    [itemId] = new Dictionary<string, object>(StringComparer.Ordinal)
                    {
                        ["hp"]   = hp,
                        ["name"] = name
                    }
                }
            }
        };
    }

    private static PatchApplicator Applicator() => new PatchApplicator(msg => { /* silent in tests */ });

    // ── Property 1: replace is idempotent ────────────────────────────────────

    /// <summary>
    /// Property: applying a "replace" op twice with the same value leaves the field
    /// unchanged after the second apply (idempotent).
    ///
    ///   ∀ v. replace(replace(x, v), v) == replace(x, v)
    ///
    /// Guards against double-application bugs in multi-phase reload scenarios.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Replace_AppliedTwiceWithSameValue_IsIdempotent(PositiveInt rawHp)
    {
        int hp = rawHp.Get;
        var content = BuildContent("pack-a", "units", "soldier", hp: 50, name: "Soldier");

        var patch = new PatchSet
        {
            TargetPack = "pack-a",
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "replace", Path = "/units/soldier/hp", Value = hp }
            }
        };

        var patchList = new List<(string, PatchSet)> { ("pack-b", patch) };
        var applicator = Applicator();

        // Apply once
        applicator.Apply(content, patchList);
        int afterFirst = (int)((Dictionary<string, object>)content["pack-a"]["units"]["soldier"])["hp"];

        // Apply again with same value
        applicator.Apply(content, patchList);
        int afterSecond = (int)((Dictionary<string, object>)content["pack-a"]["units"]["soldier"])["hp"];

        bool isIdempotent = afterFirst == hp && afterSecond == hp;
        isIdempotent.Should().BeTrue(
            $"replace with value {hp} applied twice must leave field at {hp}, got {afterFirst} then {afterSecond}");
        return isIdempotent;
    }

    // ── Property 2: multiply preserves numeric type (int → int) ─────────────

    /// <summary>
    /// Property: applying a "multiply" op to an int-typed field returns an int result
    /// (not float or double), preserving the original storage type.
    ///
    ///   ∀ i ∈ ℤ, f ∈ ℝ. typeof(multiply(i, f)) == int
    ///
    /// Guards against float bleed when integer stats are scaled (e.g., hp scaling in waves).
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Multiply_OnIntField_PreservesIntType(PositiveInt rawHp, PositiveInt rawFactor)
    {
        // Use small values to avoid overflow: cap factor at 10x
        int hp = rawHp.Get % 1000 + 1; // threshold-ok: 1..1000 stat range
        double factor = (rawFactor.Get % 10) + 0.5; // threshold-ok: 0.5..10.5 multiplier range

        var content = BuildContent("pack-a", "units", "knight", hp: hp, name: "Knight");

        var patch = new PatchSet
        {
            TargetPack = "pack-a",
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "multiply", Path = "/units/knight/hp", Factor = factor }
            }
        };

        var applicator = Applicator();
        applicator.Apply(content, new List<(string, PatchSet)> { ("pack-b", patch) });

        object result = ((Dictionary<string, object>)content["pack-a"]["units"]["knight"])["hp"];
        bool isInt = result is int;
        isInt.Should().BeTrue(
            $"multiply on int hp={hp} by {factor} must return int, got {result?.GetType().Name ?? "null"} ({result})");
        return isInt;
    }

    // ── Property 3: multiply preserves float type (double → double) ──────────

    /// <summary>
    /// Property: applying a "multiply" op to a double-typed field returns a double result.
    ///
    ///   ∀ d ∈ ℝ, f ∈ ℝ. typeof(multiply(d, f)) == double
    ///
    /// Guards against float-to-int coercion when float stats are scaled (e.g., speed values).
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Multiply_OnDoubleField_PreservesDoubleType(NormalFloat rawSpeed, PositiveInt rawFactor)
    {
        double speed = Math.Abs(rawSpeed.Get) + 0.1; // ensure positive, nonzero
        if (double.IsInfinity(speed) || double.IsNaN(speed)) speed = 1.5;
        double factor = (rawFactor.Get % 5) + 0.5; // threshold-ok: 0.5..5.5x range

        var content = new Dictionary<string, Dictionary<string, Dictionary<string, object>>>(StringComparer.Ordinal)
        {
            ["pack-a"] = new Dictionary<string, Dictionary<string, object>>(StringComparer.Ordinal)
            {
                ["units"] = new Dictionary<string, object>(StringComparer.Ordinal)
                {
                    ["archer"] = new Dictionary<string, object>(StringComparer.Ordinal)
                    {
                        ["speed"] = speed
                    }
                }
            }
        };

        var patch = new PatchSet
        {
            TargetPack = "pack-a",
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "multiply", Path = "/units/archer/speed", Factor = factor }
            }
        };

        var applicator = Applicator();
        applicator.Apply(content, new List<(string, PatchSet)> { ("pack-b", patch) });

        object result = ((Dictionary<string, object>)content["pack-a"]["units"]["archer"])["speed"];
        bool isDouble = result is double;
        isDouble.Should().BeTrue(
            $"multiply on double speed={speed} by {factor} must return double, got {result?.GetType().Name ?? "null"}");
        return isDouble;
    }

    // ── Property 4: add-then-remove returns to original state ────────────────

    /// <summary>
    /// Property: adding a field and then removing it leaves the item in its original state
    /// (the field is absent after removal).
    ///
    ///   ∀ k, v. remove(add({}, k, v), k) ≡ {}  (field absent)
    ///
    /// Guards the add/remove symmetry required for transactional pack overriding in future
    /// multi-pass reload scenarios.
    /// </summary>
    [Property(MaxTest = 100)]
    public bool AddThenRemove_FieldIsAbsentAfterRemoval(NonEmptyString rawKey)
    {
        // Sanitize key: no slashes (path separator), no empty
        string fieldKey = rawKey.Get.Replace("/", "_").Replace(" ", "_");
        if (string.IsNullOrWhiteSpace(fieldKey)) fieldKey = "extra_field";

        var content = BuildContent("pack-a", "units", "mage", hp: 80, name: "Mage");

        // Step 1: add the field
        var addPatch = new PatchSet
        {
            TargetPack = "pack-a",
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "add", Path = $"/units/mage/{fieldKey}", Value = "test_value" }
            }
        };

        var applicator = Applicator();
        applicator.Apply(content, new List<(string, PatchSet)> { ("pack-b", addPatch) });

        var item = (Dictionary<string, object>)content["pack-a"]["units"]["mage"];
        item.ContainsKey(fieldKey).Should().BeTrue($"field '{fieldKey}' must exist after add");

        // Step 2: remove the field
        var removePatch = new PatchSet
        {
            TargetPack = "pack-a",
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "remove", Path = $"/units/mage/{fieldKey}" }
            }
        };

        applicator.Apply(content, new List<(string, PatchSet)> { ("pack-c", removePatch) });

        bool fieldAbsent = !item.ContainsKey(fieldKey);
        fieldAbsent.Should().BeTrue($"field '{fieldKey}' must be absent after remove");
        return fieldAbsent;
    }

    // ── Property 5: patches to nonexistent pack are silently skipped ──────────

    /// <summary>
    /// Property: a patch set that targets a pack ID not present in the content dictionary
    /// does not throw; the content dictionary is left completely unchanged.
    ///
    /// Guards against crashes when a balance pack targets a warfare pack that was not loaded
    /// (e.g. optional dependency not installed by the user).
    /// </summary>
    [Property(MaxTest = 100)]
    public bool Patch_TargetingAbsentPack_DoesNotThrow_AndLeavesContentUnchanged(NonEmptyString rawTarget)
    {
        string target = "absent-" + rawTarget.Get.Replace("/", "_").Replace(" ", "_");
        var content = BuildContent("pack-a", "units", "guard", hp: 60, name: "Guard");

        var patch = new PatchSet
        {
            TargetPack = target, // not in content
            Operations = new List<PatchOperation>
            {
                new PatchOperation { Op = "replace", Path = "/units/guard/hp", Value = 9999 }
            }
        };

        bool threw = false;
        var applicator = Applicator();
        try
        {
            applicator.Apply(content, new List<(string, PatchSet)> { ("pack-b", patch) });
        }
        catch
        {
            threw = true;
        }

        bool noThrow = !threw;
        noThrow.Should().BeTrue("patching an absent target pack must never throw");

        // Original pack-a data must be untouched
        var item = (Dictionary<string, object>)content["pack-a"]["units"]["guard"];
        int hpAfter = (int)item["hp"];
        bool unchanged = hpAfter == 60;
        unchanged.Should().BeTrue($"pack-a must be unchanged; hp expected 60, got {hpAfter}");

        return noThrow && unchanged;
    }
}
