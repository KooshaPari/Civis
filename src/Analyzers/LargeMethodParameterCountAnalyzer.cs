using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;
using System.Collections.Immutable;

namespace DINOForge.Analyzers;

/// <summary>
/// DF1026: Detects methods with >7 parameters, suggesting a parameter object pattern.
/// Methods with too many parameters are harder to call, test, and understand. Consider introducing a parameter object.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class LargeMethodParameterCountAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1026";

    private static readonly LocalizableString Title =
        (LocalizableString)"Method has too many parameters";

    private static readonly LocalizableString MessageFormat =
        (LocalizableString)"Method '{0}' has {1} parameters (threshold: 7) — consider introducing a parameter object";

    private static readonly LocalizableString Description =
        (LocalizableString)"Methods with >7 parameters indicate poor design and are difficult to call, test, and maintain. " +
        "Consider introducing a parameter object (DTO) to group related parameters. " +
        "Exempt with `// many-params-ok: <reason>` for special cases.";

    private const string Category = "Design";
    private const int ParameterThreshold = 7;

    private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
        DiagnosticId,
        Title,
        MessageFormat,
        Category,
        DiagnosticSeverity.Info,
        isEnabledByDefault: true,
        description: Description,
        helpLinkUri: "https://github.com/KooshaPari/Dino/blob/main/docs/analyzers/DF1026.md");

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics => ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(AnalyzeMethodDeclaration, SyntaxKind.MethodDeclaration);
    }

    private static void AnalyzeMethodDeclaration(SyntaxNodeAnalysisContext context)
    {
        var methodDeclaration = (MethodDeclarationSyntax)context.Node;

        // Skip if in generated file
        if (methodDeclaration.SyntaxTree.FilePath.EndsWith(".Generated.cs", System.StringComparison.OrdinalIgnoreCase))
        {
            return;
        }

        // Skip if in test file
        if (methodDeclaration.SyntaxTree.FilePath.ToLowerInvariant().Contains("tests"))
        {
            return;
        }

        // Skip constructors (different cohort)
        var parent = methodDeclaration.Parent as ClassDeclarationSyntax;
        if (parent != null && methodDeclaration.Identifier.ValueText == parent.Identifier.ValueText)
        {
            return;
        }

        // Check for suppression marker
        var leadingTrivia = methodDeclaration.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
            {
                var comment = trivia.ToString();
                if (comment.ToLowerInvariant().Contains("many-params-ok"))
                {
                    return;
                }
            }
        }

        var parameterCount = methodDeclaration.ParameterList.Parameters.Count;

        if (parameterCount > ParameterThreshold)
        {
            var diagnostic = Diagnostic.Create(
                Rule,
                methodDeclaration.Identifier.GetLocation(),
                methodDeclaration.Identifier.ValueText,
                parameterCount);

            context.ReportDiagnostic(diagnostic);
        }
    }
}
