using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Debug panel page code-behind — mirrors the F9 in-game debug overlay.
    /// </summary>
    public sealed partial class DebugPanelPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public DebugPanelViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="DebugPanelPage"/>.</summary>
        public DebugPanelPage()
        {
            ViewModel = App.Services.GetRequiredService<DebugPanelViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            await ViewModel.RefreshAsync().ConfigureAwait(true);
        }
    }
}
