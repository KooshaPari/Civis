using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class UnusedPrivateFieldAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1024";
        private const string Category = "Maintainability";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Info,
            "Unused private field",
            "Private field '{0}' is declared but never read or written — remove it or use it",
            "Private fields that are declared but never referenced anywhere in the code are dead state. They should be removed to reduce code clutter and improve maintainability. Use // unused-field-ok: <reason> to suppress if the field is intentionally reserved or used via reflection.");

        private static readonly string[] ExemptAttributes = new[]
        {
            "SerializeField",
            "JsonProperty",
            "YamlMember",
            "FieldOffset",
            "NonSerialized"
        };

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.Analyze);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxTreeAction(AnalyzeSyntaxTree);
        }

        private static void AnalyzeSyntaxTree(SyntaxTreeAnalysisContext context)
        {
            var root = context.Tree.GetRoot();
            var sourceFile = context.Tree.FilePath;

            // Skip generated files
            if (sourceFile.EndsWith(".Generated.cs", StringComparison.OrdinalIgnoreCase))
                return;

            // Skip test files
            if (sourceFile.Contains("\\Tests\\", StringComparison.OrdinalIgnoreCase) ||
                sourceFile.Contains("/Tests/", StringComparison.OrdinalIgnoreCase))
                return;

            // Build a set of all identifier names used in the file (non-declaration contexts)
            var usedIdentifiers = new HashSet<string>();
            var walker = new IdentifierUsageWalker();
            walker.Visit(root);
            usedIdentifiers = walker.UsedIdentifiers;

            // Find all private field declarations in class/struct contexts
            var fieldWalker = new PrivateFieldWalker();
            fieldWalker.Visit(root);

            foreach (var fieldDecl in fieldWalker.PrivateFields)
            {
                // Check if any variable in this field declaration has an exempting attribute
                var hasExemption = fieldDecl.Declaration.Variables.Any(v =>
                    fieldDecl.AttributeLists.Any(attrList =>
                        attrList.Attributes.Any(attr =>
                            ExemptAttributes.Any(exemptName =>
                                attr.Name.ToString().EndsWith(exemptName, StringComparison.Ordinal)))));

                if (hasExemption)
                    continue;

                // Check each variable in the declaration
                foreach (var variable in fieldDecl.Declaration.Variables)
                {
                    var fieldName = variable.Identifier.Text;

                    // Check for suppression marker in trivia
                    var leadingTrivia = fieldDecl.GetLeadingTrivia();
                    if (HasSuppressionMarker(leadingTrivia))
                        continue;

                    // If field name is not in usage set, report diagnostic
                    if (!usedIdentifiers.Contains(fieldName))
                    {
                        var diagnostic = Diagnostic.Create(
                            Rule,
                            variable.GetLocation(),
                            fieldName);
                        context.ReportDiagnostic(diagnostic);
                    }
                }
            }
        }

        private static bool HasSuppressionMarker(SyntaxTriviaList trivia)
        {
            foreach (var t in trivia)
            {
                if (t.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    var text = t.ToFullString();
                    if (text.Contains("unused-field-ok:", StringComparison.Ordinal))
                        return true;
                }
            }
            return false;
        }

        /// <summary>
        /// Collects all IdentifierNameSyntax used in the syntax tree (excluding variable declarations).
        /// </summary>
        private class IdentifierUsageWalker : CSharpSyntaxWalker
        {
            public HashSet<string> UsedIdentifiers { get; } = new HashSet<string>();

            public override void VisitIdentifierName(IdentifierNameSyntax node)
            {
                // Skip if this identifier is part of a variable declaration
                var parent = node.Parent;
                if (parent is VariableDeclaratorSyntax or FieldDeclarationSyntax or
                    LocalDeclarationStatementSyntax or ParameterSyntax or
                    PropertyDeclarationSyntax or MethodDeclarationSyntax or
                    EnumMemberDeclarationSyntax or EventFieldDeclarationSyntax)
                {
                    base.VisitIdentifierName(node);
                    return;
                }

                UsedIdentifiers.Add(node.Identifier.Text);
                base.VisitIdentifierName(node);
            }
        }

        /// <summary>
        /// Collects all private field declarations in class/struct contexts.
        /// </summary>
        private class PrivateFieldWalker : CSharpSyntaxWalker
        {
            public List<FieldDeclarationSyntax> PrivateFields { get; } = new List<FieldDeclarationSyntax>();

            public override void VisitFieldDeclaration(FieldDeclarationSyntax node)
            {
                // Only analyze private fields (not public, protected, internal, or static)
                var isPrivate = node.Modifiers.Any(m => m.IsKind(SyntaxKind.PrivateKeyword));
                var isStatic = node.Modifiers.Any(m => m.IsKind(SyntaxKind.StaticKeyword));

                // Also only analyze in class/struct contexts
                var parent = node.Parent;
                var isInTypeDeclaration = parent is ClassDeclarationSyntax or StructDeclarationSyntax;

                if (isPrivate && !isStatic && isInTypeDeclaration)
                {
                    PrivateFields.Add(node);
                }

                base.VisitFieldDeclaration(node);
            }
        }
    }
}
