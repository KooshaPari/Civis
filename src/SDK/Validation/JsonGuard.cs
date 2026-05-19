using System;
using System.IO;
using System.Linq;

namespace DINOForge.SDK.Validation
{
    /// <summary>
    /// Shared helpers for guarding deserialized SDK content with semantic validation.
    /// Centralizes the <see cref="IValidatable"/> dispatch pattern used by PackLoader,
    /// RegistryImportService, UIContentLoader, ScenarioContentLoader, and UniverseLoader
    /// (task #210 / Pattern #75).
    /// </summary>
    public static class JsonGuard
    {
        /// <summary>
        /// If <paramref name="item"/> implements <see cref="IValidatable"/>, run its
        /// semantic validation and throw <see cref="InvalidDataException"/> on failure.
        /// Otherwise no-op. Use at deserialize sites to surface invariants that JSON
        /// Schema cannot express (cross-field rules, enum constraints, ranges).
        /// </summary>
        /// <typeparam name="T">Reference type of the deserialized item.</typeparam>
        /// <param name="item">Deserialized SDK content item (may be non-validatable).</param>
        /// <param name="source">
        /// Optional source descriptor (file path, label) included in the error message
        /// to aid pack-author diagnosis.
        /// </param>
        /// <exception cref="InvalidDataException">
        /// Thrown when <paramref name="item"/> implements <see cref="IValidatable"/> and
        /// reports one or more <see cref="ValidationError"/>s.
        /// </exception>
        public static void ValidateOrThrow<T>(T item, string? source = null) where T : class
        {
            if (item is IValidatable validatable)
            {
                ValidationResult result = validatable.Validate();
                if (!result.IsValid)
                {
                    string details = string.Join(
                        "; ",
                        result.Errors.Select(e => $"{e.Path}: {e.Message}"));
                    string prefix = source != null
                        ? $"{typeof(T).Name} at '{source}'"
                        : typeof(T).Name;
                    throw new InvalidDataException(
                        $"{prefix} failed semantic validation: {details}");
                }
            }
        }

        /// <summary>
        /// Best-effort variant: returns <c>true</c> when validation passes (or item is
        /// not <see cref="IValidatable"/>), <c>false</c> when it fails. Suitable for
        /// loaders that prefer to skip+log rather than abort the entire pack.
        /// </summary>
        /// <typeparam name="T">Reference type of the deserialized item.</typeparam>
        /// <param name="item">Deserialized SDK content item (may be non-validatable).</param>
        /// <param name="errorSummary">
        /// When the method returns <c>false</c>, contains the joined error summary
        /// suitable for logging. <c>null</c> when the method returns <c>true</c>.
        /// </param>
        public static bool TryValidate<T>(T item, out string? errorSummary) where T : class
        {
            errorSummary = null;
            if (item is IValidatable validatable)
            {
                ValidationResult result = validatable.Validate();
                if (!result.IsValid)
                {
                    errorSummary = string.Join(
                        "; ",
                        result.Errors.Select(e => $"{e.Path}: {e.Message}"));
                    return false;
                }
            }
            return true;
        }
    }
}
