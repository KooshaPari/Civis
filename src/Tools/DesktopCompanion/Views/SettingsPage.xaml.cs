using DINOForge.DesktopCompanion.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion.Views
{
    /// <summary>
    /// Settings page code-behind.
    /// </summary>
    public sealed partial class SettingsPage : Page
    {
        /// <summary>View-model resolved from the DI container.</summary>
        public SettingsViewModel ViewModel { get; }

        /// <summary>Initializes a new instance of <see cref="SettingsPage"/>.</summary>
        public SettingsPage()
        {
            ViewModel = App.Services.GetRequiredService<SettingsViewModel>();
            InitializeComponent();
        }

        /// <inheritdoc />
        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            base.OnNavigatedTo(e);
            await ViewModel.LoadAsync().ConfigureAwait(true);
        }
    }
}
