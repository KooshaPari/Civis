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
    public class ImplicitEncodingAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0106";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"File.ReadAllText/WriteAllText/ReadAllLines/WriteAllLines without explicit Encoding";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"File.ReadAllText without explicit Encoding detected. Use SafeFileIO.ReadText(path) or pass Encoding.UTF8 explicitly.";

        private static readonly LocalizableString Description =
            (LocalizableString)"File I/O without explicit Encoding falls back to system default (Windows: code page 1252, Linux: UTF-8). This causes silent data loss on non-UTF-8 systems. Always specify Encoding.UTF8 explicitly or use SafeFileIO wrapper.";

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

            // Check for implicit-encoding-ok comment in leading trivia
            if (HasImplicitEncodingOkComment(invocation))
                return;

            // Check if this is a File.ReadAllText/WriteAllText/ReadAllLines/WriteAllLines call
            if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                var methodName = memberAccess.Name.Identifier.ValueText;
                var isTargetMethod = methodName is "ReadAllText" or "WriteAllText" or "ReadAllLines" or "WriteAllLines";

                if (!isTargetMethod)
                    return;

                // Check if it's File.XXX (not some other type)
                if (memberAccess.Expression is IdentifierNameSyntax identifier)
                {
                    if (identifier.Identifier.ValueText != "File")
                        return;
                }
                else
                {
                    return;
                }

                // Check argument count and presence of Encoding
                var argCount = invocation.ArgumentList.Arguments.Count;
                var hasEncoding = HasEncodingArgument(invocation.ArgumentList);

                // ReadAllText(path) = 1 arg, no encoding → warn
                // WriteAllText(path, content) = 2 args, no encoding → warn
                // ReadAllLines(path) = 1 arg, no encoding → warn
                // WriteAllLines(path, lines) = 2 args, no encoding → warn
                // If 3rd arg is Encoding → allow
                if (!hasEncoding)
                {
                    var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
                    context.ReportDiagnostic(diagnostic);
                }
            }
        }

        private static bool HasImplicitEncodingOkComment(InvocationExpressionSyntax invocation)
        {
            // Check leading trivia for implicit-encoding-ok marker
            var leadingTrivia = invocation.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("implicit-encoding-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool HasEncodingArgument(ArgumentListSyntax argumentList)
        {
            // For ReadAllText/ReadAllLines: Encoding is 2nd arg
            // For WriteAllText/WriteAllLines: Encoding is 3rd arg
            // Check if any argument is an Encoding.<something> or IdentifierName "Encoding"
            foreach (var arg in argumentList.Arguments)
            {
                var argExpr = arg.Expression;

                // Check for Encoding.UTF8, Encoding.Unicode, etc.
                if (argExpr is MemberAccessExpressionSyntax memberAccess)
                {
                    if (memberAccess.Expression is IdentifierNameSyntax identifier &&
                        identifier.Identifier.ValueText == "Encoding")
                    {
                        return true;
                    }
                }

                // Check for plain Encoding variable
                if (argExpr is IdentifierNameSyntax idName &&
                    idName.Identifier.ValueText == "Encoding")
                {
                    return true;
                }
            }

            return false;
        }
    }
}
