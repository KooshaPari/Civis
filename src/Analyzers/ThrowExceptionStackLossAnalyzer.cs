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
    public class ThrowExceptionStackLossAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1012";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Use 'throw;' to preserve stack trace";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Replace 'throw ex;' with 'throw;' to preserve the original exception stack trace. If you intentionally want to reset the stack, document with `// rethrow-as-new-ok: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Rethrowing an exception variable with 'throw ex;' resets the stack trace to the current throw location, losing the original call site. Always use 'throw;' (bare rethrow) to preserve the stack trace. Only use 'throw ex;' when explicitly wrapping the exception in a new exception type with `// rethrow-as-new-ok: <reason>` documented.";

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
            context.RegisterSyntaxNodeAction(AnalyzeThrow, SyntaxKind.ThrowStatement);
        }

        private static void AnalyzeThrow(SyntaxNodeAnalysisContext context)
        {
            var throwStatement = (ThrowStatementSyntax)context.Node;

            // Skip if no expression (bare throw; is correct)
            if (throwStatement.Expression == null)
                return;

            // Check if the expression is a simple identifier
            if (!(throwStatement.Expression is IdentifierNameSyntax identifierName))
                return;

            // Check for rethrow-as-new-ok comment in leading trivia
            if (HasRethrowAsNewOkComment(throwStatement))
                return;

            // Walk up to find the enclosing catch clause
            var catchClause = FindEnclosingCatchClause(throwStatement);
            if (catchClause == null)
                return; // Not in a catch block, so this is a new throw, not a rethrow

            // Check if the identifier matches the catch declaration variable
            if (catchClause.Declaration == null)
                return; // catch without explicit variable (catch (Exception) not catch (Exception ex))

            var caughtVariableName = catchClause.Declaration.Identifier.ValueText;
            var thrownVariableName = identifierName.Identifier.ValueText;

            // If the thrown variable is the caught exception variable, this is a problematic rethrow
            if (caughtVariableName == thrownVariableName)
            {
                var diagnostic = Diagnostic.Create(Rule, throwStatement.GetLocation());
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static CatchClauseSyntax? FindEnclosingCatchClause(SyntaxNode node)
        {
            var current = node.Parent;

            while (current != null)
            {
                // If we hit a catch clause, check if we're in its body
                if (current is CatchClauseSyntax catchClause)
                {
                    return catchClause;
                }

                // If we hit a method or lambda (other than within the catch), we've left the catch scope
                if (current is MethodDeclarationSyntax or LocalFunctionStatementSyntax or LambdaExpressionSyntax or AnonymousFunctionExpressionSyntax)
                {
                    return null;
                }

                current = current.Parent;
            }

            return null;
        }

        private static bool HasRethrowAsNewOkComment(SyntaxNode node)
        {
            // Check leading trivia for rethrow-as-new-ok marker
            var leadingTrivia = node.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("rethrow-as-new-ok:"))
                        return true;
                }
            }

            // Also check trailing trivia of the previous token
            var previousToken = node.GetFirstToken().GetPreviousToken();
            if (previousToken != default && previousToken.HasTrailingTrivia)
            {
                foreach (var trivia in previousToken.TrailingTrivia)
                {
                    if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                        trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                    {
                        var commentText = trivia.ToFullString();
                        if (commentText.Contains("rethrow-as-new-ok:"))
                            return true;
                    }
                }
            }

            return false;
        }
    }
}
