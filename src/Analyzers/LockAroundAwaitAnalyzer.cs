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
    public class LockAroundAwaitAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1003";
        private const string Category = "Concurrency";

        private static readonly LocalizableString Title =
            (LocalizableString)"`await` inside `lock` block";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`await` inside `lock` risks IllegalMonitorStateException since continuation may run on different thread. Use `SemaphoreSlim.WaitAsync` for async-safe mutual exclusion.";

        private static readonly LocalizableString Description =
            (LocalizableString)"The C# `lock` statement uses Monitor.Enter/Exit which requires the same thread to enter and exit. If an `await` expression is used inside a lock block, the async continuation may resume on a different thread, causing IllegalMonitorStateException or deadlock. Replace with `SemaphoreSlim.WaitAsync()` or move await outside the lock. Use `// lock-await-ok: <reason>` inline comment to suppress.";

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
            context.RegisterSyntaxNodeAction(AnalyzeAwaitExpression, SyntaxKind.AwaitExpression);
        }

        private static void AnalyzeAwaitExpression(SyntaxNodeAnalysisContext context)
        {
            var awaitExpr = (AwaitExpressionSyntax)context.Node;

            // Skip if marked with lock-await-ok
            if (HasLockAwaitOkComment(awaitExpr))
                return;

            // Walk ancestors looking for LockStatementSyntax
            var lockStatement = awaitExpr.Ancestors()
                .OfType<LockStatementSyntax>()
                .FirstOrDefault();

            if (lockStatement == null)
                return;

            // Report diagnostic at the await keyword location
            var diagnostic = Diagnostic.Create(Rule, awaitExpr.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasLockAwaitOkComment(AwaitExpressionSyntax awaitExpr)
        {
            // Check leading trivia
            var leadingTrivia = awaitExpr.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // Check trailing trivia of the await keyword
            var awaitTokenTrailing = awaitExpr.AwaitKeyword.TrailingTrivia;
            foreach (var trivia in awaitTokenTrailing)
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
                if (commentText.Contains("lock-await-ok:"))
                    return true;
            }
            return false;
        }
    }
}
