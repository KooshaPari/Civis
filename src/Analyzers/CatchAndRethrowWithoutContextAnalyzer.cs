using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers;

/// <summary>
/// Analyzer DF1020: Catch and rethrow without preserving exception context.
/// Detects <c>throw new Exception(ex.Message)</c> patterns that drop the inner exception,
/// losing the original stack trace and exception chain. Pattern #104 enforcement at compile-time.
/// </summary>
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public class CatchAndRethrowWithoutContextAnalyzer : DiagnosticAnalyzer
{
    public const string DiagnosticId = "DF1020";

    private static readonly LocalizableString Title =
        (LocalizableString)"throw new Exception(ex.Message) loses exception context";

    private static readonly LocalizableString MessageFormat =
        (LocalizableString)"Throwing new {0} with only message loses inner exception chain; pass 'ex' as innerException";

    private static readonly LocalizableString Description =
        (LocalizableString)"Rethrowing with only ex.Message drops the original exception as innerException, losing the stack trace and exception chain. Always pass the caught exception as the innerException parameter. Exempt with `// catch-rethrow-ok: <reason>` comment for deliberate exception translation.";

    private const string Category = "Reliability";

    private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
        DiagnosticId,
        Title,
        MessageFormat,
        Category,
        DiagnosticSeverity.Warning,
        isEnabledByDefault: true,
        description: Description);

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
        ImmutableArray.Create(Rule);

    public override void Initialize(AnalysisContext context)
    {
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.EnableConcurrentExecution();
        context.RegisterSyntaxNodeAction(Analyze, SyntaxKind.ThrowStatement);
    }

    private static void Analyze(SyntaxNodeAnalysisContext ctx)
    {
        var throwStmt = (ThrowStatementSyntax)ctx.Node;
        var filePath = ctx.Node.SyntaxTree.FilePath;

        // Skip test files
        if (filePath.Contains("Tests", StringComparison.OrdinalIgnoreCase))
            return;

        // Skip generated files
        if (filePath.Contains(".Generated.cs", StringComparison.OrdinalIgnoreCase))
            return;

        // Skip if marked with catch-rethrow-ok marker
        if (HasCatchRethrowOkMarker(throwStmt))
            return;

        // Check if the throw expression is a new object creation
        if (throwStmt.Expression is not ObjectCreationExpressionSyntax objectCreation)
            return;

        // Find the enclosing catch clause
        var catchClause = throwStmt.Ancestors().OfType<CatchClauseSyntax>().FirstOrDefault();
        if (catchClause == null)
            return;

        // Only analyze if the catch clause declares an exception variable
        if (catchClause.Declaration?.Identifier.Text is not string caughtExceptionName)
            return;

        // Check if the throw statement is passing ex.Message but not ex itself
        if (!HasExceptionMessageArgument(objectCreation, caughtExceptionName))
            return;

        // Check if the caught exception is NOT already passed as an argument
        if (HasCaughtExceptionAsArgument(objectCreation, caughtExceptionName))
            return;

        // Get the exception type name from the object creation
        var exceptionTypeName = objectCreation.Type.ToString();

        // Report diagnostic
        var diagnostic = Diagnostic.Create(
            Rule,
            throwStmt.GetLocation(),
            exceptionTypeName);
        ctx.ReportDiagnostic(diagnostic);
    }

    /// <summary>
    /// Checks if the caught exception is already passed as an argument to the new exception constructor.
    /// </summary>
    private static bool HasCaughtExceptionAsArgument(ObjectCreationExpressionSyntax objectCreation, string caughtExceptionName)
    {
        var argumentList = objectCreation.ArgumentList;
        if (argumentList == null)
            return false;

        foreach (var arg in argumentList.Arguments)
        {
            // Check if the argument is the caught exception variable itself
            if (arg.Expression is IdentifierNameSyntax identifier &&
                identifier.Identifier.Text == caughtExceptionName)
                return true;

            // Check if the argument is a member access like "ex" (not "ex.Message")
            var argText = arg.Expression.ToString();
            if (argText == caughtExceptionName)
                return true;
        }

        return false;
    }

    /// <summary>
    /// Checks if the exception message is being passed (ex.Message, ex.InnerException.Message, etc.).
    /// </summary>
    private static bool HasExceptionMessageArgument(ObjectCreationExpressionSyntax objectCreation, string caughtExceptionName)
    {
        var argumentList = objectCreation.ArgumentList;
        if (argumentList == null)
            return false;

        foreach (var arg in argumentList.Arguments)
        {
            var argText = arg.Expression.ToString();

            // Check for direct property access: ex.Message, ex.InnerException, etc.
            if (argText.StartsWith(caughtExceptionName + ".", StringComparison.Ordinal))
                return true;
        }

        return false;
    }

    /// <summary>
    /// Checks for catch-rethrow-ok marker in trivia.
    /// </summary>
    private static bool HasCatchRethrowOkMarker(SyntaxNode node)
    {
        var leadingTrivia = node.GetLeadingTrivia();
        foreach (var trivia in leadingTrivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("catch-rethrow-ok:", StringComparison.OrdinalIgnoreCase))
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
                    if (commentText.Contains("catch-rethrow-ok:", StringComparison.OrdinalIgnoreCase))
                        return true;
                }
            }
        }

        return false;
    }
}
