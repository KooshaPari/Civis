using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Asset Browser page — shows discovered asset bundles across all packs.
    /// </summary>
    public sealed partial class AssetBrowserPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public AssetBrowserViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="AssetBrowserPage"/>.</summary>
        public AssetBrowserPage()
        {
            ViewModel = App.Services.GetRequiredService<AssetBrowserViewModel>();
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
