using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class DictionaryIndexerAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1008";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Dictionary[key] without TryGetValue/ContainsKey guard";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`{0}[{1}]` throws KeyNotFoundException on miss. For user-sourced keys use `TryGetValue` and handle missing key explicitly.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Direct dictionary indexing with `dict[key]` throws KeyNotFoundException if the key is missing. When the key comes from user input, pack IDs, JSON properties, or other untrusted sources, this exception leaks an internal type. Use `dict.TryGetValue(key, out var value)` with explicit missing-key handling, or guard with `dict.ContainsKey(key)` first.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Info,
            isEnabledByDefault: true,
            description: Description,
            helpLinkUri: null);

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeElementAccess, SyntaxKind.ElementAccessExpression);
        }

        private static void AnalyzeElementAccess(SyntaxNodeAnalysisContext context)
        {
            var elementAccess = (ElementAccessExpressionSyntax)context.Node;

            // Skip if leading-trivia contains dict-indexer-ok marker
            if (HasDictIndexerOkComment(elementAccess))
                return;

            // Get semantic model for type checking
            var semanticModel = context.SemanticModel;

            // Get the type of the accessed expression
            var typeInfo = semanticModel.GetTypeInfo(elementAccess.Expression);
            var type = typeInfo.Type;

            // Skip if not a dictionary-like type
            if (!IsDictionaryType(type))
                return;

            // Check if inside a ContainsKey guard (heuristic)
            if (IsInsideContainsKeyGuard(elementAccess, semanticModel))
                return;

            // Report diagnostic
            var dictDisplay = type?.Name ?? "Dictionary";
            var keyDisplay = elementAccess.ArgumentList?.Arguments.FirstOrDefault()?.ToString() ?? "key";
            var diagnostic = Diagnostic.Create(
                Rule,
                elementAccess.GetLocation(),
                dictDisplay,
                keyDisplay);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsDictionaryType(ITypeSymbol? type)
        {
            if (type == null)
                return false;

            // Check for Dictionary<,>, ConcurrentDictionary<,>, IDictionary<,>
            var fullName = type.ToDisplayString();
            return fullName.StartsWith("System.Collections.Generic.Dictionary<") ||
                   fullName.StartsWith("System.Collections.Concurrent.ConcurrentDictionary<") ||
                   fullName.StartsWith("System.Collections.Generic.IDictionary<");
        }

        private static bool IsInsideContainsKeyGuard(ElementAccessExpressionSyntax elementAccess, SemanticModel semanticModel)
        {
            // Walk up to find parent IfStatement
            var parent = elementAccess.Parent;
            while (parent != null)
            {
                if (parent is IfStatementSyntax ifStmt)
                {
                    // Check if condition contains ContainsKey call with matching dictionary/key
                    if (IsContainsKeyCheck(ifStmt.Condition, elementAccess))
                        return true;
                }

                parent = parent.Parent;
            }

            return false;
        }

        private static bool IsContainsKeyCheck(ExpressionSyntax condition, ElementAccessExpressionSyntax elementAccess)
        {
            // Simple heuristic: look for ContainsKey invocation in condition
            var invocation = condition as InvocationExpressionSyntax;
            if (invocation == null)
                return false;

            var methodName = (invocation.Expression as MemberAccessExpressionSyntax)?.Name.Identifier.Text;
            return methodName == "ContainsKey";
        }

        private static bool HasDictIndexerOkComment(ElementAccessExpressionSyntax expr)
        {
            var leadingTrivia = expr.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            return false;
        }

        private static bool CheckTrivia(SyntaxTrivia trivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("dict-indexer-ok:"))
                    return true;
            }
            return false;
        }
    }
}
