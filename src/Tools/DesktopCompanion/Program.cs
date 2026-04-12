using Microsoft.UI.Xaml;

namespace DINOForge.DesktopCompanion
{
    /// <summary>
    /// Application entry point for the unpackaged WinUI 3 companion app.
    /// The SDK auto-initializer (MddBootstrapAutoInitializer) handles Windows App Runtime bootstrap.
    /// </summary>
    public static class Program
    {
        [global::System.STAThread]
        static void Main(string[] args)
        {
            global::WinRT.ComWrappersSupport.InitializeComWrappers();
            Application.Start(p => { _ = new App(); });
        }
    }
}
