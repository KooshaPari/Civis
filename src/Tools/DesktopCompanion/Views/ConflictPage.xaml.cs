using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Conflict page code-behind — analyzes pack conflicts and dependency issues.
    /// </summary>
    public sealed partial class ConflictPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public ConflictViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="ConflictPage"/>.</summary>
        public ConflictPage()
        {
            ViewModel = App.Services.GetRequiredService<ConflictViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            // Auto-analyze on navigation if desired
        }
    }
}
