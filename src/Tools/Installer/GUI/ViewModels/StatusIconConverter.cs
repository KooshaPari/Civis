using Avalonia.Data.Converters;
using System;
using System.Globalization;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// Converts a boolean status value to a unicode check or cross icon string.
/// Used in the game path validation grid.
/// </summary>
public sealed class StatusIconConverter : IValueConverter
{
    /// <summary>Singleton instance for use in XAML.</summary>
    public static readonly StatusIconConverter Instance = new();

    /// <inheritdoc />
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is true ? "+" : "x";
    }

    /// <inheritdoc />
    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}
