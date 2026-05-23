using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #848: treat `catch (Exception) { }` (typed but empty) the same as bare `catch { }`,
    // and recognize `// safe-swallow:` / `// test-cleanup-ok` suppression markers.
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class SilentCatchAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0111";
        private const string Category = "Observability";

        private static readonly LocalizableString Title =
            (LocalizableString)"Catch swallows exceptions silently";

        private static readonly LocalizableString MessageFormat =
            (LocalizableString)"Catch block silently swallows exception. Log via `catch (Exception ex) {{ _logger.LogWarning(ex, \"context\"); }}`, document with `// safe-swallow: <reason>`, or remove the try/catch.";

        private static readonly LocalizableString Description =
            (LocalizableString)"Catch blocks that do not produce a warning/error-level signal hide I/O, reflection, or resource-exhaustion failures, breaking observability and making debugging impossible. Always log exceptions at warning/error level, document safe-swallows inline, or remove the try/catch entirely. Use `catch (Exception ex) { _logger.LogWarning(ex, \"context\"); }` for production, or `// safe-swallow: <reason>` for intentional swallows.";

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
            context.RegisterSyntaxNodeAction(AnalyzeCatchClause, SyntaxKind.CatchClause);
        }

        private static void AnalyzeCatchClause(SyntaxNodeAnalysisContext context)
        {
            var catchClause = (CatchClauseSyntax)context.Node;

            if (catchClause.Block == null)
                return;

            // Check for safe-swallow / test-cleanup-ok markers first (cheap escape hatch)
            if (HasSafeSwallowComment(catchClause))
                return;

            // #848 Gap Class E: `catch when (false) { ... }` — filter that NEVER matches makes
            // the entire catch dead code. The author either typoed a predicate, abandoned an
            // implementation, or is deliberately disabling a handler without removing it. Always
            // suspicious; surface it the same as a silent swallow.
            // (Always-true filter `when (true)` is NOT independently flagged — it is semantically
            // equivalent to no filter, so the underlying body still goes through the regular
            // empty/silent-erasure checks below.)
            if (catchClause.Filter is { } filter &&
                filter.FilterExpression is LiteralExpressionSyntax filterLit &&
                filterLit.IsKind(SyntaxKind.FalseLiteralExpression))
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
                return;
            }

            var statements = catchClause.Block.Statements;

            // Case A: truly empty block — catch {} / catch (Exception) {} / catch (Exception ex) {}
            if (statements.Count == 0)
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
                return;
            }

            // #848 Gap Class F: body contains only no-op statements (empty statement `;`,
            // empty blocks `{ }`) — semantically identical to truly empty but the lexer sees
            // statements. Treat as Case A.
            if (statements.All(IsNoOpStatement))
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
                return;
            }

            // #848 Gap Class G: low-signal logging-only body — `catch { _logger.LogTrace(...); }`
            // or `catch { _logger.LogVerbose(...); }` still swallows the exception without a
            // warning/error-level signal.
            if (IsWeakLogOnlyBody(statements))
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
                return;
            }

            // #848 Gap Class D: placeholder-only body — body contains no executable work and
            // is annotated solely with a TODO/FIXME/XXX/HACK comment. These are abandoned
            // handlers, not intentional swallows; suppression marker is `// safe-swallow:` or
            // `// test-cleanup-ok` (per HasSafeSwallowComment), not `// TODO`.
            if (statements.All(IsNoOpStatement) || HasOnlyPlaceholderComment(catchClause))
            {
                // (statements.All(IsNoOpStatement) already returned above; this branch only
                // catches bodies whose statements are no-ops AND whose only commentary is a
                // placeholder — already handled, but kept here for documentation/clarity.)
            }

            // #848 Gap Class A: discard pattern — `catch (Exception ex) { _ = ex; }`. Author
            // assigns the exception to the discard `_`, suppressing "unused variable" warnings
            // without actually handling the exception. Body length is 1, no logging, no rethrow.
            if (IsDiscardOnlyBody(catchClause, statements))
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
                return;
            }

            // Case B / #848 Gap Class C: catch-swallow-default — block contains ONLY a
            // return/break/continue with a default-ish value (null, default, false, 0, "",
            // empty collection literal) and no logging, rethrow, or use of the caught exception.
            // This is the silent-erasure form described in Pattern #111 (and overlaps Pattern
            // #104).
            if (IsSilentReturnDefault(catchClause, statements))
            {
                context.ReportDiagnostic(Diagnostic.Create(Rule, catchClause.GetLocation()));
            }
        }

        // #848 Gap Class F helper: recognize `;` (empty statement) and `{ }` (empty block) as
        // no-op statements that make a catch body semantically empty.
        private static bool IsNoOpStatement(StatementSyntax stmt)
        {
            switch (stmt)
            {
                case EmptyStatementSyntax _:
                    return true;
                case BlockSyntax block when block.Statements.All(IsNoOpStatement):
                    return true;
                default:
                    return false;
            }
        }

        // #848 Gap Class G helper: body contains only low-signal logging invocations such as
        // `LogTrace` / `LogVerbose`. These are still silent swallow patterns because they do
        // not surface a production-grade exception signal.
        private static bool IsWeakLogOnlyBody(SyntaxList<StatementSyntax> statements)
        {
            if (statements.Count == 0)
                return false;

            return statements.All(IsWeakLogOnlyStatement);
        }

        private static bool IsWeakLogOnlyStatement(StatementSyntax stmt)
        {
            switch (stmt)
            {
                case EmptyStatementSyntax _:
                    return true;
                case BlockSyntax block when block.Statements.All(IsWeakLogOnlyStatement):
                    return true;
                case ExpressionStatementSyntax exprStmt when exprStmt.Expression is InvocationExpressionSyntax invocation:
                    return IsWeakLoggingInvocation(invocation);
                default:
                    return false;
            }
        }

        private static bool IsWeakLoggingInvocation(InvocationExpressionSyntax invocation)
        {
            string invocationText = invocation.Expression.ToString();
            return invocationText.IndexOf("LogTrace", StringComparison.Ordinal) >= 0 ||
                   invocationText.IndexOf("LogVerbose", StringComparison.Ordinal) >= 0;
        }

        // #848 Gap Class D helper: body's only non-trivia content is a placeholder comment
        // (TODO/FIXME/XXX/HACK). Distinct from `// safe-swallow:` which is the explicit opt-out.
        private static bool HasOnlyPlaceholderComment(CatchClauseSyntax catchClause)
        {
            if (catchClause.Block == null) return false;
            if (catchClause.Block.Statements.Count > 0) return false;
            // Empty-statement case already reported via Case A; this is defensive only.
            return false;
        }

        // #848 Gap Class A helper: detect `_ = ex;` (discard assignment of the caught exception
        // variable) — and variants like `_ = ex.Message;`. Treats it as a silent swallow.
        private static bool IsDiscardOnlyBody(CatchClauseSyntax catchClause, SyntaxList<StatementSyntax> statements)
        {
            if (statements.Count != 1) return false;
            if (!(statements[0] is ExpressionStatementSyntax exprStmt)) return false;
            if (!(exprStmt.Expression is AssignmentExpressionSyntax assign)) return false;
            if (!assign.IsKind(SyntaxKind.SimpleAssignmentExpression)) return false;
            if (!(assign.Left is IdentifierNameSyntax leftId)) return false;
            if (leftId.Identifier.ValueText != "_") return false;
            // The body is `_ = <something>;`. If RHS is the caught exception (or a member of it,
            // e.g. `_ = ex.Message`), this is a swallow — no log, no rethrow.
            // We don't require the RHS to be the exception specifically; `_ = anything;` inside a
            // catch with no other action is always silent.
            return true;
        }

        private static bool IsSilentReturnDefault(CatchClauseSyntax catchClause, SyntaxList<StatementSyntax> statements)
        {
            // Only consider a single-statement body to keep false-positive rate low.
            if (statements.Count != 1)
                return false;

            var stmt = statements[0];

            // Reject if the statement (or any descendant) logs, rethrows, or references the
            // caught exception variable. This is a syntactic heuristic — good enough without
            // a semantic model.
            var exceptionVarName = catchClause.Declaration?.Identifier.ValueText;
            foreach (var node in stmt.DescendantNodesAndSelf())
            {
                switch (node)
                {
                    case ThrowStatementSyntax _:
                    case ThrowExpressionSyntax _:
                        return false;
                    case InvocationExpressionSyntax invocation:
                        var text = invocation.Expression.ToString();
                        if (text.IndexOf("Log", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Trace", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Write", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Console", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Debug", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Error", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Warn", StringComparison.Ordinal) >= 0 ||
                            text.IndexOf("Report", StringComparison.Ordinal) >= 0)
                            return false;
                        break;
                    case IdentifierNameSyntax id when exceptionVarName != null && id.Identifier.ValueText == exceptionVarName:
                        return false;
                }
            }

            // Now check the statement is a return/break/continue of a default-ish value.
            switch (stmt)
            {
                case ReturnStatementSyntax ret:
                    return IsDefaultishExpression(ret.Expression);
                case BreakStatementSyntax _:
                case ContinueStatementSyntax _:
                    return true;
            }
            return false;
        }

        private static bool IsDefaultishExpression(ExpressionSyntax? expr)
        {
            if (expr == null) return true; // bare `return;`
            switch (expr.Kind())
            {
                case SyntaxKind.NullLiteralExpression:
                case SyntaxKind.DefaultLiteralExpression:
                case SyntaxKind.DefaultExpression:
                case SyntaxKind.FalseLiteralExpression:
                    return true;
                case SyntaxKind.NumericLiteralExpression:
                    return expr.ToString() == "0";
                case SyntaxKind.StringLiteralExpression:
                    return expr.ToString() == "\"\"" || expr.ToString() == "string.Empty";
            }
            // string.Empty / String.Empty
            if (expr is MemberAccessExpressionSyntax mae &&
                mae.Name.Identifier.ValueText == "Empty" &&
                (mae.Expression.ToString() == "string" || mae.Expression.ToString() == "String"))
                return true;
            return false;
        }

        private static bool HasSafeSwallowComment(CatchClauseSyntax catchClause)
        {
            // 1. Check leading trivia (previous-line marker)
            var leadingTrivia = catchClause.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // 2. Check trailing trivia of closing brace (same-line trailing marker)
            if (catchClause.Block?.CloseBraceToken.HasTrailingTrivia ?? false)
            {
                var closingBraceTrailing = catchClause.Block.CloseBraceToken.TrailingTrivia;
                foreach (var trivia in closingBraceTrailing)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }
            }

            // 3. Check inline trivia inside the block (e.g., catch { /* safe-swallow: */ })
            if (catchClause.Block != null)
            {
                // Check leading trivia of opening brace
                var openingBraceLeading = catchClause.Block.OpenBraceToken.LeadingTrivia;
                foreach (var trivia in openingBraceLeading)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }

                // Check trailing trivia of opening brace
                var openingBraceTrailing = catchClause.Block.OpenBraceToken.TrailingTrivia;
                foreach (var trivia in openingBraceTrailing)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }

                // Check leading trivia of closing brace
                var closingBraceLeading = catchClause.Block.CloseBraceToken.LeadingTrivia;
                foreach (var trivia in closingBraceLeading)
                {
                    if (CheckTrivia(trivia))
                        return true;
                }
            }

            return false;
        }

        private static bool CheckTrivia(SyntaxTrivia trivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("safe-swallow:") ||
                    commentText.Contains("test-cleanup-ok"))
                    return true;
            }
            return false;
        }
    }
}
