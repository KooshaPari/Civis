using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Dashboard page code-behind.
    /// </summary>
    public sealed partial class DashboardPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public DashboardViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="DashboardPage"/>.</summary>
        public DashboardPage()
        {
            ViewModel = App.Services.GetRequiredService<DashboardViewModel>();
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
