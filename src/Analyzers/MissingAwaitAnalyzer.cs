using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;
using Microsoft.CodeAnalysis.Operations;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class MissingAwaitAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1017";
        private const string Category = "Reliability";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Task result discarded — missing 'await' or '_'",
            "Call to async method '{0}' is not awaited — task result is discarded, exceptions are unobservable. Add 'await' or assign to '_'.",
            "Calling an async method without awaiting its result is fire-and-forget asynchrony that hides exceptions and prevents caller synchronization. Always use 'await' for the result, or explicitly discard with '_' to document intent. Mark with `// fire-and-forget-ok: <reason>` only if truly intentional.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.InvocationExpression);
        }

        private static void Analyze(SyntaxNodeAnalysisContext ctx)
        {
            var invocation = (InvocationExpressionSyntax)ctx.Node;

            // Skip if this invocation is inside an await expression
            var parent = invocation.Parent;
            if (parent is AwaitExpressionSyntax)
                return;

            // Skip if parent is an assignment to a variable or '_' discard
            if (parent is AssignmentExpressionSyntax assignment)
            {
                // Check if the left side is a discard or variable (both are okay)
                var leftSide = assignment.Left.ToString();
                return; // Assigned to a variable or discard, so it's intentional
            }

            // Skip if parent is a simple assignment (using declarator)
            if (parent is VariableDeclaratorSyntax)
                return;

            // Skip if parent is part of a method/lambda argument or return
            if (IsUsedAsArgument(invocation))
                return;

            // Skip if parent is part of a ConditionalExpressionSyntax or other expression contexts
            if (IsPartOfLargerExpression(invocation))
                return;

            // The invocation must be in ExpressionStatementSyntax (statement position, not used)
            if (!(parent is ExpressionStatementSyntax))
                return;

            // Skip if marked with fire-and-forget-ok
            if (HasFireAndForgetOkMarker(invocation))
                return;

            // Skip test files
            var filePath = ctx.Node.SyntaxTree.FilePath;
            if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
                return;

            // Get the symbol to determine if it's an async method
            var symbol = ctx.SemanticModel.GetSymbolInfo(invocation).Symbol;
            if (symbol == null)
                return;

            var returnType = GetReturnType(symbol);
            if (returnType == null)
                return;

            // Check if return type is Task or ValueTask
            var returnTypeStr = returnType.ToString();
            var isTask = returnTypeStr.StartsWith("System.Threading.Tasks.Task", StringComparison.Ordinal) ||
                         returnTypeStr.StartsWith("System.Threading.Tasks.ValueTask", StringComparison.Ordinal) ||
                         returnTypeStr == "Task" || returnTypeStr == "ValueTask" ||
                         returnTypeStr.StartsWith("Task<", StringComparison.Ordinal) ||
                         returnTypeStr.StartsWith("ValueTask<", StringComparison.Ordinal);

            if (!isTask)
                return;

            // Get method name for the diagnostic message
            var methodName = symbol.Name;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                invocation.GetLocation(),
                methodName);
            ctx.ReportDiagnostic(diagnostic);
        }

        private static ITypeSymbol? GetReturnType(ISymbol symbol)
        {
            return symbol switch
            {
                IMethodSymbol method => method.ReturnType,
                IPropertySymbol property => property.Type,
                _ => null
            };
        }

        private static bool IsUsedAsArgument(InvocationExpressionSyntax invocation)
        {
            var parent = invocation.Parent;

            // Check if parent is an argument in another invocation
            if (parent is ArgumentSyntax)
                return true;

            // Check if parent is part of an array/collection initializer
            if (parent?.Parent is InitializerExpressionSyntax)
                return true;

            return false;
        }

        private static bool IsPartOfLargerExpression(InvocationExpressionSyntax invocation)
        {
            var parent = invocation.Parent;

            // If the immediate parent is an expression that uses the invocation's result
            return parent is BinaryExpressionSyntax or
                   ConditionalExpressionSyntax or
                   CastExpressionSyntax or
                   InvocationExpressionSyntax or
                   MemberAccessExpressionSyntax or
                   ReturnStatementSyntax or
                   ArrowExpressionClauseSyntax;
        }

        private static bool HasFireAndForgetOkMarker(SyntaxNode node)
        {
            var leadingTrivia = node.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("fire-and-forget-ok:"))
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
                        if (commentText.Contains("fire-and-forget-ok:"))
                            return true;
                    }
                }
            }

            return false;
        }
    }
}
