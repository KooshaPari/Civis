#nullable enable
using System;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using Newtonsoft.Json.Linq;

namespace DINOForge.Bridge.Client;

/// <summary>
/// Interface for communicating with the DINOForge in-game IPC bridge server
/// over named pipes using JSON-RPC 2.0.
/// </summary>
/// <remarks>
/// Enables testing and mocking of GameClient. All implementations should be thread-safe.
/// </remarks>
public interface IGameClient : IDisposable
{
    /// <summary>Gets the current connection state.</summary>
    ConnectionState State { get; }

    /// <summary>Gets whether the client is currently connected to the game bridge.</summary>
    bool IsConnected { get; }

    /// <summary>
    /// Connects to the game bridge named pipe server.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <exception cref="GameClientException">Thrown when the connection fails.</exception>
    Task ConnectAsync(CancellationToken ct = default);

    /// <summary>
    /// Disconnects from the game bridge server.
    /// </summary>
    void Disconnect();

    /// <summary>
    /// Pings the game bridge to verify connectivity and uptime.
    /// </summary>
    Task<PingResult> PingAsync(CancellationToken ct = default);

    /// <summary>
    /// Retrieves the current game status including world, entity count, and loaded packs.
    /// </summary>
    Task<GameStatus> StatusAsync(CancellationToken ct = default);

    /// <summary>
    /// Waits for the ECS world to be ready for queries.
    /// </summary>
    /// <param name="timeoutMs">Optional timeout in milliseconds.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<WaitResult> WaitForWorldAsync(int? timeoutMs = null, CancellationToken ct = default);

    /// <summary>
    /// Queries entities by optional component type or category.
    /// </summary>
    /// <param name="componentType">Optional component type filter.</param>
    /// <param name="category">Optional category filter.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<QueryResult> QueryEntitiesAsync(string? componentType = null, string? category = null, CancellationToken ct = default);

    /// <summary>
    /// Retrieves a stat value by SDK path.
    /// </summary>
    /// <param name="sdkPath">SDK path to the stat (e.g., "unit.stats.hp").</param>
    /// <param name="entityIndex">Optional entity index.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StatResult> GetStatAsync(string sdkPath, int? entityIndex = null, CancellationToken ct = default);

    /// <summary>
    /// Applies an override to a stat.
    /// </summary>
    /// <param name="sdkPath">SDK path to the stat.</param>
    /// <param name="value">The override value.</param>
    /// <param name="mode">Optional mode (e.g., "add", "set").</param>
    /// <param name="filter">Optional filter for entity selection.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<OverrideResult> ApplyOverrideAsync(string sdkPath, float value, string? mode = null, string? filter = null, CancellationToken ct = default);

    /// <summary>
    /// Reloads content packs from disk.
    /// </summary>
    /// <param name="path">Optional pack path.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<ReloadResult> ReloadPacksAsync(string? path = null, CancellationToken ct = default);

    /// <summary>
    /// Retrieves the game catalog (units, buildings, projectiles, etc.).
    /// </summary>
    Task<CatalogSnapshot> GetCatalogAsync(CancellationToken ct = default);

    /// <summary>
    /// Dumps the current ECS state, optionally filtered by category.
    /// </summary>
    /// <param name="category">Optional category filter.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<CatalogSnapshot> DumpStateAsync(string? category = null, CancellationToken ct = default);

    /// <summary>
    /// Retrieves current game resource values.
    /// </summary>
    Task<ResourceSnapshot> GetResourcesAsync(CancellationToken ct = default);

    /// <summary>
    /// Captures an in-game screenshot.
    /// </summary>
    /// <param name="path">Optional output path for the screenshot.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<ScreenshotResult> ScreenshotAsync(string? path = null, CancellationToken ct = default);

    /// <summary>
    /// Loads a game scene by name or index.
    /// </summary>
    /// <param name="scene">Scene name or index.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<LoadSceneResult> LoadSceneAsync(string scene, CancellationToken ct = default);

