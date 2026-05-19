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
    public class AsyncVoidAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1005";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"async void method outside event-handler context";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`async void` is unsafe outside event-handler signatures (caller can't catch exceptions). Use `async Task` instead, or annotate with `// async-void-ok: <reason>` if it's a legitimate event handler.";

        private static readonly LocalizableString Description =
            (LocalizableString)"The `async void` pattern is dangerous because exceptions thrown in the method cannot be caught by the caller, and there is no awaitable Task for synchronization. The only legitimate use is for event handlers with the signature `void MethodName(object sender, EventArgs e)`. For all other cases, use `async Task` or `async Task<T>`.";

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
            context.RegisterSyntaxNodeAction(AnalyzeMethod, SyntaxKind.MethodDeclaration);
        }

        private static void AnalyzeMethod(SyntaxNodeAnalysisContext context)
        {
            var method = (MethodDeclarationSyntax)context.Node;

            // Skip if not marked as async
            if (!method.Modifiers.Any(m => m.IsKind(SyntaxKind.AsyncKeyword)))
                return;

            // Skip if return type is not void
            if (!method.ReturnType.ToString().Equals("void", StringComparison.Ordinal))
                return;

            // Skip if marked with async-void-ok
            if (HasAsyncVoidOkComment(method))
                return;

            // Check if this is a legitimate event handler pattern:
            // - 2 parameters
            // - second parameter type ends with "EventArgs" or is exactly "EventArgs"
            if (IsEventHandlerPattern(method))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                method.Identifier.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsEventHandlerPattern(MethodDeclarationSyntax method)
        {
            var parameters = method.ParameterList.Parameters;

            // Event handlers typically have 2 parameters: (object sender, EventArgs e)
            if (parameters.Count != 2)
                return false;

            // Second parameter type should be EventArgs or derive from it
            var secondParamType = parameters[1].Type?.ToString() ?? string.Empty;

            return secondParamType.EndsWith("EventArgs", StringComparison.Ordinal) ||
                   secondParamType == "EventArgs";
        }

        private static bool HasAsyncVoidOkComment(MethodDeclarationSyntax method)
        {
            // Check leading trivia
            var leadingTrivia = method.GetLeadingTrivia();
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
                if (commentText.Contains("async-void-ok:"))
                    return true;
            }
            return false;
        }
    }
}
