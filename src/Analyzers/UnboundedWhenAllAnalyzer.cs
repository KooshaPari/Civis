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
    public class UnboundedWhenAllAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1004";
        private const string Category = "Performance";

        private static readonly LocalizableString Title =
            (LocalizableString)"Task.WhenAll over potentially unbounded enumeration";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`Task.WhenAll(...Select(...))` over `{0}` may allocate N tasks for large N. Consider `Parallel.ForEachAsync` with `MaxDegreeOfParallelism` cap for >10 expected items.";

        private static readonly LocalizableString Description =
            (LocalizableString)"The pattern `Task.WhenAll(items.Select(x => DoAsync(x)))` allocates one task per item in the enumeration. For large or dynamically-sized enumerations, this can cause memory and thread-pool exhaustion. For >10 expected items, use `Parallel.ForEachAsync` with `MaxDegreeOfParallelism` to cap concurrency. Use `// task-whenall-ok: <reason>` inline comment to suppress.";

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
            context.RegisterSyntaxNodeAction(AnalyzeInvocation, SyntaxKind.InvocationExpression);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            // Skip if marked with task-whenall-ok
            if (HasTaskWhenAllOkComment(invocation))
                return;

            // Check if this is a Task.WhenAll(...) call
            if (!IsTaskWhenAllCall(invocation))
                return;

            // Check if the argument is a .Select(...) invocation
            if (invocation.ArgumentList.Arguments.Count != 1)
                return;

            var arg = invocation.ArgumentList.Arguments[0].Expression;
            if (!IsSelectInvocation(arg, out var selectSourceName))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                invocation.GetLocation(),
                selectSourceName);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsTaskWhenAllCall(InvocationExpressionSyntax invocation)
        {
            // Check if this is Task.WhenAll(...)
            if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                // Check method name is "WhenAll"
                if (memberAccess.Name.Identifier.ValueText != "WhenAll")
                    return false;

                // Check the left side is "Task" (could be fully qualified like System.Threading.Tasks.Task)
                return memberAccess.Expression.ToString().EndsWith("Task");
            }

            return false;
        }

        private static bool IsSelectInvocation(ExpressionSyntax expr, out string sourceName)
        {
            sourceName = "enumerable";

            // Check if this is a .Select(...) invocation
            if (expr is InvocationExpressionSyntax selectInvocation)
            {
                if (selectInvocation.Expression is MemberAccessExpressionSyntax selectMemberAccess)
                {
                    if (selectMemberAccess.Name.Identifier.ValueText == "Select")
                    {
                        // Extract the source (left side of .Select)
                        sourceName = selectMemberAccess.Expression.ToString();
                        return true;
                    }
                }
            }

            return false;
        }

        private static bool HasTaskWhenAllOkComment(InvocationExpressionSyntax invocation)
        {
            // Check leading trivia
            var leadingTrivia = invocation.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // Check trailing trivia of the invocation
            var trailingTrivia = invocation.GetTrailingTrivia();
            foreach (var trivia in trailingTrivia)
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
                if (commentText.Contains("task-whenall-ok:"))
                    return true;
            }
            return false;
        }
    }
}
