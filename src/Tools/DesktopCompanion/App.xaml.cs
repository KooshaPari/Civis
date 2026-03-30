using System;
using System.IO;
using DINOForge.DesktopCompanion.Data;
using DINOForge.DesktopCompanion.ViewModels;
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
        private static readonly string LogPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "DINOForge.DesktopCompanion", "app.log");

        /// <summary>Application-wide DI service provider.</summary>
        public static IServiceProvider Services { get; private set; } = null!;

        private MainWindow? _mainWindow;

        /// <summary>Initialises a new instance of <see cref="App"/>.</summary>
        public App()
        {
            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(LogPath)!);
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] App ctor start\n");
                InitializeComponent();
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] InitializeComponent done\n");
                Services = BuildServiceProvider();
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] DI done\n");
            }
            catch (Exception ex)
            {
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] ERROR: {ex}\n");
                throw;
            }
        }

        /// <inheritdoc />
        protected override void OnLaunched(LaunchActivatedEventArgs args)
        {
            try
            {
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] OnLaunched\n");
                _mainWindow = new MainWindow();
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] MainWindow created\n");
                _mainWindow.Activate();
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] Activate done\n");
            }
            catch (Exception ex)
            {
                File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss.fff}] ERROR: {ex}\n");
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
            services.AddTransient<DebugPanelViewModel>();
            services.AddTransient<SettingsViewModel>();

            return services.BuildServiceProvider();
        }
    }
}
