#nullable enable
// UpdateChecker — GitHub release polling for DINOForge itself and mod packs.
// Pattern #115: single static HttpClient per destination host.
// Pattern #232: best-effort only; network failure MUST NOT crash the plugin.
// Throttle: persists last-check timestamp to BepInEx/dinoforge-updatecheck.json;
//           skips the network call if fewer than 24 h have elapsed since the last check.
using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using DINOForge.Runtime.Diagnostics;

namespace DINOForge.Runtime.Updates
{
    /// <summary>
    /// Information about an available update for DINOForge or a mod pack.
    /// </summary>
    public sealed class UpdateInfo
    {
        /// <summary>The target pack/component ID (e.g. "DINOForge" or a pack id).</summary>
        public string ComponentId { get; }

        /// <summary>Human-readable display name (pack name or "DINOForge Runtime").</summary>
        public string DisplayName { get; }

        /// <summary>Current version installed.</summary>
        public string CurrentVersion { get; }

        /// <summary>Newer version available on GitHub.</summary>
        public string NewVersion { get; }

        /// <summary>Browser URL to the GitHub release.</summary>
        public string ReleaseUrl { get; }

        /// <summary>Short release notes (first 300 chars of the GitHub release body).</summary>
        public string? Changelog { get; }

        /// <summary>Initializes a new <see cref="UpdateInfo"/>.</summary>
        public UpdateInfo(
            string componentId,
            string displayName,
            string currentVersion,
            string newVersion,
            string releaseUrl,
            string? changelog)
        {
            ComponentId = componentId;
            DisplayName = displayName;
            CurrentVersion = currentVersion;
            NewVersion = newVersion;
            ReleaseUrl = releaseUrl;
            Changelog = changelog;
        }
    }

    /// <summary>
    /// Checks GitHub releases for updates to DINOForge and mod packs.
    /// All network operations are async and best-effort — failures are logged and swallowed.
    /// </summary>
    internal sealed class UpdateChecker
    {
        // ── Constants ────────────────────────────────────────────────────────────
        private const string GitHubApiBase = "https://api.github.com";
        private const string UserAgent = "DINOForge-UpdateChecker/1.0";
        private const int ThrottleHours = 24; // threshold-ok: 24-hour cache window
        private const string StateFileName = "dinoforge-updatecheck.json";

        // ── Pattern #115: one static HttpClient shared for the process lifetime ──
        // http-client-ok: static singleton; reused across all update-check calls
        private static readonly HttpClient SharedHttp = CreateHttpClient();

        private readonly string _stateFilePath;

        // ── Constructor ──────────────────────────────────────────────────────────

        /// <summary>
        /// Creates an UpdateChecker whose throttle state is persisted to <paramref name="bepInExRoot"/>.
        /// </summary>
        internal UpdateChecker(string bepInExRoot)
        {
            _stateFilePath = Path.Combine(bepInExRoot, StateFileName);
        }

        // ── Public API ───────────────────────────────────────────────────────────

