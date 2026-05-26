using System;
using Microsoft.UI.Xaml.Data;

namespace DINOForge.DesktopCompanion.Themes.Converters
{
    /// <summary>
    /// Converts a nullable object to a string.
    /// <c>null</c> → "(no manifest)", non-<c>null</c> → calls <see cref="object.ToString()"/>.
    /// </summary>
    public sealed class NullToStringConverter : IValueConverter
    {
        /// <inheritdoc />
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            return value?.ToString() ?? "(no manifest)";
        }

        /// <inheritdoc />
        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }
}
