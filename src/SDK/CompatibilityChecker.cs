#nullable enable
using System;
using System.Collections.Generic;
using System.Reflection;

namespace DINOForge.SDK
{
    /// <summary>
    /// Provides version compatibility checking for DINOForge packs against
    /// DINOForge framework, game, BepInEx, and Unity versions.
    ///
    /// Version ranges use simple semver-like syntax:
    /// - wildcard = any version (wildcard)
    /// - exact version number
    /// - greater than or equal syntax
    /// - less than or equal syntax
    /// - strictly greater
    /// - strictly less
    /// - equal (alternate syntax)
    ///
    /// Multiple constraints can be combined with spaces.
    /// </summary>
    public static class CompatibilityChecker
    {
        // unbounded-version-ok: intentional runtime skip sentinel when version detection is unavailable
        private const string WildcardVersionConstraint = "*";

        /// <summary>
        /// Gets the current DINOForge framework version from the SDK assembly.
        /// </summary>
        public static Version FrameworkVersion
        {
            get
            {
                var asm = typeof(CompatibilityChecker).Assembly;

                // MinVer sets AssemblyVersion to {major}.0.0.0 by default, which collapses
                // to 0.0.0.0 when major=0.  Fall back to FileVersion (e.g. "0.5.1.0") which
                // MinVer always sets correctly, then to InformationalVersion as last resort.
                var assemblyVersion = asm.GetName().Version;
                if (assemblyVersion != null && assemblyVersion.Major + assemblyVersion.Minor > 0)
                    return assemblyVersion;

                // Try FileVersionAttribute (MinVer sets this to {major}.{minor}.{patch}.0)
                var fileVerAttr = asm.GetCustomAttributes(typeof(AssemblyFileVersionAttribute), false);
                if (fileVerAttr.Length > 0 &&
                    fileVerAttr[0] is AssemblyFileVersionAttribute fva &&
                    Version.TryParse(fva.Version, out var fileVer) &&
                    fileVer.Major + fileVer.Minor > 0)
                {
                    return fileVer;
                }

                // Fallback: known framework version constant
                return new Version(0, 5, 0);
            }
        }

        /// <summary>
        /// Resolves the current BepInEx version by reflecting on any loaded
        /// BepInEx assembly in the AppDomain. Returns "*" (skip check) when
        /// BepInEx is not loaded (e.g. CLI tools, unit tests).
        /// </summary>
        public static string CurrentBepInExVersion
        {
            get
            {
                try
                {
                    foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
                    {
                        var name = asm.GetName().Name;
                        if (string.Equals(name, "BepInEx", StringComparison.OrdinalIgnoreCase) ||
                            string.Equals(name, "BepInEx.Core", StringComparison.OrdinalIgnoreCase))
                        {
                            var v = asm.GetName().Version;
                            if (v != null && v.Major > 0)
                                return $"{v.Major}.{v.Minor}.{v.Build}";
                        }
                    }
                }
                catch
                {
                    // safe-swallow: best-effort version probe
                }
                return WildcardVersionConstraint;
            }
        }

        /// <summary>
        /// Resolves the current Unity engine version by reflecting on the
        /// UnityEngine.CoreModule assembly. Returns "*" when not loaded.
        /// </summary>
        public static string CurrentUnityVersion
        {
            get
            {
                try
                {
                    foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
                    {
                        var name = asm.GetName().Name;
                        if (string.Equals(name, "UnityEngine.CoreModule", StringComparison.OrdinalIgnoreCase) ||
                            string.Equals(name, "UnityEngine", StringComparison.OrdinalIgnoreCase))
                        {
                            // Unity puts the editor version in InformationalVersion (e.g. "2021.3.45f2")
                            var info = (AssemblyInformationalVersionAttribute?)Attribute.GetCustomAttribute(
                                asm, typeof(AssemblyInformationalVersionAttribute));
                            if (info != null && !string.IsNullOrWhiteSpace(info.InformationalVersion))
                                return info.InformationalVersion;
                            var v = asm.GetName().Version;
                            if (v != null && v.Major > 0)
                                return v.ToString();
                        }
                    }
                }
                catch
                {
                    // safe-swallow: best-effort version probe
                }
                return WildcardVersionConstraint;
            }
        }

