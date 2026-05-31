#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Pure-C# (no Unity / BepInEx dependency) configuration + resolution layer for the
    /// Star-Wars blaster-bolt projectile visual swap. Compiles in CI without the game
    /// installed (allowlisted in DINOForge.Runtime.csproj) and is directly unit-testable.
    ///
    /// DINO renders projectiles through <c>Unity.Rendering.RenderMesh</c> exactly like every
    /// other entity (per #975). The blaster look is achieved WITHOUT new asset bundles by
    /// recolouring the existing projectile material's emission to a glowing energy-bolt colour,
    /// keyed on the firing faction:
    ///   - Enemy-tagged projectiles (CIS / Separatist droids) → red bolts.
    ///   - Friendly projectiles (Galactic Republic clones)    → blue bolts.
    ///
    /// <see cref="ProjectileMeshSwapSystem"/> is the ECS execution layer that consumes this
    /// config; this type owns only the (faction → colour) data and hex parsing so the logic
    /// stays testable.
    /// </summary>
    public static class BlasterBoltConfig
    {
        /// <summary>The two firing factions a projectile can be attributed to.</summary>
        public enum BoltFaction
        {
            /// <summary>Galactic Republic (clone troopers) — friendly, blue bolts.</summary>
            Republic,

            /// <summary>Confederacy of Independent Systems (droids) — enemy-tagged, red bolts.</summary>
            Cis,
        }

        /// <summary>Linear-ish RGBA colour, components in [0,1]. Unity-free so it stays testable.</summary>
        public readonly struct BoltColor
        {
            /// <summary>Red component, 0..1.</summary>
            public float R { get; }

            /// <summary>Green component, 0..1.</summary>
            public float G { get; }

            /// <summary>Blue component, 0..1.</summary>
            public float B { get; }

            /// <summary>Alpha component, 0..1.</summary>
            public float A { get; }

            /// <summary>Initializes a new <see cref="BoltColor"/>.</summary>
            public BoltColor(float r, float g, float b, float a = 1f)
            {
                R = r;
                G = g;
                B = b;
                A = a;
            }
        }

        /// <summary>
        /// Default emission intensity multiplier applied to the resolved colour so the bolt
        /// reads as a glowing energy projectile rather than a flat-tinted arrow.
        /// </summary>
        public const float DefaultEmissionIntensity = 4.0f;

        /// <summary>Default Republic bolt colour (Clone Wars blue, slightly cyan-white core).</summary>
        public static readonly BoltColor DefaultRepublicColor = new BoltColor(0.20f, 0.55f, 1.0f);

        /// <summary>Default CIS bolt colour (Separatist red).</summary>
        public static readonly BoltColor DefaultCisColor = new BoltColor(1.0f, 0.12f, 0.08f);

        // Mutable so a pack's projectiles/*.yaml can override the defaults at load time
        // (declarative, no hardcoded IDs in engine glue). Guarded by a lock for thread-safety
        // since pack load and the ECS swap system run on different threads.
        private static readonly object _lock = new object();
        private static BoltColor _republicColor = DefaultRepublicColor;
        private static BoltColor _cisColor = DefaultCisColor;
        private static float _emissionIntensity = DefaultEmissionIntensity;
        private static bool _enabled = true;

        /// <summary>Whether projectile bolt recolouring is enabled (pack can disable).</summary>
        public static bool Enabled
        {
            get { lock (_lock) { return _enabled; } }
            set { lock (_lock) { _enabled = value; } }
        }

        /// <summary>Current emission intensity multiplier.</summary>
        public static float EmissionIntensity
        {
            get { lock (_lock) { return _emissionIntensity; } }
        }

        /// <summary>
        /// Resolves the bolt colour for a projectile given whether its owner carries the
        /// vanilla Enemy tag. Enemy → CIS (red); friendly → Republic (blue).
        /// </summary>
        /// <param name="isEnemy">True when the projectile entity has <c>Components.Enemy</c>.</param>
        /// <returns>The configured colour for the implied faction.</returns>
        public static BoltColor ResolveBoltColor(bool isEnemy)
        {
            lock (_lock)
            {
                return isEnemy ? _cisColor : _republicColor;
            }
        }

        /// <summary>Resolves the bolt colour for an explicit faction.</summary>
        /// <param name="faction">The firing faction.</param>
        /// <returns>The configured colour for that faction.</returns>
        public static BoltColor ResolveBoltColor(BoltFaction faction)
        {
            lock (_lock)
            {
                return faction == BoltFaction.Cis ? _cisColor : _republicColor;
            }
        }

        /// <summary>
        /// Maps a pack faction id / name (case-insensitive) to a <see cref="BoltFaction"/>.
        /// Anything that is not recognised as a CIS/Separatist/droid faction defaults to
        /// <see cref="BoltFaction.Republic"/> (the friendly side).
        /// </summary>
        /// <param name="factionId">Faction id from a pack definition (e.g. "cis", "republic").</param>
        /// <returns>The resolved bolt faction.</returns>
        public static BoltFaction FactionFromId(string? factionId)
        {
            if (string.IsNullOrWhiteSpace(factionId))
                return BoltFaction.Republic;

            string f = factionId!.Trim();
            if (f.IndexOf("cis", StringComparison.OrdinalIgnoreCase) >= 0
                || f.IndexOf("separat", StringComparison.OrdinalIgnoreCase) >= 0
                || f.IndexOf("droid", StringComparison.OrdinalIgnoreCase) >= 0
                || f.IndexOf("enemy", StringComparison.OrdinalIgnoreCase) >= 0
                || f.IndexOf("confederac", StringComparison.OrdinalIgnoreCase) >= 0)
            {
                return BoltFaction.Cis;
            }

            return BoltFaction.Republic;
        }

        /// <summary>
        /// Overrides the colour for a faction from a pack-supplied hex string (e.g. "#FF2010"
        /// or "FF2010" or "#FF2010FF"). Invalid strings are ignored (defaults retained) and the
        /// method returns false so callers can log.
        /// </summary>
        /// <param name="faction">Faction whose colour to set.</param>
        /// <param name="hex">Hex colour, with or without leading '#', RGB or RGBA.</param>
        /// <returns>True if parsed and applied; false if the hex was invalid.</returns>
        public static bool SetFactionColorHex(BoltFaction faction, string? hex)
        {
            if (!TryParseHexColor(hex, out BoltColor color))
                return false;

            lock (_lock)
            {
                if (faction == BoltFaction.Cis)
                    _cisColor = color;
                else
                    _republicColor = color;
            }
            return true;
        }

        /// <summary>Sets the emission intensity multiplier (clamped to a sane positive range).</summary>
        /// <param name="intensity">Desired multiplier; values &lt;= 0 are ignored.</param>
        public static void SetEmissionIntensity(float intensity)
        {
            if (intensity <= 0f || float.IsNaN(intensity) || float.IsInfinity(intensity))
                return;
            lock (_lock)
            {
                _emissionIntensity = intensity;
            }
        }

        /// <summary>
        /// Resets all configuration to compiled defaults. Intended for test isolation and
        /// for re-applying a clean state on hot-reload before re-reading pack definitions.
        /// </summary>
        public static void ResetToDefaults()
        {
            lock (_lock)
            {
                _republicColor = DefaultRepublicColor;
                _cisColor = DefaultCisColor;
                _emissionIntensity = DefaultEmissionIntensity;
                _enabled = true;
            }
        }

        /// <summary>
        /// Parses a hex colour string into a <see cref="BoltColor"/>. Accepts "#RGB", "#RRGGBB",
        /// and "#RRGGBBAA" forms (leading '#' optional). Returns false on any malformed input.
        /// </summary>
        /// <param name="hex">The hex string.</param>
        /// <param name="color">The parsed colour (only valid when the method returns true).</param>
        /// <returns>True on success.</returns>
        public static bool TryParseHexColor(string? hex, out BoltColor color)
        {
            color = default;
            if (string.IsNullOrWhiteSpace(hex))
                return false;

            string h = hex!.Trim();
            if (h.StartsWith("#", StringComparison.Ordinal))
                h = h.Substring(1);

            // Expand shorthand #RGB → #RRGGBB.
            if (h.Length == 3)
            {
                h = new string(new[] { h[0], h[0], h[1], h[1], h[2], h[2] });
            }

            if (h.Length != 6 && h.Length != 8)
                return false;

            if (!TryHexByte(h, 0, out int r) ||
                !TryHexByte(h, 2, out int g) ||
                !TryHexByte(h, 4, out int b))
            {
                return false;
            }

            int a = 255;
            if (h.Length == 8 && !TryHexByte(h, 6, out a))
                return false;

            color = new BoltColor(r / 255f, g / 255f, b / 255f, a / 255f);
            return true;
        }

        private static bool TryHexByte(string s, int index, out int value)
        {
            return int.TryParse(
                s.Substring(index, 2),
                NumberStyles.HexNumber,
                CultureInfo.InvariantCulture,
                out value);
        }
    }
}
