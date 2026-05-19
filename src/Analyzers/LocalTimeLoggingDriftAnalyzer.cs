using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class LocalTimeLoggingDriftAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0103";
        private const string Category = "Performance";

        private static readonly LocalizableString Title =
            (LocalizableString)"DateTime.Now used in logging context";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"DateTime.Now timestamp is local-time-dependent. Use DateTime.UtcNow for log persistence; format for human display only at read time.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Using DateTime.Now in logging timestamps creates time-dependent log entries that drift when system clocks change or logs are analyzed across timezones. Use DateTime.UtcNow for persistence and format only at display time.";

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
            context.RegisterSyntaxNodeAction(AnalyzeMemberAccess, SyntaxKind.SimpleMemberAccessExpression);
        }

        private static void AnalyzeMemberAccess(SyntaxNodeAnalysisContext context)
        {
            var memberAccess = (MemberAccessExpressionSyntax)context.Node;

            // Check if this is DateTime.Now
            if (memberAccess.Name.Identifier.ValueText != "Now")
                return;

            if (memberAccess.Expression is not IdentifierNameSyntax identifier)
                return;

            if (identifier.Identifier.ValueText != "DateTime")
                return;

            // Check for suppression comment: // local-time-ok: <reason>
            var leadingTrivia = memberAccess.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    var comment = trivia.ToString();
                    if (comment.Contains("// local-time-ok:"))
                        return;
                }
            }

            // Check if this is in a logging-related context
            if (!IsInLoggingContext(memberAccess, context))
                return;

            var diagnostic = Diagnostic.Create(Rule, memberAccess.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsInLoggingContext(MemberAccessExpressionSyntax memberAccess, SyntaxNodeAnalysisContext context)
        {
            // Walk up the syntax tree to check for logging-related patterns
            var current = memberAccess.Parent;

            while (current != null)
            {
                // Check if we're in an assignment to a variable named *log* or *timestamp*
                if (current is VariableDeclaratorSyntax varDeclarator)
                {
                    var varName = varDeclarator.Identifier.ValueText.ToLowerInvariant();
                    if (varName.Contains("log") || varName.Contains("timestamp"))
                        return true;
                }

                // Check if we're in a method call with "Log" in the name
                if (current is InvocationExpressionSyntax invocation)
                {
                    if (invocation.Expression is MemberAccessExpressionSyntax methodAccess)
                    {
                        var methodName = methodAccess.Name.Identifier.ValueText.ToLowerInvariant();
                        if (methodName.Contains("log") || methodName.Contains("write"))
                            return true;
                    }
                }

                // Check if we're in a CatchClauseSyntax (exception logging)
                if (current is CatchClauseSyntax)
                    return true;

                // Check if we're in a BinaryExpressionSyntax with AppendAllText or similar
                if (current is BinaryExpressionSyntax binaryExpr)
                {
                    if (binaryExpr.Left is InvocationExpressionSyntax leftInvoke &&
                        leftInvoke.Expression is MemberAccessExpressionSyntax leftMethod)
                    {
                        var methodName = leftMethod.Name.Identifier.ValueText.ToLowerInvariant();
                        if (methodName.Contains("append") || methodName.Contains("log"))
                            return true;
                    }
                }

                current = current.Parent;
            }

            return false;
        }
    }
}
