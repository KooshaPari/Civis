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
    public class AsyncVoidEventHandlerAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1016";
        private const string Category = "Reliability";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Async void method should be Task-returning",
            "Method '{0}' is async void — exceptions are unobservable. Use async Task instead, or mark with `// async-void-ok: <reason>` if this is a legitimate event handler.",
            "The `async void` pattern is dangerous because exceptions thrown in the method cannot be caught by the caller, and there is no awaitable Task for synchronization. Use `async Task` instead. Annotate with `// async-void-ok: <reason>` only for event handlers.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.MethodDeclaration);
        }

        private static void Analyze(SyntaxNodeAnalysisContext ctx)
        {
            var method = (MethodDeclarationSyntax)ctx.Node;

            if (!DinoAnalyzerSyntaxHelpers.IsAsyncVoidMethod(method))
                return;

            if (DinoAnalyzerSyntaxHelpers.LeadingTriviaContains(method, "async-void-ok:"))
                return;

            // Skip test files
            var filePath = ctx.Node.SyntaxTree.FilePath;
            if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                method.Identifier.GetLocation(),
                method.Identifier.Text);
            ctx.ReportDiagnostic(diagnostic);
        }

    }
}
