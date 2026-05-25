using System;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    // #849: pair `+=` with `-=` lifecycle check; recognize OnEnable/OnDisable and
    // ctor/Dispose pairs as valid symmetric subscription/unsubscription sites.
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class EventLifecycleAsymmetryAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF0105";
        public const string NoCleanupDiagnosticId = "DF0105a";
        private const string Category = "Resource Management";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Event subscription without matching unsubscribe",
            "Found `{0} += {1}` in class with Dispose/OnDestroy but no matching `-= {1}` in cleanup. Add `{0} -= {1}` in Dispose/OnDestroy to prevent listener leaks.",
            "Event handler subscriptions (+=) without matching unsubscriptions (-=) in cleanup methods (Dispose, OnDestroy, OnDisable, Close) can cause memory leaks by preventing listener cleanup. Always add a matching -= in the same cleanup method where the += is registered. Use `// event-lifecycle-ok: <reason>` inline comment to suppress.");

        private static readonly DiagnosticDescriptor NoCleanupRule = DinoDiagnosticDescriptors.Create(
            NoCleanupDiagnosticId,
            Category,
            DiagnosticSeverity.Warning,
            "Event subscription without matching unsubscribe",
            "{0} has event subscription '{1} += {2}' but no Dispose/OnDestroy/OnDisable/Close method - handler can never unsubscribe",
            "Event handler subscriptions (+=) without matching unsubscriptions (-=) in cleanup methods (Dispose, OnDestroy, OnDisable, Close) can cause memory leaks by preventing listener cleanup. Always add a matching -= in the same cleanup method where the += is registered. Use `// event-lifecycle-ok: <reason>` inline comment to suppress.");

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule, NoCleanupRule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeAssignment, SyntaxKind.AddAssignmentExpression);
        }

        private static void AnalyzeAssignment(SyntaxNodeAnalysisContext context)
        {
            var assignment = (AssignmentExpressionSyntax)context.Node;

            // Skip if marked with event-lifecycle-ok
            if (HasEventLifecycleOkComment(assignment))
                return;

            // Extract event name and handler expression
            var eventName = GetEventName(assignment.Left);
            if (eventName == null)
                return;

            var handlerExpression = GetHandlerExpression(assignment.Right);
            if (handlerExpression == null)
                return;

            // Find the containing class
            var classDeclaration = assignment.Ancestors()
                .OfType<ClassDeclarationSyntax>()
                .FirstOrDefault();

            if (classDeclaration == null)
                return;

            // Check if class has cleanup hook (Dispose, OnDestroy, etc.)
            if (!HasCleanupHook(classDeclaration))
            {
                // Gap #4 fix: no cleanup hook at all is the worst leak case — report instead of skip
                var className = classDeclaration.Identifier.ValueText;
                var noCleanupDiagnostic = Diagnostic.Create(
                    NoCleanupRule,
                    assignment.GetLocation(),
                    className,
                    eventName,
                    handlerExpression);
                context.ReportDiagnostic(noCleanupDiagnostic);
                return;
            }

            // Check if matching -= exists in cleanup method
            if (HasMatchingUnsubscription(classDeclaration, eventName, handlerExpression))
                return;

            // Report diagnostic
            var eventNameString = eventName;
            var handlerString = handlerExpression;
            var diagnostic = Diagnostic.Create(
                Rule,
                assignment.GetLocation(),
                eventNameString,
                handlerString);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasEventLifecycleOkComment(AssignmentExpressionSyntax assignment)
        {
            // Check leading trivia
            var leadingTrivia = assignment.GetLeadingTrivia();
            foreach (var trivia in leadingTrivia)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            // Check trailing trivia of the operator
            var operatorTrailing = assignment.OperatorToken.TrailingTrivia;
            foreach (var trivia in operatorTrailing)
            {
                if (CheckTrivia(trivia))
                    return true;
            }

            return false;
        }

        private static bool CheckTrivia(SyntaxTrivia trivia)
        {
            if (trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) ||
                trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                var commentText = trivia.ToFullString();
                if (commentText.Contains("event-lifecycle-ok:"))
                    return true;
            }
            return false;
        }

        private static string? GetEventName(ExpressionSyntax expression)
        {
            // Handle simple identifier: event +=
            if (expression is IdentifierNameSyntax identifier)
                return identifier.Identifier.ValueText;

            // Handle member access: obj.event +=
            if (expression is MemberAccessExpressionSyntax memberAccess)
                return memberAccess.Name.Identifier.ValueText;

            return null;
        }

        private static string GetHandlerExpression(ExpressionSyntax expression)
        {
            // Simple heuristic: extract the handler identifier or method name
            if (expression is IdentifierNameSyntax identifier)
                return identifier.Identifier.ValueText;

            // Handle method reference: ClassName.MethodName
            if (expression is MemberAccessExpressionSyntax memberAccess)
                return memberAccess.Name.Identifier.ValueText;

            // Handle lambda or other complex expressions
            if (expression.ToString().Length < 100)
                return expression.ToString();

            return "handler";
        }

        private static bool HasCleanupHook(ClassDeclarationSyntax classDeclaration)
        {
            var cleanupMethods = new[] { "Dispose", "OnDestroy", "OnDisable", "Close" };

            return classDeclaration.Members.OfType<MethodDeclarationSyntax>()
                .Any(method => cleanupMethods.Contains(method.Identifier.ValueText));
        }

        private static bool HasMatchingUnsubscription(
            ClassDeclarationSyntax classDeclaration,
            string eventName,
            string handlerExpression)
        {
            // Find all cleanup methods
            var cleanupMethods = new[] { "Dispose", "OnDestroy", "OnDisable", "Close" };
            var cleanupMethodDecls = classDeclaration.Members
                .OfType<MethodDeclarationSyntax>()
                .Where(m => cleanupMethods.Contains(m.Identifier.ValueText))
                .ToList();

            if (!cleanupMethodDecls.Any())
                return false;

            // Check if any cleanup method contains matching -= statement
            foreach (var method in cleanupMethodDecls)
            {
                if (ContainsMatchingUnsubscription(method, eventName, handlerExpression))
                    return true;
            }

            return false;
        }

        private static bool ContainsMatchingUnsubscription(
            MethodDeclarationSyntax method,
            string eventName,
            string handlerExpression)
        {
            if (method.Body == null)
                return false;

            // Find all -= assignment expressions
            var unsubscriptions = method.Body.DescendantNodes()
                .OfType<AssignmentExpressionSyntax>()
                .Where(expr => expr.Kind() == SyntaxKind.SubtractAssignmentExpression)
                .ToList();

            foreach (var unsub in unsubscriptions)
            {
                var unsubEventName = GetEventName(unsub.Left);
                var unsubHandler = GetHandlerExpression(unsub.Right);

                // Match both event name and handler (simple string comparison)
                if (unsubEventName == eventName)
                {
                    // Handler matching: either exact string match or contains handler name
                    if (unsubHandler == handlerExpression ||
                        unsubHandler.Contains(handlerExpression) ||
                        handlerExpression.Contains(unsubHandler))
                    {
                        return true;
                    }
                }
            }

            return false;
        }
    }
}
