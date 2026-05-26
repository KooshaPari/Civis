#nullable enable

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// Scene identifiers for <see cref="DINOForge.Bridge.Client.GameClient.LoadSceneAsync"/>.
/// DINO main menu is build index 1 (see NativeMenuInjector InitialGameLoader → LoadScene(1)).
/// </summary>
internal static class GameLaunchSceneNames
{
    public const string MainMenuBuildIndex = "1";
}
