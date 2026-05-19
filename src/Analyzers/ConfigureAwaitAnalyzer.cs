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
    public class ConfigureAwaitAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0098";
        private const string Category = "Async";

        private static readonly LocalizableString Title =
            (LocalizableString)"await missing ConfigureAwait(false) in library code";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Library-scope await should use .ConfigureAwait(false) to avoid capturing synchronization context. Suppress with `// configureawait-ok: <reason>` if app-scope.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Library code (SDK, Bridge, Domains) must call .ConfigureAwait(false) on all awaits to prevent capturing the synchronization context. This allows library consumers to use the library in UI contexts without deadlock. Application code (Runtime, Tools) may omit this when deliberately scoped to a synchronization context. Mark intentional omissions with `// configureawait-ok: <reason>`.";

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
            context.RegisterSyntaxNodeAction(AnalyzeAwaitExpression, SyntaxKind.AwaitExpression);
        }

        private static void AnalyzeAwaitExpression(SyntaxNodeAnalysisContext context)
        {
            var awaitExpr = (AwaitExpressionSyntax)context.Node;

            // Skip if in skipped file paths (application code)
            var filePath = context.Node.SyntaxTree.FilePath;
            if (ShouldSkipFile(filePath))
                return;

            // Check for configureawait-ok comment in leading trivia
            if (HasConfigureAwaitOkComment(awaitExpr))
                return;

            // Check if the awaited expression already ends with .ConfigureAwait(...)
            if (EndsWithConfigureAwait(awaitExpr.Expression))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, awaitExpr.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool ShouldSkipFile(string filePath)
        {
            // Skip application code directories (these are allowed to omit ConfigureAwait)
            var skipPatterns = new[]
            {
                "\\Tests\\",
                "\\Tools\\",
                "\\Runtime\\",
                "\\Domains\\Runtime\\"
            };

            return skipPatterns.Any(pattern => filePath.Contains(pattern));
        }

        private static bool HasConfigureAwaitOkComment(AwaitExpressionSyntax awaitExpr)
        {
            // Check leading trivia for configureawait-ok marker
            var leadingTrivia = awaitExpr.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("configureawait-ok:"))
                        return true;
                }
            }

            // Also check trailing trivia of preceding token (same-line marker)
            var token = awaitExpr.AwaitKeyword;
            if (token.HasTrailingTrivia)
            {
                var trailingTrivia = token.TrailingTrivia;
                foreach (var trivia in trailingTrivia)
                {
                    if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                    {
                        var commentText = trivia.ToFullString();
                        if (commentText.Contains("configureawait-ok:"))
                            return true;
                    }
                }
            }

            return false;
        }

        private static bool EndsWithConfigureAwait(ExpressionSyntax expression)
        {
            // Check if the expression is an invocation ending with .ConfigureAwait(...)
            if (expression is InvocationExpressionSyntax invocation)
            {
                if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
                {
                    return memberAccess.Name.Identifier.ValueText == "ConfigureAwait";
                }
            }

            return false;
        }
    }
}
