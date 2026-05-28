namespace DINOForge.Runtime.Localization;

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;

/// <summary>
/// Localization (i18n) service for DINOForge UI strings.
/// Provides translation lookup with fallback to English keys or provided fallback values.
/// </summary>
internal static class L10n
{
    private static Dictionary<string, string> _strings = new(StringComparer.Ordinal);
    private static string _currentLocale = "en-US";

    /// <summary>
    /// Locales successfully loaded so far.
    /// </summary>
    private static readonly HashSet<string> _loadedLocales = new(StringComparer.OrdinalIgnoreCase);

    /// <summary>
    /// Gets the currently active locale code (e.g., "en-US", "de-DE").
    /// </summary>
    public static string CurrentLocale => _currentLocale;

    /// <summary>
    /// Loads a locale from BepInEx/dinoforge-i18n/{locale}.json or repo assets/i18n/{locale}.json.
    /// Silently falls back to en-US if file not found.
    /// </summary>
    public static void LoadLocale(string locale)
    {
        if (string.IsNullOrWhiteSpace(locale))
        {
            locale = "en-US";
        }

        if (_loadedLocales.Contains(locale))
        {
            _currentLocale = locale;
            return;
        }

        _strings.Clear();

        // Try BepInEx path first (runtime)
        string bepinexPath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "dinoforge-i18n",
            $"{locale}.json");

        // Fallback to repo assets path (development)
        string assetsPath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "..",
            "..",
            "..",
            "assets",
            "i18n",
            $"{locale}.json");

        string jsonPath = null;

        if (File.Exists(bepinexPath))
        {
            jsonPath = bepinexPath;
        }
        else if (File.Exists(assetsPath))
        {
            jsonPath = assetsPath;
        }
        else if (locale != "en-US")
        {
            // Recursively fall back to en-US
            LoadLocale("en-US");
            return;
        }

        if (jsonPath != null)
        {
            try
            {
                string json = File.ReadAllText(jsonPath, System.Text.Encoding.UTF8);
                using (var doc = JsonDocument.Parse(json))
                {
                    foreach (var prop in doc.RootElement.EnumerateObject())
                    {
                        if (prop.Value.ValueKind == JsonValueKind.String)
                        {
                            _strings[prop.Name] = prop.Value.GetString() ?? prop.Name;
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                BepInEx.Logging.Logger.CreateLogSource("DINOForge.L10n")
                    ?.LogWarning($"Failed to load locale {locale}: {ex.Message}");

                // Fall back to en-US if load fails
                if (locale != "en-US")
                {
                    LoadLocale("en-US");
                    return;
                }
            }
        }

        _currentLocale = locale;
        _loadedLocales.Add(locale);
    }

    /// <summary>
    /// Translates a key to the current locale's value.
    /// Returns fallback if provided, otherwise the key itself.
    /// </summary>
    public static string T(string key, string? fallback = null)
    {
        if (string.IsNullOrEmpty(key))
        {
            return fallback ?? key;
        }

        if (_strings.TryGetValue(key, out var value))
        {
            return value;
        }

        return fallback ?? key;
    }

    /// <summary>
    /// Translates and formats a key with positional string arguments.
    /// Example: T("menu.profile.confirm_delete", "MyProfile") → "Delete profile 'MyProfile'?"
    /// </summary>
    public static string T(string key, params object[] args)
    {
        string template = T(key);

        try
        {
            return string.Format(System.Globalization.CultureInfo.InvariantCulture, template, args);
        }
        catch
        {
            // If formatting fails, return the unformatted template
            return template;
        }
    }

    /// <summary>
    /// Gets a read-only list of available locale codes by scanning the i18n directory.
    /// </summary>
    public static IReadOnlyList<string> GetAvailableLocales()
    {
        var locales = new List<string> { "en-US" };

        string bepinexDir = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "dinoforge-i18n");

        string assetsDir = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "..",
            "..",
            "..",
            "assets",
            "i18n");

        foreach (string dir in new[] { bepinexDir, assetsDir })
        {
            if (Directory.Exists(dir))
            {
                foreach (var file in Directory.EnumerateFiles(dir, "*.json"))
                {
                    string locale = Path.GetFileNameWithoutExtension(file);
                    if (!string.IsNullOrEmpty(locale) && !locales.Contains(locale))
                    {
                        locales.Add(locale);
                    }
                }
            }
        }

        return locales.AsReadOnly();
    }

    /// <summary>
    /// Initializes L10n with the default locale (en-US).
    /// Called once at startup.
    /// </summary>
    public static void Initialize()
    {
        // Check for DINOFORGE_LOCALE env var
        string envLocale = Environment.GetEnvironmentVariable("DINOFORGE_LOCALE") ?? "en-US";
        LoadLocale(envLocale);
    }
}
