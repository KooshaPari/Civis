using System.Collections.ObjectModel;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Represents one collapsible debug section (mirrors an F9 panel section).
    /// </summary>
    public sealed class DebugSectionViewModel
    {
        /// <summary>Display name for the section header.</summary>
        public string SectionName { get; init; } = "";

        /// <summary>Lines of diagnostic text displayed in the section body.</summary>
        public ObservableCollection<string> Lines { get; init; } = new ObservableCollection<string>();
    }
}
