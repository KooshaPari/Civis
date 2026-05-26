using System.Threading.Tasks;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Diagnostics;
using Microsoft.CodeAnalysis.Testing;
using Microsoft.CodeAnalysis.CSharp.Testing.XUnit;

namespace DINOForge.Tests.Analyzers;

public abstract class AnalyzerTestBase<TAnalyzer>
    where TAnalyzer : DiagnosticAnalyzer, new()
{
    protected static DiagnosticResult Diagnostic()
    {
        return AnalyzerVerifier<TAnalyzer>.Diagnostic();
    }

    protected static DiagnosticResult Diagnostic(string diagnosticId)
    {
        return AnalyzerVerifier<TAnalyzer>.Diagnostic(diagnosticId);
    }

    protected static DiagnosticResult Diagnostic(DiagnosticDescriptor descriptor)
    {
        return AnalyzerVerifier<TAnalyzer>.Diagnostic(descriptor);
    }

    protected static Task VerifyAnalyzerAsync(string source, params DiagnosticResult[] expectedDiagnostics)
    {
        return AnalyzerVerifier<TAnalyzer>.VerifyAnalyzerAsync(source, expectedDiagnostics);
    }
}
