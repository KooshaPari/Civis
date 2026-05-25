using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;

namespace DINOForge.DesktopCompanion.Themes.Converters
{
    /// <summary>
    /// Converts a nullable object to a <see cref="Visibility"/> value.
    /// <c>null</c> → <see cref="Visibility.Collapsed"/>, non-<c>null</c> → <see cref="Visibility.Visible"/>.
    /// </summary>
    public sealed class NullToVisibilityConverter : IValueConverter
    {
        /// <inheritdoc />
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            return value == null ? Visibility.Collapsed : Visibility.Visible;
        }

        /// <inheritdoc />
        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }
}
