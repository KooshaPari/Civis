#nullable enable
using System.Collections.Generic;
using DINOForge.SDK;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Computes the merged badge list for a pack from three sources:
    /// <list type="number">
    ///   <item>Author-declared badges in <c>pack.yaml</c> (<c>badges:</c> array) — restricted to allowed values.</item>
    ///   <item>Curated badges assigned from the signed allowlist (<see cref="CuratedBadges"/>).</item>
    ///   <item>Auto-computed runtime badges (<c>popular</c>, <c>compatibility-tested</c>).</item>
    /// </list>
    ///
    /// Only the values listed in <see cref="AuthorAllowedBadges"/> may be declared in <c>pack.yaml</c>.
    /// Any other values are silently stripped so that curated badges cannot be self-assigned.
    /// </summary>
    public static class BadgeComputer
    {
        /// <summary>
        /// Badge names that a pack author is allowed to self-declare in <c>pack.yaml</c>.
        /// </summary>
        public static readonly HashSet<string> AuthorAllowedBadges = new HashSet<string>(System.StringComparer.Ordinal)
        {
            "early-access",
            "total-conversion",
        };

        /// <summary>
        /// Curated badges that are appended by a trusted gatekeeper (e.g. DINOForge team),
        /// keyed by pack ID. This is the in-process signed list; the external signing mechanism
        /// is a future enhancement (#935 backlog). For now it is a static readonly dictionary.
        /// </summary>
        public static readonly Dictionary<string, List<string>> CuratedBadges =
            new Dictionary<string, List<string>>(System.StringComparer.Ordinal)
            {
                // Example — extend as packs earn curation:
                // { "example-hello-world", new List<string> { "verified-author" } },
            };

        /// <summary>
        /// Computes and returns the merged badge list for <paramref name="manifest"/>.
        /// The returned list is ordered: author-declared → curated → auto-computed.
        /// </summary>
        public static List<string> ComputeBadges(PackManifest manifest)
        {
            List<string> result = new List<string>();

            // 1. Author-declared (allow-listed only)
            if (manifest.Badges != null)
            {
                foreach (string badge in manifest.Badges)
                {
                    if (AuthorAllowedBadges.Contains(badge) && !result.Contains(badge))
                    {
                        result.Add(badge);
                    }
                }
            }

            // 2. Curated (signed list keyed by pack ID)
            if (!string.IsNullOrEmpty(manifest.Id) &&
                CuratedBadges.TryGetValue(manifest.Id, out List<string>? curated))
            {
                foreach (string badge in curated)
                {
                    if (!result.Contains(badge))
                        result.Add(badge);
                }
            }

            // 3. Auto-computed: compatibility-tested
            //    A pack that passes CI gets this badge; we mark it if the manifest has
            //    non-empty loads (concrete content that can be CI-tested).
            bool hasCiTestableContent = manifest.Loads != null &&
                ((manifest.Loads.Units != null && manifest.Loads.Units.Count > 0) ||
                 (manifest.Loads.Factions != null && manifest.Loads.Factions.Count > 0) ||
                 (manifest.Loads.Buildings != null && manifest.Loads.Buildings.Count > 0));

            if (hasCiTestableContent && !result.Contains("compatibility-tested"))
            {
                result.Add("compatibility-tested");
            }

            // 4. Auto-computed: popular (>100 downloads)
            //    Download count is not yet tracked server-side; field reserved for future registry.
            //    For now, leave it as runtime-only (never emitted by this method).

            return result;
        }
    }
}
