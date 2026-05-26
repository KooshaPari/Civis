#nullable enable
using System;
using System.IO;
using System.Linq;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Path-containment helper for user-authored YAML paths (e.g. <c>asset.File</c>,
    /// <c>UpdateDefinition.File</c>) that get joined onto a trusted pack root.
    ///
    /// Mirrors the #166 pattern (AssetctlPipeline + InstallLifecycle): combine,
    /// <see cref="Path.GetFullPath(string)"/>, then verify the resolved path stays
    /// inside the trusted root. Returns null on traversal attempts so callers can
    /// emit a validation error or skip the asset.
    /// </summary>
    public static class SafePathResolver
    {
        /// <summary>
        /// Resolve a user-supplied relative path against a trusted root. Returns
        /// <c>null</c> if the resolved path would escape the root (e.g. <c>..</c>
        /// traversal or an absolute drive-letter path).
        /// </summary>
        /// <param name="trustedRoot">Trusted root directory (e.g. <c>packPath</c>).
        /// Must be non-empty.</param>
        /// <param name="segments">User-authored path segments to combine onto the
        /// root. Each segment may itself contain separators.</param>
        /// <returns>Absolute resolved path inside <paramref name="trustedRoot"/>,
        /// or <c>null</c> if the resolved path escapes the root or contains
        /// invalid characters.</returns>
        public static string? TryResolveSafePath(string trustedRoot, params string[] segments)
        {
            if (string.IsNullOrWhiteSpace(trustedRoot))
            {
                throw new ArgumentException("trustedRoot must be non-empty", nameof(trustedRoot));
            }

            string rootFull = Path.GetFullPath(trustedRoot);
            if (!rootFull.EndsWith(Path.DirectorySeparatorChar.ToString(), StringComparison.Ordinal))
            {
                rootFull += Path.DirectorySeparatorChar;
            }

            try
            {
                string[] all = new string[segments.Length + 1];
                all[0] = trustedRoot;
                for (int i = 0; i < segments.Length; i++)
                {
                    all[i + 1] = segments[i] ?? string.Empty;
                }

                string combined = Path.Combine(all);
                string resolved = Path.GetFullPath(combined);

                return resolved.StartsWith(rootFull, StringComparison.OrdinalIgnoreCase)
                    ? resolved
                    : null;
            }
            catch (ArgumentException)
            {
                // Invalid path characters
                return null;
            }
            catch (NotSupportedException)
            {
                // Path contains a colon at an invalid position, etc.
                return null;
            }
        }
    }
}
