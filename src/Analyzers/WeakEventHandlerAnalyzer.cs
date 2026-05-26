using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Diagnostics;

namespace DINOForge.Analyzers
{
    [DiagnosticAnalyzer(LanguageNames.CSharp)]
    public class WeakEventHandlerAnalyzer : DiagnosticAnalyzer
    {
        public const string DiagnosticId = "DF1002";
        private const string Category = "Resource Management";

        private static readonly DiagnosticDescriptor Rule = DinoDiagnosticDescriptors.Create(
            DiagnosticId,
            Category,
            DiagnosticSeverity.Info,
            "Static event subscription without weak reference",
            "Subscription to long-lived event `{0}` from instance `{1}` risks listener-leak. Use WeakEventManager pattern or ensure -= in Dispose/OnDestroy.",
            "Subscriptions to static or long-lived event sources (e.g., SceneManager.sceneLoaded, AppDomain.UnhandledException) from instance objects without proper cleanup (Dispose/OnDestroy) can cause memory leaks by holding references indefinitely. Use WeakEventManager pattern or ensure matching -= in cleanup methods. Use `// weak-event-ok: <reason>` inline comment to suppress.");

        // Known long-lived event sources (static/singleton-lifetime)
        private static readonly HashSet<string> KnownLongLivedEvents = new(StringComparer.Ordinal)
        {
            "sceneLoaded",
            "sceneUnloaded",
            "activeSceneChanged",
            "sceneLoadingStarted",
            "sceneUnloadingStarted",
            "beforeSceneUnload",
            "UnhandledException",
            "quitting",
            "focusChanged",
            "deepLinkActivated",
            "logMessageReceived",
            "logMessageReceivedThreaded",
            "wantToQuit",
            "playModeStateChanged",
        };

        public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics =>
            ImmutableArray.Create(Rule);

        public override void Initialize(AnalysisContext context)
        {
            context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
            context.EnableConcurrentExecution();
            context.RegisterSyntaxNodeAction(AnalyzeAssignment, SyntaxKind.AddAssignmentExpression);
        }

        private static void AnalyzeAssignment(SyntaxNodeAnalysisContext context)
        {
            var assignment = (AssignmentExpressionSyntax)context.Node;

            // Skip if marked with weak-event-ok
            if (HasWeakEventOkComment(assignment))
                return;

            // Extract event name from the left side
            var eventName = GetEventName(assignment.Left);
            if (eventName == null)
                return;

            // Check if it's a known long-lived event
            if (!KnownLongLivedEvents.Contains(eventName))
                return;

            // Check if the left side is a static/external event (e.g., SceneManager.sceneLoaded)
            var isStaticEventAccess = IsStaticEventAccess(assignment.Left);
            if (!isStaticEventAccess)
                return;

            // Get handler name for the message
            var handlerName = GetHandlerName(assignment.Right);

            // Find the containing class to check for cleanup
            var classDeclaration = assignment.Ancestors()
                .OfType<ClassDeclarationSyntax>()
                .FirstOrDefault();

            var subscriberName = classDeclaration?.Identifier.ValueText ?? "instance";

            // Report diagnostic
            var diagnostic = Diagnostic.Create(
                Rule,
                assignment.GetLocation(),
                eventName,
                subscriberName);
            context.ReportDiagnostic(diagnostic);
        }

        private static bool HasWeakEventOkComment(AssignmentExpressionSyntax assignment)
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
                if (commentText.Contains("weak-event-ok:"))
                    return true;
            }
            return false;
        }

        private static string? GetEventName(ExpressionSyntax expression)
        {
            // Handle simple identifier: event +=
            if (expression is IdentifierNameSyntax identifier)
                return identifier.Identifier.ValueText;

            // Handle member access: SceneManager.sceneLoaded +=
            if (expression is MemberAccessExpressionSyntax memberAccess)
                return memberAccess.Name.Identifier.ValueText;

            return null;
        }

        private static bool IsStaticEventAccess(ExpressionSyntax expression)
        {
            // Check if it's a member access (likely static or external)
            // e.g., SceneManager.sceneLoaded, AppDomain.CurrentDomain.UnhandledException
            if (expression is MemberAccessExpressionSyntax memberAccess)
            {
                // If left side is an identifier like "SceneManager" or qualified like "AppDomain.CurrentDomain"
                return memberAccess.Expression is IdentifierNameSyntax ||
                       memberAccess.Expression is MemberAccessExpressionSyntax;
            }

            // Simple identifiers without qualification are likely local/field events (skip)
            return false;
        }

        private static string GetHandlerName(ExpressionSyntax expression)
        {
            // Simple heuristic: extract identifier or method name
            if (expression is IdentifierNameSyntax identifier)
                return identifier.Identifier.ValueText;

            if (expression is MemberAccessExpressionSyntax memberAccess)
                return memberAccess.Name.Identifier.ValueText;

            // For lambdas and complex expressions, return a generic name
            if (expression.ToString().Length < 100)
                return expression.ToString();

            return "handler";
        }
    }
}
