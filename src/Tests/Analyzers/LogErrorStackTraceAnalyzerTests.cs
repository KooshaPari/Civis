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
    /// Tests for <see cref="LogErrorStackTraceAnalyzer"/> (DF0096).
    /// Covers both metadata invariants (RS1031 / RS1032 surface) and the full
    /// diagnostic firing matrix described in Pattern #96.
    /// </summary>
    public class LogErrorStackTraceAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new LogErrorStackTraceAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0096_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0096");
            LogErrorStackTraceAnalyzer.DiagnosticId.Should().Be("DF0096");
        }

        [Fact]
        public void DF0096_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0096_HasLoggingCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Logging");
        }

        [Fact]
        public void DF0096_HasSuppressionMarker()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Description.ToString().Should().Contain("pattern-96-ok:");
        }

        [Fact]
        public void DF0096_TitleHasNoTrailingPeriod()
        {
            // RS1031 compliance: titles must not end with a period.
            var title = GetAnalyzer().SupportedDiagnostics[0].Title.ToString();
            title.Should().NotEndWith(".");
            title.Should().NotEndWith(" ");
            title.Should().NotContain("\r");
            title.Should().NotContain("\n");
        }

        [Fact]
        public void DF0096_MessageFormatEndsWithPeriod()
        {
            // RS1032 compliance: message format should end with a period.
            var messageFormat = GetAnalyzer().SupportedDiagnostics[0].MessageFormat.ToString();
            messageFormat.TrimEnd().Should().EndWith(".");
        }

        // ---------- Firing behavior ----------

        [Fact]
        public async Task LogError_WithExMessageInterpolation_ReportsDF0096()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError($""Failed: {ex.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogError_WithExMessageMemberAccess_ReportsDF0096()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError(ex.Message);
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogError_WithStringConcatExMessage_ReportsDF0096()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError(""Failed: "" + ex.Message);
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogError_WithExceptionFirstArg_DoesNotReport()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError(ex, ""Failed"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogError_WithExceptionFirstArgPlusParams_DoesNotReport()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError(ex, ""Failed {Code} {Detail}"", 42, ""x"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogError_WithFullExInterpolation_DoesNotReport()
        {
            // {ex} (without .Message) renders ToString() = type+msg+stack — healthy.
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError($""Failed: {ex}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogError_WithLiteralStringOnly_DoesNotReport()
        {
            const string source = @"
using Microsoft.Extensions.Logging;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        _log.LogError(""Static message with no exception context"");
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogError_WithSuppressionMarkerSameLine_DoesNotReport()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError($""Failed: {ex.Message}""); // pattern-96-ok: legacy compat
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogError_WithSuppressionMarkerPreviousLine_DoesNotReport()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            // pattern-96-ok: framework boundary
            _log.LogError($""Failed: {ex.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task LogCritical_WithExMessage_ReportsDF0096()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogCritical($""Boom: {ex.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogWarning_WithExMessage_ReportsDF0096()
        {
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogWarning($""Recovering from: {ex.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogError_WithNestedExMessage_ReportsDF0096()
        {
            // ex.InnerException.Message is still lossy — the .Message terminator
            // collapses everything to a string.
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    private readonly ILogger _log = null!;
    public void M()
    {
        try { } catch (Exception ex)
        {
            _log.LogError($""Inner: {ex.InnerException.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
        }

        [Fact]
        public async Task LogError_OnDifferentReceiver_StillReports()
        {
            // Receiver name doesn't matter — `logger`, `_log`, `Log`, etc.
            const string source = @"
using Microsoft.Extensions.Logging;
using System;

public class C
{
    public void M(ILogger logger)
    {
        try { } catch (Exception e)
        {
            logger.LogError($""x: {e.Message}"");
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0096");
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
                MetadataReference.CreateFromFile(typeof(System.Console).Assembly.Location),
                // Microsoft.Extensions.Logging shim — provide a minimal ILogger
                // surface so the source compiles without taking a real dependency.
            };
            // Add a minimal logger shim — easier than dragging in the real package.
            var shim = CSharpSyntaxTree.ParseText(@"
namespace Microsoft.Extensions.Logging
{
    public interface ILogger { }
    public static class LoggerExtensions
    {
        public static void LogError(this ILogger logger, string message) { }
        public static void LogError(this ILogger logger, System.Exception ex, string message) { }
        public static void LogError(this ILogger logger, System.Exception ex, string message, params object[] args) { }
        public static void LogError(this ILogger logger, string message, params object[] args) { }
        public static void LogCritical(this ILogger logger, string message) { }
        public static void LogCritical(this ILogger logger, System.Exception ex, string message) { }
        public static void LogWarning(this ILogger logger, string message) { }
        public static void LogWarning(this ILogger logger, System.Exception ex, string message) { }
    }
}");
            var compilation = CSharpCompilation.Create(
                "DF0096Test",
                new[] { tree, shim },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new LogErrorStackTraceAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0096")
                .ToImmutableArray();
        }
    }
}
