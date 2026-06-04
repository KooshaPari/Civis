using System;
using System.IO;
using System.Text.RegularExpressions;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Runtime;

/// <summary>
/// Characterization tests for the engine-UI injection-race fix
/// (fix/engine-ui-injection-race). The "no Mods button / no engine UI" symptom is an
/// INTERMITTENT injection race: the MainMenu-mode pack-load / native button injection
/// could miss its timing window (ECS-world gate at main menu, late canvas, DINO's custom
/// <c>MainMenuButton : Selectable</c>) and never recover until the next scene change.
///
/// These are SOURCE-TEXT characterization fixtures — same strategy as
/// <see cref="NativeMenuInjectorCharacterizationTests"/>. The Runtime layer references
/// Unity/BepInEx/Unity.Entities (none available to the net8.0 test host) and the SUTs are
/// private members of a <c>MonoBehaviour</c>, so we pin the load-bearing structural
/// invariants at the file level. They fail loudly if any anti-race guarantee is dropped.
/// </summary>
[Trait("Category", "NativeMenu")]
public sealed class EngineUiSelfHealCharacterizationTests
{
    private static readonly Lazy<string> PluginSource =
        new(() => File.ReadAllText(LocateSource(Path.Combine("src", "Runtime", "Plugin.cs")), System.Text.Encoding.UTF8));

    private static readonly Lazy<string> InjectorSource =
        new(() => File.ReadAllText(LocateSource(Path.Combine("src", "Runtime", "UI", "NativeMenuInjector.cs")), System.Text.Encoding.UTF8));

