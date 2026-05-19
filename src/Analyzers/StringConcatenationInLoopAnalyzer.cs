using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class StringConcatenationInLoopAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1025";
        private const string Category = "Performance";

        private static readonly LocalizableString Title =
            (LocalizableString)"String concatenation in loop creates GC pressure";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"String concatenation via `+=` inside a loop allocates new strings on each iteration. Use StringBuilder or string.Join instead to avoid quadratic allocation and GC pressure.";

        private static readonly LocalizableString Description =
            (LocalizableString)"String concatenation in loops (for, while, do, foreach) creates O(N²) heap allocations because each += creates a new string object and copies the entire previous string. This pattern is common in dynamically building content (YAML, JSON, HTML) and causes significant GC pressure in hot paths. Always use StringBuilder or string.Join for loop-based concatenation.";

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
            context.RegisterSyntaxNodeAction(AnalyzeAssignmentExpression, SyntaxKind.AddAssignmentExpression);
        }

        private static void AnalyzeAssignmentExpression(SyntaxNodeAnalysisContext context)
        {
            var assignmentExpr = (AssignmentExpressionSyntax)context.Node;

            // Skip if leading-trivia contains gc-concat-ok marker
            if (HasGcConcatOkComment(assignmentExpr))
                return;

            // Get semantic model for type checking
            var semanticModel = context.SemanticModel;

            // Analyze left-hand side type
            var lhsTypeInfo = semanticModel.GetTypeInfo(assignmentExpr.Left);
            var lhsType = lhsTypeInfo.Type;

            // Skip if LHS is not a string type
            if (lhsType?.SpecialType != SpecialType.System_String)
                return;

            // Check if we're inside a loop
            if (!IsInsideLoop(assignmentExpr))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, assignmentExpr.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsInsideLoop(SyntaxNode node)
        {
            var parent = node.Parent;
            while (parent != null)
            {
                // Check for all loop types
                if (parent is ForStatementSyntax ||
                    parent is WhileStatementSyntax ||
                    parent is DoStatementSyntax ||
                    parent is ForEachStatementSyntax)
                {
                    return true;
                }

                // Stop at method/property boundary
                if (parent is MethodDeclarationSyntax ||
                    parent is PropertyDeclarationSyntax ||
                    parent is AccessorDeclarationSyntax)
                {
                    return false;
                }

                parent = parent.Parent;
            }

            return false;
        }

        private static bool HasGcConcatOkComment(AssignmentExpressionSyntax expr)
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
                if (commentText.Contains("gc-concat-ok:"))
                    return true;
            }
            return false;
        }
    }
}
