using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1022: Class holds IDisposable field but doesn't implement IDisposable.
/// Detects classes that have private/protected fields of types like HttpClient, Process,
/// CancellationTokenSource, Timer, etc., but don't implement IDisposable or IAsyncDisposable.
/// This is a resource leak risk — the fields won't be disposed when the class instance is discarded.
/// Pattern #224 enforcement at compile-time.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class IDisposableNotImplementedAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1022";

    private const string Category = "Reliability";

    private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
        DiagnosticId,
        Category,
        DiagnosticSeverity.Info,
        "Class holds IDisposable field but doesn't implement IDisposable",
        "Class '{0}' has IDisposable field '{1}' of type '{2}' but doesn't implement IDisposable — resource leak risk",
        "A class that holds fields of types like HttpClient, Process, Timer, SemaphoreSlim, etc., should implement IDisposable to ensure these resources are properly released. Exempt classes that inherit from MonoBehaviour, ComponentSystemBase, or SystemBase (which use OnDestroy). Suppress with `// idisposable-ok: <reason>` for deliberate cases.");

    private static readonly string[] DisposableFieldTypes = new[]
    {
        "HttpClient",
        "Process",
        "CancellationTokenSource",
        "Timer",
        "NamedPipeServerStream",
        "NamedPipeClientStream",
        "SemaphoreSlim",
        "ManualResetEventSlim",
        "AutoResetEvent",
        "Mutex",
        "FileStream",
        "StreamReader",
        "StreamWriter"
    };

    private static readonly string[] ExemptBaseTypes = new[]
    {
        "MonoBehaviour",
        "ComponentSystemBase",
        "SystemBase"
    };

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

        // Check if the class already implements IDisposable or IAsyncDisposable
        if (ImplementsIDisposable(classDecl))
            return;

        // Check if the class inherits from an exempt base type
        if (InheritsFromExemptType(classDecl, ctx.SemanticModel))
            return;

        // Find all fields that are IDisposable types
        foreach (var field in classDecl.Members.OfType<FieldDeclarationSyntax>())
        {
            // Skip if marked with idisposable-ok marker
            if (HasIDisposableOkMarker(field))
                continue;

            var fieldTypeName = field.Declaration.Type.ToString();

            // Check if this field is one of the known disposable types
            if (DisposableFieldTypes.Contains(fieldTypeName))
            {
                // Report for each variable in the field declaration
                foreach (var variable in field.Declaration.Variables)
                {
                    var diagnostic = Diagnostic.Create(
                        Rule,
                        variable.GetLocation(),
                        classDecl.Identifier.Text,
                        variable.Identifier.Text,
                        fieldTypeName);
                    ctx.ReportDiagnostic(diagnostic);
                }
            }
        }
    }

    /// <summary>
    /// Checks if the class implements IDisposable or IAsyncDisposable.
    /// </summary>
    private static bool ImplementsIDisposable(ClassDeclarationSyntax classDecl)
    {
        if (classDecl.BaseList == null)
            return false;

        foreach (var baseType in classDecl.BaseList.Types)
        {
            var typeName = baseType.Type.ToString();
            if (typeName == "IDisposable" || typeName == "IAsyncDisposable")
                return true;
        }

        return false;
    }

    /// <summary>
    /// Checks if the class inherits from an exempt base type (MonoBehaviour, ComponentSystemBase, SystemBase).
    /// </summary>
    private static bool InheritsFromExemptType(ClassDeclarationSyntax classDecl, SemanticModel semanticModel)
    {
        var classSymbol = semanticModel.GetDeclaredSymbol(classDecl);
        if (classSymbol == null)
            return false;

        var baseType = classSymbol.BaseType;
        while (baseType != null)
        {
            var fullName = baseType.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat);

            // Check against exempt base types (check by name since they may be from different assemblies)
            foreach (var exempt in ExemptBaseTypes)
            {
                if (fullName.Contains(exempt))
                    return true;
            }

            baseType = baseType.BaseType;
        }

        return false;
    }

    /// <summary>
    /// Checks for idisposable-ok marker in trivia.
    /// </summary>
    private static bool HasIDisposableOkMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("idisposable-ok:", StringComparison.OrdinalIgnoreCase))
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
                    if (commentText.Contains("idisposable-ok:", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }
}
