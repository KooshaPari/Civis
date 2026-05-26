#nullable enable
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock of <see cref="IModCanvas"/> for unit tests. Tracks every
/// spec passed to Create* and every handle passed to Destroy. Pattern #125 seam
/// double — pairs with <c>ModCanvasAdapter</c> (sole production impl) so future
/// canvases (e.g. world-space overlay) inherit a canonical fake.
/// </summary>
public sealed class MockModCanvas : IModCanvas
{
    /// <summary>Every panel spec sent to <see cref="CreatePanel"/>.</summary>
    public List<PanelSpec> PanelsCreated { get; } = new();

    /// <summary>Every label spec sent to <see cref="CreateLabel"/>.</summary>
    public List<LabelSpec> LabelsCreated { get; } = new();

    /// <summary>Every button spec sent to <see cref="CreateButton"/>.</summary>
    public List<ButtonSpec> ButtonsCreated { get; } = new();

    /// <summary>Every handle sent to <see cref="Destroy"/>.</summary>
    public List<ExtendedHandle> Destroyed { get; } = new();

    /// <inheritdoc />
    public ExtendedPanelHandle CreatePanel(PanelSpec spec)
    {
        PanelsCreated.Add(spec);
        return new ExtendedPanelHandle(spec.Id, new object());
    }

    /// <inheritdoc />
    public ExtendedLabelHandle CreateLabel(LabelSpec spec)
    {
        LabelsCreated.Add(spec);
        return new ExtendedLabelHandle(spec.Id, new object());
    }

    /// <inheritdoc />
    public ExtendedButtonHandle CreateButton(ButtonSpec spec)
    {
        ButtonsCreated.Add(spec);
        return new ExtendedButtonHandle(spec.Id, new object());
    }

    /// <inheritdoc />
    public void Destroy(ExtendedHandle handle)
    {
        Destroyed.Add(handle);
    }
}
