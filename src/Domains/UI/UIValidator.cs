using System;
using System.Collections.Generic;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.UI
{
    /// <summary>
    /// Consolidated validator for a UI pack — aggregates Validate() calls across
    /// <see cref="HudElementDefinition"/>, <see cref="MenuDefinition"/> and
    /// <see cref="ThemeDefinition"/> registries plus cross-cutting checks
    /// (menu hierarchy, theme references). Mirrors the shape of the Economy domain's
    /// validation result (see <c>EconomyValidationResult</c>) so callers can treat
    /// UI/Economy/Scenario validation uniformly.
    /// </summary>
    /// <remarks>
    /// Task #844 — previously, IValidatable invocations were scattered across
    /// <see cref="UIContentLoader"/> (per-deserialize JsonGuard.ValidateOrThrow calls).
    /// This type provides a single <see cref="ValidatePack"/> entry point so packs
    /// can be validated post-load without re-deserializing.
    /// </remarks>
    public sealed class UIValidator
    {
        private readonly HudElementRegistry _hudElementRegistry;
        private readonly MenuRegistry _menuRegistry;
        private readonly ThemeRegistry _themeRegistry;

        /// <summary>
        /// Creates a new UI validator bound to the three UI registries.
        /// </summary>
        public UIValidator(
            HudElementRegistry hudElementRegistry,
            MenuRegistry menuRegistry,
            ThemeRegistry themeRegistry)
        {
            _hudElementRegistry = hudElementRegistry ?? throw new ArgumentNullException(nameof(hudElementRegistry));
            _menuRegistry = menuRegistry ?? throw new ArgumentNullException(nameof(menuRegistry));
            _themeRegistry = themeRegistry ?? throw new ArgumentNullException(nameof(themeRegistry));
        }

        /// <summary>
        /// Validate the entire UI pack content held by the registries this
        /// validator was bound to. Aggregates per-definition <see cref="IValidatable.Validate"/>
        /// results and cross-cutting structural checks (menu hierarchy cycles).
        /// </summary>
        /// <param name="packId">Pack identifier for diagnostic context.</param>
        /// <returns>A <see cref="UIValidationResult"/> capturing errors and warnings.</returns>
        public UIValidationResult ValidatePack(string packId)
        {
            if (packId == null) throw new ArgumentNullException(nameof(packId));

            List<string> errors = new List<string>();
            List<string> warnings = new List<string>();

            foreach (HudElementDefinition element in _hudElementRegistry.All)
                AppendErrors(errors, "hud_element", element.Id, element.Validate());

            foreach (MenuDefinition menu in _menuRegistry.All)
                AppendErrors(errors, "menu", menu.Id, menu.Validate());

            foreach (ThemeDefinition theme in _themeRegistry.All)
                AppendErrors(errors, "theme", theme.Id, theme.Validate());

            // Cross-cutting: menu hierarchy cycle / orphan parent detection.
            foreach (string hierarchyError in _menuRegistry.ValidateHierarchy())
                errors.Add($"[menu_hierarchy] {hierarchyError}");

            // TODO(#844-followup): cross-cutting theme reference checks
            // (HUD elements referencing unregistered theme ids → warnings).

            return new UIValidationResult(
                packId,
                isValid: errors.Count == 0,
                errors: errors.AsReadOnly(),
                warnings: warnings.AsReadOnly());
        }

        private static void AppendErrors(List<string> sink, string kind, string id, ValidationResult result)
        {
            if (result.IsValid) return;
            foreach (ValidationError err in result.Errors)
                sink.Add($"[{kind}:{id}] {err.Path}: {err.Message}");
        }
    }

    /// <summary>
    /// Result of validating a complete UI pack (hud elements, menus, themes).
    /// Shape mirrors <c>EconomyValidationResult</c> for cross-domain uniformity.
    /// </summary>
    public sealed class UIValidationResult
    {
        /// <summary>The pack identifier that was validated.</summary>
        public string PackId { get; }

        /// <summary>True if all validations passed with no errors.</summary>
        public bool IsValid { get; }

        /// <summary>All validation errors found. Errors indicate problems that must be fixed.</summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>Non-blocking warnings (e.g. unreferenced themes, suspicious z-orders).</summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>Creates a new UI validation result.</summary>
        public UIValidationResult(
            string packId,
            bool isValid,
            IReadOnlyList<string> errors,
            IReadOnlyList<string> warnings)
        {
            PackId = packId;
            IsValid = isValid;
            Errors = errors;
            Warnings = warnings;
        }
    }
}