    private static string LocateSource(string relativePath)
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        for (int i = 0; i < 20 && dir != null; i++, dir = dir.Parent)
        {
            if (File.Exists(Path.Combine(dir.FullName, "global.json")))
            {
                var path = Path.Combine(dir.FullName, relativePath);
                if (File.Exists(path))
                {
                    return path;
                }
            }
        }
        throw new InvalidOperationException(
            $"{relativePath} not located from {AppContext.BaseDirectory}; "
            + "characterization tests cannot run without source access.");
    }

    // ------------------------------------------------------------------ //
    // Injector: public injection-state accessor for the self-heal loop
    // ------------------------------------------------------------------ //

    /// <summary>
    /// The self-healing retry in RuntimeDriver must be able to ask the injector whether the
    /// MODS button is actually present (and still alive). Without a public accessor the loop
    /// cannot decide when to stop retrying, so the race recovery would be impossible.
    /// </summary>
    [Fact]
    public void NativeMenuInjector_ExposesIsModsButtonInjectedAccessor()
    {
        var src = InjectorSource.Value;
        Regex.IsMatch(src, @"public\s+bool\s+IsModsButtonInjected\s*=>\s*_injected\s*&&\s*_injectedButton\s*!=\s*null")
            .Should().BeTrue(because:
                "the self-heal retry loop needs a public liveness check (injected AND button alive)");
    }

    // ------------------------------------------------------------------ //
    // RuntimeDriver: deterministic, idempotent MainMenu-mode init path
    // ------------------------------------------------------------------ //

    /// <summary>
    /// RunMainMenuInit must exist and load packs WITHOUT requiring an ECS world — at the main
    /// menu DINO has no ECS world, so the ECS-gated world-ready coroutine never runs there.
    /// This is the only path that brings up the engine UI at the main menu.
    /// </summary>
    [Fact]
    public void RuntimeDriver_HasIdempotentRunMainMenuInit_ThatLoadsPacksWithoutEcsWorld()
    {
        var src = PluginSource.Value;
        Regex.IsMatch(src, @"private\s+void\s+RunMainMenuInit\s*\(\s*string\s+reason\s*\)")
            .Should().BeTrue(because: "the deterministic MainMenu-mode init path must exist");

        var body = GetMethodBody(src, "private void RunMainMenuInit(string reason)");
        body.Should().Contain("_modPlatform.LoadPacks()",
            because: "MainMenu-mode init loads packs (pure YAML parse — no ECS world needed)");
        body.Should().Contain("WireUguiToModPlatform()",
            because: "MainMenu-mode init must wire the UGUI mod menu to the platform");
        body.Should().Contain("PushLoadedPacksToUgui(",
            because: "MainMenu-mode init must push loaded packs into the F10 panel");
        body.Should().Contain("TryInjectMenuButton()",
            because: "MainMenu-mode init must attempt native MODS-button injection");
        body.Should().Contain("LogEngineUiHeartbeat(",
            because: "MainMenu-mode init must emit the engine-UI readiness heartbeat");
    }

    /// <summary>
    /// Every failure inside RunMainMenuInit must be surfaced (logged), never silently
    /// swallowed (Pattern #104/#111). A swallowed exception here is exactly what produced
    /// the invisible "no engine UI" symptom.
    /// </summary>
    [Fact]
    public void RunMainMenuInit_LogsFailures_NoSilentSwallow()
    {
        var body = GetMethodBody(PluginSource.Value, "private void RunMainMenuInit(string reason)");
        Regex.IsMatch(body, @"catch\s*\(\s*Exception\s+\w+\s*\)")
            .Should().BeTrue(because: "RunMainMenuInit must catch to stay non-fatal");
        Regex.IsMatch(body, @"catch\s*\(\s*Exception\s+\w+\s*\)\s*\{[\s\S]{0,300}?LogError\(")
            .Should().BeTrue(because: "the top-level catch must LogError — no silent swallow (Pattern #104/#111)");
    }

    // ------------------------------------------------------------------ //
    // Self-healing: scene-change re-init + bounded retry
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Re-entering a menu scene (e.g. returning from gameplay) must re-run the idempotent
    /// menu-mode init so the engine UI is rebuilt. The subscription must be paired with an
    /// unsubscribe (Pattern #105) to avoid stale-handler invocation after destruction.
    /// </summary>
    [Fact]
    public void RuntimeDriver_ReRunsMenuInit_OnSceneChange_AndUnsubscribes()
    {
        var src = PluginSource.Value;
        src.Should().Contain("SceneManager.activeSceneChanged += OnRuntimeDriverSceneChanged",
            because: "scene-change self-heal must subscribe");
        src.Should().Contain("SceneManager.activeSceneChanged -= OnRuntimeDriverSceneChanged",
            because: "subscription must be paired with unsubscribe in OnDestroy (Pattern #105)");

        var handler = GetMethodBody(src, "private void OnRuntimeDriverSceneChanged(Scene previous, Scene next)");
        handler.Should().Contain("RunMainMenuInit(\"scene-change\")",
            because: "the scene-change handler must re-run the idempotent menu-mode init");
        Regex.IsMatch(handler, @"_menuInitRetryFrames\s*=\s*0")
            .Should().BeTrue(because: "scene change must re-arm the bounded retry budget");
        Regex.IsMatch(handler, @"_engineUiHeartbeatLogged\s*=\s*false")
            .Should().BeTrue(because: "scene change must re-arm the heartbeat for the new scene");
    }

    /// <summary>
    /// The main-thread pump loop must bounded-retry MODS injection while the button is not yet
    /// present, capped by a retry budget — this is what closes the intermittent timing window.
    /// </summary>
    [Fact]
    public void RuntimeDriver_BoundedRetriesInjection_UntilModsButtonExists()
    {
        var src = PluginSource.Value;
        Regex.IsMatch(src, @"private\s+const\s+int\s+MenuInitMaxRetryFrames\s*=\s*\d+")
            .Should().BeTrue(because: "the retry must be bounded by a named budget");
        Regex.IsMatch(src, @"!\s*_nativeMenuInjector\.IsModsButtonInjected[\s\S]{0,80}?_menuInitRetryFrames\s*<\s*MenuInitMaxRetryFrames")
            .Should().BeTrue(because:
                "the pump loop must retry only while the button is missing AND budget remains");
    }

    // ------------------------------------------------------------------ //
    // Heartbeat: single unambiguous engine-UI readiness line
    // ------------------------------------------------------------------ //

    /// <summary>
    /// A single clear log line must report engine-UI state so the user can confirm it from the
    /// BepInEx console at a glance. The exact prefix/keys are part of the user-facing contract.
    /// </summary>
    [Fact]
    public void RuntimeDriver_EmitsEngineUiHeartbeatLine_WithExpectedShape()
    {
        var src = PluginSource.Value;
        Regex.IsMatch(src,
            @"\[DINOForge\]\s*ENGINE-UI READY:\s*packs=\{packs\}\s*modsButton=\{modsButton\}\s*f9=\{f9\}\s*f10=\{f10\}")
            .Should().BeTrue(because:
                "the heartbeat is the user-facing confirmation line and its shape is contractual");
    }

    // ------------------------------------------------------------------ //
    // Helpers
    // ------------------------------------------------------------------ //

    private static string GetMethodBody(string src, string signature)
    {
        var sigIdx = src.IndexOf(signature, StringComparison.Ordinal);
        sigIdx.Should().BeGreaterThan(0, because: $"method signature must exist: {signature}");
        var openBrace = src.IndexOf('{', sigIdx);
        openBrace.Should().BeGreaterThan(sigIdx, because: "method body must follow the signature");

        int depth = 0;
        for (int i = openBrace; i < src.Length; i++)
        {
            if (src[i] == '{') depth++;
            else if (src[i] == '}')
            {
                depth--;
                if (depth == 0) return src.Substring(openBrace, i - openBrace + 1);
            }
        }
        throw new InvalidOperationException("Brace match failed — file likely truncated.");
    }
}
