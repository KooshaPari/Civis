using Octokit;
using System;
using System.Reflection;
using System.Threading.Tasks;

namespace DINOForge.Installer.Services;

/// <summary>
/// Holds the result of a GitHub release update check.
/// </summary>
public sealed record UpdateInfo(bool HasUpdate, string CurrentVersion, string LatestVersion, string ReleaseUrl);

/// <summary>
/// Checks GitHub releases for newer versions of DINOForge.
/// Uses Octokit to query the GitHub API.
/// </summary>
public sealed class UpdateChecker
{
    private const string Owner = "KooshaPari";
    private const string Repo = "Dino";
    private const string ProductHeader = "DINOForge-Installer";

    /// <summary>
    /// Checks GitHub for the latest DINOForge release and compares against the
    /// embedded assembly version.
    /// </summary>
    /// <returns>An <see cref="UpdateInfo"/> describing whether an update is available.</returns>
    public async Task<UpdateInfo> CheckAsync()
    {
        string currentVersion = GetCurrentVersion();

        try
        {
            GitHubClient client = new GitHubClient(new ProductHeaderValue(ProductHeader));
            Release latest = await client.Repository.Release.GetLatest(Owner, Repo).ConfigureAwait(false);

            string latestTag = latest.TagName?.TrimStart('v') ?? currentVersion;
            bool hasUpdate = IsNewer(latestTag, currentVersion);

            return new UpdateInfo(hasUpdate, currentVersion, latestTag, latest.HtmlUrl ?? string.Empty);
        }
        catch
        {
            // Network unavailable or rate-limited — treat as no update available.
            return new UpdateInfo(false, currentVersion, currentVersion, string.Empty);
        }
    }

    /// <summary>
    /// Gets the version string embedded in the assembly (e.g. "0.5.0").
    /// </summary>
    private static string GetCurrentVersion()
    {
        Version? v = Assembly.GetExecutingAssembly().GetName().Version;
        return v is null ? "0.0.0" : $"{v.Major}.{v.Minor}.{v.Build}";
    }

    /// <summary>
    /// Returns true if <paramref name="candidate"/> is strictly newer than <paramref name="current"/>
    /// using simple semantic version comparison.
    /// </summary>
    private static bool IsNewer(string candidate, string current)
    {
        if (Version.TryParse(candidate, out Version? c) && Version.TryParse(current, out Version? cur))
            return c > cur;
        return false;
    }
}
