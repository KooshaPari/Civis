using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1023: Empty catch block silently swallows exceptions.
/// Detects <c>try { ... } catch { }</c> or <c>try { ... } catch (Exception) { }</c>
/// patterns where the catch block body is empty (no statements or logging).
/// This hides errors and makes debugging impossible. Pattern #228 enforcement at compile-time.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class EmptyCatchBlockAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1023";

    private const string Category = "Reliability";

    private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
        DiagnosticId,
        Category,
        DiagnosticSeverity.Warning,
        "Empty catch block silently swallows exceptions",
        "Catch block has no body. Exception is silently swallowed. Add logging or '// safe-swallow: <reason>' marker.",
        "An empty catch block hides exceptions entirely, breaking observability and making debugging impossible. Add logging via _logger.LogWarning(ex, \"context\") or document the deliberate swallow with an inline '// safe-swallow: <reason>' comment. Test cleanup handlers can use '// test-cleanup-ok' marker.");

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.CatchClause);
    }

    private static void Analyze(SyntaxNodeAnalysisContext ctx)
    {
        var catchClause = (CatchClauseSyntax)ctx.Node;
        var filePath = ctx.Node.SyntaxTree.FilePath;

        // Skip test files
        if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
        {
            // Test files can have empty catch blocks if marked with // test-cleanup-ok
            if (!HasTestCleanupOkMarker(catchClause))
                return;
        }

        // Skip generated files
        if (filePath.Contains(".Generated.cs", StringComparison.OrdinalIgnoreCase))
            return;

        // Check if marked with safe-swallow marker
        if (HasSafeSwallowMarker(catchClause))
            return;

        // Check if the catch block body is empty
        var block = catchClause.Block;
        if (block == null)
            return;

        // If statements list is empty, this is an empty catch block
        if (block.Statements.Count == 0)
        {
            // Check if there are any meaningful comments inside the braces (beyond whitespace)
            var hasMeaningfulContent = HasMeaningfulContent(block);
            if (!hasMeaningfulContent)
            {
                var diagnostic = Diagnostic.Create(Rule, catchClause.GetLocation());
                ctx.ReportDiagnostic(diagnostic);
            }
        }
    }

    /// <summary>
    /// Checks if the catch block has any meaningful content beyond whitespace.
    /// </summary>
    private static bool HasMeaningfulContent(BlockSyntax block)
    {
        // Check all trivia (comments, etc.) inside the block
        var blockTrivia = block.DescendantTrivia();

        foreach (var trivia in blockTrivia)
        {
            // If there's any comment inside (beyond the structure), it's meaningful
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                return true;
            }
        }

        return false;
    }

    /// <summary>
    /// Checks for safe-swallow marker in trivia.
    /// </summary>
    private static bool HasSafeSwallowMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("safe-swallow:", StringComparison.OrdinalIgnoreCase))
                    return true;
            }
        }

        // Check trailing trivia of the previous token
        var previousToken = node.GetFirstToken().GetPreviousToken();
        if (previousToken != default && previousToken.HasTrailingTrivia)
        {
            foreach (var trivia in previousToken.TrailingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("safe-swallow:", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }

    /// <summary>
    /// Checks for test-cleanup-ok marker in trivia (only for test files).
    /// </summary>
    private static bool HasTestCleanupOkMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("test-cleanup-ok", StringComparison.OrdinalIgnoreCase))
                    return true;
            }
        }

        // Check trailing trivia of the previous token
        var previousToken = node.GetFirstToken().GetPreviousToken();
        if (previousToken != default && previousToken.HasTrailingTrivia)
        {
            foreach (var trivia in previousToken.TrailingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("test-cleanup-ok", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }
}
