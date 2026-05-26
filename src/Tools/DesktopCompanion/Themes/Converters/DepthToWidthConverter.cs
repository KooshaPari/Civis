using System;
using Microsoft.UI.Xaml.Data;

namespace DINOForge.DesktopCompanion.Themes.Converters
{
    /// <summary>
    /// Converts tree depth level to indentation width for dependency tree visualization.
    /// Each level indents by 16 pixels.
    /// </summary>
    public sealed class DepthToWidthConverter : IValueConverter
    {
        private const double IndentPerLevel = 16.0;

        /// <inheritdoc />
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is int depth)
            {
                return depth * IndentPerLevel;
            }

            return 0.0;
        }

        /// <inheritdoc />
        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotSupportedException();
        }
    }
}
