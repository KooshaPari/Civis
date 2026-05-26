using System;
using System.Collections.Generic;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Result of validating a mod's asset bundle for compatibility.
    /// </summary>
    public sealed class AssetValidationResult
    {
        /// <summary>Whether the bundle passed all validation checks.</summary>
        public bool IsValid { get; }

        /// <summary>Unity version the bundle was built with.</summary>
        public string UnityVersion { get; }

        /// <summary>Validation errors found, if any.</summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>Assets found in the bundle.</summary>
        public IReadOnlyList<AssetInfo> Assets { get; }

        /// <summary>
        /// Initializes a new <see cref="AssetValidationResult"/> instance.
        /// </summary>
        public AssetValidationResult(bool isValid, string unityVersion, IReadOnlyList<string> errors, IReadOnlyList<AssetInfo> assets)
        {
            IsValid = isValid;
            UnityVersion = unityVersion ?? throw new ArgumentNullException(nameof(unityVersion));
            Errors = errors ?? throw new ArgumentNullException(nameof(errors));
            Assets = assets ?? throw new ArgumentNullException(nameof(assets));
        }

        /// <summary>
        /// Creates a failed validation result with the specified errors.
        /// </summary>
        public static AssetValidationResult Failure(IReadOnlyList<string> errors)
        {
            return new AssetValidationResult(
                isValid: false,
                unityVersion: "unknown",
                errors: errors,
                assets: Array.Empty<AssetInfo>());
        }
    }
}
