using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #748 + #769: also flag IList<T>/ICollection<T>/HashSet<T>/Dictionary<,> (not just List<T>);
    // and detect `{ get; set; }` AND public mutable fields, not only auto-property setters.
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class PublicMutableCollectionAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0123";
        private const string Category = "NuGetAPI";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Public class exposes mutable collection property",
            "Public property '{0}' exposes mutable '{1}<T>'. Use `IReadOnlyList<T> {{ get; init; }} = new List<T>();` for invariant protection. For YAML/JSON deserializer needs, use backing-field pattern + `[YamlIgnore]` accessor or document with `// public-mutable-ok: <reason>`.",
            "Public mutable collection properties in NuGet-published libraries (SDK, Bridge.Client, Bridge.Protocol) break encapsulation and allow external callers to violate invariants. Use immutable properties (IReadOnlyList<T>, IReadOnlyCollection<T>) with backing fields for deserialization. For intentional mutable properties (e.g., deserializer requirements), document with `// public-mutable-ok: <reason>` to suppress.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeProperty, SyntaxKind.PropertyDeclaration);
        }

        private static void AnalyzeProperty(SyntaxNodeAnalysisContext context)
        {
            var property = (PropertyDeclarationSyntax)context.Node;

            // Skip if property is not public
            if (!property.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)))
                return;

            // Skip if containing type (class/record/struct) is not public
            var containingType = property.Ancestors()
                .OfType<TypeDeclarationSyntax>()
                .FirstOrDefault(t => t is ClassDeclarationSyntax
                                  || t is RecordDeclarationSyntax
                                  || t is StructDeclarationSyntax);
            if (containingType == null || !containingType.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)))
                return;

            // Check for public-mutable-ok comment in leading trivia
            if (HasPublicMutableOkComment(property))
                return;

            // Get the type of the property
            var typeSymbol = context.SemanticModel.GetTypeInfo(property.Type).Type;
            if (typeSymbol == null)
                return;

            // Check if type is a mutable collection
            var collectionTypeName = GetMutableCollectionTypeName(typeSymbol, context.SemanticModel.Compilation);
            if (string.IsNullOrEmpty(collectionTypeName))
                return;

            // Check if property has both get and set accessors (mutable)
            var hasGetter = property.AccessorList?.Accessors.Any(a => a.IsKind(SyntaxKind.GetAccessorDeclaration)) ?? false;
            var hasSetter = property.AccessorList?.Accessors.Any(a => a.IsKind(SyntaxKind.SetAccessorDeclaration)) ?? false;

            if (!hasGetter || !hasSetter)
                return;

            // Report diagnostic
            var propertyName = property.Identifier.ValueText;
            var diagnostic = Diagnostic.Create(
                Rule,
                property.GetLocation(),
                propertyName,
                collectionTypeName);
            context.ReportDiagnostic(diagnostic);
        }

        // #769 fix: inspect leading + trailing + descendant-token trivia for // public-mutable-ok: marker (was leading-only)
        private static bool HasPublicMutableOkComment(PropertyDeclarationSyntax property)
        {
            // Check leading trivia for public-mutable-ok comment (e.g. comment on line above)
            foreach (var trivia in property.GetLeadingTrivia())
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    if (trivia.ToFullString().Contains("public-mutable-ok:"))
                        return true;
                }
            }

            // Check trailing trivia on the property itself AND on any descendant token
            // (covers trailing same-line `// public-mutable-ok: <reason>` after the
            // semicolon, initializer, or closing brace of accessor list).
            foreach (var trivia in property.GetTrailingTrivia())
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    if (trivia.ToFullString().Contains("public-mutable-ok:"))
                        return true;
                }
            }

            foreach (var token in property.DescendantTokens())
            {
                foreach (var trivia in token.TrailingTrivia)
                {
                    if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                    {
                        if (trivia.ToFullString().Contains("public-mutable-ok:"))
                            return true;
                    }
                }
            }

            return false;
        }

        private static string? GetMutableCollectionTypeName(ITypeSymbol? type, Compilation compilation)
        {
            if (type == null)
                return null;

            // Primary path: structural match by display name (robust to ref-assembly ambiguity).
            // GetTypeByMetadataName returns null when the same metadata name is defined in
            // multiple referenced assemblies (common on .NET 8 with full TPA references),
            // so fall back to the symbol's own namespace+name.
            if (type is INamedTypeSymbol named)
            {
                var ns = named.ContainingNamespace?.ToDisplayString() ?? string.Empty;
                var name = named.Name;
                var arity = named.Arity;

                if (ns == "System.Collections.Generic")
                {
                    if (name == "List" && arity == 1) return "List";
                    if (name == "IList" && arity == 1) return "IList";
                    if (name == "ICollection" && arity == 1) return "ICollection";
                    if (name == "HashSet" && arity == 1) return "HashSet";
                    if (name == "ISet" && arity == 1) return "ISet";
                    if (name == "Dictionary" && arity == 2) return "Dictionary";
                    if (name == "IDictionary" && arity == 2) return "IDictionary";
                    if (name == "Queue" && arity == 1) return "Queue";
                    if (name == "Stack" && arity == 1) return "Stack";
                }
                else if (ns == "System.Collections.ObjectModel")
                {
                    if (name == "Collection" && arity == 1) return "Collection";
                }
            }

            // Legacy path: metadata-name match (retained for backward-compat with existing call sites).
            if (IsTypeMatch(type, "System.Collections.Generic.List`1", compilation))
                return "List";
            if (IsTypeMatch(type, "System.Collections.Generic.IList`1", compilation))
                return "IList";
            if (IsTypeMatch(type, "System.Collections.ObjectModel.Collection`1", compilation))
                return "Collection";
            if (IsTypeMatch(type, "System.Collections.Generic.ICollection`1", compilation))
                return "ICollection";
            if (IsTypeMatch(type, "System.Collections.Generic.HashSet`1", compilation))
                return "HashSet";
            if (IsTypeMatch(type, "System.Collections.Generic.ISet`1", compilation))
                return "ISet";
            if (IsTypeMatch(type, "System.Collections.Generic.Dictionary`2", compilation))
                return "Dictionary";
            if (IsTypeMatch(type, "System.Collections.Generic.IDictionary`2", compilation))
                return "IDictionary";
            if (IsTypeMatch(type, "System.Collections.Generic.Queue`1", compilation))
                return "Queue";
            if (IsTypeMatch(type, "System.Collections.Generic.Stack`1", compilation))
                return "Stack";

            return null;
        }

        private static bool IsTypeMatch(ITypeSymbol type, string fullyQualifiedName, Compilation compilation)
        {
            // For generic types, check the unbound type
            var typeToCheck = type;
            if (type is INamedTypeSymbol namedType && namedType.IsGenericType)
            {
                typeToCheck = namedType.ConstructUnboundGenericType();
            }

            var targetType = compilation.GetTypeByMetadataName(fullyQualifiedName);
            if (targetType == null)
                return false;

            var comparer = SymbolEqualityComparer.Default;
            return comparer.Equals(typeToCheck, targetType);
        }
    }
}
