using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class StaticMutableCollectionAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1001";
        private const string Category = "Concurrency";

        private static readonly LocalizableString Title =
            (LocalizableString)"Static mutable collection field modified without lock";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Static field `{0}` is a mutable collection; modifications from multiple threads risk race. Either: (a) initialize as `ConcurrentDictionary`/`ConcurrentBag`, (b) wrap modifications in `lock`, or (c) make it `static readonly` immutable view.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Static mutable collection fields (e.g., `static List<T>`, `static Dictionary<TKey, TValue>`, `static HashSet<T>`) accessed from multiple threads without synchronization are prone to race conditions. Tier 2 semantic analysis detects these declarations and recommends thread-safe alternatives.";

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
            context.RegisterSyntaxNodeAction(AnalyzeFieldDeclaration, SyntaxKind.FieldDeclaration);
        }

        private static void AnalyzeFieldDeclaration(SyntaxNodeAnalysisContext context)
        {
            var fieldDecl = (FieldDeclarationSyntax)context.Node;

            // Skip if has static-mutable-ok comment
            if (HasStaticMutableOkComment(fieldDecl))
                return;

            // Check if field is static
            var isStatic = fieldDecl.Modifiers.Any(SyntaxKind.StaticKeyword);
            if (!isStatic)
                return;

            // Check if field is readonly (readonly static collections are OK)
            var isReadonly = fieldDecl.Modifiers.Any(SyntaxKind.ReadOnlyKeyword);
            if (isReadonly)
                return;

            // Check type for mutable collections
            var typeSyntax = fieldDecl.Declaration.Type;
            if (IsMutableCollectionType(typeSyntax))
            {
                // Report diagnostic for each declarator
                foreach (var declarator in fieldDecl.Declaration.Variables)
                {
                    var diagnostic = Diagnostic.Create(Rule, declarator.GetLocation(), declarator.Identifier.ValueText);
                    context.ReportDiagnostic(diagnostic);
                }
            }
        }

        private static bool HasStaticMutableOkComment(FieldDeclarationSyntax fieldDecl)
        {
            var leadingTrivia = fieldDecl.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("static-mutable-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool IsMutableCollectionType(TypeSyntax typeSyntax)
        {
            // Handle generic types like List<T>, Dictionary<K,V>, HashSet<T>
            if (typeSyntax is GenericNameSyntax genericName)
            {
                var identifier = genericName.Identifier.ValueText;
                return identifier is "List" or "Dictionary" or "HashSet" or "Queue" or "Stack" or "SortedDictionary" or "SortedSet";
            }

            // Handle simple identifiers (e.g., unqualified List if using is present)
            if (typeSyntax is IdentifierNameSyntax identifierName)
            {
                var identifier = identifierName.Identifier.ValueText;
                return identifier is "List" or "Dictionary" or "HashSet" or "Queue" or "Stack" or "SortedDictionary" or "SortedSet";
            }

            // Handle qualified names like System.Collections.Generic.List<T>
            if (typeSyntax is QualifiedNameSyntax qualifiedName)
            {
                var rightPart = qualifiedName.Right;
                if (rightPart is GenericNameSyntax rightGeneric)
                {
                    var identifier = rightGeneric.Identifier.ValueText;
                    return identifier is "List" or "Dictionary" or "HashSet" or "Queue" or "Stack" or "SortedDictionary" or "SortedSet";
                }
            }

            return false;
        }
    }
}
