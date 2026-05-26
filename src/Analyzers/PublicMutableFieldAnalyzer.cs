using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1018: Public mutable fields should be properties.
/// Detects <c>public</c> fields that lack <c>readonly</c>, <c>const</c>, or <c>static readonly</c> modifiers,
/// and flags them as encapsulation violations (non-NuGet-published classes exempted).
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class PublicMutableFieldAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1018";

    private const string Category = "Design";

    private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
        DiagnosticId,
        Category,
        DiagnosticSeverity.Info,
        "Public mutable field should be a property",
        "Public field '{0}' breaks encapsulation and binary compatibility — convert to property",
        "Public fields expose implementation details and prevent adding validation or change notifications later. Convert to auto-properties. Mark intentional exceptions with `// public-field-ok: <reason>` if semantically required.");

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics => ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(AnalyzeField, SyntaxKind.FieldDeclaration);
    }

    private void AnalyzeField(SyntaxNodeAnalysisContext ctx)
    {
        var fieldDecl = (FieldDeclarationSyntax)ctx.Node;

        // Skip if file is generated
        if (fieldDecl.SyntaxTree.FilePath.Contains(".Generated.cs"))
            return;

        // Check for public modifier
        if (!fieldDecl.Modifiers.Any(SyntaxKind.PublicKeyword))
            return;

        // Skip if const, readonly, or static readonly
        var modifiers = fieldDecl.Modifiers;
        if (modifiers.Any(SyntaxKind.ConstKeyword) ||
            modifiers.Any(SyntaxKind.ReadOnlyKeyword) ||
            (modifiers.Any(SyntaxKind.StaticKeyword) && modifiers.Any(SyntaxKind.ReadOnlyKeyword)))
            return;

        // Skip if parent is a struct
        var parent = fieldDecl.Parent;
        if (parent is StructDeclarationSyntax)
            return;

        // Skip if has public-field-ok marker in trivia
        var triviaStr = fieldDecl.GetLeadingTrivia().ToString();
        if (triviaStr.Contains("public-field-ok:"))
            return;

        // Report diagnostic for each variable
        foreach (var variable in fieldDecl.Declaration.Variables)
        {
            ctx.ReportDiagnostic(Diagnostic.Create(Rule, variable.GetLocation(), variable.Identifier.Text));
        }
    }
}
