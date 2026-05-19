using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    /// <summary>
    /// Analyzer DF0116: Sync-over-async blocking call (Pattern #116).
    /// Detects <c>.Result</c> / <c>.Wait()</c> on tasks. Suppress with
    /// <c>// sync-over-async-unavoidable: &lt;reason&gt;</c> on the same line
    /// (trailing trivia), the previous line (leading trivia of the enclosing
    /// statement), or directly on the offending node. Marker semantics mirror
    /// Pattern #96 (DF0096) for consistency.
    /// </summary>
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class SyncOverAsyncAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0116";
        private const string Category = "Reliability";
        private const string SuppressionMarker = "sync-over-async-unavoidable:";

        private static readonly LocalizableString Title =
            (LocalizableString)"Sync-over-async blocking call (.Result / .Wait())";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Use `await` instead of `.Result` / `.Wait()`. Blocking on a task risks deadlock when continuations need the captured context. If unavoidable, document with `// sync-over-async-unavoidable: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Sync-over-async (calling .Result or .Wait() on a task) blocks the calling thread and can cause deadlock if the task's continuation requires the same SynchronizationContext. Always use `await` in async contexts, or document with `// sync-over-async-unavoidable: <reason>` if truly necessary.";

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
            context.RegisterSyntaxNodeAction(AnalyzeMemberAccess, SyntaxKind.SimpleMemberAccessExpression);
            context.RegisterSyntaxNodeAction(AnalyzeInvocation, SyntaxKind.InvocationExpression);
        }

        private static void AnalyzeMemberAccess(SyntaxNodeAnalysisContext context)
        {
            var memberAccess = (MemberAccessExpressionSyntax)context.Node;

            // Match .Result but exclude .ResultType, .ResultSummary, .Results
            var memberName = memberAccess.Name.Identifier.ValueText;
            if (memberName != "Result")
                return;

            // Exclude false-positive member names (defense in depth — the prior
            // equality check already narrows to "Result" exactly).
            if (IsFalsePositiveMemberName(memberName))
                return;

            if (HasSuppressionMarker(memberAccess))
                return;

            var diagnostic = Diagnostic.Create(Rule, memberAccess.Name.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            // Match .Wait() invocations
            if (!(invocation.Expression is MemberAccessExpressionSyntax memberAccess))
                return;

            if (memberAccess.Name.Identifier.ValueText != "Wait")
                return;

            if (HasSuppressionMarker(invocation))
                return;

            var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsFalsePositiveMemberName(string memberName)
        {
            // Exclude properties/fields that contain "Result" but are not Task.Result
            var falsePositives = new[] { "ResultType", "ResultSummary", "Results" };
            return falsePositives.Contains(memberName);
        }

        /// <summary>
        /// Returns true when the offending node (or the enclosing statement)
        /// carries a <c>// sync-over-async-unavoidable: ...</c> marker in
        /// adjacent trivia. Checks (in order): node leading + trailing trivia,
        /// enclosing statement leading + trailing trivia. Mirrors DF0096's
        /// HasSuppressionMarker so authors can place the marker on the
        /// statement above the call site OR inline at the end of the line.
        /// </summary>
        private static bool HasSuppressionMarker(SyntaxNode node)
        {
            if (TriviaContainsMarker(node.GetLeadingTrivia()))
                return true;
            if (TriviaContainsMarker(node.GetTrailingTrivia()))
                return true;

            var statement = node.FirstAncestorOrSelf<StatementSyntax>();
            if (statement != null)
            {
                if (TriviaContainsMarker(statement.GetLeadingTrivia()))
                    return true;
                if (TriviaContainsMarker(statement.GetTrailingTrivia()))
                    return true;
            }

            return false;
        }

        private static bool TriviaContainsMarker(SyntaxTriviaList trivia)
        {
            foreach (var t in trivia)
            {
                if (t.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    t.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var text = t.ToFullString();
                    if (text.IndexOf(SuppressionMarker, StringComparison.Ordinal) >= 0)
                        return true;
                }
            }
            return false;
        }
    }
}