        /// <summary>
        /// Checks if a pack manifest is compatible with the specified versions.
        /// Pass null for bepinexVersion/unityVersion to auto-detect from loaded assemblies.
        /// </summary>
        /// <param name="manifest">The pack manifest to validate.</param>
        /// <param name="dinoGameVersion">Current DINO game version, e.g. "1.0.0".</param>
        /// <param name="bepinexVersion">Current BepInEx version, or null to auto-detect.</param>
        /// <param name="unityVersion">Current Unity version, or null to auto-detect.</param>
        /// <returns>A CompatibilityResult with compatibility status and any warnings/errors.</returns>
        public static CompatibilityResult CheckPack(
            PackManifest manifest,
            string dinoGameVersion = WildcardVersionConstraint,
            string? bepinexVersion = null,
            string? unityVersion = null)
        {
            bepinexVersion ??= CurrentBepInExVersion;
            unityVersion ??= CurrentUnityVersion;
            var result = new CompatibilityResult();

            // Check framework version
            if (!IsVersionInRange(FrameworkVersion.ToString(), manifest.FrameworkVersion))
            {
                result.Errors.Add(
                    $"Pack requires DINOForge {manifest.FrameworkVersion}, but {FrameworkVersion} is installed.");
            }

            // Check game version
            if (!IsVersionInRange(dinoGameVersion, manifest.GameVersion))
            {
                result.Warnings.Add(
                    $"Pack specifies game_version '{manifest.GameVersion}', but current game is {dinoGameVersion}. " +
                    "This may cause compatibility issues.");
            }

            // Check BepInEx version
            if (!IsVersionInRange(bepinexVersion, manifest.BepInExVersion))
            {
                result.Warnings.Add(
                    $"Pack specifies bepinex_version '{manifest.BepInExVersion}', but current BepInEx is {bepinexVersion}. " +
                    "This may cause compatibility issues.");
            }

            // Check Unity version
            if (!IsVersionInRange(unityVersion, manifest.UnityVersion))
            {
                result.Warnings.Add(
                    $"Pack specifies unity_version '{manifest.UnityVersion}', but current Unity is {unityVersion}. " +
                    "This may cause compatibility issues.");
            }

            // Pack is compatible if there are no errors (warnings are allowed)
            result.IsCompatible = result.Errors.Count == 0;

            return result;
        }

        /// <summary>
        /// Determines if a version satisfies a version range constraint.
        /// </summary>
        /// <param name="version">The version to check, e.g. "1.5.0".</param>
        /// <param name="range">The range constraint, e.g. multiple constraints separated by spaces.</param>
        /// <returns>True if version satisfies the range; false otherwise.</returns>
        public static bool IsVersionInRange(string version, string range)
        {
            // Wildcard always matches
            if (range == WildcardVersionConstraint || string.IsNullOrWhiteSpace(range))
            {
                return true;
            }

            // Split on spaces to get individual constraints
            var constraints = range.Split(' ');

            // Each constraint must be satisfied
            foreach (var constraint in constraints)
            {
                if (!IsSingleConstraintSatisfied(version, constraint))
                {
                    return false;
                }
            }

            return true;
        }

        /// <summary>
        /// Checks if a version satisfies a single constraint like "&gt;=1.0.0" or "&lt;2.0.0".
        /// </summary>
        private static bool IsSingleConstraintSatisfied(string version, string constraint)
        {
            if (string.IsNullOrWhiteSpace(constraint))
            {
                return true;
            }

            // Extract operator and version number
            var (op, constraintVersion) = ExtractOperatorAndVersion(constraint);

            // Handle wildcard versions like "2021.3.*"
            if (constraintVersion.Contains(WildcardVersionConstraint))
            {
                // Wildcard version comparison: match prefix
                var prefix = constraintVersion.Replace(WildcardVersionConstraint, "");
                return version.StartsWith(prefix, StringComparison.OrdinalIgnoreCase);
            }

            // Try to parse versions, handle formats like "2021.3.45f2" (Unity)
            if (!TryParseVersion(version, out var v) || !TryParseVersion(constraintVersion, out var cv))
            {
                // If we can't parse, fall back to string comparison (conservative)
                return version.Equals(constraintVersion, StringComparison.OrdinalIgnoreCase);
            }

            switch (op)
            {
                case ">=":
                    return v >= cv;
                case "<=":
                    return v <= cv;
                case ">":
                    return v > cv;
                case "<":
                    return v < cv;
                case "=":
                case "==":
                    return v == cv;
                case "~":
                    return IsCaretCompatible(v, cv);
                case "^":
                    return IsTildeCompatible(v, cv);
                default:
                    return false;
            }
        }

