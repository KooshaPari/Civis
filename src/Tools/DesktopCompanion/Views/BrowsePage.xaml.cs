using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Browse page code-behind — displays available packs from a catalog with search and filter.
    /// </summary>
    public sealed partial class BrowsePage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public BrowseViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="BrowsePage"/>.</summary>
        public BrowsePage()
        {
            ViewModel = App.Services.GetRequiredService<BrowseViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            // Optionally auto-load from a passed parameter
            if (e.Parameter is string catalogSource && !string.IsNullOrEmpty(catalogSource))
            {
                ViewModel.CatalogSource = catalogSource;
            }
        }
    }
}
