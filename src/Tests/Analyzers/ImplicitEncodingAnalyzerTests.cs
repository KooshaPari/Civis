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
    /// Tests for <see cref="ImplicitEncodingAnalyzer"/> (DF0106).
    /// Pattern #232 family — file I/O without explicit Encoding.
    /// Supports inline suppression marker
    /// <c>// implicit-encoding-ok: &lt;reason&gt;</c> in leading trivia.
    /// </summary>
    public class ImplicitEncodingAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new ImplicitEncodingAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0106_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0106");
            ImplicitEncodingAnalyzer.DiagnosticId.Should().Be("DF0106");
        }

        [Fact]
        public void DF0106_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0106_HasReliabilityCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Reliability");
        }

        // ---------- Firing behavior — positive cases ----------

        [Fact]
        public async Task FileReadAllText_NoEncoding_ReportsDF0106()
        {
            const string source = @"
using System.IO;

public class C
{
    public string M(string path)
    {
        return File.ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        [Fact]
        public async Task FileWriteAllText_NoEncoding_ReportsDF0106()
        {
            const string source = @"
using System.IO;

public class C
{
    public void M(string path, string content)
    {
        File.WriteAllText(path, content);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        [Fact]
        public async Task FileReadAllLines_NoEncoding_ReportsDF0106()
        {
            const string source = @"
using System.IO;

public class C
{
    public string[] M(string path)
    {
        return File.ReadAllLines(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        [Fact]
        public async Task Reports_WhenQualifiedSystemIOFile()
        {
            // System.IO.File.ReadAllText(...) without `using System.IO;` — MemberAccess walk to leaf "File".
            const string source = @"
public class C
{
    public string M(string path)
    {
        return System.IO.File.ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        [Fact]
        public async Task Reports_WhenUsingStaticSystemIOFile()
        {
            // `using static System.IO.File;` — bare invocation, resolved via SemanticModel.
            const string source = @"
using static System.IO.File;

public class C
{
    public string M(string path)
    {
        return ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        [Fact]
        public async Task Reports_WhenAliasedSystemIOFile()
        {
            // `using F = System.IO.File;` — alias resolved via SemanticModel.
            const string source = @"
using F = System.IO.File;

public class C
{
    public string M(string path)
    {
        return F.ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0106");
        }

        // ---------- Firing behavior — negative cases ----------

        [Fact]
        public async Task FileReadAllText_WithExplicitEncoding_DoesNotReport()
        {
            const string source = @"
using System.IO;
using System.Text;

public class C
{
    public string M(string path)
    {
        return File.ReadAllText(path, Encoding.UTF8);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task FileReadAllText_WithImplicitEncodingOkMarker_DoesNotReport()
        {
            const string source = @"
using System.IO;

public class C
{
    public void M(string path)
    {
        // implicit-encoding-ok: legacy file format known to be ASCII
        File.ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task NotFileType_ReadAllText_DoesNotReport()
        {
            // Analyzer is scoped to `File.XXX` calls — unrelated types are exempt.
            const string source = @"
public class MyReader
{
    public string ReadAllText(string path) => string.Empty;
}

public class C
{
    public string M(MyReader r, string path)
    {
        return r.ReadAllText(path);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        // ---------- Helpers ----------

        private static async Task<ImmutableArray<Diagnostic>> RunAnalyzerAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Collections.Generic.List<>).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.IO.File).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Text.Encoding).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0106Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new ImplicitEncodingAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0106")
                .ToImmutableArray();
        }
    }
}
