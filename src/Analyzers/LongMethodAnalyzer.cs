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
    public class LongMethodAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1015";
        private const string Category = "Maintainability";
        private const int LineThreshold = 60;

        private static readonly LocalizableString Title =
            (LocalizableString)"Method body exceeds 60 lines";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Method '{0}' has a body of {1} lines — consider decomposing into smaller helpers";

        private static readonly LocalizableString Description =
            (LocalizableString)"Methods with bodies exceeding 60 lines are hard to test, maintain, and understand. They often indicate mixed concerns or missing abstractions. Refactor into smaller, focused helpers or use intermediate state machine helpers. If the long method is justified (e.g., a dispatcher with many case labels or generated code), document with `// long-method-ok: <reason>`.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Info,
            isEnabledByDefault: true,
            description: Description,
            helpLinkUri: null);

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeMethod, SyntaxKind.MethodDeclaration);
        }

        private static void AnalyzeMethod(SyntaxNodeAnalysisContext context)
        {
            var methodDecl = (MethodDeclarationSyntax)context.Node;

            // Skip if method is expression-bodied (only arrow method bodies)
            if (methodDecl.ExpressionBody != null)
                return;

            // Skip if no block body
            if (methodDecl.Body == null)
                return;

            // #1015 fix: dispatcher exemption (5+ case labels) + GeneratedCode + CompilerGenerated suppression
            // Check for long-method-ok comment
            if (HasLongMethodOkComment(methodDecl))
                return;

            // Check for generated code attribute
            if (HasGeneratedCodeAttribute(methodDecl))
                return;

            // Check if in a .Generated.cs file
            var sourceTree = methodDecl.SyntaxTree;
            if (sourceTree.FilePath.Contains(".Generated.cs"))
                return;

            // Calculate body line count
            var bodyLineCount = CountMethodBodyLines(methodDecl.Body);

            // Skip if body is not too long
            if (bodyLineCount <= LineThreshold)
                return;

            // Check for dispatcher pattern (5+ case labels = likely switch dispatcher)
            var caseCount = CountCaseLabels(methodDecl.Body);
            if (caseCount >= 5)
                return;

            // Fire diagnostic
            var diagnostic = Diagnostic.Create(Rule, methodDecl.Identifier.GetLocation(), methodDecl.Identifier.Text, bodyLineCount);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasLongMethodOkComment(MethodDeclarationSyntax methodDecl)
        {
            // Check leading trivia of the method
            var leadingTrivia = methodDecl.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("long-method-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool HasGeneratedCodeAttribute(MethodDeclarationSyntax methodDecl)
        {
            // Check for [GeneratedCode] or [CompilerGenerated] attributes
            var generatedAttributes = new[] { "GeneratedCode", "CompilerGenerated" };

            foreach (var attrList in methodDecl.AttributeLists)
            {
                foreach (var attr in attrList.Attributes)
                {
                    var attrName = attr.Name switch
                    {
                        IdentifierNameSyntax id => id.Identifier.Text,
                        SimpleNameSyntax sn => sn.Identifier.Text,
                        _ => null
                    };

                    if (attrName != null && generatedAttributes.Contains(attrName))
                        return true;
                }
            }

            return false;
        }

        private static int CountMethodBodyLines(BlockSyntax body)
        {
            if (body == null)
                return 0;

            var startLine = body.GetLocation().GetLineSpan().StartLinePosition.Line;
            var endLine = body.GetLocation().GetLineSpan().EndLinePosition.Line;

            // Return the difference (inclusive)
            return endLine - startLine + 1;
        }

        private static int CountCaseLabels(BlockSyntax body)
        {
            if (body == null)
                return 0;

            // Walk the syntax tree and count CasePatternSwitchLabel and CaseSwitchLabel nodes
            return body.DescendantNodes()
                .OfType<CaseSwitchLabelSyntax>()
                .Count() + body.DescendantNodes()
                .OfType<CasePatternSwitchLabelSyntax>()
                .Count();
        }
    }
}
