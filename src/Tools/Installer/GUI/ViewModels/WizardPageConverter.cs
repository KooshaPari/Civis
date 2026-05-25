using Avalonia.Data.Converters;
using System;
using System.Globalization;

namespace DINOForge.Installer.ViewModels;

/// <summary>
/// Converts a <see cref="WizardPage"/> enum value to a 1-based step number string.
/// Used in the navigation bar step indicator.
/// </summary>
public sealed class WizardPageConverter : IValueConverter
{
    /// <summary>Singleton instance for use in XAML.</summary>
    public static readonly WizardPageConverter Instance = new();

    /// <inheritdoc />
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is WizardPage page)
            return ((int)page + 1).ToString(culture);
        return "1";
    }

    /// <inheritdoc />
    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}
