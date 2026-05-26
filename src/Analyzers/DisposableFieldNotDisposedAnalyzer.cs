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
    public class DisposableFieldNotDisposedAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1006";
        private const string Category = "Reliability";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Disposable field in class not implementing IDisposable",
            "Field `{0}` of type `{1}` is disposable. The containing class `{2}` should implement IDisposable and dispose the field in a Dispose method, or use `using` at field initialization (e.g., field with initializer `= new {...}`).",
            "Classes that hold disposable resource fields (Stream, Reader, Writer, HttpClient, Pipe, CancellationTokenSource, etc.) should either (a) implement IDisposable and explicitly dispose the field in Dispose(), or (b) use a field initializer with a `using` declaration if the field is a local scoped resource. This prevents resource leaks.");

        private static readonly string[] DisposableTypeSuffixes = new[]
        {
            "Stream",
            "Reader",
            "Writer",
            "HttpClient",
            "NamedPipeServerStream",
            "NamedPipeClientStream",
            "CancellationTokenSource",
            "SemaphoreSlim",
            "ManualResetEvent",
            "ManualResetEventSlim",
            "Timer",
            "Connection",
            "Client",
            "Pipe",
            "Source"
        };

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeField, SyntaxKind.FieldDeclaration);
        }

        private static void AnalyzeField(SyntaxNodeAnalysisContext context)
        {
            var fieldDecl = (FieldDeclarationSyntax)context.Node;

            // Only analyze private or static fields (public fields are already a design issue)
            var isPrivate = fieldDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.PrivateKeyword));
            var isStatic = fieldDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.StaticKeyword));

            if (!isPrivate && !isStatic)
                return;

            // Get field type name
            var fieldTypeName = fieldDecl.Declaration.Type.ToString();

            // Check if field type ends with a known disposable suffix
            var isDisposableType = DisposableTypeSuffixes.Any(suffix =>
                fieldTypeName.EndsWith(suffix, StringComparison.Ordinal));

            if (!isDisposableType)
                return;

            // Skip if field has disposable-field-ok comment marker
            if (HasDisposableFieldOkComment(fieldDecl))
                return;

            // Find the containing class
            var containingClass = fieldDecl.Parent as ClassDeclarationSyntax;
            if (containingClass == null)
                return;

            // Check if class implements IDisposable or IAsyncDisposable
            var implementsDisposable = ContainsDisposableInterface(containingClass);

            if (implementsDisposable)
                return;

            // Report diagnostic for each variable in the field declaration
            foreach (var variable in fieldDecl.Declaration.Variables)
            {
                var diagnostic = Diagnostic.Create(
                    Rule,
                    variable.Identifier.GetLocation(),
                    variable.Identifier.Text,
                    fieldTypeName,
                    containingClass.Identifier.Text);
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool ContainsDisposableInterface(ClassDeclarationSyntax classDecl)
        {
            if (classDecl.BaseList == null)
                return false;

            foreach (var baseType in classDecl.BaseList.Types)
            {
                var baseTypeName = baseType.Type.ToString();

                // Check for IDisposable, IAsyncDisposable, or fully qualified versions
                if (baseTypeName == "IDisposable" ||
                    baseTypeName == "IAsyncDisposable" ||
                    baseTypeName == "System.IDisposable" ||
                    baseTypeName == "System.IAsyncDisposable")
                {
                    return true;
                }
            }

            return false;
        }

        private static bool HasDisposableFieldOkComment(FieldDeclarationSyntax field)
        {
            var leadingTrivia = field.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            return false;
        }

        private static bool CheckTrivia(SyntaxTrivia trivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("disposable-field-ok:"))
                    return true;
            }
            return false;
        }
    }
}
