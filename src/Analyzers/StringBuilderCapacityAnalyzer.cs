using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class StringBuilderCapacityAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0117";
        private const string Category = "Performance";

        private static readonly LocalizableString Title =
            (LocalizableString)"StringBuilder created without capacity hint";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"StringBuilder created without capacity hint. `new StringBuilder()` allocates default 16-char capacity. If the expected output size is knowable, pass `new StringBuilder(estimatedCapacity)` to avoid reallocation. For unbounded loops, default to 4096.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Preallocating StringBuilder capacity avoids multiple internal buffer reallocations as the string grows, reducing GC pressure and improving performance.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Info,
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

            // Match: new StringBuilder() with NO arguments
            if (!IsStringBuilderCreation(objectCreation))
                return;

            // If ArgumentList is null OR has 0 arguments, flag it
            var argumentList = objectCreation.ArgumentList;
            if (argumentList != null && argumentList.Arguments.Count > 0)
                return; // Has arguments, don't flag

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, objectCreation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsStringBuilderCreation(ObjectCreationExpressionSyntax objectCreation)
        {
            var typeName = objectCreation.Type;

            if (typeName is IdentifierNameSyntax identifier)
            {
                return identifier.Identifier.ValueText == "StringBuilder";
            }

            if (typeName is QualifiedNameSyntax qualified)
            {
                // Handle System.Text.StringBuilder
                return qualified.Right is IdentifierNameSyntax rightId &&
                       rightId.Identifier.ValueText == "StringBuilder";
            }

            return false;
        }
    }
}
