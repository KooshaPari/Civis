#nullable enable
using System;
using System.Diagnostics.CodeAnalysis;
using System.Reflection;
using Sentry;

namespace DINOForge.SDK.Diagnostics;

/// <summary>
/// Initializes Sentry error tracking for the DINOForge SDK and all consuming applications.
/// Call <see cref="Initialize"/> once at application startup (e.g., from Runtime.Plugin.Awake).
/// </summary>
[ExcludeFromCodeCoverage] // Requires live Sentry DSN connection to cover; graceful no-op is verified via integration tests
public static class SentryInitializer
{
    private static bool _initialized;

    /// <summary>
    /// Initializes Sentry with the DSN from the <c>SENTRY_DSN</c> environment variable.
    /// Safe to call multiple times — subsequent calls are no-ops.
    /// Gracefully degrades if SENTRY_DSN is not set (logs instead of throwing).
    /// </summary>
    /// <param name="environment">
    /// Environment name (e.g., "development", "production"). Defaults to "production".
    /// </param>
    /// <param name="releaseOverride">
    /// Optional release string (e.g., git commit hash). Defaults to assembly version.
    /// </param>
    public static void Initialize(string? environment = null, string? releaseOverride = null)
    {
        if (_initialized)
            return;

        string? dsn = Environment.GetEnvironmentVariable("SENTRY_DSN");
        if (string.IsNullOrWhiteSpace(dsn))
        {
            // Graceful degradation — no DSN configured, skip init
            return;
        }

        try
        {
            SentrySdk.Init(options =>
            {
                options.Dsn = dsn;
                options.TracesSampleRate = 1.0;
                options.AttachStacktrace = true;
                options.Environment = environment ?? GetDefaultEnvironment();
                options.Release = releaseOverride ?? GetDefaultRelease();
                options.MaxBreadcrumbs = 50;
                options.MaxQueueItems = 10;
                options.ShutdownTimeout = TimeSpan.FromSeconds(2);
            });

            // Subscribe to AppDomain unhandled exceptions as a safety net
            AppDomain.CurrentDomain.UnhandledException += OnUnhandledException;

            _initialized = true;
        }
        catch (Exception ex)
        {
            // Never let Sentry init failure crash the application
            Console.Error.WriteLine($"[DINOForge] Sentry initialization failed: {ex.Message}");
        }
    }

    /// <summary>
    /// Adds a breadcrumb for SDK events (pack load, schema validation, etc.).
    /// Safe to call before Initialize — breadcrumbs are dropped silently if Sentry is not active.
    /// </summary>
    /// <param name="message">Human-readable breadcrumb message.</param>
    /// <param name="category">Breadcrumb category (e.g., "pack", "schema", "validation").</param>
    /// <param name="data">Optional key-value data.</param>
    public static void AddBreadcrumb(string message, string category = "sdk", object? data = null)
    {
        if (!_initialized)
            return;

        try
        {
            SentrySdk.AddBreadcrumb(message, category, data: null);
        }
        catch
        {
            // Never let breadcrumb failure crash the app
        }
    }

    /// <summary>
    /// Captures an exception manually from SDK code.
    /// </summary>
    public static void CaptureException(Exception ex, string? context = null)
    {
        if (!_initialized)
            return;

        try
        {
            SentrySdk.CaptureException(ex, scope =>
            {
                if (context != null)
                    scope.SetTag("context", context);
            });
        }
        catch
        {
            // Never let capture failure crash the app
        }
    }

    private static void OnUnhandledException(object sender, UnhandledExceptionEventArgs e)
    {
        if (e.ExceptionObject is Exception ex)
        {
            SentrySdk.CaptureException(ex, scope =>
            {
                scope.Level = SentryLevel.Fatal;
                scope.SetTag("unhandled", "true");
            });
        }
    }

    private static string GetDefaultEnvironment()
    {
#if DEBUG
        return "development";
#else
        return "production";
#endif
    }

    private static string GetDefaultRelease()
    {
        return typeof(SentryInitializer).Assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion
            ?? typeof(SentryInitializer).Assembly.GetName().Version?.ToString()
            ?? "unknown";
    }
}
