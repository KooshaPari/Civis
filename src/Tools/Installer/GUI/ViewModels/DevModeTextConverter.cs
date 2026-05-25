using Avalonia.Data.Converters;
using System;
using System.Globalization;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// Converts an IsDevMode boolean to a human-readable mode label.
/// </summary>
public sealed class DevModeTextConverter : IValueConverter
{
    /// <summary>Singleton instance for use in XAML.</summary>
    public static readonly DevModeTextConverter Instance = new();

    /// <inheritdoc />
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is true ? "Developer" : "Player";
    }

    /// <inheritdoc />
    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}
