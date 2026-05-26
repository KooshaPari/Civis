using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #706 + #719: detect PredefinedTypeSyntax (e.g. `string`) in addition to IdentifierName,
    // and scan ALL constructor args for StringComparer (not just position 0).
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class UnprotectedStringDictAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0099";
        private const string Category = "Performance";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Dictionary<string, T> without explicit StringComparer",
            "Constructor missing StringComparer. For user-sourced keys use `StringComparer.Ordinal`. For UI-facing case-insensitive use `StringComparer.OrdinalIgnoreCase`.",
            "Dictionary<string, T> and ConcurrentDictionary<string, T> default to culture-sensitive comparison (StringComparer.CurrentCulture equivalent via default comparer). User-sourced keys (pack IDs, asset names, faction names) must use StringComparer.Ordinal to ensure deterministic key lookups across machines and cultures. UI-facing keys (menu names, HUD labels) may use OrdinalIgnoreCase for case-insensitive matching, but must be explicit.");

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

            // #719 fix: scan ALL arguments for StringComparer (was Arguments[0] only)
            // Check ALL arguments: Dictionary<string,T> constructors accept the comparer at
            // varying positions depending on overload (1-arg comparer; 2-arg (capacity, comparer);
            // 2-arg (sourceDict, comparer); 3-arg (capacity, comparer) etc.). The comparer is
            // recognized if any argument is `StringComparer.<member>` or textually contains the
            // StringComparer/Ordinal/IgnoreCase tokens.
            foreach (var arg in argumentList.Arguments)
            {
                if (arg.Expression is MemberAccessExpressionSyntax memberAccess &&
                    memberAccess.Expression is IdentifierNameSyntax id &&
                    id.Identifier.ValueText == "StringComparer")
                {
                    return;
                }

                var argText = arg.ToString();
                if (argText.Contains("StringComparer", StringComparison.Ordinal) ||
                    argText.Contains("Ordinal", StringComparison.Ordinal) ||
                    argText.Contains("IgnoreCase", StringComparison.Ordinal))
                {
                    return;
                }
            }

            // No argument matched a StringComparer — flag it
            var noComparerDiagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
            context.ReportDiagnostic(noComparerDiagnostic);
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
                    // #706 fix: also recognize PredefinedTypeSyntax 'string' keyword (was IdentifierNameSyntax only)
                    if (firstTypeArg is PredefinedTypeSyntax predefinedArg &&
                        predefinedArg.Keyword.IsKind(SyntaxKind.StringKeyword))
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
                        if (firstTypeArg is PredefinedTypeSyntax predefinedArg &&
                            predefinedArg.Keyword.IsKind(SyntaxKind.StringKeyword))
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