    /// <summary>
    /// Triggers game world load via ECS singleton (bypasses menu).
    /// </summary>
    Task<StartGameResult> StartGameAsync(CancellationToken ct = default);

    /// <summary>
    /// Lists available save files discovered by the game bridge.
    /// </summary>
    Task<JObject> ListSavesAsync(CancellationToken ct = default);

    /// <summary>
    /// Dismisses the "Press Any Key to Continue" loading screen.
    /// </summary>
    Task<StartGameResult> DismissLoadScreenAsync(CancellationToken ct = default);

    /// <summary>
    /// Loads a save file by name (e.g. "AUTOSAVE_1" or "CONTINUE").
    /// </summary>
    /// <param name="saveName">Save file name.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StartGameResult> LoadSaveAsync(string saveName = "AUTOSAVE_1", CancellationToken ct = default);

    /// <summary>
    /// Clicks a named Unity UI button. Pass empty string to list all active buttons.
    /// Use "DINOForge_ModsButton" to click the injected Mods button.
    /// </summary>
    /// <param name="buttonName">Button name or empty to list all.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StartGameResult> ClickButtonAsync(string buttonName, CancellationToken ct = default);

    /// <summary>
    /// Toggles a DINOForge UI panel.
    /// </summary>
    /// <param name="target">UI target: "modmenu" (F10) or "debug" (F9).</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StartGameResult> ToggleUiAsync(string target = "modmenu", CancellationToken ct = default);

    /// <summary>
    /// Dumps active MonoBehaviours and their void() methods.
    /// </summary>
    /// <param name="filter">Optional filter by type/GO name.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StartGameResult> ScanSceneAsync(string filter = "", CancellationToken ct = default);

    /// <summary>
    /// Invokes a void(0-param) method on any active MonoBehaviour matching target.
    /// </summary>
    /// <param name="target">MonoBehaviour type or GameObject name.</param>
    /// <param name="method">Method name to invoke.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<StartGameResult> InvokeMethodAsync(string target, string method, CancellationToken ct = default);

    /// <summary>
    /// Performs end-to-end mod verification.
    /// </summary>
    /// <param name="packPath">Pack path to verify.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<VerifyResult> VerifyModAsync(string packPath, CancellationToken ct = default);

    /// <summary>
    /// Retrieves SDK-to-ECS component mappings.
    /// </summary>
    /// <param name="sdkPath">Optional SDK path filter.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<ComponentMapResult> GetComponentMapAsync(string? sdkPath = null, CancellationToken ct = default);

    /// <summary>
    /// Captures a live snapshot of the active Unity UI hierarchy.
    /// </summary>
    /// <param name="selector">Optional selector string for filtering.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<UiTreeResult> GetUiTreeAsync(string? selector = null, CancellationToken ct = default);

    /// <summary>
    /// Queries the live Unity UI hierarchy using a simple selector grammar.
    /// </summary>
    /// <param name="selector">Selector string to query UI nodes.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<UiActionResult> QueryUiAsync(string selector, CancellationToken ct = default);

    /// <summary>
    /// Clicks the first live Unity UI node matching the given selector.
    /// </summary>
    /// <param name="selector">Selector string to find the UI node.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<UiActionResult> ClickUiAsync(string selector, CancellationToken ct = default);

    /// <summary>
    /// Waits for a live Unity UI selector to reach the requested state.
    /// </summary>
    /// <param name="selector">Selector string to find the UI node.</param>
    /// <param name="state">Optional target state (e.g., "visible", "enabled").</param>
    /// <param name="timeoutMs">Optional timeout in milliseconds.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<UiWaitResult> WaitForUiAsync(string selector, string? state = null, int? timeoutMs = null, CancellationToken ct = default);

    /// <summary>
    /// Asserts a condition against the first node matching the given selector.
    /// </summary>
    /// <param name="selector">Selector string to find the UI node.</param>
    /// <param name="condition">Condition assertion to evaluate.</param>
    /// <param name="ct">Cancellation token.</param>
    Task<UiExpectationResult> ExpectUiAsync(string selector, string condition, CancellationToken ct = default);
}
