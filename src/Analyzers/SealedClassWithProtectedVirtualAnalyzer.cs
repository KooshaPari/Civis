using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1021: Sealed class has unreachable protected virtual members.
/// Detects <c>protected virtual</c> or <c>protected abstract</c> members in sealed classes.
/// Sealed classes cannot be inherited, so such members are dead code (cannot be overridden,
/// cannot be accessed from derived classes). This indicates either:
/// 1. The class should not be sealed, or
/// 2. The members should be private/internal instead of protected virtual.
/// Pattern #124 enforcement at compile-time.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class SealedClassWithProtectedVirtualAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1021";

    private const string Category = "Design";

    private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
        DiagnosticId,
        Category,
        DiagnosticSeverity.Warning,
        "Sealed class has unreachable protected virtual/abstract members",
        "Member '{0}' in sealed class '{1}' uses 'protected virtual' or 'protected abstract' but can never be overridden — make it private/internal or unseal the class",
        "A sealed class cannot be inherited, so protected virtual or abstract members are unreachable dead code. Either unseal the class to allow inheritance, or change the member accessibility to private or internal. Exempt with `// sealed-virtual-ok: <reason>` for deliberate cases.");

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.ClassDeclaration);
    }

    private static void Analyze(SyntaxNodeAnalysisContext ctx)
    {
        var classDecl = (ClassDeclarationSyntax)ctx.Node;
        var filePath = ctx.Node.SyntaxTree.FilePath;

        // Skip test files
        if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
            return;

        // Skip generated files
        if (filePath.Contains(".Generated.cs", StringComparison.OrdinalIgnoreCase))
            return;

        // Check if the class is sealed
        if (!classDecl.Modifiers.Any(SyntaxKind.SealedKeyword))
            return;

        // Check each member for protected virtual/abstract
        foreach (var member in classDecl.Members)
        {
            if (IsProtectedVirtualOrAbstractMember(member))
            {
                // Skip if marked with sealed-virtual-ok marker
                if (HasSealedVirtualOkMarker(member))
                    continue;

                // Check if this is truly a new virtual/abstract (not an override from base)
                var semanticModel = ctx.SemanticModel;
                var memberSymbol = semanticModel.GetDeclaredSymbol(member);

                // Only report if the member is virtual/abstract but not an override
                if (memberSymbol is IMethodSymbol methodSymbol)
                {
                    if ((methodSymbol.IsVirtual || methodSymbol.IsAbstract) && !methodSymbol.IsOverride)
                    {
                        var diagnostic = Diagnostic.Create(
                            Rule,
                            member.GetLocation(),
                            methodSymbol.Name,
                            classDecl.Identifier.Text);
                        ctx.ReportDiagnostic(diagnostic);
                    }
                }
                else if (memberSymbol is IPropertySymbol propSymbol)
                {
                    if ((propSymbol.IsVirtual || propSymbol.IsAbstract) && !propSymbol.IsOverride)
                    {
                        var diagnostic = Diagnostic.Create(
                            Rule,
                            member.GetLocation(),
                            propSymbol.Name,
                            classDecl.Identifier.Text);
                        ctx.ReportDiagnostic(diagnostic);
                    }
                }
            }
        }
    }

    /// <summary>
    /// Checks if a member has both 'protected' and ('virtual' or 'abstract') modifiers.
    /// </summary>
    private static bool IsProtectedVirtualOrAbstractMember(MemberDeclarationSyntax member)
    {
        var modifiers = member.Modifiers;

        // Must have protected modifier
        if (!modifiers.Any(SyntaxKind.ProtectedKeyword))
            return false;

        // Must have either virtual or abstract modifier
        var hasVirtual = modifiers.Any(SyntaxKind.VirtualKeyword);
        var hasAbstract = modifiers.Any(SyntaxKind.AbstractKeyword);

        return hasVirtual || hasAbstract;
    }

    /// <summary>
    /// Checks for sealed-virtual-ok marker in trivia.
    /// </summary>
    private static bool HasSealedVirtualOkMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("sealed-virtual-ok:", StringComparison.OrdinalIgnoreCase))
                    return true;
            }
        }

        // Check trailing trivia of the previous token
        var previousToken = node.GetFirstToken().GetPreviousToken();
        if (previousToken != default && previousToken.HasTrailingTrivia)
        {
            foreach (var trivia in previousToken.TrailingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("sealed-virtual-ok:", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }
}
