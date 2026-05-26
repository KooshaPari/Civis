using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using System;
using System.Linq;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests;

[Trait("Category", "Property")]
[Trait("Layer", "SemVer")]
public class SemVerInvariantsFsCheckProperties
{
    private static Version V(int maj, int min, int bld = 0, int rev = 0) =>
        new(Math.Max(0, maj), Math.Max(0, min), Math.Max(0, bld), Math.Max(0, rev));

    [Property(MaxTest = 100)]
    public bool Version_Reflexive_Equal(byte maj, byte min, byte bld, byte rev)
    {
        var v = V(maj, min, bld, rev);
        return v.Equals(v) && v.CompareTo(v) == 0;
    }

    [Property(MaxTest = 100)]
    public bool Version_Antisymmetric(byte am, byte an, byte bm, byte bn)
    {
        var a = V(am, an);
        var b = V(bm, bn);
        if (a.Equals(b)) return true;
        return Math.Sign(a.CompareTo(b)) == -Math.Sign(b.CompareTo(a));
    }

    [Property(MaxTest = 100)]
    public bool Version_Transitive_LessThan(byte am, byte bm, byte cm)
    {
        var sorted = new[] { am, bm, cm }.OrderBy(x => x).ToArray();
        var a = V(sorted[0], 0);
        var b = V(sorted[1], 0);
        var c = V(sorted[2], 0);
        if (a.CompareTo(b) > 0 || b.CompareTo(c) > 0) return true;
        return a.CompareTo(c) <= 0;
    }

    [Property(MaxTest = 100)]
    public bool Version_Hash_Consistent_With_Equality(byte am, byte an, byte ab, byte ar)
    {
        var v1 = V(am, an, ab, ar);
        var v2 = V(am, an, ab, ar);
        return !v1.Equals(v2) || v1.GetHashCode() == v2.GetHashCode();
    }

    [Property(MaxTest = 100)]
    public bool Version_Major_Dominates(byte am, byte bm, byte aMin, byte bMin)
    {
        if (am == bm) return true;
        var (high, low) = am > bm ? (V(am, aMin), V(bm, bMin)) : (V(bm, bMin), V(am, aMin));
        return high.CompareTo(low) > 0;
    }
}
