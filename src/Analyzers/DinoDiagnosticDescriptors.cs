using Microsoft.CodeAnalysis;

namespace DINOForge.Analyzers
{
    /// <summary>
    /// Shared factory for DINOForge analyzer <see cref="DiagnosticDescriptor"/> instances.
    /// Centralizes title/message/description scaffolding to reduce duplication (Sonar CPD).
    /// </summary>
    internal static class DinoDiagnosticDescriptors
    {
        public static DiagnosticDescriptor Create(
            string id,
            string category,
            DiagnosticSeverity severity,
            string title,
            string messageFormat,
            string description,
            string? helpLinkUri = null,
            bool isEnabledByDefault = true)
        {
            return new DiagnosticDescriptor(
                id,
                (LocalizableString)title,
                (LocalizableString)messageFormat,
                category,
                severity,
                isEnabledByDefault,
                description: (LocalizableString)description,
                helpLinkUri: helpLinkUri);
        }
    }
}
