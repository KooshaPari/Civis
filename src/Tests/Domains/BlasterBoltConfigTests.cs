#nullable enable
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Behavior coverage for <see cref="BlasterBoltConfig"/> — the pure-C# (Unity-free) blaster-bolt
/// projectile recolour config consumed by <c>ProjectileMeshSwapSystem</c>. Guards: hex parsing
/// across all accepted forms + malformed input, faction-id substring matching, faction→colour
/// resolution, emission-intensity validation, and the pack-override/hot-reload reset lifecycle.
///
/// All tests call <see cref="BlasterBoltConfig.ResetToDefaults"/> first because the config is a
/// process-wide static (pack load mutates it); without reset, ordering would leak state. The type
/// exposes ResetToDefaults precisely for this isolation contract.
/// </summary>
[Collection("BlasterBoltConfig")] // serialize: shared static state, must not run concurrently
public class BlasterBoltConfigTests
{
    public BlasterBoltConfigTests() => BlasterBoltConfig.ResetToDefaults();

    // ---- ResolveBoltColor (isEnemy overload): Enemy → CIS red, friendly → Republic blue ----

    [Fact]
    public void ResolveBoltColor_Enemy_ReturnsCisRed()
    {
        BlasterBoltConfig.BoltColor c = BlasterBoltConfig.ResolveBoltColor(isEnemy: true);

        c.R.Should().Be(BlasterBoltConfig.DefaultCisColor.R);
        c.G.Should().Be(BlasterBoltConfig.DefaultCisColor.G);
        c.B.Should().Be(BlasterBoltConfig.DefaultCisColor.B);
        // Red dominant — sanity that the default is actually red, not just "the CIS constant".
        c.R.Should().BeGreaterThan(c.B);
    }

    [Fact]
    public void ResolveBoltColor_Friendly_ReturnsRepublicBlue()
    {
        BlasterBoltConfig.BoltColor c = BlasterBoltConfig.ResolveBoltColor(isEnemy: false);

        c.B.Should().Be(BlasterBoltConfig.DefaultRepublicColor.B);
        // Blue dominant for the friendly clone bolts.
        c.B.Should().BeGreaterThan(c.R);
    }

    [Theory]
    [InlineData(BlasterBoltConfig.BoltFaction.Cis)]
    [InlineData(BlasterBoltConfig.BoltFaction.Republic)]
    public void ResolveBoltColor_ByFaction_MatchesIsEnemyOverload(BlasterBoltConfig.BoltFaction faction)
    {
        bool isEnemy = faction == BlasterBoltConfig.BoltFaction.Cis;

        BlasterBoltConfig.BoltColor byFaction = BlasterBoltConfig.ResolveBoltColor(faction);
        BlasterBoltConfig.BoltColor byFlag = BlasterBoltConfig.ResolveBoltColor(isEnemy);

        byFaction.R.Should().Be(byFlag.R);
        byFaction.G.Should().Be(byFlag.G);
        byFaction.B.Should().Be(byFlag.B);
    }

    // ---- FactionFromId: CIS substrings (case-insensitive) vs Republic default ----

    [Theory]
    [InlineData("cis")]
    [InlineData("CIS")]
    [InlineData("separatist")]
    [InlineData("Separatists")]
    [InlineData("droid-army")]
    [InlineData("Enemy")]
    [InlineData("Confederacy")]
    [InlineData("  cis  ")] // trimmed
    public void FactionFromId_CisLikeIds_ResolveToCis(string id)
    {
        BlasterBoltConfig.FactionFromId(id).Should().Be(BlasterBoltConfig.BoltFaction.Cis);
    }

