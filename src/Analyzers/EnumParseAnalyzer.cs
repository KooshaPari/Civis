using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class EnumParseAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1009";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Enum.Parse without TryParse fallback";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`Enum.Parse` throws on unknown input. Use `Enum.TryParse(...)` for user-sourced strings (YAML enums, JSON discriminators).";

        private static readonly LocalizableString Description =
            (LocalizableString)"`Enum.Parse<TEnum>(string)` throws ArgumentException if the input string does not match any enum value. When parsing user-sourced data (YAML enum fields, JSON discriminators, pack content), this exception leaks internal type information. Use `Enum.TryParse<TEnum>(string, out var result)` with explicit error handling and fallback logic.";

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
            context.RegisterSyntaxNodeAction(AnalyzeInvocation, SyntaxKind.InvocationExpression);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            // Skip if leading-trivia contains enum-parse-ok marker
            if (HasEnumParseOkComment(invocation))
                return;

            // Check if this is an Enum.Parse call (generic or non-generic)
            if (!IsEnumParseCall(invocation, context.SemanticModel))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsEnumParseCall(InvocationExpressionSyntax invocation, SemanticModel semanticModel)
        {
            // Extract the method name from the invocation
            var methodName = GetMethodName(invocation);
            if (methodName != "Parse")
                return false;

            // Get symbol information to confirm it's System.Enum.Parse
            var symbolInfo = semanticModel.GetSymbolInfo(invocation);
            var method = symbolInfo.Symbol as IMethodSymbol;
            if (method == null)
                return false;

            // Check if this is a method in System.Enum
            var containingType = method.ContainingType;
            if (containingType == null)
                return false;

            var fullName = containingType.ToDisplayString();
            return fullName == "System.Enum" && method.Name == "Parse";
        }

        private static string? GetMethodName(InvocationExpressionSyntax invocation)
        {
            var expr = invocation.Expression;

            // Handle generic case: Enum.Parse<T>(...)
            if (expr is GenericNameSyntax genericName)
                return genericName.Identifier.Text;

            // Handle non-generic case: Enum.Parse(...)
            if (expr is IdentifierNameSyntax identifierName)
                return identifierName.Identifier.Text;

            // Handle member access: something.Parse(...) — get the Name part
            if (expr is MemberAccessExpressionSyntax memberAccess)
                return memberAccess.Name.Identifier.Text;

            return null;
        }

        private static bool HasEnumParseOkComment(InvocationExpressionSyntax expr)
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
                if (commentText.Contains("enum-parse-ok:"))
                    return true;
            }
            return false;
        }
    }
}
