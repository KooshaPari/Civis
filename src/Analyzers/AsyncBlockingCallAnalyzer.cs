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
    public class AsyncBlockingCallAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1011";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Blocking Task.Result or Wait() in async context";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Avoid '.Result' or '.Wait()' inside an async method — use 'await' instead to prevent deadlocks. If unavoidable, document with `// async-blocking-ok: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Calling .Result or .Wait() on a Task inside an async method blocks the async context, risking deadlock if the task's continuation requires the same context. Always use `await` in async contexts, or document with `// async-blocking-ok: <reason>` if truly unavoidable.";

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

            // Check if this is .Result access
            var memberName = memberAccess.Name.Identifier.ValueText;
            if (memberName != "Result")
                return;

            // Check for async-blocking-ok comment in leading trivia
            if (HasAsyncBlockingOkComment(memberAccess))
                return;

            // Check if the containing method is async
            if (!IsInsideAsyncMethod(memberAccess))
                return;

            var diagnostic = Diagnostic.Create(Rule, memberAccess.Name.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            // Match .Wait() invocations (no args or single TimeSpan arg)
            if (!(invocation.Expression is MemberAccessExpressionSyntax memberAccess))
                return;

            if (memberAccess.Name.Identifier.ValueText != "Wait")
                return;

            // Check argument count: 0 or 1 (TimeSpan/int)
            var argCount = invocation.ArgumentList.Arguments.Count;
            if (argCount > 1)
                return; // Not the blocking Wait() we're looking for

            // Check for async-blocking-ok comment in leading trivia
            if (HasAsyncBlockingOkComment(invocation))
                return;

            // Check if the containing method is async
            if (!IsInsideAsyncMethod(invocation))
                return;

            var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasAsyncBlockingOkComment(SyntaxNode node)
        {
            // Check leading trivia for async-blocking-ok marker
            var leadingTrivia = node.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("async-blocking-ok:"))
                        return true;
                }
            }

            // Also check trailing trivia of the previous token
            var parent = node.Parent;
            if (parent != null)
            {
                var previousToken = node.GetFirstToken().GetPreviousToken();
                if (previousToken != default && previousToken.HasTrailingTrivia)
                {
                    foreach (var trivia in previousToken.TrailingTrivia)
                    {
                        if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                            trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                        {
                            var commentText = trivia.ToFullString();
                            if (commentText.Contains("async-blocking-ok:"))
                                return true;
                        }
                    }
                }
            }

            return false;
        }

        private static bool IsInsideAsyncMethod(SyntaxNode node)
        {
            // Walk up the syntax tree to find a method or lambda
            var current = node.Parent;

            while (current != null)
            {
                // Check if we're inside an async method
                if (current is MethodDeclarationSyntax methodDecl)
                {
                    return methodDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.AsyncKeyword));
                }

                // Check if we're inside an async local function
                if (current is LocalFunctionStatementSyntax localFunc)
                {
                    return localFunc.Modifiers.Any(m => m.IsKind(SyntaxKind.AsyncKeyword));
                }

                // Check if we're inside an async lambda
                if (current is LambdaExpressionSyntax lambda)
                {
                    return lambda.AsyncKeyword.IsKind(SyntaxKind.AsyncKeyword);
                }

                // Check if we're inside an async anonymous function
                if (current is AnonymousFunctionExpressionSyntax anonFunc)
                {
                    return anonFunc.AsyncKeyword.IsKind(SyntaxKind.AsyncKeyword);
                }

                current = current.Parent;
            }

            return false;
        }
    }
}
