using System;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace DINOForge.Analyzers
{
    /// <summary>
    /// Shared Roslyn syntax and analysis-context helpers for DINOForge analyzers (Sonar CPD).
    /// </summary>
    internal static class DinoAnalyzerSyntaxHelpers
    {
        public static bool LeadingTriviaContains(
            SyntaxNode node,
            string marker,
            StringComparison comparison = StringComparison.Ordinal) =>
            AnyTriviaContains(node.GetLeadingTrivia(), marker, comparison);

        public static bool AnyTriviaContains(
            SyntaxTriviaList triviaList,
            string marker,
            StringComparison comparison = StringComparison.Ordinal)
        {
            foreach (var trivia in triviaList)
            {
                if (TriviaContains(trivia, marker, comparison))
                    return true;
            }

            return false;
        }

        public static bool TriviaContains(
            SyntaxTrivia trivia,
            string marker,
            StringComparison comparison = StringComparison.Ordinal)
        {
            if (!trivia.IsKind(SyntaxKind.SingleLineCommentTrivia) &&
                !trivia.IsKind(SyntaxKind.MultiLineCommentTrivia))
            {
                return false;
            }

            return trivia.ToFullString().Contains(marker, comparison);
        }

        public static bool IsAsyncVoidMethod(MethodDeclarationSyntax method)
        {
            if (!method.Modifiers.Any(m => m.IsKind(SyntaxKind.AsyncKeyword)))
                return false;

            return method.ReturnType.ToString().Equals("void", StringComparison.Ordinal);
        }

    }
}
