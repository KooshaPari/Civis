using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1019: Missing ConfigureAwait(false) in library code.
/// Detects <c>await</c> expressions that do not use <c>.ConfigureAwait(false)</c>,
/// which can capture synchronization context and cause deadlocks when called from
/// UI or sync-blocking contexts. Pattern #98 enforcement at compile-time.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class MissingConfigureAwaitAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1019";

    private const string Category = "Reliability";

    private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
        DiagnosticId,
        Category,
        DiagnosticSeverity.Info,
        "await without ConfigureAwait(false) in library code",
        "await expression should use .ConfigureAwait(false) in library code to avoid context capture — use 'await {0}.ConfigureAwait(false)'",
        "Library code should avoid capturing the caller's synchronization context. Use 'await X.ConfigureAwait(false)' to prevent deadlocks when called from UI or sync-blocking contexts. Exempt with `// configureawait-ok: <reason>` for timing-sensitive operations like Task.Delay or Task.Yield.");

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.AwaitExpression);
    }

    private static void Analyze(SyntaxNodeAnalysisContext ctx)
    {
        var awaitExpr = (AwaitExpressionSyntax)ctx.Node;
        var filePath = ctx.Node.SyntaxTree.FilePath;

        // Skip test files
        if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
            return;

        // Skip generated files
        if (filePath.Contains(".Generated.cs", StringComparison.OrdinalIgnoreCase))
            return;

        // Only analyze library code (SDK, Bridge, Domains)
        if (!IsLibraryCode(filePath))
            return;

        // Skip if marked with configureawait-ok marker
        if (HasConfigureAwaitOkMarker(awaitExpr))
            return;

        // Check if the awaited expression is already a ConfigureAwait invocation
        if (IsAlreadyConfigureAwait(awaitExpr.Expression))
            return;

        // Skip Task.Yield() and Task.Delay() (timing-sensitive exemptions)
        if (IsTimingSensitiveCall(awaitExpr.Expression))
            return;

        // Get the expression text for the diagnostic message
        var expressionText = awaitExpr.Expression.ToString();

        // Report diagnostic
        var diagnostic = Diagnostic.Create(
            Rule,
            awaitExpr.GetLocation(),
            expressionText);
        ctx.ReportDiagnostic(diagnostic);
    }

    /// <summary>
    /// Determines if the file is library code (SDK, Bridge, Domains).
    /// </summary>
    private static bool IsLibraryCode(string filePath)
    {
        var normalized = filePath.Replace('\\', '/');
        return normalized.Contains("/SDK/") ||
               normalized.Contains("/Bridge/") ||
               normalized.Contains("/Domains/");
    }

    /// <summary>
    /// Checks if the expression is already a ConfigureAwait invocation.
    /// </summary>
    private static bool IsAlreadyConfigureAwait(ExpressionSyntax expr)
    {
        if (expr is not InvocationExpressionSyntax invocation)
            return false;

        var memberAccess = invocation.Expression as MemberAccessExpressionSyntax;
        if (memberAccess?.Name.Identifier.Text == "ConfigureAwait")
            return true;

        return false;
    }

    /// <summary>
    /// Detects timing-sensitive operations (Task.Yield, Task.Delay) that are exempt.
    /// </summary>
    private static bool IsTimingSensitiveCall(ExpressionSyntax expr)
    {
        var exprStr = expr.ToString();

        // Task.Delay and Task.Yield are timing-sensitive
        if (exprStr.Contains("Task.Delay", StringComparison.Ordinal) ||
            exprStr.Contains("Task.Yield", StringComparison.Ordinal))
            return true;

        return false;
    }

    /// <summary>
    /// Checks for configureawait-ok marker in trivia.
    /// </summary>
    private static bool HasConfigureAwaitOkMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("configureawait-ok:", StringComparison.OrdinalIgnoreCase))
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
                    if (commentText.Contains("configureawait-ok:", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }
}
