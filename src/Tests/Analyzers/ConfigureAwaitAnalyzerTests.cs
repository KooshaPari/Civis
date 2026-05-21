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
    /// Tests for <see cref="ConfigureAwaitAnalyzer"/> (DF0098, Pattern #98).
    /// Suppression marker: <c>// configureawait-ok: &lt;reason&gt;</c>.
    /// Path exclusion: files under \Tests\, \Tools\, \Runtime\, \Domains\Runtime\.
    /// </summary>
    public class ConfigureAwaitAnalyzerTests
    {
        [Fact]
        public async Task Reports_OnTriggerPattern()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    public async Task M()
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source, @"C:\repo\src\SDK\Foo.cs");
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0098");
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    public async Task M()
    {
        // configureawait-ok: deliberately captures context
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source, @"C:\repo\src\SDK\Foo.cs");
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenInExcludedPath()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    public async Task M()
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source, @"C:\repo\src\Tools\Foo.cs");
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenConfigureAwaitChained()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    public async Task M()
    {
        await Task.Delay(1).ConfigureAwait(false);
    }
}";
            var diagnostics = await RunAsync(source, @"C:\repo\src\SDK\Foo.cs");
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source, string filePath)
        {
            var tree = CSharpSyntaxTree.ParseText(source, path: filePath);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(Task).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF0098Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new ConfigureAwaitAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0098").ToImmutableArray();
        }
    }
}