        /// <summary>
        /// Extracts the operator and version from a constraint.
        /// </summary>
        private static (string op, string version) ExtractOperatorAndVersion(string constraint)
        {
            if (constraint.StartsWith(">="))
                return (">=", constraint.Substring(2));
            if (constraint.StartsWith("<="))
                return ("<=", constraint.Substring(2));
            if (constraint.StartsWith("=="))
                return ("==", constraint.Substring(2));
            if (constraint.StartsWith("~"))
                return ("~", constraint.Substring(1));
            if (constraint.StartsWith("^"))
                return ("^", constraint.Substring(1));
            if (constraint.StartsWith(">"))
                return (">", constraint.Substring(1));
            if (constraint.StartsWith("<"))
                return ("<", constraint.Substring(1));
            if (constraint.StartsWith("="))
                return ("=", constraint.Substring(1));

            // No operator, assume exact match
            return ("=", constraint);
        }

        /// <summary>
        /// Tries to parse a version string into a Version object.
        /// Handles standard semver and Unity-style versions.
        /// </summary>
        private static bool TryParseVersion(string versionStr, out Version version)
        {
            version = new Version(0, 0, 0);

            if (string.IsNullOrWhiteSpace(versionStr))
            {
                return false;
            }

            // Strip semver pre-release tag (e.g. "-preview.0.5", "-alpha.1") before parsing
            var dashIdx = versionStr.IndexOf('-');
            if (dashIdx >= 0)
                versionStr = versionStr.Substring(0, dashIdx);

            // Remove common suffixes like 'f2', 'rc1', etc. for parsing
            var cleaned = System.Text.RegularExpressions.Regex.Replace(versionStr, "[a-zA-Z]+\\d*$", "");

            if (Version.TryParse(cleaned, out var parsed))
            {
                version = parsed;
                return true;
            }

            return false;
        }

        /// <summary>
        /// Caret compatibility: allows minor and patch changes only (^1.2.3 = >=1.2.3 &lt;2.0.0).
        /// </summary>
        private static bool IsCaretCompatible(Version actual, Version specified)
        {
            if (actual < specified)
                return false;

            // Allow patch changes, but not minor/major
            return actual.Major == specified.Major &&
                   actual.Minor == specified.Minor;
        }

        /// <summary>
        /// Tilde compatibility: allows patch changes only (~1.2.3 = >=1.2.3 &lt;1.3.0).
        /// </summary>
        private static bool IsTildeCompatible(Version actual, Version specified)
        {
            if (actual < specified)
                return false;

            // Allow minor/patch changes, but not major
            return actual.Major == specified.Major;
        }
    }

    /// <summary>
    /// Encapsulates the result of a compatibility check.
    /// </summary>
    public sealed class CompatibilityResult
    {
        /// <summary>
        /// True if the pack is compatible (no errors, warnings allowed).
        /// </summary>
        public bool IsCompatible { get; set; } = true;

        /// <summary>
        /// List of warning messages (compatibility issues that don't block loading).
        /// </summary>
        public List<string> Warnings { get; set; } = new List<string>(); // public-mutable-ok: result accumulator, callers append during validation

        /// <summary>
        /// List of error messages (compatibility issues that block loading).
        /// </summary>
        public List<string> Errors { get; set; } = new List<string>(); // public-mutable-ok: result accumulator, callers append during validation
    }
}
