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
    public class UnsealedConcreteMutableClassAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1013";
        private const string Category = "Design";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Info,
            "Public concrete class with mutable state should be sealed or designed for inheritance",
            "Public class '{0}' is not sealed and has mutable private state but defines no virtual/abstract members for inheritance — seal it or add an extension point. If unsealable by design, document with `// unsealed-ok: <reason>`.",
            "Public concrete classes with mutable private state should either be sealed (if inheritance is not intended) or define protected virtual/abstract members (if inheritance is intended). An unsealed class with mutable state but no extension points creates a maintenance liability: subclasses can capture mutable state without proper lifecycle management. Seal the class or explicitly design for inheritance by adding protected virtual methods. If unsealable by design, document with `// unsealed-ok: <reason>`.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeClass, SyntaxKind.ClassDeclaration);
        }

        private static void AnalyzeClass(SyntaxNodeAnalysisContext context)
        {
            var classDecl = (ClassDeclarationSyntax)context.Node;

            // Check for unsealed-ok comment in leading trivia
            if (HasUnsealdedOkComment(classDecl))
                return;

            // Must be public
            var hasPublicModifier = classDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword));
            if (!hasPublicModifier)
                return;

            // Must NOT be sealed, abstract, or static
            if (classDecl.Modifiers.Any(m =>
                m.IsKind(SyntaxKind.SealedKeyword) ||
                m.IsKind(SyntaxKind.AbstractKeyword) ||
                m.IsKind(SyntaxKind.StaticKeyword)))
                return;

            // EXEMPT: Framework base classes (MonoBehaviour, ComponentSystemBase, etc.)
            if (IsFrameworkBaseClass(classDecl))
                return;

            // EXEMPT: Has [Serializable] attribute (likely Unity-serialized)
            if (HasSerializableAttribute(classDecl))
                return;

            // Check for private mutable fields
            if (!HasPrivateMutableFields(classDecl))
                return;

            // Check for protected virtual/abstract members (if any exist, inheritance is designed)
            if (HasProtectedVirtualOrAbstractMembers(classDecl))
                return;

            // All conditions met: fire the diagnostic
            var diagnostic = Diagnostic.Create(Rule, classDecl.Identifier.GetLocation(), classDecl.Identifier.Text);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasUnsealdedOkComment(ClassDeclarationSyntax classDecl)
        {
            // Check leading trivia for unsealed-ok marker
            var leadingTrivia = classDecl.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var commentText = trivia.ToFullString();
                    if (commentText.Contains("unsealed-ok:"))
                        return true;
                }
            }

            return false;
        }

        private static bool IsFrameworkBaseClass(ClassDeclarationSyntax classDecl)
        {
            // Check if this class inherits from a framework base class
            if (classDecl.BaseList == null)
                return false;

            var frameworkBaseNames = new[] { "MonoBehaviour", "ComponentSystemBase", "SystemBase", "ViewModelBase" };

            foreach (var baseType in classDecl.BaseList.Types)
            {
                var typeName = baseType.Type switch
                {
                    IdentifierNameSyntax id => id.Identifier.Text,
                    GenericNameSyntax gn => gn.Identifier.Text,
                    _ => null
                };

                // Check exact match with framework bases
                if (typeName != null && frameworkBaseNames.Contains(typeName))
                    return true;

                // Check for Avalonia.Controls.* inheritance
                if (baseType.Type is QualifiedNameSyntax qualName)
                {
                    var fullName = qualName.ToString();
                    if (fullName.StartsWith("Avalonia.Controls"))
                        return true;
                }
            }

            return false;
        }

        private static bool HasSerializableAttribute(ClassDeclarationSyntax classDecl)
        {
            // Check if class has [Serializable] attribute
            foreach (var attrList in classDecl.AttributeLists)
            {
                foreach (var attr in attrList.Attributes)
                {
                    var attrName = attr.Name switch
                    {
                        IdentifierNameSyntax id => id.Identifier.Text,
                        SimpleNameSyntax sn => sn.Identifier.Text,
                        _ => null
                    };

                    if (attrName == "Serializable" || attrName == "SerializableAttribute")
                        return true;
                }
            }

            return false;
        }

        private static bool HasPrivateMutableFields(ClassDeclarationSyntax classDecl)
        {
            // Look for any private field whose type hints at mutability
            var mutableTypePatterns = new[] { "Dictionary", "List", "HashSet", "Queue", "Stack", "SortedSet" };

            foreach (var member in classDecl.Members)
            {
                if (!(member is FieldDeclarationSyntax fieldDecl))
                    continue;

                // Must be private
                if (!fieldDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.PrivateKeyword)))
                    continue;

                // Skip readonly fields (immutable after init)
                if (fieldDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.ReadOnlyKeyword)))
                    continue;

                // Check type hints at mutability
                var fieldTypeName = fieldDecl.Declaration.Type.ToString();

                // Pattern 1: Starts with mutable container name
                foreach (var pattern in mutableTypePatterns)
                {
                    if (fieldTypeName.Contains(pattern))
                        return true;
                }

                // Pattern 2: Field name starts with underscore (convention for mutable backing field)
                // This is heuristic; only if the type is also commonly mutable (not primitive)
                foreach (var varDecl in fieldDecl.Declaration.Variables)
                {
                    var fieldName = varDecl.Identifier.Text;
                    if (fieldName.StartsWith("_") && !IsPrimitiveType(fieldTypeName))
                        return true;
                }
            }

            return false;
        }

        private static bool IsPrimitiveType(string typeName)
        {
            // Simple heuristic: exclude common primitives
            var primitiveNames = new[]
            {
                "int", "long", "short", "byte",
                "uint", "ulong", "ushort", "sbyte",
                "float", "double", "decimal",
                "bool", "char", "string",
                "object", "void"
            };

            return primitiveNames.Contains(typeName) || typeName.StartsWith("System.");
        }

        private static bool HasProtectedVirtualOrAbstractMembers(ClassDeclarationSyntax classDecl)
        {
            // Check if this class has any protected virtual or abstract members
            foreach (var member in classDecl.Members)
            {
                // Check if member is protected
                var hasProtectedModifier = member.Modifiers.Any(m => m.IsKind(SyntaxKind.ProtectedKeyword));
                if (!hasProtectedModifier)
                    continue;

                // Check if member is virtual or abstract
                var isVirtualOrAbstract = member.Modifiers.Any(m =>
                    m.IsKind(SyntaxKind.VirtualKeyword) ||
                    m.IsKind(SyntaxKind.AbstractKeyword));

                if (isVirtualOrAbstract)
                    return true;
            }

            return false;
        }
    }
}