        /// <summary>
        /// Checks GitHub for a newer release of a component.
        /// Returns <see langword="null"/> when no update is available or on network failure.
        /// </summary>
        /// <param name="repoOwner">GitHub org/user name (e.g. "KooshaPari").</param>
        /// <param name="repoName">Repository name (e.g. "Dino").</param>
        /// <param name="componentId">Short identifier used in the throttle state (e.g. pack id or "DINOForge").</param>
        /// <param name="displayName">Human-readable name shown in the UI.</param>
        /// <param name="currentVersion">Currently installed version string (SemVer-like).</param>
        /// <param name="ct">Cancellation token.</param>
        public async Task<UpdateInfo?> CheckForUpdateAsync(
            string repoOwner,
            string repoName,
            string componentId,
            string displayName,
            string currentVersion,
            CancellationToken ct)
        {
            try
            {
                string url = $"{GitHubApiBase}/repos/{Uri.EscapeDataString(repoOwner)}/{Uri.EscapeDataString(repoName)}/releases/latest";

                using (HttpRequestMessage request = new HttpRequestMessage(HttpMethod.Get, url))
                {
                    request.Headers.TryAddWithoutValidation("User-Agent", UserAgent);
                    request.Headers.TryAddWithoutValidation("Accept", "application/vnd.github.v3+json");

                    HttpResponseMessage response = await SharedHttp.SendAsync(request, ct).ConfigureAwait(false);

                    if (response.StatusCode == HttpStatusCode.NotFound)
                    {
                        // No releases published yet — not an error
                        return null;
                    }

                    if (!response.IsSuccessStatusCode)
                    {
                        DebugLog.Write("UpdateChecker",
                            $"[UpdateChecker] GitHub API returned {(int)response.StatusCode} for {repoOwner}/{repoName}");
                        return null;
                    }

                    string json = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
                    return ParseRelease(json, componentId, displayName, currentVersion);
                }
            }
            catch (OperationCanceledException)
            {
                return null;
            }
            catch (Exception ex)
            {
                // safe-swallow: network failure must NOT crash the plugin (Pattern #232 extension)
                DebugLog.Write("UpdateChecker",
                    $"[UpdateChecker] CheckForUpdateAsync failed for {repoOwner}/{repoName}: {ex.GetType().Name}: {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Runs update checks for DINOForge and all packs that declare an <c>update_check</c>.
        /// Respects the 24-hour throttle window. Returns all updates found (possibly empty list).
        /// </summary>
        /// <param name="packChecks">
        /// List of (componentId, displayName, repoOwner, repoName, currentVersion) for each pack.
        /// </param>
        /// <param name="dinoForgeVersion">Currently installed DINOForge version.</param>
        /// <param name="ct">Cancellation token.</param>
        public async Task<IReadOnlyList<UpdateInfo>> RunAllChecksAsync(
            IReadOnlyList<PackUpdateTarget> packChecks,
            string dinoForgeVersion,
            CancellationToken ct)
        {
            // Throttle: skip if last check was within the window.
            if (!ShouldCheck())
            {
                DebugLog.Write("UpdateChecker", "[UpdateChecker] Throttle active — skipping network checks.");
                return Array.Empty<UpdateInfo>();
            }

            List<Task<UpdateInfo?>> tasks = new List<Task<UpdateInfo?>>();

            // DINOForge itself
            tasks.Add(CheckForUpdateAsync(
                "KooshaPari", "Dino",
                "DINOForge", "DINOForge Runtime",
                dinoForgeVersion, ct));

            // Each pack
            foreach (PackUpdateTarget pack in packChecks)
            {
                tasks.Add(CheckForUpdateAsync(
                    pack.RepoOwner, pack.RepoName,
                    pack.ComponentId, pack.DisplayName,
                    pack.CurrentVersion, ct));
            }

            UpdateInfo?[] results = await Task.WhenAll(tasks).ConfigureAwait(false);

            // Persist throttle timestamp now (even if results are empty)
            PersistLastCheck();

            List<UpdateInfo> updates = new List<UpdateInfo>();
            foreach (UpdateInfo? info in results)
            {
                if (info != null)
                    updates.Add(info);
            }

            if (updates.Count > 0)
            {
                DebugLog.Write("UpdateChecker", $"[UpdateChecker] {updates.Count} update(s) found.");
            }

            return updates;
        }

        // ── Private helpers ──────────────────────────────────────────────────────

        private static HttpClient CreateHttpClient()
        {
            HttpClient client = new HttpClient();
            client.Timeout = TimeSpan.FromSeconds(15); // threshold-ok: 15s network timeout
            return client;
        }

        private UpdateInfo? ParseRelease(
            string json,
            string componentId,
            string displayName,
            string currentVersion)
        {
            try
            {
                JObject release = JObject.Parse(json);
                string? tagName = release["tag_name"]?.Value<string>();
                string? htmlUrl = release["html_url"]?.Value<string>();
                string? body = release["body"]?.Value<string>();

                if (string.IsNullOrEmpty(tagName) || string.IsNullOrEmpty(htmlUrl))
                    return null;

                // Normalize tag: strip leading "v" for comparison
                string newVersion = tagName.TrimStart('v');
                string curNorm = currentVersion.TrimStart('v');

                // Strip pre-release suffix for comparison (e.g. "0.25.0-dev" → "0.25.0")
                int dashIdx = curNorm.IndexOf('-');
                if (dashIdx >= 0)
                    curNorm = curNorm.Substring(0, dashIdx);

                if (string.Compare(newVersion, curNorm, StringComparison.Ordinal) <= 0)
                    return null; // tag-name-ok: up to date

                // Trim changelog to 300 chars
                string? changelog = body;
                if (changelog != null && changelog.Length > 300) // threshold-ok: changelog preview limit
                    changelog = changelog.Substring(0, 300) + "…";

                return new UpdateInfo(componentId, displayName, currentVersion, newVersion, htmlUrl, changelog);
            }
            catch (Exception ex)
            {
                // safe-swallow: malformed release JSON should not crash
                DebugLog.Write("UpdateChecker", $"[UpdateChecker] ParseRelease failed: {ex.Message}");
                return null;
            }
        }

        // ── Throttle state ───────────────────────────────────────────────────────

        private bool ShouldCheck()
        {
            try
            {
                if (!File.Exists(_stateFilePath))
                    return true;

                string raw = File.ReadAllText(_stateFilePath, System.Text.Encoding.UTF8);
                JObject state = JObject.Parse(raw);
                string? lastCheckStr = state["last_check"]?.Value<string>();
                if (string.IsNullOrEmpty(lastCheckStr))
                    return true;

                DateTime lastCheck = DateTime.Parse(lastCheckStr, null,
                    System.Globalization.DateTimeStyles.RoundtripKind);

                return (DateTime.UtcNow - lastCheck).TotalHours >= ThrottleHours;
            }
            catch
            {
                // safe-swallow: state file corrupt → allow check
                return true;
            }
        }

        private void PersistLastCheck()
        {
            try
            {
                JObject state = new JObject(
                    new JProperty("last_check", DateTime.UtcNow.ToString("O"))
                );
                // Use the static JsonConvert.SerializeObject (present in EVERY Newtonsoft
                // build) instead of the JToken.ToString(Formatting) instance overload, which
                // is absent from Unity's stripped Newtonsoft.Json 13.0.2 (Managed/) and throws
                // MethodNotFound at runtime in the BepInEx/Mono context. (iter-149 hotfix)
                File.WriteAllText(_stateFilePath, JsonConvert.SerializeObject(state, Formatting.None),
                    System.Text.Encoding.UTF8);
            }
            catch
            {
                // safe-swallow: best-effort only
            }
        }
    }

    /// <summary>
    /// Identifies a single pack to check for updates.
    /// </summary>
    internal sealed class PackUpdateTarget
    {
        /// <summary>Pack id used as the component key in throttle state.</summary>
        public string ComponentId { get; }

        /// <summary>Human-readable pack name for the UI.</summary>
        public string DisplayName { get; }

        /// <summary>GitHub repo owner (from pack.yaml update_check.owner).</summary>
        public string RepoOwner { get; }

        /// <summary>GitHub repo name (from pack.yaml update_check.repo).</summary>
        public string RepoName { get; }

        /// <summary>Currently installed version of this pack.</summary>
        public string CurrentVersion { get; }

        /// <summary>Initializes a new <see cref="PackUpdateTarget"/>.</summary>
        public PackUpdateTarget(
            string componentId,
            string displayName,
            string repoOwner,
            string repoName,
            string currentVersion)
        {
            ComponentId = componentId;
            DisplayName = displayName;
            RepoOwner = repoOwner;
            RepoName = repoName;
            CurrentVersion = currentVersion;
        }
    }
}
