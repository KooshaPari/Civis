using System;
using Microsoft.UI;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace DINOForge.DesktopCompanion.Themes.Converters
{
    /// <summary>
    /// Converts an error count integer to a <see cref="SolidColorBrush"/>.
    /// 0 → accent green (#4CAF50), &gt;0 → warning red (#F44336).
    /// </summary>
    public sealed class ErrorCountToColorConverter : IValueConverter
    {
        private static readonly SolidColorBrush GreenBrush =
            new SolidColorBrush(Color.FromArgb(0xFF, 0x4C, 0xAF, 0x50));

        private static readonly SolidColorBrush RedBrush =
            new SolidColorBrush(Color.FromArgb(0xFF, 0xF4, 0x43, 0x36));

        /// <inheritdoc />
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            int count = value is int i ? i : 0;
            return count == 0 ? GreenBrush : RedBrush;
        }

        /// <inheritdoc />
        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotSupportedException();
        }
    }
}
