using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;

namespace DINOForge.DesktopCompanion.Themes.Converters
{
    /// <summary>
    /// Converts a <see cref="bool"/> to a <see cref="Visibility"/> value.
    /// <c>true</c> → <see cref="Visibility.Visible"/>, <c>false</c> → <see cref="Visibility.Collapsed"/>.
    /// Pass parameter "Invert" to reverse the mapping.
    /// </summary>
    public sealed class BoolToVisibilityConverter : IValueConverter
    {
        /// <inheritdoc />
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            bool flag = value is bool b && b;
            bool invert = parameter is string s && s.Equals("Invert", StringComparison.OrdinalIgnoreCase);
            return (flag ^ invert) ? Visibility.Visible : Visibility.Collapsed;
        }

        /// <inheritdoc />
        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            bool visible = value is Visibility v && v == Visibility.Visible;
            bool invert = parameter is string s && s.Equals("Invert", StringComparison.OrdinalIgnoreCase);
            return visible ^ invert;
        }
    }
}