    [Theory]
    [InlineData("republic")]
    [InlineData("Galactic Republic")]
    [InlineData("clone")]
    [InlineData("unknown-faction")]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData(null)]
    public void FactionFromId_NonCisOrBlank_DefaultsToRepublic(string? id)
    {
        BlasterBoltConfig.FactionFromId(id).Should().Be(BlasterBoltConfig.BoltFaction.Republic);
    }

    // ---- TryParseHexColor: all accepted forms + malformed input ----

    [Theory]
    [InlineData("#FF0000")]   // RRGGBB with hash
    [InlineData("FF0000")]    // RRGGBB no hash
    [InlineData("#f00")]      // shorthand RGB → FF0000
    public void TryParseHexColor_RedForms_ParseToPureRed(string hex)
    {
        BlasterBoltConfig.TryParseHexColor(hex, out BlasterBoltConfig.BoltColor c).Should().BeTrue();

        c.R.Should().BeApproximately(1f, 0.001f);
        c.G.Should().BeApproximately(0f, 0.001f);
        c.B.Should().BeApproximately(0f, 0.001f);
        c.A.Should().BeApproximately(1f, 0.001f); // RGB forms default alpha to 1.
    }

    [Fact]
    public void TryParseHexColor_RgbaForm_ParsesAlpha()
    {
        BlasterBoltConfig.TryParseHexColor("#00FF0080", out BlasterBoltConfig.BoltColor c).Should().BeTrue();

        c.G.Should().BeApproximately(1f, 0.001f);
        c.A.Should().BeApproximately(0x80 / 255f, 0.001f); // ~0.502
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData("#GG0000")]   // non-hex digits
    [InlineData("#FF00")]     // length 4 — neither 3/6/8
    [InlineData("#FF0000000")] // length 9 — too long
    [InlineData("xyz")]
    public void TryParseHexColor_Malformed_ReturnsFalse(string? hex)
    {
        BlasterBoltConfig.TryParseHexColor(hex, out BlasterBoltConfig.BoltColor c).Should().BeFalse();
        // On failure the out value is the struct default (all zero).
        c.R.Should().Be(0f);
        c.A.Should().Be(0f);
    }

    // ---- SetFactionColorHex: applies on valid, no-ops + returns false on invalid ----

    [Fact]
    public void SetFactionColorHex_ValidHex_OverridesResolvedColor()
    {
        bool ok = BlasterBoltConfig.SetFactionColorHex(BlasterBoltConfig.BoltFaction.Cis, "#00FF00");

        ok.Should().BeTrue();
        BlasterBoltConfig.BoltColor c = BlasterBoltConfig.ResolveBoltColor(BlasterBoltConfig.BoltFaction.Cis);
        c.G.Should().BeApproximately(1f, 0.001f);
        c.R.Should().BeApproximately(0f, 0.001f);
    }

    [Fact]
    public void SetFactionColorHex_InvalidHex_ReturnsFalseAndRetainsDefault()
    {
        bool ok = BlasterBoltConfig.SetFactionColorHex(BlasterBoltConfig.BoltFaction.Republic, "not-a-color");

        ok.Should().BeFalse();
        BlasterBoltConfig.BoltColor c = BlasterBoltConfig.ResolveBoltColor(BlasterBoltConfig.BoltFaction.Republic);
        c.B.Should().Be(BlasterBoltConfig.DefaultRepublicColor.B); // unchanged
    }

    // ---- SetEmissionIntensity: positive applied, non-positive / non-finite ignored ----

    [Fact]
    public void SetEmissionIntensity_PositiveValue_IsApplied()
    {
        BlasterBoltConfig.SetEmissionIntensity(7.5f);
        BlasterBoltConfig.EmissionIntensity.Should().Be(7.5f);
    }

    [Theory]
    [InlineData(0f)]
    [InlineData(-3f)]
    [InlineData(float.NaN)]
    [InlineData(float.PositiveInfinity)]
    public void SetEmissionIntensity_InvalidValue_IsIgnored(float bad)
    {
        BlasterBoltConfig.SetEmissionIntensity(bad);
        BlasterBoltConfig.EmissionIntensity.Should().Be(BlasterBoltConfig.DefaultEmissionIntensity);
    }

    // ---- Enabled flag + ResetToDefaults lifecycle ----

    [Fact]
    public void Enabled_RoundTrips()
    {
        BlasterBoltConfig.Enabled = false;
        BlasterBoltConfig.Enabled.Should().BeFalse();
    }

    [Fact]
    public void ResetToDefaults_RestoresColorsIntensityAndEnabled()
    {
        BlasterBoltConfig.SetFactionColorHex(BlasterBoltConfig.BoltFaction.Cis, "#000000");
        BlasterBoltConfig.SetEmissionIntensity(99f);
        BlasterBoltConfig.Enabled = false;

        BlasterBoltConfig.ResetToDefaults();

        BlasterBoltConfig.Enabled.Should().BeTrue();
        BlasterBoltConfig.EmissionIntensity.Should().Be(BlasterBoltConfig.DefaultEmissionIntensity);
        BlasterBoltConfig.BoltColor c = BlasterBoltConfig.ResolveBoltColor(BlasterBoltConfig.BoltFaction.Cis);
        c.R.Should().Be(BlasterBoltConfig.DefaultCisColor.R);
    }
}
