using System;
using System.Collections.Immutable;
using System.Text.RegularExpressions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class UnboundedConstraintAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0094";
        private const string Category = "Design";
        private static readonly TimeSpan RegexMatchTimeout = TimeSpan.FromSeconds(1);

        private static readonly LocalizableString Title =
            (LocalizableString)"Unbounded version constraint";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Version constraint `{0}` has no upper bound. Specify a maximum (e.g. \">=0.1.0 <1.0.0\") to avoid accepting unintended future versions.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Version constraints without upper bounds (e.g., \">=0.1.0\", \">0.1.0\", \"*\") accept any future version, creating compatibility risk. Pack manifests and SDK defaults MUST specify explicit upper bounds to prevent breaking changes from unvetted versions. Example: change \">=0.1.0\" to \">=0.1.0 <1.0.0\".";

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
            context.RegisterSyntaxNodeAction(AnalyzeStringLiteral, SyntaxKind.StringLiteralExpression);
        }

        private static void AnalyzeStringLiteral(SyntaxNodeAnalysisContext context)
        {
            var literalExpr = (LiteralExpressionSyntax)context.Node;

            // Skip non-string literals
            if (!literalExpr.IsKind(SyntaxKind.StringLiteralExpression))
                return;

            // Extract the string value
            var stringValue = literalExpr.Token.ValueText;

            // Wildcard is a valid unbounded constraint signal even though it is one character.
            if (stringValue == "*")
            {
                if (HasUnboundedVersionOkComment(literalExpr))
                    return;

                var wildcardDiagnostic = Diagnostic.Create(Rule, literalExpr.GetLocation(), stringValue);
                context.ReportDiagnostic(wildcardDiagnostic);
                return;
            }

            // Skip very short strings (unlikely to be version constraints)
            if (stringValue.Length < 3)
                return;

            // Check for unbounded-version-ok marker in leading trivia
            if (HasUnboundedVersionOkComment(literalExpr))
                return;

            // Check if this string matches an unbounded version constraint pattern
            if (IsUnboundedVersionConstraint(stringValue))
            {
                var diagnostic = Diagnostic.Create(Rule, literalExpr.GetLocation(), stringValue);
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool HasUnboundedVersionOkComment(LiteralExpressionSyntax literalExpr)
        {
            foreach (var node in literalExpr.AncestorsAndSelf())
            {
                if (HasSuppressionComment(node.GetLeadingTrivia()) ||
                    HasSuppressionComment(node.GetTrailingTrivia()))
                {
                    return true;
                }
            }

            return false;
        }

        private static bool HasSuppressionComment(SyntaxTriviaList triviaList)
        {
            foreach (var trivia in triviaList)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("unbounded-version-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool IsUnboundedVersionConstraint(string constraint)
        {
            var trimmed = constraint.Trim();

            // Pattern 1: ">=X.Y.Z" with no upper bound (no space after version)
            // Match ">=0.1.0" but not ">=0.1.0 <1.0.0"
            if (Regex.IsMatch(trimmed, @"^\s*>=\d+\.\d+(\.\d+)?\s*$", RegexOptions.None, RegexMatchTimeout))
                return true;

            // Pattern 2: ">X.Y.Z" with no upper bound (strict lower, no upper)
            if (Regex.IsMatch(trimmed, @"^\s*>\d+\.\d+(\.\d+)?\s*$", RegexOptions.None, RegexMatchTimeout))
                return true;

            // Pattern 3: Wildcard "*" (accepts any version)
            if (trimmed == "*")
                return true;

            // Pattern 4: "~X.Y.Z" (loose tilde without explicit upper — tilde IS bounded per semver, so skip)
            // Pattern 5: "^X.Y.Z" (caret — caret IS bounded per semver, so skip)

            // Don't match patterns with explicit upper bounds:
            // ">=0.1.0 <1.0.0" has a space and upper bound
            // ">=0.1.0,<1.0.0" has a comma and upper bound
            if (trimmed.Contains(" <") || trimmed.Contains(",<") || trimmed.Contains(" <="))
                return false;

            return false;
        }
    }
}
