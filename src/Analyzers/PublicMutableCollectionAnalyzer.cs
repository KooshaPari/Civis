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
    public class PublicMutableCollectionAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0123";
        private const string Category = "NuGetAPI";

        private static readonly LocalizableString Title =
            (LocalizableString)"Public class exposes mutable collection property";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Public property '{0}' exposes mutable '{1}<T>'. Use `IReadOnlyList<T> {{ get; init; }} = new List<T>();` for invariant protection. For YAML/JSON deserializer needs, use backing-field pattern + `[YamlIgnore]` accessor or document with `// public-mutable-ok: <reason>`.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Public mutable collection properties in NuGet-published libraries (SDK, Bridge.Client, Bridge.Protocol) break encapsulation and allow external callers to violate invariants. Use immutable properties (IReadOnlyList<T>, IReadOnlyCollection<T>) with backing fields for deserialization. For intentional mutable properties (e.g., deserializer requirements), document with `// public-mutable-ok: <reason>` to suppress.";

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
            context.RegisterSyntaxNodeAction(AnalyzeProperty, SyntaxKind.PropertyDeclaration);
        }

        private static void AnalyzeProperty(SyntaxNodeAnalysisContext context)
        {
            var property = (PropertyDeclarationSyntax)context.Node;

            // Skip if property is not public
            if (!property.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)))
                return;

            // Skip if containing class is not public
            var containingClass = property.Ancestors()
                .OfType<ClassDeclarationSyntax>()
                .FirstOrDefault();
            if (containingClass == null || !containingClass.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)))
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

        private static bool HasPublicMutableOkComment(PropertyDeclarationSyntax property)
        {
            // Check leading trivia for public-mutable-ok comment
            var leadingTrivia = property.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("public-mutable-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static string? GetMutableCollectionTypeName(ITypeSymbol? type, Compilation compilation)
        {
            if (type == null)
                return null;

            // Check for List<T>
            if (IsTypeMatch(type, "System.Collections.Generic.List`1", compilation))
                return "List";

            // Check for IList<T>
            if (IsTypeMatch(type, "System.Collections.Generic.IList`1", compilation))
                return "IList";

            // Check for Collection<T>
            if (IsTypeMatch(type, "System.Collections.ObjectModel.Collection`1", compilation))
                return "Collection";

            // Check for ICollection<T>
            if (IsTypeMatch(type, "System.Collections.Generic.ICollection`1", compilation))
                return "ICollection";

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
