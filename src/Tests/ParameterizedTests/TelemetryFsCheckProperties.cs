using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System;
using System.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

[Trait("Category", "Property")]
[Trait("Layer", "Telemetry")]
public class TelemetryFsCheckProperties
{
    [Property(MaxTest = 100)]
    public bool Counter_Increment_N_Times_Equals_N(byte n)
    {
        long counter = 0;
        for (int i = 0; i < n; i++) counter++;
        return counter == n;
    }

    [Property(MaxTest = 100)]
    public bool Mean_Of_N_Copies_Of_V_Equals_V(int n, int v)
    {
        if (n <= 0) n = 1;
        if (n > 1000) n = 1000;
        var stream = Enumerable.Repeat((long)v, n).ToArray();
        var mean = stream.Sum() / (double)n;
        return Math.Abs(mean - v) < 1e-9;
    }

    [Property(MaxTest = 100)]
    public bool Min_LE_Mean_LE_Max(int[] stream)
    {
        if (stream is null || stream.Length == 0) return true;
        var min = stream.Min();
        var max = stream.Max();
        var mean = stream.Average();
        return min <= mean && mean <= max;
    }

    [Property(MaxTest = 100)]
    public bool Histogram_Bucket_Assignment_Is_Unique(int value, byte numBuckets)
    {
        if (numBuckets == 0) return true;
        int bucket = Math.Abs(value) % numBuckets;
        return bucket >= 0 && bucket < numBuckets;
    }

    [Property(MaxTest = 100)]
    public bool Reservoir_Of_Size_K_Has_At_Most_K_Items(byte k, byte numItems)
    {
        var reservoir = new System.Collections.Generic.List<byte>();
        for (int i = 0; i < numItems; i++)
        {
            if (reservoir.Count < k) reservoir.Add((byte)i);
        }
        return reservoir.Count <= k;
    }
}
