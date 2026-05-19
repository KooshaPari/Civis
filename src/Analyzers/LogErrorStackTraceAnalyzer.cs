using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    /// <summary>
    /// Analyzer DF0096: LogError discards exception stack trace (Pattern #96).
    /// Detects calls to LogError / LogCritical / LogWarning / LogException where the
    /// exception is referenced via <c>ex.Message</c> (member access or string
    /// interpolation) without passing the exception itself as an argument.
    /// Suppress with <c>// pattern-96-ok: &lt;reason&gt;</c> on the same or
    /// preceding line. Mirrors the behavior of
    /// <c>scripts/ci/detect_logerror_no_stack.py</c>.
    /// </summary>
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public sealed class LogErrorStackTraceAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0096";
        private const string Category = "Logging";

        // Log methods that should carry an exception when one is in scope.
        private static readonly ImmutableHashSet<string> LogMethods =
            ImmutableHashSet.Create(
                StringComparer.Ordinal,
                "LogError",
                "LogCritical",
                "LogFatal",
                "LogException",
                "LogWarning");

        // Identifier names typically used for caught exceptions. Matches the
        // Python detector (_EX_NAMES) for parity.
        private static readonly ImmutableHashSet<string> ExceptionNames =
            ImmutableHashSet.Create(
                StringComparer.Ordinal,
                "ex",
                "e",
                "exception");

        private const string SuppressionMarker = "pattern-96-ok:";

        private static readonly LocalizableString Title =
            (LocalizableString)"LogError discards exception stack trace";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"'{0}' uses '{1}.Message' but does not pass '{1}' as an argument; the stack trace is lost. Pass the exception (e.g. '{0}({1}, \"...\")') or interpolate '{{{1}}}' (without .Message) to render ToString(). Suppress with '// pattern-96-ok: <reason>'.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Pattern #96: passing only exception.Message to a logger drops the stack trace, exception type, and InnerException chain, making production debugging guesswork. Always pass the exception as a positional argument or interpolate {ex} (without .Message) so the default ToString() rendering covers type + message + stack. Suppress with '// pattern-96-ok: <reason>' marker on the same or preceding line.";

        private static readonly DiagnosticDescriptor Rule = new DiagnosticDescriptor(
            DiagnosticId,
            Title,
            MessageFormat,
            Category,
            DiagnosticSeverity.Warning,
            isEnabledByDefault: true,
            description: Description);

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            if (context == null)
                return;

            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.Analyze | GeneratedCodeAnalysisFlags.ReportDiagnostics);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeInvocation, SyntaxKind.InvocationExpression);
        }

        private static void AnalyzeInvocation(SyntaxNodeAnalysisContext context)
        {
            var invocation = (InvocationExpressionSyntax)context.Node;

            if (!TryGetLogMethodName(invocation, out var methodName))
                return;

            var args = invocation.ArgumentList.Arguments;
            if (args.Count == 0)
                return;

            // Healthy: any positional argument is itself the exception variable.
            // Covers BOTH LogError(ex, "msg") and LogError("msg", ex) patterns.
            if (ArgumentsContainExceptionVariable(args, out var exceptionVarName))
            {
                // If the exception itself is being passed, the stack is preserved
                // regardless of what other ex.Message references appear in the
                // format string.
                return;
            }

            // Scan all arguments for an offending ex.Message reference.
            foreach (var arg in args)
            {
                if (TryFindLossyExceptionReference(arg.Expression, out var lossyName))
                {
                    // Check both same-line and preceding-line suppression markers.
                    if (HasSuppressionMarker(invocation))
                        return;

                    var diagnostic = Diagnostic.Create(
                        Rule,
                        invocation.GetLocation(),
                        methodName,
                        lossyName);
                    context.ReportDiagnostic(diagnostic);
                    return;
                }
            }

            _ = exceptionVarName; // silence unused-out warning in the healthy path
        }

        /// <summary>
        /// Returns true and populates <paramref name="methodName"/> when the
        /// invocation matches <c>logger.LogError(...)</c> or any sibling
        /// <see cref="LogMethods"/> name. Plain identifier invocations
        /// (<c>LogError(...)</c> without a receiver) also match.
        /// </summary>
        private static bool TryGetLogMethodName(InvocationExpressionSyntax invocation, out string methodName)
        {
            methodName = string.Empty;

            switch (invocation.Expression)
            {
                case MemberAccessExpressionSyntax memberAccess:
                    var name = memberAccess.Name.Identifier.ValueText;
                    if (LogMethods.Contains(name))
                    {
                        methodName = name;
                        return true;
                    }
                    return false;

                case IdentifierNameSyntax ident:
                    if (LogMethods.Contains(ident.Identifier.ValueText))
                    {
                        methodName = ident.Identifier.ValueText;
                        return true;
                    }
                    return false;

                default:
                    return false;
            }
        }

        /// <summary>
        /// Returns true when any argument expression is a bare identifier matching
        /// one of <see cref="ExceptionNames"/>. Outputs the matched identifier for
        /// completeness.
        /// </summary>
        private static bool ArgumentsContainExceptionVariable(
            SeparatedSyntaxList<ArgumentSyntax> args,
            out string exceptionVarName)
        {
            exceptionVarName = string.Empty;
            foreach (var arg in args)
            {
                if (arg.Expression is IdentifierNameSyntax id &&
                    ExceptionNames.Contains(id.Identifier.ValueText))
                {
                    exceptionVarName = id.Identifier.ValueText;
                    return true;
                }
            }
            return false;
        }

        /// <summary>
        /// Returns true when the supplied expression contains a lossy
        /// <c>ex.Message</c> reference (either as a top-level member access or
        /// inside a string-interpolation expression). <paramref name="exceptionVarName"/>
        /// is set to the matched identifier.
        /// </summary>
        private static bool TryFindLossyExceptionReference(ExpressionSyntax expr, out string exceptionVarName)
        {
            exceptionVarName = string.Empty;

            // Direct member access: ex.Message
            if (expr is MemberAccessExpressionSyntax memberAccess &&
                IsExMessageAccess(memberAccess, out var direct))
            {
                exceptionVarName = direct;
                return true;
            }

            // Interpolated string: $"...{ex.Message}..."
            if (expr is InterpolatedStringExpressionSyntax interp)
            {
                if (HasLossyInterpolation(interp, out var inInterp))
                {
                    exceptionVarName = inInterp;
                    return true;
                }
                return false;
            }

            // Binary concatenation: "..." + ex.Message
            if (expr is BinaryExpressionSyntax bin && bin.IsKind(SyntaxKind.AddExpression))
            {
                if (TryFindLossyExceptionReference(bin.Left, out var leftName))
                {
                    exceptionVarName = leftName;
                    return true;
                }
                if (TryFindLossyExceptionReference(bin.Right, out var rightName))
                {
                    exceptionVarName = rightName;
                    return true;
                }
            }

            return false;
        }

        private static bool HasLossyInterpolation(InterpolatedStringExpressionSyntax interp, out string exceptionVarName)
        {
            exceptionVarName = string.Empty;

            // First scan all interpolated holes; if ANY hole renders the full
            // exception (e.g. {ex} without .Message), the call is healthy
            // because ToString() carries type + message + stack.
            foreach (var content in interp.Contents)
            {
                if (content is InterpolationSyntax interpolation &&
                    interpolation.Expression is IdentifierNameSyntax id &&
                    ExceptionNames.Contains(id.Identifier.ValueText))
                {
                    // Healthy interpolation found — suppress the entire match.
                    return false;
                }
            }

            // Now look for the lossy pattern: {ex.Message} (or nested member like
            // {ex.InnerException.Message}). Any such hole is a violation.
            foreach (var content in interp.Contents)
            {
                if (content is InterpolationSyntax interpolation &&
                    interpolation.Expression is MemberAccessExpressionSyntax member &&
                    IsExMessageAccess(member, out var name))
                {
                    exceptionVarName = name;
                    return true;
                }
            }

            return false;
        }

        /// <summary>
        /// Returns true when the supplied member access is of the form
        /// <c>ex.Message</c> where <c>ex</c> is one of <see cref="ExceptionNames"/>.
        /// Also matches <c>ex.InnerException.Message</c> by walking nested member
        /// access chains: the immediate <c>.Message</c> selector is what loses the
        /// stack.
        /// </summary>
        private static bool IsExMessageAccess(MemberAccessExpressionSyntax memberAccess, out string exceptionVarName)
        {
            exceptionVarName = string.Empty;

            // We must end in ".Message" — that is the lossy selector.
            if (memberAccess.Name.Identifier.ValueText != "Message")
                return false;

            // Walk the left-hand expression to find the root identifier.
            var current = memberAccess.Expression;
            while (current is MemberAccessExpressionSyntax inner)
            {
                current = inner.Expression;
            }

            if (current is IdentifierNameSyntax id && ExceptionNames.Contains(id.Identifier.ValueText))
            {
                exceptionVarName = id.Identifier.ValueText;
                return true;
            }
            return false;
        }

        /// <summary>
        /// Returns true when the invocation (or the parent statement) carries a
        /// <c>// pattern-96-ok: ...</c> suppression marker in adjacent trivia.
        /// Checks: same-line trailing trivia, leading trivia of the invocation,
        /// and leading trivia of the enclosing statement (the typical placement).
        /// </summary>
        private static bool HasSuppressionMarker(InvocationExpressionSyntax invocation)
        {
            if (TriviaContainsMarker(invocation.GetTrailingTrivia()))
                return true;
            if (TriviaContainsMarker(invocation.GetLeadingTrivia()))
                return true;

            var statement = invocation.FirstAncestorOrSelf<StatementSyntax>();
            if (statement != null)
            {
                if (TriviaContainsMarker(statement.GetLeadingTrivia()))
                    return true;
                if (TriviaContainsMarker(statement.GetTrailingTrivia()))
                    return true;
            }

            return false;
        }

        private static bool TriviaContainsMarker(SyntaxTriviaList trivia)
        {
            foreach (var t in trivia)
            {
                if (t.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                    t.IsKind(SyntaxKind.MultiLineCommentTrivia))
                {
                    var text = t.ToFullString();
                    if (text.IndexOf(SuppressionMarker, StringComparison.Ordinal) >= 0)
                        return true;
                }
            }
            return false;
        }
    }
}
