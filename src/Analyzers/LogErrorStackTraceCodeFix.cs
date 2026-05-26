using System;
using System.Collections.Immutable;
using System.Composition;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CodeActions;
using Microsoft.CodeAnalysis.CodeFixes;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace DINOForge.Analyzers
{
    [ExportCodeFixProvider(LanguageNames.CSharp, Name = nameof(LogErrorStackTraceCodeFix))]
    [Shared]
    public class LogErrorStackTraceCodeFix : CodeFixProvider
    {
        public sealed override ImmutableArray<string> FixableDiagnosticIds =>
            ImmutableArray.Create(LogErrorStackTraceAnalyzer.DiagnosticId);

        public sealed override FixAllProvider GetFixAllProvider() =>
            WellKnownFixAllProviders.BatchFixer;

        public sealed override async Task RegisterCodeFixesAsync(CodeFixContext context)
        {
            var root = await context.Document.GetSyntaxRootAsync(context.CancellationToken).ConfigureAwait(false);
            if (root == null)
                return;

            var diagnostic = context.Diagnostics.First();
            var diagnosticSpan = diagnostic.Location.SourceSpan;

            var invocation = root.FindToken(diagnosticSpan.Start).Parent?
                .AncestorsAndSelf()
                .OfType<InvocationExpressionSyntax>()
                .FirstOrDefault();

            if (invocation == null)
                return;

            // Register the fix: "Replace with LogError(ex, message)"
            context.RegisterCodeFix(
                CodeAction.Create(
                    title: "Pass exception to LogError instead of ex.Message",
                    createChangedDocument: async ct => await FixLogErrorCall(context.Document, invocation, ct).ConfigureAwait(false),
                    equivalenceKey: "FixLogErrorStackTrace"),
                diagnostic);
        }

        private static async Task<Document> FixLogErrorCall(
            Document document,
            InvocationExpressionSyntax invocation,
            CancellationToken cancellationToken)
        {
            var root = await document.GetSyntaxRootAsync(cancellationToken).ConfigureAwait(false);
            if (root == null)
                return document;

            var semanticModel = await document.GetSemanticModelAsync(cancellationToken).ConfigureAwait(false);
            if (semanticModel == null)
                return document;

            var args = invocation.ArgumentList.Arguments;
            if (args.Count == 0)
                return document;

            var firstArg = args[0];

            // Extract the exception variable and the original message expression
            ISymbol? exceptionSymbol = null;
            ExpressionSyntax? exceptionExpr = null;

            if (firstArg.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                // Case: ex.Message, ex.InnerException.Message, etc.
                // Extract the base exception object (e.g., "ex" from "ex.Message")
                exceptionExpr = ExtractExceptionBase(memberAccess);
                if (exceptionExpr != null)
                {
                    var typeInfo = semanticModel.GetTypeInfo(exceptionExpr, cancellationToken);
                    if (IsExceptionType(typeInfo.Type, semanticModel.Compilation))
                    {
                        // Good — we can extract the exception
                        exceptionSymbol = semanticModel.GetSymbolInfo(exceptionExpr, cancellationToken).Symbol;
                    }
                }
            }

            if (exceptionExpr == null)
                return document; // Can't fix — pattern doesn't match expectation

            // Build new arguments: (ex, "description")
            var newArgs = SyntaxFactory.ArgumentList(
                SyntaxFactory.SeparatedList(new[]
                {
                    SyntaxFactory.Argument(exceptionExpr),
                    SyntaxFactory.Argument(
                        SyntaxFactory.LiteralExpression(
                            SyntaxKind.StringLiteralExpression,
                            SyntaxFactory.Literal("Failed to process")))
                }));

            var newInvocation = invocation.WithArgumentList(newArgs);

            var newRoot = root.ReplaceNode(invocation, newInvocation);
            return document.WithSyntaxRoot(newRoot);
        }

        /// <summary>
        /// Extract the base exception expression from a member access.
        /// E.g., from "ex.Message" returns "ex"; from "ex.InnerException.Message" returns "ex.InnerException".
        /// </summary>
        private static ExpressionSyntax? ExtractExceptionBase(MemberAccessExpressionSyntax memberAccess)
        {
            // The expression is the left side of the dot
            return memberAccess.Expression;
        }

        private static bool IsExceptionType(ITypeSymbol? type, Compilation compilation)
        {
            if (type == null)
                return false;

            var exceptionType = compilation.GetTypeByMetadataName("System.Exception");
            if (exceptionType == null)
                return false;

            var comparer = SymbolEqualityComparer.Default;
            if (comparer.Equals(type, exceptionType))
                return true;

            // Check base types
            if (type.BaseType != null && IsExceptionType(type.BaseType, compilation))
                return true;

            return false;
        }
    }
}
