namespace DINOForge.SDK.Validation
{
    /// <summary>
    /// Marker interface for SDK model types that expose semantic-level
    /// self-validation through a <see cref="ValidationResult"/>.
    ///
    /// Implementing types perform required-field, range, and cross-field
    /// checks that go beyond what JSON Schema can express. Deserialize
    /// sites (PackLoader, RegistryImportService) call <see cref="Validate"/>
    /// after parsing to surface invariants as <c>InvalidDataException</c>.
    /// </summary>
    /// <remarks>
    /// Wired in task #211 (Pattern #75) — moves dead model Validate() methods
    /// onto the actual pack-load codepath.
    /// </remarks>
    public interface IValidatable
    {
        /// <summary>
        /// Performs semantic validation of the instance.
        /// Returns a <see cref="ValidationResult"/> with violations or
        /// <see cref="ValidationResult.Success"/> when valid.
        /// </summary>
        ValidationResult Validate();
    }
}
