#nullable enable
using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// NATIVE-001..004: Main and pause menu "Mods" button injection and persistence.
/// Conditionally skips on CI where DINO_GAME_PATH is not set.
/// On a self-hosted Windows runner with the game installed, these tests run.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchNativeMenuTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// NATIVE-001: Main menu contains injected "Mods" button.
    /// </summary>
    [SkippableFact]
    public async Task MainMenu_HasModsButton_AfterInjection()
    {
        fixture.SkipIfNotInitialized();

        UiActionResult result = await fixture.Client!.QueryUiAsync("button:contains('Mods')");
        result.Should().NotBeNull("Mods button should be queryable in UI");
        result.MatchCount.Should().BeGreaterThan(0,
            "UI should contain a 'Mods' button or element in the main menu");
    }

    /// <summary>
    /// NATIVE-002: Clicking the "Mods" button activates the mod menu overlay.
    /// </summary>
    [SkippableFact]
    public async Task MainMenu_ModsButton_OpensOverlay()
    {
        fixture.SkipIfNotInitialized();

        UiActionResult result = await fixture.Client!.QueryUiAsync("button:contains('Mods')");
        result.Should().NotBeNull("Mods button should be found in main menu");
        result.MatchCount.Should().BeGreaterThan(0, "Mods button should match the selector");

        UiActionResult clickResult = await fixture.Client.ClickUiAsync("button:contains('Mods')");
        clickResult.Success.Should().BeTrue("clicking the Mods button should succeed");

        await Task.Delay(300);

        UiWaitResult waitResult = await fixture.Client.WaitForUiAsync(
            "element:contains('ModMenu')", "visible", timeoutMs: 2000);
        waitResult.Ready.Should().BeTrue(
            "mod menu should be visible after Mods button click");
    }

    /// <summary>
    /// NATIVE-004 / SPEC-002 manual AC #7: pause menu contains injected Mods button after gameplay pause.
    /// Bridge has no simulateKey/ESC endpoint; opens pause via <see cref="GameClient.InvokeMethodAsync"/> discovery.
    /// Skips when pause menu cannot be opened programmatically (verify manually or MCP <c>pause_menu</c> scenario).
    /// </summary>
    [SkippableFact]
    public async Task PauseMenu_HasModsButton_WhenOpened()
    {
        fixture.SkipIfNotInitialized();

        StartGameResult startResult = await fixture.Client!.StartGameAsync();
        startResult.Success.Should().BeTrue("StartGameAsync should enter gameplay from main menu");

        await Task.Delay(3000);

        GameStatus status = await fixture.Client.StatusAsync();
        status.WorldReady.Should().BeTrue("ECS world should be ready before opening pause menu");

        bool pauseOpened = await TryOpenPauseMenuAsync(fixture.Client);
        Skip.IfNot(pauseOpened,
            "Could not open pause menu via bridge invokeMethod (no ESC/simulateKey); run manual AC #7 or MCP pause_menu scenario.");

        await Task.Delay(2500);

        (await IsPauseMenuVisibleAsync(fixture.Client)).Should().BeTrue(
            "pause menu should expose Resume/Continue after pause open attempt");

        UiActionResult modsQuery = await fixture.Client.QueryUiAsync("button:contains('Mods')");
        modsQuery.MatchCount.Should().BeGreaterThan(0,
            "pause menu should contain the injected Mods button adjacent to Settings/Options");
    }

    /// <summary>
    /// NATIVE-003: Mods button and menu survive scene transitions (main menu -> gameplay).
    /// </summary>
    [SkippableFact]
    public async Task MainMenu_ModsButton_PersistsAcrossSceneChanges()
    {
        fixture.SkipIfNotInitialized();

        UiActionResult initialQuery = await fixture.Client!.QueryUiAsync("button:contains('Mods')");
        initialQuery.MatchCount.Should().BeGreaterThan(0,
            "Mods button should exist in main menu initially");

        await fixture.Client.StartGameAsync();
        await Task.Delay(3000);

        GameStatus status = await fixture.Client.StatusAsync();
        status.WorldReady.Should().BeTrue(
            "ECS world should be ready after scene transition");

        await fixture.Client.LoadSceneAsync("MainMenu");
        await Task.Delay(2000);

        UiActionResult finalQuery = await fixture.Client.QueryUiAsync("button:contains('Mods')");
        finalQuery.MatchCount.Should().BeGreaterThan(0,
            "Mods button should persist after scene transition cycle");
    }

    /// <summary>
    /// SPEC-002 F-03 / manual AC #4: injected Mods button matches native Settings (or Options donor) styling.
    /// Uses <see cref="GameClient.GetUiTreeAsync"/> style metadata populated by <c>UiTreeSnapshotBuilder</c>.
    /// </summary>
    [SkippableFact]
    public async Task MainMenu_ModsButton_StyleMatchesSettings_AfterInjection()
    {
        fixture.SkipIfNotInitialized();

        UiActionResult modsQuery = await fixture.Client!.QueryUiAsync("button:contains('Mods')");
        modsQuery.MatchCount.Should().BeGreaterThan(0,
            "Mods button must be injected before comparing visual style");

        UiTreeResult tree = await fixture.Client.GetUiTreeAsync();
        tree.Success.Should().BeTrue("GetUiTreeAsync should succeed on main menu");

        UiNode? modsButton = FindModsMenuButton(tree.Root);
        UiNode? referenceButton = FindNativeMenuReferenceButton(tree.Root);

        modsButton.Should().NotBeNull("UI tree should contain the injected Mods button");
        referenceButton.Should().NotBeNull(
            "UI tree should contain a native Settings or Options button to compare against");

        modsButton!.Style.Should().NotBeNull("Mods button should expose bridge style metadata");
        referenceButton!.Style.Should().NotBeNull("reference native menu button should expose style metadata");

        UiStyleSnapshot modsStyle = modsButton.Style!;
        UiStyleSnapshot referenceStyle = referenceButton.Style!;

        // Mirrors NativeMenuInjector.SyncButtonVisualStyle: transition + ColorBlock + label Text styling.
        modsStyle.Transition.Should().Be(referenceStyle.Transition,
            "Mods button should use the same Selectable transition mode as the native donor");
        modsStyle.NormalColor.Should().Be(referenceStyle.NormalColor);
        modsStyle.HighlightedColor.Should().Be(referenceStyle.HighlightedColor);
        modsStyle.PressedColor.Should().Be(referenceStyle.PressedColor);
        modsStyle.DisabledColor.Should().Be(referenceStyle.DisabledColor);
        modsStyle.FontSize.Should().Be(referenceStyle.FontSize,
            "Mods label font size should match the native menu donor");
        modsStyle.TextColor.Should().Be(referenceStyle.TextColor,
            "Mods label color should match the native menu donor");

        if (modsButton.Bounds != null && referenceButton.Bounds != null)
        {
            modsButton.Bounds.Width.Should().BeApproximately(referenceButton.Bounds.Width, 2f,
                "cloned Mods button should preserve donor layout width");
            modsButton.Bounds.Height.Should().BeApproximately(referenceButton.Bounds.Height, 2f,
                "cloned Mods button should preserve donor layout height");
        }
    }

    private static UiNode? FindModsMenuButton(UiNode root)
    {
        foreach (UiNode node in EnumerateNodes(root))
        {
            if (!string.Equals(node.Role, "button", StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            if (node.Name.StartsWith("DINOForge_ModsButton", StringComparison.OrdinalIgnoreCase))
            {
                return node;
            }

            if (string.Equals(node.Label, "Mods", StringComparison.OrdinalIgnoreCase))
            {
                return node;
            }
        }

        return null;
    }

    private static UiNode? FindNativeMenuReferenceButton(UiNode root)
    {
        UiNode? settings = null;
        UiNode? options = null;

        foreach (UiNode node in EnumerateNodes(root))
        {
            if (!string.Equals(node.Role, "button", StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            if (string.Equals(node.Label, "Settings", StringComparison.OrdinalIgnoreCase))
            {
                settings = node;
            }
            else if (node.Label.IndexOf("Options", StringComparison.OrdinalIgnoreCase) >= 0)
            {
                options ??= node;
            }
        }

        return settings ?? options;
    }

    private static IEnumerable<UiNode> EnumerateNodes(UiNode node)
    {
        yield return node;
        foreach (UiNode child in node.Children)
        {
            foreach (UiNode descendant in EnumerateNodes(child))
            {
                yield return descendant;
            }
        }
    }

    private static async Task<bool> TryOpenPauseMenuAsync(GameClient client)
    {
        (string target, string method)[] attempts =
        [
            ("PauseMenu", "Show"),
            ("PauseMenu", "Open"),
            ("PauseMenu", "Toggle"),
            ("PauseMenuManager", "TogglePause"),
            ("PauseMenuManager", "Show"),
            ("GamePause", "Toggle"),
            ("Pause", "Toggle"),
            ("Pause", "Show"),
        ];

        foreach ((string target, string method) in attempts)
        {
            StartGameResult result = await client.InvokeMethodAsync(target, method);
            if (result.Success)
            {
                return true;
            }
        }

        return false;
    }

    private static async Task<bool> IsPauseMenuVisibleAsync(GameClient client)
    {
        UiActionResult resume = await client.QueryUiAsync("button:contains('Resume')");
        if (resume.MatchCount > 0)
        {
            return true;
        }

        UiActionResult continueButton = await client.QueryUiAsync("button:contains('Continue')");
        return continueButton.MatchCount > 0;
    }
}
