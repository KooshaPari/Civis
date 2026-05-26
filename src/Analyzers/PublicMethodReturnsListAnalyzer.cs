using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class PublicMethodReturnsListAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1027";
        private const string Category = "Design";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Info,
            "Public method should return IReadOnlyList<T> instead of List<T>",
            "Method '{0}' returns mutable List<T> publicly. Prefer IReadOnlyList<T> or IEnumerable<T>.",
            "Exposing mutable List<T> from public methods allows callers to modify internal state. Prefer immutable return types like IReadOnlyList<T> or IEnumerable<T> to maintain encapsulation and prevent mutations. Use list-return-ok: marker to suppress.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeMethod, SyntaxKind.MethodDeclaration);
        }

        private static void AnalyzeMethod(SyntaxNodeAnalysisContext context)
        {
            var method = (MethodDeclarationSyntax)context.Node;

            // Skip if not public
            if (!method.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)))
                return;

            // Skip getter/setter accessor methods
            if (method.Parent is AccessorListSyntax)
                return;

            // Skip if marked with list-return-ok
            if (HasListReturnOkComment(method))
                return;

            // Check if return type is List<T>
            if (!IsListReturnType(method.ReturnType))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                method.Identifier.GetLocation(),
                method.Identifier.Text);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsListReturnType(TypeSyntax returnType)
        {
            // Match "List<T>" pattern
            if (returnType is GenericNameSyntax genericName)
            {
                return genericName.Identifier.Text == "List";
            }

            return false;
        }

        private static bool HasListReturnOkComment(MethodDeclarationSyntax method)
        {
            // Check leading trivia
            var leadingTrivia = method.GetLeadingTrivia();
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
                if (commentText.Contains("list-return-ok:"))
                    return true;
            }
            return false;
        }
    }
}
