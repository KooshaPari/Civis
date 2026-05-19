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
    public class SyncOverAsyncAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0116";
        private const string Category = "Reliability";

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

            // Exclude false-positive member names
            if (IsFalsePositiveMemberName(memberName))
                return;

            // Check for sync-over-async-unavoidable comment in leading trivia
            if (HasSyncOverAsyncUnavoidableComment(memberAccess))
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

            // Check for sync-over-async-unavoidable comment in leading trivia
            if (HasSyncOverAsyncUnavoidableComment(invocation))
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

        private static bool HasSyncOverAsyncUnavoidableComment(SyntaxNode node)
        {
            // Check leading trivia for sync-over-async-unavoidable marker
            var leadingTrivia = node.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("sync-over-async-unavoidable:"))
                        return true;
                }
            }

            // Also check trailing trivia of preceding token
            var parent = node.Parent;
            if (parent != null)
            {
                var token = parent.GetFirstToken();
                if (token.HasTrailingTrivia)
                {
                    var trailingTrivia = token.TrailingTrivia;
                    foreach (var trivia in trailingTrivia)
                    {
                        if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                        {
                            var commentText = trivia.ToFullString();
                            if (commentText.Contains("sync-over-async-unavoidable:"))
                                return true;
                        }
                    }
                }
            }

            return false;
        }
    }
}
