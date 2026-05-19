using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class AsyncLambdaActionAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1010";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Async lambda assigned to Action / fire-and-forget";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Async lambda discards Task — exceptions are unobservable. Use `Func<Task>` instead, or wrap in `Task.Run(async () => { ... })` with explicit error handling.";

        private static readonly LocalizableString Description =
            (LocalizableString)"An async lambda expression is assigned to a delegate type that does not return a Task (e.g., `Action`). This creates a fire-and-forget pattern where exceptions thrown in the lambda are not observed and may be lost. Use `Func<Task>` to preserve the Task result, or wrap in `Task.Run(...)` with explicit error handling via continuation or ContinueWith.";

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
            context.RegisterSyntaxNodeAction(AnalyzeLambda, SyntaxKind.ParenthesizedLambdaExpression, SyntaxKind.SimpleLambdaExpression);
        }

        private static void AnalyzeLambda(SyntaxNodeAnalysisContext context)
        {
            var lambda = (LambdaExpressionSyntax)context.Node;

            // Skip if leading-trivia contains async-action-ok marker
            if (HasAsyncActionOkComment(lambda))
                return;

            // Check if this lambda is async
            if (!lambda.AsyncKeyword.IsKind(SyntaxKind.AsyncKeyword))
                return;

            // Walk up the syntax tree to find the parent assignment/declaration
            var parent = lambda.Parent;
            if (parent == null)
                return;

            // Check if the parent is an AssignmentExpressionSyntax or VariableDeclaratorSyntax
            bool isAssignedToAction = false;

            if (parent is AssignmentExpressionSyntax assignment)
            {
                isAssignedToAction = IsTargetAction(assignment.Left, context.SemanticModel);
            }
            else if (parent is VariableDeclaratorSyntax varDeclarator)
            {
                isAssignedToAction = IsVariableAction(varDeclarator, context.SemanticModel);
            }

            if (isAssignedToAction)
            {
                var diagnostic = Diagnostic.Create(Rule, lambda.GetLocation());
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool IsTargetAction(ExpressionSyntax target, SemanticModel semanticModel)
        {
            var symbolInfo = semanticModel.GetSymbolInfo(target);
            if (symbolInfo.Symbol is IPropertySymbol propSymbol)
            {
                return IsActionType(propSymbol.Type);
            }

            if (symbolInfo.Symbol is IFieldSymbol fieldSymbol)
            {
                return IsActionType(fieldSymbol.Type);
            }

            if (symbolInfo.Symbol is IParameterSymbol paramSymbol)
            {
                return IsActionType(paramSymbol.Type);
            }

            if (symbolInfo.Symbol is ILocalSymbol localSymbol)
            {
                return IsActionType(localSymbol.Type);
            }

            return false;
        }

        private static bool IsVariableAction(VariableDeclaratorSyntax varDeclarator, SemanticModel semanticModel)
        {
            // Get the parent variable declaration (e.g., VariableDeclarationSyntax)
            var varDecl = varDeclarator.Parent as VariableDeclarationSyntax;
            if (varDecl == null)
                return false;

            // Check the explicit type annotation
            var typeInfo = semanticModel.GetTypeInfo(varDecl.Type);
            if (typeInfo.Type != null && IsActionType(typeInfo.Type))
                return true;

            // If no explicit type (var), check the initializer type
            if (varDecl.Type.IsKind(SyntaxKind.IdentifierName) &&
                (varDecl.Type as IdentifierNameSyntax)?.Identifier.Text == "var")
            {
                if (varDeclarator.Initializer != null)
                {
                    var initInfo = semanticModel.GetTypeInfo(varDeclarator.Initializer.Value);
                    if (initInfo.Type != null && IsActionType(initInfo.Type))
                        return true;
                }
            }

            return false;
        }

        private static bool IsActionType(ITypeSymbol type)
        {
            if (type == null)
                return false;

            var fullName = type.ToDisplayString();

            // Check for System.Action (non-generic)
            if (fullName == "System.Action")
                return true;

            // Check for System.Action<T> (only if it doesn't return anything)
            if (fullName.StartsWith("System.Action<"))
                return true;

            return false;
        }

        private static bool HasAsyncActionOkComment(LambdaExpressionSyntax expr)
        {
            var leadingTrivia = expr.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
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
                if (commentText.Contains("async-action-ok:"))
                    return true;
            }
            return false;
        }
    }
}
