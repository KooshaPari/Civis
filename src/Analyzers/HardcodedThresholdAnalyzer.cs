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
    public class HardcodedThresholdAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1014";
        private const string Category = "Maintainability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Hardcoded numeric threshold should be extracted to a named constant";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Numeric literal '{0}' used as threshold or argument — extract to a named const/readonly to make tuning explicit. If intentional, document with `// threshold-ok: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Magic numeric literals (≥100) used as thresholds, timeouts, or method arguments reduce code clarity and maintainability. Extract these values to named const or readonly fields so future readers understand the semantics (e.g., 'MaxRetries', 'TimeoutMs', 'PollingIntervalMs') and can adjust them in one place. If the value is genuinely a semantic constant (e.g., 100 as part of a percentage calculation), document with `// threshold-ok: <reason>`.";

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
            context.RegisterSyntaxNodeAction(AnalyzeLiteral, SyntaxKind.NumericLiteralExpression);
        }

        private static void AnalyzeLiteral(SyntaxNodeAnalysisContext context)
        {
            var literalNode = (LiteralExpressionSyntax)context.Node;

            // #749 fix: attribute exemption walks Ancestors().OfType<AttributeSyntax>() (was short-circuit on ExpressionSyntax) + self-exemption on /Analyzers/ paths
            // Self-exemption: skip the analyzer's own source files to avoid recursive flagging
            var filePath = literalNode.SyntaxTree?.FilePath;
            if (filePath is not null && filePath.Length > 0)
            {
                var normalized = filePath.Replace('\\', '/');
                if (normalized.Contains("/src/Analyzers/") ||
                    normalized.Contains("/Analyzers/HardcodedThresholdAnalyzer"))
                    return;
            }

            // Attribute argument exemption: literals inside attributes (e.g. [Range(0, 100)]) are not thresholds
            if (IsInsideAttribute(literalNode))
                return;

            // Check for threshold-ok comment in leading trivia
            if (HasThresholdOkComment(literalNode))
                return;

            // Also check trailing trivia of the previous token
            var previousToken = literalNode.GetFirstToken().GetPreviousToken();
            if (previousToken != default && HasThresholdOkCommentInTrivia(previousToken.TrailingTrivia))
                return;

            // Parse the numeric value
            if (!TryParseNumericValue(literalNode, out var numValue))
                return;

            // Skip values < 100 (too common for semantic constants)
            if (numValue < 100)
                return;

            // Check parent context
            var parent = literalNode.Parent;

            // Case 1: Binary comparison (>, >=, <, <=)
            if (parent is BinaryExpressionSyntax binaryExpr)
            {
                if (binaryExpr.OperatorToken.IsKind(SyntaxKind.GreaterThanToken) ||
                    binaryExpr.OperatorToken.IsKind(SyntaxKind.GreaterThanEqualsToken) ||
                    binaryExpr.OperatorToken.IsKind(SyntaxKind.LessThanToken) ||
                    binaryExpr.OperatorToken.IsKind(SyntaxKind.LessThanEqualsToken))
                {
                    // Don't flag if this is inside a const/readonly field declaration
                    if (IsInsideConstOrReadonlyDeclaration(literalNode))
                        return;

                    var diagnostic = Diagnostic.Create(Rule, literalNode.GetLocation(), numValue);
                    context.ReportDiagnostic(diagnostic);
                    return;
                }
            }

            // Case 2: Method argument (positional argument in a method call)
            if (parent is ArgumentSyntax argSyntax)
            {
                // Don't flag if this is inside a const/readonly field declaration
                if (IsInsideConstOrReadonlyDeclaration(literalNode))
                    return;

                var diagnostic = Diagnostic.Create(Rule, literalNode.GetLocation(), numValue);
                context.ReportDiagnostic(diagnostic);
                return;
            }
        }

        private static bool TryParseNumericValue(LiteralExpressionSyntax literal, out long numValue)
        {
            numValue = 0;
            var token = literal.Token;

            try
            {
                var text = token.ValueText ?? token.Text;

                // Remove underscores (e.g., 1_000 → 1000)
                text = text.Replace("_", "");

                // Detect hex, octal, binary prefixes
                if (text.StartsWith("0x", StringComparison.OrdinalIgnoreCase))
                {
                    return long.TryParse(text.Substring(2), System.Globalization.NumberStyles.HexNumber, null, out numValue);
                }
                if (text.StartsWith("0b", StringComparison.OrdinalIgnoreCase))
                {
                    numValue = Convert.ToInt64(text.Substring(2), 2);
                    return true;
                }

                // Try decimal
                if (long.TryParse(text, out var decValue))
                {
                    numValue = decValue;
                    return true;
                }

                // Try double (for floating-point literals)
                if (double.TryParse(text, out var doubleValue))
                {
                    numValue = (long)doubleValue;
                    return true;
                }

                return false;
            }
            catch
            {
                return false;
            }
        }

        private static bool HasThresholdOkComment(LiteralExpressionSyntax literalNode)
        {
            var leadingTrivia = literalNode.GetLeadingTrivia();
            return HasThresholdOkCommentInTrivia(leadingTrivia);
        }

        private static bool HasThresholdOkCommentInTrivia(SyntaxTriviaList trivia)
        {
            foreach (var t in trivia)
            {
                if (t.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    t.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = t.ToFullString();
                    if (commentText.Contains("threshold-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool IsInsideConstOrReadonlyDeclaration(SyntaxNode node)
        {
            var current = node.Parent;

            while (current != null)
            {
                // Stop at field declaration and check modifiers
                if (current is FieldDeclarationSyntax fieldDecl)
                {
                    return fieldDecl.Modifiers.Any(m =>
                        m.IsKind(SyntaxKind.ConstKeyword) ||
                        m.IsKind(SyntaxKind.ReadOnlyKeyword));
                }

                // Stop at type declaration (class, struct, etc.)
                if (current is TypeDeclarationSyntax)
                    return false;

                current = current.Parent;
            }

            return false;
        }

        private static bool IsInsideAttribute(SyntaxNode node)
        {
            // Walk ancestors looking for an AttributeSyntax (or AttributeArgumentSyntax).
            // Stop only at a MemberDeclaration or StatementSyntax — those mark the end of an
            // attribute argument expression context. Do NOT stop on the first ExpressionSyntax
            // ancestor (the literal itself, BinaryExpression, etc. ARE ExpressionSyntax).
            foreach (var ancestor in node.Ancestors())
            {
                if (ancestor is AttributeSyntax || ancestor is AttributeArgumentSyntax || ancestor is AttributeArgumentListSyntax)
                    return true;

                if (ancestor is MemberDeclarationSyntax || ancestor is StatementSyntax)
                    return false;
            }

            return false;
        }
    }
}
