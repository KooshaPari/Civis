#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.SDK
{
    /// <summary>
    /// Thunderstore-style mod profile: a named snapshot of which packs are enabled,
    /// plus per-pack settings overrides. Profiles are serialised to
    /// <c>BepInEx/dinoforge-profiles/&lt;name&gt;.json</c> and can be exported /
    /// imported as JSON strings for sharing.
    ///
    /// Version field is intentionally <c>string</c> so callers can gate on schema version
    /// without a numeric parse:
    ///   Version == "1"  → current layout
    ///   Version &gt; "1" → reject on import (unknown future schema)
    /// </summary>
    public sealed class ModProfile
    {
        /// <summary>Human-readable profile name (also used as the file stem).</summary>
        public string Name { get; set; } = "";

        /// <summary>Schema version — currently always <c>"1"</c>.</summary>
        public string Version { get; set; } = "1";

        /// <summary>DINOForge platform version string captured at save time.</summary>
        public string DinoForgeVersion { get; set; } = "";

        /// <summary>UTC timestamp when this profile was saved.</summary>
        public DateTimeOffset CreatedAt { get; set; }

        /// <summary>Ordered list of pack IDs that should be enabled when this profile is loaded.</summary>
        public List<string> EnabledPacks { get; set; } = new List<string>();

        /// <summary>
        /// Optional per-pack settings overrides.
        /// Key: pack ID (ordinal).  Value: setting key→value map (ordinal).
        /// </summary>
        public Dictionary<string, Dictionary<string, string>> PackSettings { get; set; } =
            new Dictionary<string, Dictionary<string, string>>(StringComparer.Ordinal);
    }
}
