using System.Collections.Immutable;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Analyzers;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    /// <summary>
    /// Tests for <see cref="UnboundedWhenAllAnalyzer"/> (DF1004).
    /// Detects <c>Task.WhenAll(items.Select(x =&gt; DoAsync(x)))</c> patterns over potentially
    /// unbounded enumerations, which can cause memory and thread-pool exhaustion.
    /// Suppression marker: <c>// task-whenall-ok: &lt;reason&gt;</c>
    /// </summary>
    public class UnboundedWhenAllAnalyzerTests
    {
        [Fact]
        public void HasCorrectId()
        {
            var analyzer = new UnboundedWhenAllAnalyzer();
            analyzer.SupportedDiagnostics.Should().ContainSingle();
            analyzer.SupportedDiagnostics[0].Id.Should().Be("DF1004");
        }

        [Fact]
        public void HasExpectedSeverity()
        {
            var analyzer = new UnboundedWhenAllAnalyzer();
            analyzer.SupportedDiagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
            analyzer.SupportedDiagnostics[0].IsEnabledByDefault.Should().BeTrue();
        }

        [Fact]
        public void HasExpectedCategory()
        {
            var analyzer = new UnboundedWhenAllAnalyzer();
            analyzer.SupportedDiagnostics[0].Category.Should().Be("Performance");
        }

        [Fact]
        public async Task DetectsTaskWhenAllOverSelect()
        {
            const string source = @"
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
public class MyClass
{
    public async Task DoWorkAsync(List<int> items)
    {
        await Task.WhenAll(items.Select(x => ProcessAsync(x)));
    }
    private Task ProcessAsync(int x) => Task.CompletedTask;
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF1004");
        }

        [Fact]
        public async Task IgnoresTaskWhenAllOverArrayLiteral()
        {
            // Task.WhenAll with explicit task arguments (no .Select) should not flag.
            const string source = @"
using System.Threading.Tasks;
public class MyClass
{
    public async Task DoWorkAsync()
    {
        await Task.WhenAll(Task.CompletedTask, Task.CompletedTask);
    }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task HonorsSuppressionMarker()
        {
            const string source = @"
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
public class MyClass
{
    public async Task DoWorkAsync(List<int> items)
    {
        // task-whenall-ok: bounded to <10 items by upstream validation
        await Task.WhenAll(items.Select(x => ProcessAsync(x)));
    }
    private Task ProcessAsync(int x) => Task.CompletedTask;
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(Task).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Collections.Generic.List<>).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF1004Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new UnboundedWhenAllAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1004").ToImmutableArray();
        }
    }
}
