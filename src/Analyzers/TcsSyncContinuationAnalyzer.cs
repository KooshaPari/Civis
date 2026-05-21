using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class TcsSyncContinuationAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0097";
        private const string Category = "Concurrency";

        private static readonly LocalizableString Title =
            (LocalizableString)"TaskCompletionSource missing RunContinuationsAsynchronously";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"TaskCompletionSource ctor without TaskCreationOptions.RunContinuationsAsynchronously risks sync-continuation deadlock. Pass `TaskCreationOptions.RunContinuationsAsynchronously` to the constructor, or document an intentional sync continuation with `// tcs-sync-ok: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"TaskCompletionSource without TaskCreationOptions.RunContinuationsAsynchronously runs continuations synchronously on the producer's thread, causing main-thread starvation and potential deadlocks in cross-thread marshalling contexts. Always pass TaskCreationOptions.RunContinuationsAsynchronously. For intentional sync continuation, suppress with `// tcs-sync-ok: <reason>` (the trailing colon + reason are required, per Pattern #111 convention).";

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
            context.RegisterSyntaxNodeAction(AnalyzeObjectCreation, SyntaxKind.ObjectCreationExpression);
        }

        private static void AnalyzeObjectCreation(SyntaxNodeAnalysisContext context)
        {
            var objectCreation = (ObjectCreationExpressionSyntax)context.Node;

            // Match: new TaskCompletionSource(...) or new TaskCompletionSource<T>(...)
            if (!IsTaskCompletionSourceCreation(objectCreation))
                return;

            // Check if argument list contains RunContinuationsAsynchronously
            if (HasRunContinuationsAsynchronouslyArgument(objectCreation.ArgumentList))
                return;

            // Check for `// tcs-sync-ok: <reason>` suppression marker
            if (HasTcsSyncOkMarker(objectCreation))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        /// <summary>
        /// Scans for a <c>// tcs-sync-ok: &lt;reason&gt;</c> marker in trivia around the
        /// object-creation expression. The trailing colon + reason are REQUIRED — bare
        /// <c>// tcs-sync-ok</c> (no colon) is rejected to force authors to document why.
        /// Mirrors <see cref="SilentCatchAnalyzer.HasSafeSwallowComment"/>.
        /// </summary>
        private static bool HasTcsSyncOkMarker(ObjectCreationExpressionSyntax objectCreation)
        {
            // 1. Leading trivia directly on the object-creation node
            foreach (var trivia in objectCreation.GetLeadingTrivia())
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // 2. Trailing trivia on the object-creation node (same-line marker after the expression)
            foreach (var trivia in objectCreation.GetTrailingTrivia())
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // 3. Leading + trailing trivia on the containing statement (e.g., the var-decl line)
            var containingStatement = objectCreation.FirstAncestorOrSelf<StatementSyntax>();
            if (containingStatement != null)
            {
                foreach (var trivia in containingStatement.GetLeadingTrivia())
                {
                    if (CheckTrivia(trivia))
                        return true;
                }
                foreach (var trivia in containingStatement.GetTrailingTrivia())
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
                // Require the trailing colon to force a documented reason
                // (bare `// tcs-sync-ok` without colon is NOT recognized).
                if (trivia.ToFullString().Contains("tcs-sync-ok:"))
                    return true;
            }
            return false;
        }

        private static bool IsTaskCompletionSourceCreation(ObjectCreationExpressionSyntax objectCreation)
        {
            // Check type name: TaskCompletionSource or TaskCompletionSource<T>
            var typeName = objectCreation.Type;

            if (typeName is IdentifierNameSyntax identifier)
            {
                return identifier.Identifier.ValueText == "TaskCompletionSource";
            }

            if (typeName is GenericNameSyntax generic)
            {
                return generic.Identifier.ValueText == "TaskCompletionSource";
            }

            return false;
        }

        private static bool HasRunContinuationsAsynchronouslyArgument(ArgumentListSyntax? argumentList)
        {
            if (argumentList == null)
                return false;

            foreach (var argument in argumentList.Arguments)
            {
                // Check if argument contains the text "RunContinuationsAsynchronously"
                var argumentText = argument.ToString();
                if (argumentText.Contains("RunContinuationsAsynchronously"))
                    return true;
            }

            return false;
        }
    }
}
