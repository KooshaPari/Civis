using System;
using DINOForge.DesktopCompanion.Views;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace DINOForge.DesktopCompanion
{
    /// <summary>
    /// Shell window containing the NavigationView and content Frame.
    /// </summary>
    public sealed partial class MainWindow : Window
    {
        /// <summary>Initializes a new instance of <see cref="MainWindow"/>.</summary>
        public MainWindow()
        {
            InitializeComponent();
            ExtendsContentIntoTitleBar = true;

            // Navigate to dashboard on startup
            NavView.SelectedItem = NavView.MenuItems[0];
            ContentFrame.Navigate(typeof(DashboardPage));
        }

        private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
        {
            if (args.IsSettingsSelected)
            {
                ContentFrame.Navigate(typeof(SettingsPage));
                return;
            }

            NavigationViewItem? item = args.SelectedItem as NavigationViewItem;
            string? tag = item?.Tag?.ToString();

            Type? pageType = tag switch
            {
                "Dashboard" => typeof(DashboardPage),
                "PackList" => typeof(PackListPage),
                "DebugPanel" => typeof(DebugPanelPage),
                "Settings" => typeof(SettingsPage),
                _ => null
            };

            if (pageType != null && ContentFrame.CurrentSourcePageType != pageType)
            {
                ContentFrame.Navigate(pageType);
            }
        }
    }
}
