using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class UnprotectedStringDictAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0099";
        private const string Category = "Performance";

        private static readonly LocalizableString Title =
            (LocalizableString)"Dictionary<string, T> without explicit StringComparer";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Constructor missing StringComparer. For user-sourced keys use `StringComparer.Ordinal`. For UI-facing case-insensitive use `StringComparer.OrdinalIgnoreCase`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Dictionary<string, T> and ConcurrentDictionary<string, T> default to culture-sensitive comparison (StringComparer.CurrentCulture equivalent via default comparer). User-sourced keys (pack IDs, asset names, faction names) must use StringComparer.Ordinal to ensure deterministic key lookups across machines and cultures. UI-facing keys (menu names, HUD labels) may use OrdinalIgnoreCase for case-insensitive matching, but must be explicit.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Warning,
            isEnabledByDefault: true,
            description: Description,
            helpLinkUri: null);

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeObjectCreation, SyntaxKind.ObjectCreationExpression);
        }

        private static void AnalyzeObjectCreation(SyntaxNodeAnalysisContext context)
        {
            var objectCreation = (ObjectCreationExpressionSyntax)context.Node;

            // Match: new Dictionary<string, T>() or new ConcurrentDictionary<string, T>()
            if (!IsStringDictionaryCreation(objectCreation))
                return;

            // Check if first argument is a StringComparer
            var argumentList = objectCreation.ArgumentList;
            if (argumentList == null || argumentList.Arguments.Count == 0)
            {
                // No arguments — flag it
                var diagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
                context.ReportDiagnostic(diagnostic);
                return;
            }

            // Check if first argument looks like a StringComparer (heuristic: contains "StringComparer" or "Ordinal")
            var firstArg = argumentList.Arguments[0];
            var argText = firstArg.ToString();
            if (!argText.Contains("StringComparer", StringComparison.Ordinal) &&
                !argText.Contains("Ordinal", StringComparison.Ordinal) &&
                !argText.Contains("IgnoreCase", StringComparison.Ordinal))
            {
                // First argument doesn't look like a StringComparer — flag it
                var diagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool IsStringDictionaryCreation(ObjectCreationExpressionSyntax objectCreation)
        {
            var typeName = objectCreation.Type;

            // Handle generic types like Dictionary<string, T> or ConcurrentDictionary<string, T>
            if (typeName is GenericNameSyntax genericName)
            {
                var typeArgumentList = genericName.TypeArgumentList;
                if (typeArgumentList?.Arguments.Count >= 1)
                {
                    var firstTypeArg = typeArgumentList.Arguments[0];
                    // Check if first type argument is "string"
                    if (firstTypeArg is IdentifierNameSyntax identifierArg &&
                        identifierArg.Identifier.ValueText == "string")
                    {
                        var baseName = genericName.Identifier.ValueText;
                        return baseName == "Dictionary" || baseName == "ConcurrentDictionary";
                    }
                }
            }

            // Handle qualified names like System.Collections.Generic.Dictionary<string, T>
            if (typeName is QualifiedNameSyntax qualified)
            {
                if (qualified.Right is GenericNameSyntax rightGeneric)
                {
                    var typeArgumentList = rightGeneric.TypeArgumentList;
                    if (typeArgumentList?.Arguments.Count >= 1)
                    {
                        var firstTypeArg = typeArgumentList.Arguments[0];
                        if (firstTypeArg is IdentifierNameSyntax identifierArg &&
                            identifierArg.Identifier.ValueText == "string")
                        {
                            var baseName = rightGeneric.Identifier.ValueText;
                            return baseName == "Dictionary" || baseName == "ConcurrentDictionary";
                        }
                    }
                }
            }

            return false;
        }
    }
}
