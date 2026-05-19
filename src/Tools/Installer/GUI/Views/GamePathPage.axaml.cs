using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using DINOForge.Installer.ViewModels;
using System.Linq;
using System.Threading.Tasks;

namespace DINOForge.Installer.Views;

/// <summary>
/// Game path detection page — wires the folder picker dialog to the ViewModel.
/// </summary>
public partial class GamePathPage : UserControl
{
    public GamePathPage()
    {
        InitializeComponent();
    }

    /// <summary>
    /// Subscribes to <see cref="Control.DataContextChanged"/> when the control attaches
    /// to the visual tree (Pattern #105: paired with <see cref="OnDetachedFromVisualTree"/>
    /// to ensure event subscription lifecycle symmetry).
    /// </summary>
    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        DataContextChanged += OnDataContextChanged;
    }

    /// <summary>
    /// Unsubscribes from <see cref="Control.DataContextChanged"/> when the control detaches
    /// from the visual tree (Pattern #105: paired with <see cref="OnAttachedToVisualTree"/>).
    /// </summary>
    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        DataContextChanged -= OnDataContextChanged;
        base.OnDetachedFromVisualTree(e);
    }

    private void OnDataContextChanged(object? sender, System.EventArgs e)
    {
        if (DataContext is GamePathPageViewModel vm)
        {
            vm.BrowseDialogOpener = OpenFolderDialogAsync;
        }
    }

    private async Task<string?> OpenFolderDialogAsync()
    {
        TopLevel? topLevel = TopLevel.GetTopLevel(this);
        if (topLevel is null) return null;

        System.Collections.Generic.IReadOnlyList<IStorageFolder> folders =
            await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = "Select Diplomacy is Not an Option install folder",
                AllowMultiple = false
            }).ConfigureAwait(false);

        return folders.Count > 0 ? folders[0].Path.LocalPath : null;
    }
}
