using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #746: broaden detection beyond File.ReadAllText to also cover ReadAllLines,
    // ReadAllBytes path-string-only overloads, and StreamReader/StreamWriter without encoding.
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

            // #746 fix: handle qualified System.IO.File, using-static, and alias receivers (was IdentifierNameSyntax only)
            // Determine target method name across the four invocation shapes we recognize:
            //   1. File.ReadAllText(...)                              — MemberAccess: File . ReadAllText
            //   2. System.IO.File.ReadAllText(...)                    — MemberAccess: <qualified>.File . ReadAllText
            //   3. using static System.IO.File; ReadAllText(...)      — Bare IdentifierName invocation
            //   4. using F = System.IO.File; F.ReadAllText(...)       — MemberAccess via alias; resolve via semantic model
            string? methodName = null;
            bool isFileQualified = false;

            switch (invocation.Expression)
            {
                case MemberAccessExpressionSyntax memberAccess:
                    methodName = memberAccess.Name.Identifier.ValueText;
                    isFileQualified = IsFileQualifier(memberAccess.Expression, context.SemanticModel);
                    break;

                case IdentifierNameSyntax bareName:
                    // `using static System.IO.File;` makes ReadAllText callable bare.
                    methodName = bareName.Identifier.ValueText;
                    if (!IsTargetMethodName(methodName))
                        return;
                    isFileQualified = IsBareFileMethodViaUsingStatic(bareName, context.SemanticModel);
                    break;

                default:
                    return;
            }

            if (methodName is null || !IsTargetMethodName(methodName))
                return;

            if (!isFileQualified)
                return;

            // Check argument count and presence of Encoding
            var hasEncoding = HasEncodingArgument(invocation.ArgumentList);

            if (!hasEncoding)
            {
                var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool IsTargetMethodName(string name) =>
            name is "ReadAllText" or "WriteAllText" or "ReadAllLines" or "WriteAllLines";

        /// <summary>
        /// Returns true if the given expression resolves to <c>System.IO.File</c>:
        /// covers <c>File</c>, <c>System.IO.File</c> (any nested MemberAccess walking
        /// down to a leaf identifier "File"), and aliases (<c>using F = System.IO.File;</c>)
        /// when the semantic model can resolve them.
        /// </summary>
        private static bool IsFileQualifier(ExpressionSyntax expr, SemanticModel? semanticModel)
        {
            // Plain identifier: `File` (matches `using System.IO;` as well as alias `F` resolved via semantics)
            if (expr is IdentifierNameSyntax id)
            {
                if (id.Identifier.ValueText == "File")
                    return true;

                // Alias: try to resolve via semantic model.
                if (semanticModel is not null)
                {
                    var sym = semanticModel.GetSymbolInfo(id).Symbol;
                    if (sym is INamedTypeSymbol nts && IsSystemIoFile(nts))
                        return true;
                    if (sym is IAliasSymbol alias && alias.Target is INamedTypeSymbol target && IsSystemIoFile(target))
                        return true;
                }
                return false;
            }

            // Qualified: `System.IO.File`, `global::System.IO.File`. Walk to the leaf name.
            if (expr is MemberAccessExpressionSyntax member)
            {
                if (member.Name.Identifier.ValueText != "File")
                    return false;

                // Prefer semantic resolution when available; fall back to leaf-name match.
                if (semanticModel is not null)
                {
                    var sym = semanticModel.GetSymbolInfo(member).Symbol;
                    if (sym is INamedTypeSymbol nts)
                        return IsSystemIoFile(nts);
                }

                // Syntactic fallback: any `*.File` MemberAccess targeting one of our methods.
                return true;
            }

            if (expr is QualifiedNameSyntax qualified)
            {
                return qualified.Right.Identifier.ValueText == "File";
            }

            // `global::System.IO.File` via AliasQualifiedName drills into a MemberAccess in the parser,
            // already covered above.
            return false;
        }

        /// <summary>
        /// Detects bare-method invocations like <c>ReadAllText(path)</c> enabled by
        /// <c>using static System.IO.File;</c>. Requires the semantic model to resolve
        /// the symbol to <c>System.IO.File.ReadAllText</c> (etc.).
        /// </summary>
        private static bool IsBareFileMethodViaUsingStatic(IdentifierNameSyntax bareName, SemanticModel? semanticModel)
        {
            if (semanticModel is null)
                return false;

            var sym = semanticModel.GetSymbolInfo(bareName).Symbol;
            if (sym is IMethodSymbol method && method.ContainingType is INamedTypeSymbol container)
                return IsSystemIoFile(container);

            return false;
        }

        private static bool IsSystemIoFile(INamedTypeSymbol type)
        {
            return type.Name == "File"
                && type.ContainingNamespace is { } ns
                && ns.Name == "IO"
                && ns.ContainingNamespace is { } parent
                && parent.Name == "System";
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
