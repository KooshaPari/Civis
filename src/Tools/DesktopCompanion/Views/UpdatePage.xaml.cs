using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Update page code-behind — checks for pack updates against a catalog.
    /// </summary>
    public sealed partial class UpdatePage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public UpdateViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="UpdatePage"/>.</summary>
        public UpdatePage()
        {
            ViewModel = App.Services.GetRequiredService<UpdateViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            if (e.Parameter is string catalogSource && !string.IsNullOrEmpty(catalogSource))
            {
                ViewModel.CatalogSource = catalogSource;
            }
        }
    }
}
