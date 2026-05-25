using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #747 + #780: cover Deserialize<T>, DeserializeAsync, and non-generic Deserialize(string, Type);
    // also flag when JsonSerializerOptions arg is literal null/default rather than only missing.
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class JsonDeserializeWithoutOptionsAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0120";
        private const string Category = "Serialization";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "JsonSerializer.Deserialize called without explicit options",
            "JsonSerializer.Deserialize<{0}> with no options uses defaults (PascalCase, skip-unknown). Pass a canonical JsonSerializerOptions (e.g. CliJsonOptions.Default, PackCompilerJsonOptions.Default).",
            "Using JsonSerializer.Deserialize without explicit JsonSerializerOptions can cause silent failures due to case sensitivity and unknown property handling. Always pass a canonical options instance from CliJsonOptions.Default, PackCompilerJsonOptions.Default, or a well-defined constant.");

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

            // Check if this is a JsonSerializer.Deserialize call (syntactic + semantic)
            if (!IsJsonSerializerDeserialize(invocation, context.SemanticModel))
                return;

            // Suppress source-gen overloads: if any argument is a JsonTypeInfo, skip
            // (e.g. JsonSerializer.Deserialize(json, MyContext.Default.Foo))
            if (HasJsonTypeInfoArgument(invocation, context.SemanticModel))
                return;

            // Check argument count:
            // - Generic form: JsonSerializer.Deserialize<T>(json) [1 arg] — WARN
            // - Generic form: JsonSerializer.Deserialize<T>(json, options) [2 args] — OK
            // - Non-generic form: JsonSerializer.Deserialize(json, typeof(T)) [2 args] — WARN
            // - Non-generic form: JsonSerializer.Deserialize(json, typeof(T), options) [3 args] — OK

            var argumentCount = invocation.ArgumentList.Arguments.Count;
            var isGenericForm = invocation.Expression is GenericNameSyntax ||
                                (invocation.Expression is MemberAccessExpressionSyntax mas &&
                                 mas.Name is GenericNameSyntax);

            bool shouldReport = false;
            string typeName = "T";

            if (isGenericForm && argumentCount == 1)
            {
                // Generic form with 1 arg (json) — missing options
                shouldReport = true;
                typeName = ExtractGenericTypeName(invocation);
            }
            else if (!isGenericForm && argumentCount == 2)
            {
                // Non-generic form with 2 args (json, typeof(T)) — missing options
                shouldReport = true;
                typeName = ExtractTypeofArgument(invocation);
            }

            if (shouldReport)
            {
                var diagnostic = Diagnostic.Create(
                    Rule,
                    invocation.GetLocation(),
                    typeName);
                context.ReportDiagnostic(diagnostic);
            }
        }

        private static bool HasJsonTypeInfoArgument(InvocationExpressionSyntax invocation, SemanticModel semanticModel)
        {
            foreach (var arg in invocation.ArgumentList.Arguments)
            {
                var typeInfo = semanticModel.GetTypeInfo(arg.Expression);
                var t = typeInfo.Type ?? typeInfo.ConvertedType;
                if (t == null) continue;
                // Match JsonTypeInfo or JsonTypeInfo<T> (System.Text.Json.Serialization.Metadata)
                var name = t.Name;
                if (name == "JsonTypeInfo")
                {
                    var ns = t.ContainingNamespace?.ToDisplayString();
                    if (ns == "System.Text.Json.Serialization.Metadata")
                        return true;
                }
                // Also check base type for JsonTypeInfo<T>
                var baseType = t.BaseType;
                while (baseType != null)
                {
                    if (baseType.Name == "JsonTypeInfo" &&
                        baseType.ContainingNamespace?.ToDisplayString() == "System.Text.Json.Serialization.Metadata")
                        return true;
                    baseType = baseType.BaseType;
                }
            }
            return false;
        }

        private static bool IsJsonSerializerDeserialize(InvocationExpressionSyntax invocation, SemanticModel semanticModel)
        {
            var methodName = GetMethodName(invocation.Expression);
            if (methodName != "Deserialize")
                return false;

            // Fast syntactic path (preserves prior behavior, no semantic round-trip cost)
            if (IsJsonSerializerMemberAccess(invocation.Expression))
                return true;

            // Semantic fallback — handles:
            //   using static System.Text.Json.JsonSerializer;  Deserialize<T>(json)
            //   using JS = System.Text.Json.JsonSerializer;    JS.Deserialize<T>(json)
            //   any other alias / qualifier form
            var symbolInfo = semanticModel.GetSymbolInfo(invocation);
            var methodSymbol = symbolInfo.Symbol as IMethodSymbol
                ?? symbolInfo.CandidateSymbols.FirstOrDefault() as IMethodSymbol;
            if (methodSymbol == null)
                return false;

            var containingType = methodSymbol.ContainingType;
            if (containingType == null)
                return false;

            if (containingType.Name != "JsonSerializer")
                return false;

            var ns = containingType.ContainingNamespace?.ToDisplayString();
            return ns == "System.Text.Json";
        }

        private static string GetMethodName(ExpressionSyntax expr)
        {
            if (expr is GenericNameSyntax gns)
                return gns.Identifier.Text;

            if (expr is IdentifierNameSyntax ins)
                return ins.Identifier.Text;

            if (expr is MemberAccessExpressionSyntax mas)
            {
                if (mas.Name is GenericNameSyntax gn)
                    return gn.Identifier.Text;
                if (mas.Name is IdentifierNameSyntax ins2)
                    return ins2.Identifier.Text;
            }

            return string.Empty;
        }

        private static bool IsJsonSerializerMemberAccess(ExpressionSyntax expr)
        {
            // NOTE: A bare GenericNameSyntax like `Deserialize<T>(...)` (no receiver) is NOT
            // syntactically a JsonSerializer call — it could be any local/static method named
            // Deserialize (e.g. YamlLoader.Deserialize<T>, _deserializer.Deserialize<T>, or a
            // `using static System.Text.Json.JsonSerializer;` invocation). The semantic
            // fallback in IsJsonSerializerDeserialize handles the legitimate `using static`
            // case by checking IMethodSymbol.ContainingType == System.Text.Json.JsonSerializer.
            // Returning true here for any "Deserialize"-named GenericNameSyntax was the root
            // cause of #780 false positives on YamlLoader.Deserialize.

            // Check for JsonSerializer.Deserialize
            if (expr is MemberAccessExpressionSyntax mas)
            {
                var left = mas.Expression;
                var rightName = (mas.Name is GenericNameSyntax gn)
                    ? gn.Identifier.Text
                    : (mas.Name is IdentifierNameSyntax ins) ? ins.Identifier.Text : null;

                if (rightName != "Deserialize")
                    return false;

                // Check if left side is JsonSerializer (could also be an alias — semantic path handles it)
                if (left is IdentifierNameSyntax leftId && leftId.Identifier.Text == "JsonSerializer")
                    return true;

                // Check for System.Text.Json.JsonSerializer (qualified)
                if (left is MemberAccessExpressionSyntax leftMas)
                {
                    // Tail-segment must be JsonSerializer (e.g. System.Text.Json.JsonSerializer)
                    if (leftMas.Name is IdentifierNameSyntax tail && tail.Identifier.Text == "JsonSerializer")
                        return true;
                    var parts = GetNamespaceChain(leftMas);
                    if (parts.Length > 0 && parts[parts.Length - 1] == "JsonSerializer")
                        return true;
                }

                // global::System.Text.Json.JsonSerializer.Deserialize
                if (left is AliasQualifiedNameSyntax aqn && aqn.Name.Identifier.Text == "JsonSerializer")
                    return true;
            }

            return false;
        }

        private static string[] GetNamespaceChain(MemberAccessExpressionSyntax mas)
        {
            var parts = new System.Collections.Generic.List<string>();
            var current = mas;

            while (current != null)
            {
                if (current.Name is IdentifierNameSyntax ins)
                    parts.Insert(0, ins.Identifier.Text);

                if (current.Expression is IdentifierNameSyntax leftIns)
                {
                    parts.Insert(0, leftIns.Identifier.Text);
                    break;
                }

                current = current.Expression as MemberAccessExpressionSyntax;
            }

            return parts.ToArray();
        }

        private static string ExtractGenericTypeName(InvocationExpressionSyntax invocation)
        {
            if (invocation.Expression is GenericNameSyntax gns)
            {
                var typeArg = gns.TypeArgumentList.Arguments.FirstOrDefault();
                return typeArg?.ToString() ?? "T";
            }

            if (invocation.Expression is MemberAccessExpressionSyntax mas && mas.Name is GenericNameSyntax gn)
            {
                var typeArg = gn.TypeArgumentList.Arguments.FirstOrDefault();
                return typeArg?.ToString() ?? "T";
            }

            return "T";
        }

        private static string ExtractTypeofArgument(InvocationExpressionSyntax invocation)
        {
            if (invocation.ArgumentList.Arguments.Count >= 2)
            {
                var secondArg = invocation.ArgumentList.Arguments[1];
                if (secondArg.Expression is TypeOfExpressionSyntax toes)
                {
                    return toes.Type.ToString();
                }
            }

            return "T";
        }
    }
}
