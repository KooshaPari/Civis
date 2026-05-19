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
    public class StaticInitializerSideEffectAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1028";
        private const string Category = "Reliability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Static field initializer has side effect";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Static field '{0}' initializer creates an external resource. Defer to Lazy<T> or factory method.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Static field initializers that instantiate external resources (HttpClient, Process, File I/O, synchronization primitives, streams) block program startup and prevent resource cleanup control. Use Lazy<T> or factory methods instead. Use static-side-effect-ok: marker to suppress.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Info,
            isEnabledByDefault: true,
            description: Description,
            helpLinkUri: null);

        private static readonly string[] SideEffectTypeNames = new[]
        {
            "HttpClient",
            "Process",
            "Timer",
            "FileSystemWatcher",
            "NamedPipeServerStream",
            "NamedPipeClientStream",
            "Mutex",
            "Semaphore"
        };

        private static readonly string[] SideEffectNamespaces = new[]
        {
            "File",
            "Directory",
            "Environment",
            "Process",
            "Registry"
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
            var field = (FieldDeclarationSyntax)context.Node;

            // Skip if not static
            if (!field.Modifiers.Any(m => m.IsKind(SyntaxKind.StaticKeyword)))
                return;

            // Skip if no initializer
            var declaration = field.Declaration;
            if (declaration.Variables.Count == 0)
                return;

            var variable = declaration.Variables[0];
            if (variable.Initializer == null)
                return;

            // Skip if marked with static-side-effect-ok
            if (HasStaticSideEffectOkComment(field))
                return;

            var initializer = variable.Initializer.Value;

            // Check for object creation expression (new HttpClient(), new Process(), etc.)
            if (initializer is ObjectCreationExpressionSyntax objectCreation)
            {
                var typeName = GetTypeName(objectCreation.Type);
                if (SideEffectTypeNames.Contains(typeName))
                {
                    var diagnostic = Diagnostic.Create(
                        Rule,
                        variable.Identifier.GetLocation(),
                        variable.Identifier.Text);
                    context.ReportDiagnostic(diagnostic);
                    return;
                }
            }

            // Check for invocation expression (File.Read, Environment.GetEnvironmentVariable, etc.)
            if (initializer is InvocationExpressionSyntax invocation)
            {
                if (invocation.Expression is MemberAccessExpressionSyntax memberAccess)
                {
                    var receiverName = GetReceiverName(memberAccess.Expression);
                    if (SideEffectNamespaces.Contains(receiverName))
                    {
                        var diagnostic = Diagnostic.Create(
                            Rule,
                            variable.Identifier.GetLocation(),
                            variable.Identifier.Text);
                        context.ReportDiagnostic(diagnostic);
                        return;
                    }
                }
            }
        }

        private static string GetTypeName(TypeSyntax typeSyntax)
        {
            if (typeSyntax is IdentifierNameSyntax identifierName)
                return identifierName.Identifier.Text;

            if (typeSyntax is GenericNameSyntax genericName)
                return genericName.Identifier.Text;

            if (typeSyntax is QualifiedNameSyntax qualifiedName)
                return qualifiedName.Right.Identifier.Text;

            return string.Empty;
        }

        private static string GetReceiverName(ExpressionSyntax expression)
        {
            if (expression is IdentifierNameSyntax identifierName)
                return identifierName.Identifier.Text;

            if (expression is MemberAccessExpressionSyntax memberAccess)
                return GetReceiverName(memberAccess.Expression);

            return string.Empty;
        }

        private static bool HasStaticSideEffectOkComment(FieldDeclarationSyntax field)
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
                if (commentText.Contains("static-side-effect-ok:"))
                    return true;
            }

            return false;
        }
    }
}
