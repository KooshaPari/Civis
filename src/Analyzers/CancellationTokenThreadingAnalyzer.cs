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
    public class CancellationTokenThreadingAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0114";
        private const string Category = "Async";

#pragma warning disable RS1032
#pragma warning restore RS1032

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "CancellationToken not threaded to inner async call",
            "CancellationToken is not threaded to inner async call",
            "When a method accepts a CancellationToken parameter, all inner async calls should pass it to respect cancellation. Failing to thread the token prevents graceful cancellation propagation.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeMethod, SyntaxKind.MethodDeclaration);
        }

        private static void AnalyzeMethod(SyntaxNodeAnalysisContext context)
        {
            var methodDecl = (MethodDeclarationSyntax)context.Node;

            // Check if method is async and has Task return type
            if (!methodDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.AsyncKeyword)))
                return;

            var returnType = methodDecl.ReturnType;
            if (!IsTaskType(returnType))
                return;

            // Find CancellationToken parameter
            var ctParam = methodDecl.ParameterList?.Parameters
                .FirstOrDefault(p => IsCancellationTokenType(p.Type, context.SemanticModel));

            if (ctParam == null)
                return;

            var ctParamName = ctParam.Identifier.ValueText;

            // Walk all await expressions in the method body
            if (methodDecl.Body == null)
                return;

            var awaitExpressions = methodDecl.Body
                .DescendantNodes()
                .OfType<AwaitExpressionSyntax>();

            foreach (var awaitExpr in awaitExpressions)
            {
                // Get the awaited expression (should be an invocation)
                if (!(awaitExpr.Expression is InvocationExpressionSyntax invocation))
                    continue;

                // Check if this invocation passes the CancellationToken
                if (HasCancellationTokenArgument(invocation, ctParamName, context.SemanticModel))
                    continue;

                // Report diagnostic
                var diagnostic = Diagnostic.Create(Rule, awaitExpr.GetLocation());
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool IsTaskType(TypeSyntax? returnType)
        {
            if (returnType == null)
                return false;

            if (returnType is IdentifierNameSyntax identifier)
                return identifier.Identifier.ValueText == "Task";

            if (returnType is GenericNameSyntax generic)
                return generic.Identifier.ValueText == "Task";

            return false;
        }

        private static bool IsCancellationTokenType(TypeSyntax? paramType, SemanticModel semanticModel)
        {
            if (paramType == null)
                return false;

            var typeInfo = semanticModel.GetTypeInfo(paramType);
            if (typeInfo.Type == null)
                return false;

            var typeStr = typeInfo.Type.ToDisplayString();
            return typeStr == "System.Threading.CancellationToken";
        }

        private static bool HasCancellationTokenArgument(InvocationExpressionSyntax invocation, string ctParamName, SemanticModel semanticModel)
        {
            var args = invocation.ArgumentList?.Arguments ?? default;

            foreach (var arg in args)
            {
                // Check if argument is the CancellationToken parameter by name
                if (arg.Expression is IdentifierNameSyntax identifier &&
                    identifier.Identifier.ValueText == ctParamName)
                {
                    return true;
                }

                // Check for member access like "linkedCts.Token"
                if (arg.Expression is MemberAccessExpressionSyntax memberAccess &&
                    memberAccess.Name.Identifier.ValueText == "Token")
                {
                    // Simple heuristic: if it ends with .Token, assume it's a CT derivative
                    return true;
                }
            }

            return false;
        }

        private static string GetInvocationMethodName(InvocationExpressionSyntax invocation)
        {
            if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                return memberAccess.Name.Identifier.ValueText;
            }

            if (invocation.Expression is IdentifierNameSyntax identifier)
            {
                return identifier.Identifier.ValueText;
            }

            return "InvokedMethod";
        }
    }
}
