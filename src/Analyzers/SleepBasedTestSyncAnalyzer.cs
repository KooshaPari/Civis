using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class SleepBasedTestSyncAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0108";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Sleep-based test synchronization is fragile";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Thread.Sleep or Task.Delay in test methods is fragile across environments. Use TestWait.UntilAsync(predicate, timeout) instead for robust polling.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Fixed-duration sleeps in tests create environment-dependent flakiness (slow CI, fast local). Use polling with timeout instead.";

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
            context.RegisterSyntaxNodeAction(AnalyzeInvocation, SyntaxKind.InvocationExpression);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            // Check if this is in a test file
            var filePath = context.SemanticModel.SyntaxTree.FilePath;
            if (!IsInTestFile(filePath))
                return;

            // Check for Thread.Sleep or Task.Delay invocation
            if (!IsSleepOrDelayCall(invocation))
                return;

            // Check for suppression comment: // test-sleep-ok: <reason>
            var leadingTrivia = invocation.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    var comment = trivia.ToString();
                    if (comment.Contains("// test-sleep-ok:"))
                        return;
                }
            }

            var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsInTestFile(string filePath)
        {
            return filePath.Contains("\\Tests\\") || filePath.Contains("/Tests/");
        }

        private static bool IsSleepOrDelayCall(InvocationExpressionSyntax invocation)
        {
            // Match Thread.Sleep(...)
            if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                var methodName = memberAccess.Name.Identifier.ValueText;
                if (methodName == "Sleep" || methodName == "Delay")
                {
                    // Check if it's Thread.Sleep
                    if (memberAccess.Expression is IdentifierNameSyntax identifier)
                    {
                        if (identifier.Identifier.ValueText == "Thread")
                            return true;
                        if (identifier.Identifier.ValueText == "Task")
                            return true;
                    }
                }
            }

            // Match direct Sleep/Delay calls (unlikely but possible with using static)
            if (invocation.Expression is IdentifierNameSyntax methodIdent)
            {
                var methodName = methodIdent.Identifier.ValueText;
                if (methodName == "Sleep" || methodName == "Delay")
                    return true;
            }

            return false;
        }
    }
}
