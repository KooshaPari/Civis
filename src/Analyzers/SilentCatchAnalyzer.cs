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
    public class SilentCatchAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0111";
        private const string Category = "Observability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Bare catch swallows exceptions silently";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Empty catch block silently swallows exception. Log via `catch (Exception ex) {{ _logger.LogWarning(ex, \"context\"); }}`, document with `// safe-swallow: <reason>`, or remove the try/catch.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Bare catch blocks with no body hide I/O, reflection, or resource-exhaustion failures, breaking observability and making debugging impossible. Always log exceptions, document safe-swallows inline, or remove the try/catch entirely. Use `catch (Exception ex) { _logger.LogWarning(ex, \"context\"); }` for production, or `// safe-swallow: <reason>` for intentional swallows.";

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
            context.RegisterSyntaxNodeAction(AnalyzeCatchClause, SyntaxKind.CatchClause);
        }

        private static void AnalyzeCatchClause(SyntaxNodeAnalysisContext context)
        {
            var catchClause = (CatchClauseSyntax)context.Node;

            // Skip if catch block has a body with statements
            if (catchClause.Block?.Statements.Count > 0)
                return;

            // Skip if catch has no exception declaration (bare catch {} without variable)
            // but we still want to flag it, so we check for empty body only

            // Check for safe-swallow or test-cleanup-ok comments in leading trivia
            if (HasSafeSwallowComment(catchClause))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, catchClause.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasSafeSwallowComment(CatchClauseSyntax catchClause)
        {
            // 1. Check leading trivia (previous-line marker)
            var leadingTrivia = catchClause.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // 2. Check trailing trivia of closing brace (same-line trailing marker)
            if (catchClause.Block?.CloseBraceToken.HasTrailingTrivia ?? false)
            {
                var closingBraceTrailing = catchClause.Block.CloseBraceToken.TrailingTrivia;
                foreach (var trivia in closingBraceTrailing)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }
            }

            // 3. Check inline trivia inside the block (e.g., catch { /* safe-swallow: */ })
            if (catchClause.Block != null)
            {
                // Check leading trivia of opening brace
                var openingBraceLeading = catchClause.Block.OpenBraceToken.LeadingTrivia;
                foreach (var trivia in openingBraceLeading)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }

                // Check trailing trivia of opening brace
                var openingBraceTrailing = catchClause.Block.OpenBraceToken.TrailingTrivia;
                foreach (var trivia in openingBraceTrailing)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }

                // Check leading trivia of closing brace
                var closingBraceLeading = catchClause.Block.CloseBraceToken.LeadingTrivia;
                foreach (var trivia in closingBraceLeading)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }
            }

            return false;
        }

        private static bool CheckTrivia(SyntaxTrivia trivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("safe-swallow:") ||
                    commentText.Contains("test-cleanup-ok"))
                    return true;
            }
            return false;
        }
    }
}
