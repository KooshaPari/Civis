using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Pack list page code-behind — mirrors the F10 mod menu pack list.
    /// </summary>
    public sealed partial class PackListPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public PackListViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="PackListPage"/>.</summary>
        public PackListPage()
        {
            ViewModel = App.Services.GetRequiredService<PackListViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            await ViewModel.ReloadAsync().ConfigureAwait(true);
        }
    }
}
