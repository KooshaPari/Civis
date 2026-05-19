using System;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class LogErrorStackTraceAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0096";
        private const string Category = "Logging";

        private static readonly LocalizableString Title =
            (LocalizableString)"LogError missing exception parameter";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Call to LogError discards stack trace. Pass the exception as the first argument.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Passing exception.Message instead of the exception itself to LogError loses the stack trace. Always pass the exception as the first parameter.";

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

            // Match method calls like _logger.LogError(...)
            if (!IsLogErrorCall(invocation))
                return;

            // Check if first argument is an exception variable
            var args = invocation.ArgumentList.Arguments;
            if (args.Count == 0)
                return;

            var firstArg = args[0];

            // If first arg is a simple identifier (variable/parameter), check if it's exception-typed
            if (firstArg.Expression is IdentifierNameSyntax identifier)
            {
                var symbol = context.SemanticModel.GetSymbolInfo(identifier).Symbol;
                if (symbol != null && IsExceptionType(symbol, context.SemanticModel.Compilation))
                {
                    // First arg is an exception variable — good pattern
                    return;
                }
            }

            // If first arg is a property/member access (e.g., ex.Message), it's likely losing stack trace
            if (firstArg.Expression is MemberAccessExpressionSyntax memberAccess)
            {
                var typeInfo = context.SemanticModel.GetTypeInfo(memberAccess.Expression);
                if (typeInfo.Type != null && IsExceptionType(typeInfo.Type, context.SemanticModel.Compilation))
                {
                    // Accessing member of exception without passing exception itself
                    var diagnostic = Diagnostic.Create(Rule, invocation.GetLocation());
                    context.ReportDiagnostic(diagnostic);
                    return;
                }
            }

            // If first arg is a literal string or other non-exception expression, no stack trace risk
            // (intentional LogError("message") is fine)
        }

        private static bool IsLogErrorCall(InvocationExpressionSyntax invocation)
        {
            // Match: _logger.LogError(...) or logger.LogError(...) etc.
            if (invocation.Expression is MemberAccessExpressionSyntax memberAccess &&
                memberAccess.Name.Identifier.ValueText == "LogError")
            {
                return true;
            }

            return false;
        }

        private static bool IsExceptionType(ISymbol? symbol, Compilation compilation)
        {
            if (symbol == null)
                return false;

            if (symbol is ILocalSymbol local)
                return IsExceptionType(local.Type, compilation);

            if (symbol is IParameterSymbol param)
                return IsExceptionType(param.Type, compilation);

            if (symbol is IPropertySymbol prop)
                return IsExceptionType(prop.Type, compilation);

            return false;
        }

        private static bool IsExceptionType(ITypeSymbol? type, Compilation compilation)
        {
            if (type == null)
                return false;

            var exceptionType = compilation.GetTypeByMetadataName("System.Exception");
            if (exceptionType == null)
                return false;

            var comparer = SymbolEqualityComparer.Default;
            if (comparer.Equals(type, exceptionType))
                return true;

            // Check base types
            if (type.BaseType != null && IsExceptionType(type.BaseType, compilation))
                return true;

            return false;
        }
    }
}
