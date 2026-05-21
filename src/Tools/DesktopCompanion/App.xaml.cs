using System;
using DINOForge.DesktopCompanion.Data;
using DINOForge.DesktopCompanion.ViewModels;
using DINOForge.Tools.DesktopCompanion;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;

namespace DINOForge.DesktopCompanion
{
    /// <summary>
    /// WinUI 3 application entry point.
    /// Bootstraps the DI container and launches <see cref="MainWindow"/>.
    /// </summary>
    public partial class App : Application
    {
        /// <summary>Application-wide DI service provider.</summary>
        public static IServiceProvider Services { get; private set; } = null!;

        private MainWindow? _mainWindow;

        /// <summary>Initialises a new instance of <see cref="App"/>.</summary>
        public App()
        {
            try
            {
                CompanionLogger.Append("App ctor start");
                InitializeComponent();
                CompanionLogger.Append("InitializeComponent done");
                Services = BuildServiceProvider();
                CompanionLogger.Append("DI done");
            }
            catch (Exception ex)
            {
                CompanionLogger.Append($"ERROR: {ex}");
                throw;
            }
        }

        /// <inheritdoc />
        protected override void OnLaunched(LaunchActivatedEventArgs args)
        {
            try
            {
                CompanionLogger.Append("OnLaunched");
                _mainWindow = new MainWindow();
                CompanionLogger.Append("MainWindow created");
                _mainWindow.Activate();
                CompanionLogger.Append("Activate done");
            }
            catch (Exception ex)
            {
                CompanionLogger.Append($"ERROR: {ex}");
                throw;
            }
        }

        private static IServiceProvider BuildServiceProvider()
        {
            ServiceCollection services = new ServiceCollection();

            // Data layer
            services.AddSingleton<AppConfigService>();
            services.AddSingleton<DisabledPacksService>();
            services.AddSingleton<IPackDataService, FileSystemPackDataService>();

            // ViewModels
            services.AddTransient<MainViewModel>();
            services.AddTransient<DashboardViewModel>();
            services.AddTransient<PackListViewModel>();
            services.AddTransient<AssetBrowserViewModel>();
            services.AddTransient<DebugPanelViewModel>();
            services.AddTransient<SettingsViewModel>();

            return services.BuildServiceProvider(new ServiceProviderOptions { ValidateOnBuild = true, ValidateScopes = true });
        }
    }
}
