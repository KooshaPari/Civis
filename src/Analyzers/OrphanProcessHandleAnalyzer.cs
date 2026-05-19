using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class OrphanProcessHandleAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0102";
        private const string Category = "Resource Management";

        private static readonly LocalizableString Title =
            (LocalizableString)"Process.Start without using or assignment";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"`Process.Start(...)` returns a Process handle. Wrap in `using var p = Process.Start(...)` or assign to a field/variable that's properly disposed to avoid resource leaks.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Process.Start() returns a Process handle. Discarding it without wrapping in `using` causes handle leaks. Wrap fire-and-forget calls in `using var _ = Process.Start(...)` or assign to a properly-disposed field or local variable.";

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

            // Match: Process.Start(...)
            if (!IsProcessStartCall(invocation))
                return;

            // Check if the invocation is wrapped in a using statement
            if (IsWrappedInUsing(invocation))
                return;

            // Check if the invocation is assigned to a variable
            if (IsAssigned(invocation))
                return;

            // Report diagnostic
            var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
            context.ReportDiagnostic(diagnostic);
        }

        private static bool IsProcessStartCall(InvocationExpressionSyntax invocation)
        {
            var memberAccess = invocation.Expression as MemberAccessExpressionSyntax;
            if (memberAccess == null)
                return false;

            // Check for .Start method name
            if (memberAccess.Name.Identifier.ValueText != "Start")
                return false;

            // Check for Process type (simple heuristic: identifier is Process)
            if (memberAccess.Expression is IdentifierNameSyntax identifier)
            {
                return identifier.Identifier.ValueText == "Process";
            }

            // Handle System.Diagnostics.Process.Start
            if (memberAccess.Expression is MemberAccessExpressionSyntax qualifiedAccess)
            {
                var rightmost = GetRightmostIdentifier(qualifiedAccess);
                return rightmost == "Process";
            }

            return false;
        }

        private static bool IsWrappedInUsing(InvocationExpressionSyntax invocation)
        {
            // Walk up the syntax tree to see if we're inside a using statement
            var parent = invocation.Parent;
            while (parent != null)
            {
                if (parent is UsingStatementSyntax)
                    return true;

                // Check for using declaration (e.g., using var x = Process.Start(...))
                // This is represented in the AST as a LocalDeclarationStatementSyntax with UsingKeyword
                if (parent is LocalDeclarationStatementSyntax localDecl)
                {
                    if (localDecl.UsingKeyword.RawKind != 0)  // Non-zero means the using keyword is present
                        return true;
                }

                parent = parent.Parent;
            }

            return false;
        }

        private static bool IsAssigned(InvocationExpressionSyntax invocation)
        {
            var parent = invocation.Parent;
            if (parent == null)
                return false;

            // Check if parent is an assignment (e.g., var p = Process.Start(...))
            if (parent is AssignmentExpressionSyntax)
                return true;

            // Check if parent is a variable declarator (e.g., var p = Process.Start(...))
            if (parent is VariableDeclaratorSyntax)
                return true;

            // Check if parent is an initializer (e.g., new SomeClass { Process = Process.Start(...) })
            if (parent is EqualsValueClauseSyntax)
                return true;

            return false;
        }

        private static string GetRightmostIdentifier(MemberAccessExpressionSyntax memberAccess)
        {
            return memberAccess.Name.Identifier.ValueText;
        }
    }
}
