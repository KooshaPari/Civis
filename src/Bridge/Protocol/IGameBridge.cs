#nullable enable

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Defines the contract for all RPC methods exposed by the in-game bridge server.
    /// All methods are synchronous because the server dispatches to the Unity main thread.
    /// </summary>
    public interface IGameBridge
    {
        /// <summary>Get current game and mod platform status.</summary>
        GameStatus Status();

        /// <summary>Wait for the ECS world to become available.</summary>
        /// <param name="timeoutMs">Optional timeout in milliseconds.</param>
        WaitResult WaitForWorld(int? timeoutMs);

        /// <summary>Query entities matching a component type and/or category filter.</summary>
        /// <param name="componentType">ECS component type name to filter by.</param>
        /// <param name="category">Category filter (unit, building, projectile, other).</param>
        QueryResult QueryEntities(string? componentType, string? category);

        /// <summary>Read a stat value by SDK model path.</summary>
        /// <param name="sdkPath">Dot-separated SDK model path (e.g. "unit.stats.hp").</param>
        /// <param name="entityIndex">Optional specific entity index to read from.</param>
        StatResult GetStat(string sdkPath, int? entityIndex);

        /// <summary>Apply a stat override to matching entities.</summary>
        /// <param name="sdkPath">SDK model path of the stat to modify.</param>
        /// <param name="value">The value to apply.</param>
        /// <param name="mode">Application mode: "override", "add", or "multiply".</param>
        /// <param name="filter">Optional filter component type name.</param>
        OverrideResult ApplyOverride(string sdkPath, float value, string? mode, string? filter);

        /// <summary>Reload content packs from disk.</summary>
        /// <param name="path">Optional packs directory path override.</param>
        ReloadResult ReloadPacks(string? path);

        /// <summary>Get a snapshot of the vanilla entity catalog.</summary>
        CatalogSnapshot GetCatalog();

        /// <summary>Dump full game state for a given category.</summary>
        /// <param name="category">Category to dump (unit, building, projectile, all).</param>
        CatalogSnapshot DumpState(string? category);

        /// <summary>Read current resource stockpile values.</summary>
        ResourceSnapshot GetResources();

        /// <summary>Capture a screenshot of the game window.</summary>
        /// <param name="path">File path to save the screenshot.</param>
        ScreenshotResult Screenshot(string? path);

        /// <summary>Load and verify a mod pack, reporting any issues.</summary>
        /// <param name="packPath">Path to the pack directory or manifest.</param>
        VerifyResult VerifyMod(string? packPath);

        /// <summary>Ping the server to check connectivity and uptime.</summary>
        PingResult Ping();

        /// <summary>Get the component type mapping table.</summary>
        /// <param name="sdkPath">Optional SDK path filter; null returns all mappings.</param>
        ComponentMapResult GetComponentMap(string? sdkPath);

        /// <summary>Capture a live snapshot of the active Unity UI hierarchy.</summary>
        /// <param name="selector">Optional selector string for future filtering.</param>
        UiTreeResult GetUiTree(string? selector);

        /// <summary>Query the live Unity UI hierarchy using a simple selector grammar.</summary>
        /// <param name="selector">Selector such as "role=button&amp;&amp;text=Mods".</param>
        UiActionResult QueryUi(string selector);

        /// <summary>Click the first live Unity UI node matching the given selector.</summary>
        /// <param name="selector">Selector such as "name=DINOForge_ModsButton" or "role=button&amp;&amp;text=Mods".</param>
        UiActionResult ClickUi(string selector);

        /// <summary>Wait for a live Unity UI selector to reach the requested state.</summary>
        /// <param name="selector">Selector such as "role=button&amp;&amp;text=Mods".</param>
        /// <param name="state">Target state: visible, hidden, or interactable.</param>
        /// <param name="timeoutMs">Timeout in milliseconds.</param>
        UiWaitResult WaitForUi(string selector, string? state, int? timeoutMs);

        /// <summary>Assert a condition against the first node matching the given selector.</summary>
        /// <param name="selector">Selector such as "role=button&amp;&amp;text=Mods".</param>
        /// <param name="condition">Condition such as visible, hidden, interactable, or text=Mods.</param>
        UiExpectationResult ExpectUi(string selector, string condition);
    }
}
