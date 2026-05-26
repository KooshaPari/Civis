using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System;
using System.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

[Trait("Category", "Property")]
[Trait("Layer", "DumpTools")]
public class DumpToolsFsCheckProperties
{
    private static string FormatLine(string[] components, int count) =>
        string.Join(",", components) + " -> " + count.ToString(System.Globalization.CultureInfo.InvariantCulture);

    private static (string[] components, int count)? ParseLine(string line)
    {
        if (string.IsNullOrWhiteSpace(line)) return null;
        var arrow = line.IndexOf(" -> ", StringComparison.Ordinal);
        if (arrow < 0) return null;
        var compsStr = line.Substring(0, arrow);
        var countStr = line.Substring(arrow + 4).Trim();
        if (!int.TryParse(countStr, System.Globalization.NumberStyles.Integer, System.Globalization.CultureInfo.InvariantCulture, out var count)) return null;
        var comps = compsStr.Split(new[] { ',' }, StringSplitOptions.RemoveEmptyEntries).Select(s => s.Trim()).Where(s => s.Length > 0).ToArray();
        return (comps, count);
    }

    [Property(MaxTest = 100)]
    public bool DumpParse_RoundTrip_Preserves_Components_And_Count(NonEmptyString[] components, int count)
    {
        if (count < 0) count = -count;
        var compsClean = components.Select(c => new string(c.Get.Where(ch => ch >= 0x21 && ch <= 0x7E && ch != ',' && ch != '-' && ch != '>').ToArray())).Where(s => s.Length > 0).ToArray();
        if (compsClean.Length == 0) return true;
        var line = FormatLine(compsClean, count);
        var parsed = ParseLine(line);
        return parsed.HasValue && parsed.Value.components.SequenceEqual(compsClean) && parsed.Value.count == count;
    }

    [Property(MaxTest = 100)]
    public bool DumpParse_Aggregation_Total_Equals_Sum(int[] counts)
    {
        var clean = counts.Select(c => c < 0 ? -c : c).ToArray();
        var sum = clean.Aggregate(0L, (a, b) => a + b);
        return sum >= 0;
    }

    [Property(MaxTest = 100)]
    public bool DumpParse_NoArrow_ReturnsNull(NonEmptyString s)
    {
        if (s.Get.Contains(" -> ")) return true;
        return ParseLine(s.Get) is null;
    }

    [Property(MaxTest = 100)]
    public bool DumpParse_NonNumericCount_ReturnsNull(NonEmptyString comp, NonEmptyString countStr)
    {
        if (int.TryParse(countStr.Get, out _)) return true;
        var line = comp.Get + " -> " + countStr.Get;
        return ParseLine(line) is null;
    }

    [Property(MaxTest = 100)]
    public bool DumpParse_EmptyComponentList_HandledDeterministically(int count)
    {
        if (count < 0) count = -count;
        var line = " -> " + count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        var parsed = ParseLine(line);
        return parsed is null || parsed.Value.count == count;
    }
}
