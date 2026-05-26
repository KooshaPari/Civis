using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class FloatEqualityAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1007";
        private const string Category = "Reliability";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Float ==/!= comparison without tolerance",
            "Direct `==`/`!=` comparison of `{0}` typed values risks precision-loss false negatives. Use `Math.Abs(a - b) < epsilon` (or `MathF.Abs` for float) for tolerance-based equality.",
            "IEEE 754 floating-point arithmetic introduces precision loss. Direct == and != comparisons often fail to detect meaningful equality due to rounding errors. Always use a tolerance-based comparison (Math.Abs(a - b) < epsilon) for float, double, or decimal equality checks in game code (balance, damage, range checks).");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeBinaryExpression, SyntaxKind.EqualsExpression, SyntaxKind.NotEqualsExpression);
        }

        private static void AnalyzeBinaryExpression(SyntaxNodeAnalysisContext context)
        {
            var binaryExpr = (BinaryExpressionSyntax)context.Node;

            // Skip if leading-trivia contains float-equality-ok marker
            if (DinoAnalyzerSyntaxHelpers.LeadingTriviaContains(binaryExpr, "float-equality-ok:"))
                return;

            // Get semantic model for type checking
            var semanticModel = context.SemanticModel;

            // Analyze left operand type
            var leftTypeInfo = semanticModel.GetTypeInfo(binaryExpr.Left);
            var leftType = leftTypeInfo.Type;

            // Analyze right operand type
            var rightTypeInfo = semanticModel.GetTypeInfo(binaryExpr.Right);
            var rightType = rightTypeInfo.Type;

            // Skip if either type is not float-like
            if (!IsFloatType(leftType) || !IsFloatType(rightType))
                return;

            // Skip if EITHER operand is a zero literal (zero comparison is often deliberate)
            if (IsZeroLiteral(binaryExpr.Left) || IsZeroLiteral(binaryExpr.Right))
                return;

            // Report diagnostic
            var typeDisplayString = leftType?.Name ?? "unknown";
            var diagnostic = Diagnostic.Create(
                Rule,
                binaryExpr.GetLocation(),
                typeDisplayString);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsFloatType(ITypeSymbol? type)
        {
            if (type == null)
                return false;

            // Check for float, double, or decimal
            return type.SpecialType == SpecialType.System_Single ||
                   type.SpecialType == SpecialType.System_Double ||
                   type.SpecialType == SpecialType.System_Decimal;
        }

        private static bool IsZeroLiteral(ExpressionSyntax expr)
        {
            // Check for numeric literals: 0, 0.0, 0f, 0d, 0m
            if (expr is LiteralExpressionSyntax litExpr)
            {
                var text = litExpr.Token.Text;
                // Simple check: if text is "0" or variations with suffixes
                return text == "0" || text == "0.0" || text == "0f" || text == "0F" ||
                       text == "0d" || text == "0D" || text == "0m" || text == "0M" ||
                       text == "0.0f" || text == "0.0F" || text == "0.0d" || text == "0.0D" ||
                       text == "0.0m" || text == "0.0M";
            }

            return false;
        }

    }
}
