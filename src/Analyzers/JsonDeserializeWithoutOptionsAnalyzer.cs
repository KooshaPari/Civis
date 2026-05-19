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
    public class JsonDeserializeWithoutOptionsAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0120";
        private const string Category = "Serialization";

        private static readonly LocalizableString Title =
            (LocalizableString)"JsonSerializer.Deserialize called without explicit options";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"JsonSerializer.Deserialize<{0}> with no options uses defaults (PascalCase, skip-unknown). Pass a canonical JsonSerializerOptions (e.g. CliJsonOptions.Default, PackCompilerJsonOptions.Default).";

        private static readonly LocalizableString Description =
            (LocalizableString)"Using JsonSerializer.Deserialize without explicit JsonSerializerOptions can cause silent failures due to case sensitivity and unknown property handling. Always pass a canonical options instance from CliJsonOptions.Default, PackCompilerJsonOptions.Default, or a well-defined constant.";

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

            // Check if this is a JsonSerializer.Deserialize call
            if (!IsJsonSerializerDeserialize(invocation))
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

        private static bool IsJsonSerializerDeserialize(InvocationExpressionSyntax invocation)
        {
            var methodName = GetMethodName(invocation.Expression);
            return methodName == "Deserialize" && IsJsonSerializerMemberAccess(invocation.Expression);
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
            // Check for JsonSerializer.Deserialize<T>
            if (expr is GenericNameSyntax gns)
                return gns.Identifier.Text == "Deserialize";

            // Check for JsonSerializer.Deserialize
            if (expr is MemberAccessExpressionSyntax mas)
            {
                var left = mas.Expression;
                var rightName = (mas.Name is GenericNameSyntax gn)
                    ? gn.Identifier.Text
                    : (mas.Name is IdentifierNameSyntax ins) ? ins.Identifier.Text : null;

                if (rightName != "Deserialize")
                    return false;

                // Check if left side is JsonSerializer
                if (left is IdentifierNameSyntax leftId && leftId.Identifier.Text == "JsonSerializer")
                    return true;

                // Check for System.Text.Json.JsonSerializer
                if (left is MemberAccessExpressionSyntax leftMas)
                {
                    var parts = GetNamespaceChain(leftMas);
                    if (parts.Any(p => p == "JsonSerializer"))
                        return true;
                }
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
