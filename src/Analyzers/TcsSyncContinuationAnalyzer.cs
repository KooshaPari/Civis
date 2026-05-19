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
            (LocalizableString)"TaskCompletionSource ctor without TaskCreationOptions.RunContinuationsAsynchronously risks sync-continuation deadlock. Pass `TaskCreationOptions.RunContinuationsAsynchronously` to the constructor.";

        private static readonly LocalizableString Description =
            (LocalizableString)"TaskCompletionSource without TaskCreationOptions.RunContinuationsAsynchronously runs continuations synchronously on the producer's thread, causing main-thread starvation and potential deadlocks in cross-thread marshalling contexts. Always pass TaskCreationOptions.RunContinuationsAsynchronously.";

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

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
            context.ReportDiagnostic(diagnostic);
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
