using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Runtime;

/// <summary>
/// Characterization tests for <c>NativeMenuInjector.InjectButton(Button, long)</c>
/// pinned BEFORE Pattern #222 decomposition refactor (task #538 / #479-REOPEN).
///
/// Background:
///   <c>NativeMenuInjector</c> lives in DINOForge.Runtime which references Unity (UGUI),
///   BepInEx, and Unity.Entities. None of those are available to the test host
///   (DINOForge.Tests, net8.0) because Runtime's csproj sets <c>&lt;Private&gt;false&lt;/Private&gt;</c>
///   on every Unity reference. <c>NativeMenuInjector</c> derives from <c>MonoBehaviour</c>
///   so it cannot be instantiated outside the Unity engine; <c>InjectButton</c> is
///   <c>private</c> and takes a <c>UnityEngine.UI.Button</c> argument.
///
/// Strategy:
///   These tests are *source-text* characterization fixtures. They read the
///   <c>NativeMenuInjector.cs</c> file from disk and assert observable structural
///   invariants that the refactor (per <c>docs/qa/refactor_native_menu_injector.md</c>)
///   must preserve. Each fixture maps directly to one of the 8 non-negotiable
///   behaviors enumerated in the refactor map "Risks &amp; Non-Negotiable Behaviors"
///   section, plus the 6 named fixture cases.
///
///   This is the same pattern used by <c>Pattern234TestPackLeakTests</c> for
///   pack-deployment governance — pinning behavior at the source-text level when
///   the Type system cannot reach into a layer that depends on the game install.
///
///   After the decomposition refactor (extracting 7 private helpers per
///   <c>docs/qa/refactor_native_menu_injector.md</c>) lands, these tests must
///   still pass. The regex anchors are deliberately resilient to whitespace and
///   inline comment changes so they survive cosmetic edits, but they will fail
///   loudly if any of the 8 behaviors is dropped or inverted.
///
/// SPEC-002 test-plan coverage (see <c>docs/specs/SPEC-002-native-menu-injector.md</c>):
///   Covered here (source-text, no Unity host): unit + integration items that pin
///   structure in <c>NativeMenuInjector.cs</c>.
///   Gaps when <c>GameInstalled=false</c> (no UnityEngine.dll under ManagedDir):
///     - TryInjectMenuButton_FindsSettingsButton_SetsInjected
///     - FullBoot_InjectionSucceeds
///     - OnScanNeeded_TriggersInjection (runtime invoke; see RuntimeDriver_OnScanNeeded_AssignsDelegate)
///   Gaps requiring live DINO (GameLaunch project, excluded from CI.sln):
///     - GameLaunchNativeMenuTests NATIVE-001..003
///   SPEC-002 manual AC #7 (pause menu Mods button):
///     - NativeMenuInjectorCharacterizationTests pause-menu fixtures (CI)
///     - GameLaunchNativeMenuTests NATIVE-004 (self-hosted; needs ESC/pause open via invokeMethod)
/// </summary>
[Trait("Category", "NativeMenu")]
public sealed class NativeMenuInjectorCharacterizationTests
{
    private static readonly Lazy<string> SourceText = new(() => File.ReadAllText(LocateSource(), System.Text.Encoding.UTF8));

    private static string LocateSource()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        for (int i = 0; i < 20 && dir != null; i++, dir = dir.Parent)
        {
            if (File.Exists(Path.Combine(dir.FullName, "global.json")))
            {
                var path = Path.Combine(dir.FullName, "src", "Runtime", "UI", "NativeMenuInjector.cs");
                if (File.Exists(path))
                {
                    return path;
                }
            }
        }
        throw new InvalidOperationException(
            $"NativeMenuInjector.cs not located from {AppContext.BaseDirectory}; "
            + "characterization tests cannot run without source access.");
    }

    /// <summary>
    /// Extracts the "injection code path" from the source text — i.e. the body of
    /// <c>InjectButton(Button, long)</c> PLUS the bodies of its decomposed private
    /// helpers (post Pattern #222 refactor, iter-143). Pre-refactor, this returns
    /// just the InjectButton body. Post-refactor, it returns InjectButton's body
    /// concatenated with each private helper body so the invariant regex anchors
    /// still find their targets regardless of whether the logic is inlined or
    /// extracted into helpers.
    ///
    /// Scoping: starts at <c>InjectButton</c>'s opening brace; brace-matches through
    /// the end of <c>CommitInjectionAndLog</c>'s body (the last helper introduced
    /// by the refactor map, which must remain last per behavior #7 atomicity).
    /// If <c>CommitInjectionAndLog</c> is not present (pre-refactor source), scope
    /// is just the InjectButton body.
    /// </summary>
    private static string GetInjectButtonBody()
    {
        var src = SourceText.Value;
        var sigIdx = src.IndexOf("private void InjectButton(Button settingsButton, long attemptId)",
            StringComparison.Ordinal);
        sigIdx.Should().BeGreaterThan(0,
            because: "InjectButton(Button, long) is the SUT and must exist at its known signature");

        var openBrace = src.IndexOf('{', sigIdx);
        openBrace.Should().BeGreaterThan(sigIdx, because: "method body must follow the signature");

        // Post-refactor: scope through to end of CommitInjectionAndLog (last helper, Cluster 8).
        // Pre-refactor: there's no CommitInjectionAndLog, so we fall back to just InjectButton body.
        var commitHelperIdx = src.IndexOf("private void CommitInjectionAndLog(", openBrace, StringComparison.Ordinal);

        int scopeEnd;
        if (commitHelperIdx > 0)
        {
            // Find CommitInjectionAndLog's closing brace.
            var commitOpenBrace = src.IndexOf('{', commitHelperIdx);
            commitOpenBrace.Should().BeGreaterThan(commitHelperIdx, because: "CommitInjectionAndLog body must follow signature");
            scopeEnd = MatchClosingBrace(src, commitOpenBrace);
        }
        else
        {
            // Pre-refactor scope: just InjectButton body.
            scopeEnd = MatchClosingBrace(src, openBrace);
        }

        return src.Substring(openBrace, scopeEnd - openBrace + 1);
    }

    /// <summary>
    /// Brace-match from <paramref name="openBraceIdx"/> (must be '{') to its closing '}'.
    /// Returns the index of the closing brace. Throws on mismatch (truncated file).
    /// </summary>
    private static int MatchClosingBrace(string src, int openBraceIdx)
    {
        int depth = 0;
        for (int i = openBraceIdx; i < src.Length; i++)
        {
            char c = src[i];
            if (c == '{') depth++;
            else if (c == '}')
            {
                depth--;
                if (depth == 0) { return i; }
            }
        }
        throw new InvalidOperationException("Brace match failed from index " + openBraceIdx + " — file likely truncated.");
    }

    /// <summary>
    /// Extracts the body of a method identified by a unique signature substring.
    /// Used for SPEC-002 tests outside the <c>InjectButton</c> cluster.
    /// </summary>
    private static string GetMethodBodyBySignature(string methodSignature)
    {
        var src = SourceText.Value;
        var sigIdx = src.IndexOf(methodSignature, StringComparison.Ordinal);
        sigIdx.Should().BeGreaterThan(0, because: $"method signature must exist: {methodSignature}");

        var openBrace = src.IndexOf('{', sigIdx);
        openBrace.Should().BeGreaterThan(sigIdx, because: "method body must follow the signature");

        int closeBrace = MatchClosingBrace(src, openBrace);
        return src.Substring(openBrace, closeBrace - openBrace + 1);
    }

    // ------------------------------------------------------------------ //
    // Behavior #1: Null Guard (Fixture: "Null settingsButton guard")
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior 1 (non-negotiable): InjectButton MUST early-return when
    /// settingsButton is null and MUST log a warning before returning.
    /// This is the load-bearing input validation; without it a NullReferenceException
    /// escapes to BepInEx and the Mods button never injects.
    /// </summary>
    [Fact]
    public void InjectButton_HasNullSettingsButtonGuardWithEarlyReturn()
    {
        var body = GetInjectButtonBody();
        // The guard must check settingsButton == null AND issue a LogWarning AND return early.
        Regex.IsMatch(body, @"if\s*\(\s*settingsButton\s*==\s*null\s*\)")
            .Should().BeTrue(because: "null guard for settingsButton must remain (refactor map behavior #1, fixture 1)");
        Regex.IsMatch(body, @"if\s*\(\s*settingsButton\s*==\s*null\s*\)[\s\S]{0,400}?LogWarning\([\s\S]{0,200}?NULL settingsButton")
            .Should().BeTrue(because: "null guard must LogWarning with 'NULL settingsButton' context (refactor map: graceful failure pattern)");
        Regex.IsMatch(body, @"if\s*\(\s*settingsButton\s*==\s*null\s*\)[\s\S]{0,600}?return\s*;")
            .Should().BeTrue(because: "null guard must early-return — must NOT throw to caller (refactor map behavior #3, graceful failure)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #2: Clone-Source Selection (Fixture: "1-button vs 2+-button positioning")
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior 2 (non-negotiable): Clone-source selection MUST use
    /// _allOptionsButtons.Count >= 1 as the discriminator. When the list has any
    /// Options button, clone from the LAST one and set positionAfterSibling so
    /// the Mods button is placed AFTER. When the list is null/empty, clone from
    /// settingsButton and leave positionAfterSibling null (so it goes BEFORE Settings).
    /// </summary>
    [Fact]
    public void InjectButton_CloneSourceSelectionUsesAllOptionsButtonsCount()
    {
        var body = GetInjectButtonBody();
        // The decision predicate must check _allOptionsButtons != null AND Count >= 1.
        Regex.IsMatch(body, @"_allOptionsButtons\s*!=\s*null[\s\S]{0,100}?_allOptionsButtons\.Count\s*>=\s*1")
            .Should().BeTrue(because: "clone-source decision predicate (refactor map Cluster 1, behavior #2)");

        // Inside the true branch, cloneSource is reassigned to the LAST element via Count-1 indexing.
        Regex.IsMatch(body, @"cloneSource\s*=\s*_allOptionsButtons\[\s*_allOptionsButtons\.Count\s*-\s*1\s*\]")
            .Should().BeTrue(because: "must clone from LAST Options button when 2+ exist (behavior #2 position precedence)");

        // positionAfterSibling MUST be assigned to cloneSource in the same branch.
        Regex.IsMatch(body, @"positionAfterSibling\s*=\s*cloneSource\s*;")
            .Should().BeTrue(because: "positionAfterSibling must equal cloneSource when Options buttons exist (behavior #2)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #3 + Fixture 2: Existing-button re-enforcement path
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #3 (idempotent re-enforcement): if a child of the parent transform
    /// already starts with "DINOForge_ModsButton", call EnforceModsButtonState on it,
    /// set _injectedButton/_injected, and early-return. Skipping this path causes
    /// duplicate-injection (multiple Mods buttons stack visually).
    ///
    /// Cluster 2 from refactor map.
    /// </summary>
    [Fact]
    public void InjectButton_DuplicatePreventionGuardReEnforcesAndEarlyReturns()
    {
        var body = GetInjectButtonBody();
        // Must scan parent.childCount and check name prefix "DINOForge_ModsButton" (case-insensitive).
        Regex.IsMatch(body, @"DINOForge_ModsButton", RegexOptions.IgnoreCase)
            .Should().BeTrue(because: "duplicate-prevention guard must look for 'DINOForge_ModsButton'-prefixed children (Cluster 2 / fixture 2)");
        Regex.IsMatch(body, @"StartsWith\(""DINOForge_ModsButton""\s*,\s*StringComparison\.OrdinalIgnoreCase\)")
            .Should().BeTrue(because: "name check must be case-insensitive ordinal (StringComparison.OrdinalIgnoreCase) — Pattern #99/#171 governance");
        // Must call EnforceModsButtonState on the existing button.
        Regex.IsMatch(body, @"EnforceModsButtonState\s*\(\s*existing\s*,\s*attemptId\s*\)")
            .Should().BeTrue(because: "re-enforcement helper must be called on the already-injected button (non-negotiable behavior — prevents text/visual drift)");
        // Must commit _injectedButton + _injected together AND exit (return; or return true;).
        // Post-refactor (Pattern #222) the helper TryReEnforceExistingInjection returns bool,
        // so `return true;` is acceptable alongside `return;` from the pre-refactor inline form.
        Regex.IsMatch(body, @"_injectedButton\s*=\s*existing\s*;[\s\S]{0,200}?_injected\s*=\s*true\s*;[\s\S]{0,300}?return\s*(?:true\s*)?;")
            .Should().BeTrue(because: "duplicate path must commit BOTH state fields atomically before early-return (behavior #7 atomicity)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #4: Text enforcement order ("Mods" set AFTER cloning)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #4 (non-negotiable): the cloned button inherits source text ("Options"),
    /// so "Mods" MUST be set AFTER NativeUiHelper.CloneButton returns. Setting it before
    /// would either no-op (clone overwrites) or, worse, miss TMPro children added by the clone.
    /// </summary>
    [Fact]
    public void InjectButton_EnforcesModsTextAfterCloning()
    {
        var body = GetInjectButtonBody();
        var cloneIdx = body.IndexOf("NativeUiHelper.CloneButton(cloneSource, \"Mods\")", StringComparison.Ordinal);
        cloneIdx.Should().BeGreaterThan(0, because: "CloneButton must be invoked at the known site (Cluster 3)");

        // Find the first legacy Text "Mods" assignment AFTER the clone call.
        var legacyTextMatch = Regex.Match(body.Substring(cloneIdx), @"legacyText\.text\s*=\s*""Mods""");
        legacyTextMatch.Success.Should().BeTrue(because: "behavior #4: 'Mods' MUST be set on legacy Text components AFTER cloning");

        // TMPro reflective set must also occur AFTER clone.
        var tmpMatch = Regex.Match(body.Substring(cloneIdx), @"SetValue\(\s*c\s*,\s*""Mods""\s*\)");
        tmpMatch.Success.Should().BeTrue(because: "behavior #4: TMPro text must also be set to 'Mods' via reflection AFTER cloning (avoids hard TMPro compile dependency)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #5: Position precedence (AFTER when 2+, BEFORE otherwise)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #5 (non-negotiable): position precedence MUST be:
    ///   - When positionAfterSibling != null (i.e. 2+ Options): PositionAfterSibling(mods, lastOptions)
    ///   - Otherwise (1 or 0 Options): PositionBeforeSibling(mods, settings)
    /// Inverting these breaks visual menu ordering and breaks the visible "Mods is below Options" UX contract.
    /// </summary>
    [Fact]
    public void InjectButton_PositioningPrecedenceFollowsSiblingCount()
    {
        var body = GetInjectButtonBody();
        // Must call PositionAfterSibling under positionAfterSibling != null branch.
        Regex.IsMatch(body, @"if\s*\(\s*positionAfterSibling\s*!=\s*null\s*\)[\s\S]{0,1000}?NativeUiHelper\.PositionAfterSibling\(\s*modsRect")
            .Should().BeTrue(because: "behavior #5: 2+ Options branch must call PositionAfterSibling(modsRect, lastOptionsRect)");
        // The else (or fallback) branch must call PositionBeforeSibling against settingsRect.
        Regex.IsMatch(body, @"NativeUiHelper\.PositionBeforeSibling\(\s*modsRect\s*,\s*settingsRect\s*\)")
            .Should().BeTrue(because: "behavior #5: 0/1-Options branch must call PositionBeforeSibling(modsRect, settingsRect)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #8: Layout-rebuild scope (parent + grandparent + ForceUpdateCanvases)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #8 (non-negotiable): layout rebuild MUST hit parent RectTransform,
    /// grandparent RectTransform, AND call Canvas.ForceUpdateCanvases. Skipping any
    /// of these leaves VerticalLayoutGroup/ContentSizeFitter unaware of the new
    /// child and the Mods button renders at the wrong position or off-screen.
    /// </summary>
    [Fact]
    public void InjectButton_RebuildsLayoutAtParentAndGrandparentAndForcesCanvasUpdate()
    {
        var body = GetInjectButtonBody();
        // Must call LayoutRebuilder.ForceRebuildLayoutImmediate twice (parent + grandparent).
        var rebuildHits = Regex.Matches(body, @"LayoutRebuilder\.ForceRebuildLayoutImmediate").Count;
        rebuildHits.Should().BeGreaterThanOrEqualTo(2,
            because: "behavior #8: must rebuild BOTH parent and grandparent RectTransforms (single rebuild missed VerticalLayoutGroup updates pre-#206)");
        // Must call Canvas.ForceUpdateCanvases() exactly once in the same path.
        Regex.IsMatch(body, @"Canvas\.ForceUpdateCanvases\(\s*\)")
            .Should().BeTrue(because: "behavior #8: Canvas.ForceUpdateCanvases() is required after RectTransform rebuilds");
    }

    // ------------------------------------------------------------------ //
    // Behavior #5 + Fixture 5: Navigation isolation (Mode.None, no force-select)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #5 (refactor-map non-negotiable): EventSystem.currentSelectedGameObject
    /// MUST NOT be force-selected onto the new Mods button — doing so couples it into
    /// native submit/navigation flows and triggers non-DINOForge handlers (e.g. Settings
    /// page open). The injected button's Navigation.mode MUST be set to None.
    /// </summary>
    [Fact]
    public void InjectButton_NavigationIsIsolated_NoForceSelect()
    {
        var body = GetInjectButtonBody();
        // Mods button navigation must be set to Mode.None (isolated).
        Regex.IsMatch(body, @"modsNav\.mode\s*=\s*Navigation\.Mode\.None")
            .Should().BeTrue(because: "behavior #5: Mods button must have Navigation.Mode.None to prevent native menu coupling");
        Regex.IsMatch(body, @"modsButton\.navigation\s*=\s*modsNav")
            .Should().BeTrue(because: "behavior #5: navigation struct must be written back to modsButton.navigation (Unity Navigation is a struct, not a ref)");

        // SetSelectedGameObject must NOT be called inside the InjectButton body
        // (force-selecting native UI is the explicit anti-pattern documented at the [7.4] log).
        body.Should().NotContain("SetSelectedGameObject(",
            because: "behavior #5: SetSelectedGameObject is explicitly forbidden (couples mods button to native submit/nav flows)");
    }

    // ------------------------------------------------------------------ //
    // Fixture 6: EventSystem-exception isolation (try/catch wraps step 7)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Fixture 6 from refactor map: the EventSystem-configuration step (STEP 7) MUST
    /// be wrapped in try/catch so its failure does not abort the entire injection.
    /// This is Pattern #104 governed: the catch must LogWarning (not silently swallow),
    /// must include the exception TYPE and Message and StackTrace.
    /// </summary>
    [Fact]
    public void InjectButton_EventSystemStepIsExceptionIsolated()
    {
        var body = GetInjectButtonBody();
        // Find the STEP 7 region; it must contain a try { ... } catch (Exception ...).
        var step7Idx = body.IndexOf("STEP 7", StringComparison.Ordinal);
        step7Idx.Should().BeGreaterThan(0, because: "STEP 7 EventSystem-config block must exist");

        // After STEP 7 marker, there must be a try{ block before STEP 8.
        var step8Idx = body.IndexOf("STEP 8", step7Idx, StringComparison.Ordinal);
        step8Idx.Should().BeGreaterThan(step7Idx, because: "STEP 7 must precede STEP 8");
        var region = body.Substring(step7Idx, step8Idx - step7Idx);
        Regex.IsMatch(region, @"\btry\s*\{")
            .Should().BeTrue(because: "fixture 6: STEP 7 must be wrapped in try{ ... }");
        Regex.IsMatch(region, @"catch\s*\(\s*Exception\s+\w+\s*\)[\s\S]{0,800}?LogWarning")
            .Should().BeTrue(because: "fixture 6: STEP 7 catch must LogWarning (Pattern #104 — no silent swallow)");
    }

    // ------------------------------------------------------------------ //
    // Behavior #7: State atomicity (_injected + _injectedButton committed together at end)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #7 (non-negotiable): _injected and _injectedButton MUST be assigned
    /// together at the END of InjectButton (after all diagnostic steps succeed). The
    /// assignment must NOT be split such that _injected goes true while _injectedButton
    /// is still null — Update() observes that as "injected but button dead" and triggers
    /// a re-scan loop, double-injecting the button.
    /// </summary>
    [Fact]
    public void InjectButton_CommitsInjectionStateAtomicallyAtEnd()
    {
        var body = GetInjectButtonBody();
        // Final commit: _injectedButton = modsButton; _injected = true; — must appear in this order
        // and immediately precede the success-log + outer catch.
        var match = Regex.Match(body,
            @"_injectedButton\s*=\s*modsButton\s*;[\s\S]{0,200}?_injected\s*=\s*true\s*;");
        match.Success.Should().BeTrue(because: "behavior #7: final commit must set _injectedButton THEN _injected = true (Cluster 7)");

        // The commit must be AFTER STEP 8 (final state verification) — locate STEP 8 and assert commit follows it.
        var step8Idx = body.IndexOf("STEP 8", StringComparison.Ordinal);
        step8Idx.Should().BeGreaterThan(0);
        match.Index.Should().BeGreaterThan(step8Idx, because: "behavior #7: state commit must follow STEP 8 verification, not precede it");
    }

    // ------------------------------------------------------------------ //
    // Behavior #3 + #6: Graceful-failure outer try/catch (never throws to caller)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Behavior #3 (non-negotiable): InjectButton MUST be wrapped in an outer
    /// try { ... } catch (Exception ex) { LogWarning(...); } that catches ANY
    /// exception including those bubbling from Unity calls (NRE, MissingMethodException,
    /// TypeInitializationException). The method MUST NOT propagate exceptions to the
    /// caller (Update() loop) — graceful failure pattern.
    /// </summary>
    [Fact]
    public void InjectButton_HasOuterTryCatchForGracefulFailure()
    {
        var body = GetInjectButtonBody();
        // The body MUST start with try{ (after the opening brace + whitespace).
        Regex.IsMatch(body, @"\A\{\s*try\s*\{")
            .Should().BeTrue(because: "behavior #3: entire method body must be wrapped in try{ ... } as first statement");
        // There must be a final catch(Exception ex) that LogWarnings the exception.
        // The catch must reference the exception with full detail (Pattern #74 / #111:
        // no silent swallow, no Message-only). Acceptable forms:
        //   - $"...{ex}"  (interpolation, expands to ex.ToString() → message + stack)
        //   - ex.ToString()  (explicit ToString — same result)
        //   - ex.Message followed somewhere by ex.StackTrace  (legacy explicit form)
        // We split this into two checks for clarity:
        //   1) the LogWarning call mentions "InjectButton EXCEPTION"
        //   2) the LogWarning arg references the exception via one of the forms above
        var hasInjectButtonExceptionLog = Regex.IsMatch(body,
            @"catch\s*\(\s*Exception\s+ex\s*\)[\s\S]{0,1200}?LogWarning\([\s\S]{0,800}?InjectButton EXCEPTION");
        hasInjectButtonExceptionLog.Should().BeTrue(
            because: "behavior #3: outer catch must LogWarning with 'InjectButton EXCEPTION' marker text");

        var hasFullExceptionDetail = Regex.IsMatch(body, @"\{ex\}")
            || Regex.IsMatch(body, @"ex\.ToString\(\)")
            || (Regex.IsMatch(body, @"ex\.Message") && Regex.IsMatch(body, @"ex\.StackTrace"));
        hasFullExceptionDetail.Should().BeTrue(
            because: "behavior #3: outer catch must include full exception detail — {ex} interpolation (preferred), ex.ToString(), or ex.Message + ex.StackTrace");
    }

    // ------------------------------------------------------------------ //
    // Behavior: Repurposed-name registration for Harmony text patch
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Cross-cluster behavior: after cloning, the new Mods GameObject's name MUST
    /// be assigned to the static <c>RepurposedModsButtonGoName</c> property so the
    /// <c>ModsButtonTextPatch</c> Harmony patch can intercept UiGrid text overwrite.
    /// The refactor must preserve this side-effect ordering (assignment occurs in
    /// Cluster 3, BEFORE text enforcement).
    /// </summary>
    [Fact]
    public void InjectButton_RegistersClonedButtonNameForHarmonyTextPatch()
    {
        var body = GetInjectButtonBody();
        Regex.IsMatch(body, @"RepurposedModsButtonGoName\s*=\s*modsButton\.gameObject\.name")
            .Should().BeTrue(because: "Cluster 3: clone's GO name must be registered for Harmony text-intercept patch");

        // Registration MUST occur AFTER CloneButton (it depends on modsButton existing).
        // Scope includes InjectButtonFromSelectable (also registers name); pin the Button-path
        // assignment paired with NativeUiHelper.CloneButton, not the first null-clear or Selectable path.
        var cloneIdx = body.IndexOf("NativeUiHelper.CloneButton(cloneSource, \"Mods\")", StringComparison.Ordinal);
        cloneIdx.Should().BeGreaterThan(0, because: "CloneButton must be invoked in CloneAndRegisterModsButton (Cluster 3)");
        const string regAssignment = "RepurposedModsButtonGoName = modsButton.gameObject.name";
        var regIdx = body.IndexOf(regAssignment, cloneIdx, StringComparison.Ordinal);
        regIdx.Should().BeGreaterThan(cloneIdx, because: "name registration must happen after clone returns (depends on modsButton)");
    }

    // ------------------------------------------------------------------ //
    // Behavior: CloneButton-null path returns gracefully (no NRE downstream)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Cluster 3 sub-invariant: if <c>NativeUiHelper.CloneButton</c> returns null
    /// (Unity Instantiate failure mode), the method MUST LogWarning and early-return
    /// without touching any state. Pinned because removing this guard would NRE on
    /// the next <c>modsButton.gameObject.name</c> assignment, leaving _injected
    /// permanently false and the menu unusable.
    /// </summary>
    [Fact]
    public void InjectButton_GuardsAgainstCloneButtonReturningNull()
    {
        var body = GetInjectButtonBody();
        // After the CloneButton call, there must be an `if (modsButton == null)` guard
        // that LogWarnings "STEP 1 FAILED" or similar and returns. Use regex so both the
        // inline pre-refactor form (`modsButton = NativeUiHelper.CloneButton(...)`) and the
        // post-refactor declaration form (`Button? modsButton = NativeUiHelper.CloneButton(...)`)
        // are recognized.
        var cloneMatch = Regex.Match(body, @"(?:Button\?\s+)?modsButton\s*=\s*NativeUiHelper\.CloneButton\(\s*cloneSource\s*,\s*""Mods""\s*\)");
        cloneMatch.Success.Should().BeTrue(because: "CloneButton invocation must exist (Cluster 3)");
        var afterClone = body.Substring(cloneMatch.Index);
        // Post-refactor, the helper may early-return `null` instead of `;` (since it returns Button?).
        Regex.IsMatch(afterClone, @"if\s*\(\s*modsButton\s*==\s*null\s*\)[\s\S]{0,400}?LogWarning[\s\S]{0,400}?return\s*(?:null\s*)?;")
            .Should().BeTrue(because: "Cluster 3 sub-invariant: CloneButton-null must LogWarning + return BEFORE touching modsButton.gameObject");
    }

    // ------------------------------------------------------------------ //
    // Sanity: method size guard (refactor will SHRINK the method, not GROW it)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// Method-size sanity. Pre-refactor: <c>InjectButton</c> body alone was ~298 lines.
    /// Post-refactor (Pattern #222 decomp, iter-143): the scope returned by
    /// <see cref="GetInjectButtonBody"/> widens to include the 6 extracted private helpers
    /// (ResolveCloneSource → CommitInjectionAndLog), so the combined line count is larger
    /// (~422 expected) — but <c>InjectButton</c> itself shrinks dramatically (from ~298 to
    /// ~50 lines). Envelope: 30..600 covers both forms; outside this range signals either
    /// silent regrowth (helpers re-inlined) or an unexpected explosion.
    /// </summary>
    [Fact]
    public void InjectButton_BodySizeIsWithinExpectedEnvelope()
    {
        var body = GetInjectButtonBody();
        var lineCount = body.Count(c => c == '\n');
        // Pre-refactor: ~302 lines (InjectButton inlined). Post-refactor: ~420 lines
        // (InjectButton + 6 helpers combined). Both must fall in 30..600.
        lineCount.Should().BeInRange(30, 600,
            because: "body size must remain within the refactor envelope (pre: ~302 inlined, post: ~420 with helpers per map § Post-Refactor Method Size Estimate). "
                  + $"Actual line count: {lineCount}. If this fails post-refactor with a small count, helpers may have been deleted; "
                  + "if it fails post-refactor with a large count, audit for accidental additions that should be in a separate method.");
    }

    // ------------------------------------------------------------------ //
    // SPEC-002 unit: TryInjectMenuButton graceful failure (F-06, N-02)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// SPEC-002: <c>TryInjectMenuButton_NoCanvases_DoesNotThrow</c> /
    /// <c>TryInjectMenuButton_ActiveCanvasNoSettingsButton_DoesNotThrow</c> — outer try/catch
    /// must swallow exceptions and log a warning instead of propagating.
    /// </summary>
    [Fact]
    public void TryInjectMenuButton_HasOuterTryCatchForGracefulFailure()
    {
        var body = GetMethodBodyBySignature("internal void TryInjectMenuButton()");
        body.Should().Contain("try", because: "TryInjectMenuButton must wrap scan logic in try/catch");
        Regex.IsMatch(body, @"catch\s*\(\s*Exception\s+ex\s*\)[\s\S]{0,800}?TryInjectMenuButton EXCEPTION")
            .Should().BeTrue(because: "SPEC-002 F-06/N-02: failures must LogWarning with TryInjectMenuButton EXCEPTION marker");
    }

    /// <summary>
    /// SPEC-002 F-06: when no Settings/Options button is found, log diagnostic and schedule retry.
    /// </summary>
    [Fact]
    public void TryInjectMenuButton_LogsDiagnosticWhenNoSettingsButtonFound()
    {
        var body = GetMethodBodyBySignature("internal void TryInjectMenuButton()");
        Regex.IsMatch(body, @"0 Settings buttons found")
            .Should().BeTrue(because: "SPEC-002 F-06: must log when scan finds no injection target");
        Regex.IsMatch(body, @"Will retry in \{RescanInterval\}s")
            .Should().BeTrue(because: "SPEC-002 F-06: must indicate automatic retry interval");
    }

    /// <summary>
    /// SPEC-002 architecture: scan uses <c>Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()</c>
    /// and skips inactive canvases via <c>IsCanvasActive</c>.
    /// </summary>
    [Fact]
    public void TryInjectMenuButton_ScansActiveCanvasesViaFindObjectsOfTypeAll()
    {
        var body = GetMethodBodyBySignature("internal void TryInjectMenuButton()");
        body.Should().Contain("Resources.FindObjectsOfTypeAll<Canvas>()",
            because: "SPEC-002: canvas discovery uses FindObjectsOfTypeAll<Canvas>");
        Regex.IsMatch(body, @"IsCanvasActive\(\s*canvas\s*\)")
            .Should().BeTrue(because: "SPEC-002: inactive canvases must be skipped");
    }

    /// <summary>
    /// SPEC-002: Settings primary, Options fallback in <c>FindSettingsButton</c>.
    /// </summary>
    [Fact]
    public void FindSettingsButton_SearchesSettingsThenOptions()
    {
        var body = GetMethodBodyBySignature("private Button? FindSettingsButton(Canvas canvas)");
        body.Should().Contain("NativeUiHelper.FindButtonByText(canvas.transform, \"Settings\")",
            because: "SPEC-002: primary anchor is Settings label");
        Regex.IsMatch(body, @"IndexOf\(\s*""Options""\s*,\s*StringComparison\.OrdinalIgnoreCase\s*\)")
            .Should().BeTrue(because: "SPEC-002: Options label is the fallback anchor");
    }

    // ------------------------------------------------------------------ //
    // SPEC-002 unit: RewireModsButtonClick, SyncButtonVisualStyle
    // ------------------------------------------------------------------ //

    /// <summary>
    /// SPEC-002: <c>RewireModsButtonClick_ClearsInheritedListeners</c>.
    /// </summary>
    [Fact]
    public void RewireModsButtonClick_RemovesAllListenersBeforeAddingToggle()
    {
        var body = GetMethodBodyBySignature("private void RewireModsButtonClick(Button modsButton, long attemptId)");
        var removeIdx = body.IndexOf("onClick.RemoveAllListeners()", StringComparison.Ordinal);
        var addIdx = body.IndexOf("onClick.AddListener(OnModsButtonClicked)", StringComparison.Ordinal);
        removeIdx.Should().BeGreaterThan(0);
        addIdx.Should().BeGreaterThan(removeIdx,
            because: "SPEC-002: inherited clone listeners must be cleared before wiring Mods toggle");
    }

    /// <summary>
    /// SPEC-002 F-03 / unit: <c>SyncButtonVisualStyle_CopiesColorBlock</c>.
    /// </summary>
    [Fact]
    public void SyncButtonVisualStyle_CopiesColorBlockAndTransition()
    {
        var body = GetMethodBodyBySignature("private void SyncButtonVisualStyle(Button target, Button source, long attemptId)");
        body.Should().Contain("target.transition = source.transition");
        body.Should().Contain("target.colors = source.colors");
        body.Should().Contain("target.spriteState = source.spriteState");
    }

    // ------------------------------------------------------------------ //
    // SPEC-002 unit: OnModsButtonClicked debounce + null host (N-05, F-02)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// SPEC-002 N-05: <c>OnModsButtonClicked_Debounce_SecondClickIgnored</c> — 200 ms window.
    /// </summary>
    [Fact]
    public void OnModsButtonClicked_Uses200MsDebounceWindow()
    {
        var src = SourceText.Value;
        src.Should().Contain("private const float ClickDebounceSeconds = 0.2f",
            because: "SPEC-002 N-05: debounce constant must be 200 ms");

        var body = GetMethodBodyBySignature("private void OnModsButtonClicked()");
        Regex.IsMatch(body, @"Time\.unscaledTime[\s\S]{0,200}?ClickDebounceSeconds")
            .Should().BeTrue(because: "SPEC-002 N-05: debounce must compare unscaled time against ClickDebounceSeconds");
        Regex.IsMatch(body, @"_menuHost\.Toggle\(\)")
            .Should().BeTrue(because: "SPEC-002 F-02: successful click must toggle mod menu host");
    }

    /// <summary>
    /// SPEC-002: <c>OnModsButtonClicked_NullMenuHost_DoesNotThrow</c>.
    /// </summary>
    [Fact]
    public void OnModsButtonClicked_NullMenuHostLogsWarningWithoutThrowing()
    {
        var body = GetMethodBodyBySignature("private void OnModsButtonClicked()");
        Regex.IsMatch(body, @"catch\s*\(\s*Exception\s+ex\s*\)[\s\S]{0,400}?OnModsButtonClicked exception")
            .Should().BeTrue(because: "OnModsButtonClicked must not throw to Unity event system");
        Regex.IsMatch(body, @"if\s*\(\s*_menuHost\s*==\s*null\s*\)[\s\S]{0,300}?LogWarning[\s\S]{0,300}?return\s*;")
            .Should().BeTrue(because: "SPEC-002: null menu host must LogWarning and return");
    }

    // ------------------------------------------------------------------ //
    // SPEC-002 integration (source-text): scene change, Update destroy, OnScanNeeded
    // ------------------------------------------------------------------ //

    /// <summary>
    /// SPEC-002: <c>SceneChange_ResetsInjectionState</c> (F-05).
    /// </summary>
    [Fact]
    public void OnActiveSceneChanged_ResetsInjectionStateAndRescans()
    {
        var body = GetMethodBodyBySignature("private void OnActiveSceneChanged(Scene previous, Scene next)");
        Regex.IsMatch(body, @"_injected\s*=\s*false\s*;[\s\S]{0,200}?_injectedButton\s*=\s*null")
            .Should().BeTrue(because: "SPEC-002 F-05: scene change must reset injection state");
        body.Should().Contain("TryInjectMenuButton()",
            because: "SPEC-002 F-05: scene change must trigger immediate re-scan");
    }

    /// <summary>
    /// SPEC-002: <c>Update_ButtonDestroyed_ResetsAndRescans</c>.
    /// </summary>
    [Fact]
    public void Update_DestroyedInjectedButtonResetsInjectedFlag()
    {
        var body = GetMethodBodyBySignature("private void Update()");
        Regex.IsMatch(body, @"if\s*\(\s*_injected\s*&&\s*_injectedButton\s*==\s*null\s*\)[\s\S]{0,300}?_injected\s*=\s*false")
            .Should().BeTrue(because: "SPEC-002: destroyed injected button must reset _injected for re-scan");
        Regex.IsMatch(body, @"_rescanTimer\s*<\s*RescanInterval")
            .Should().BeTrue(because: "SPEC-002: periodic re-scan uses RescanInterval gate");
    }

    /// <summary>
    /// SPEC-002 F-07: static <c>OnScanNeeded</c> delegate for external re-scan triggers.
    /// </summary>
    [Fact]
    public void OnScanNeeded_IsExposedAsNullableStaticAction()
    {
        var src = SourceText.Value;
        Regex.IsMatch(src, @"public\s+static\s+System\.Action\?\s+OnScanNeeded\s*;")
            .Should().BeTrue(because: "SPEC-002 F-07: OnScanNeeded must be a public static nullable Action");
    }

    /// <summary>
    /// SPEC-002 F-07 / <c>OnScanNeeded_TriggersInjection</c>: <c>RuntimeDriver</c> assigns the delegate
    /// after creating <c>NativeMenuInjector</c> so main-thread callers can request <c>TryInjectMenuButton</c>.
    /// </summary>
    [Fact]
    public void RuntimeDriver_OnScanNeeded_AssignsDelegateToTryInjectMenuButton()
    {
        var pluginSrc = File.ReadAllText(LocateRuntimePluginSource(), System.Text.Encoding.UTF8);
        Regex.IsMatch(pluginSrc, @"NativeMenuInjector\.OnScanNeeded\s*=\s*\(\)\s*=>")
            .Should().BeTrue(because: "SPEC-002 F-07: RuntimeDriver must assign OnScanNeeded after injector creation");
        pluginSrc.Should().Contain("TryInjectMenuButton()",
            because: "SPEC-002 F-07: OnScanNeeded handler must invoke TryInjectMenuButton");
    }

    private static string LocateRuntimePluginSource()
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        for (int i = 0; i < 20 && dir != null; i++, dir = dir.Parent)
        {
            if (File.Exists(Path.Combine(dir.FullName, "global.json")))
            {
                var path = Path.Combine(dir.FullName, "src", "Runtime", "Plugin.cs");
                if (File.Exists(path))
                {
                    return path;
                }
            }
        }
        throw new InvalidOperationException(
            $"Plugin.cs not located from {AppContext.BaseDirectory}; "
            + "RuntimeDriver OnScanNeeded characterization test cannot run without source access.");
    }

    /// <summary>
    /// SPEC-002 N-03: periodic re-scan interval is at least 1 second (implementation: 2 s).
    /// </summary>
    [Fact]
    public void RescanInterval_IsAtLeastOneSecond()
    {
        var src = SourceText.Value;
        var match = Regex.Match(src, @"private\s+const\s+float\s+RescanInterval\s*=\s*(\d+(?:\.\d+)?)f\s*;");
        match.Success.Should().BeTrue(because: "RescanInterval constant must exist");
        float interval = float.Parse(match.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture);
        interval.Should().BeGreaterThanOrEqualTo(1f, because: "SPEC-002 N-03: re-scan interval must be >= 1 second");
    }

    // ------------------------------------------------------------------ //
    // SPEC-002 manual AC #7: pause menu injection (same scan path as main menu)
    // ------------------------------------------------------------------ //

    /// <summary>
    /// SPEC-002 manual AC #7: pause menu is an explicit injection target alongside main menu.
    /// </summary>
    [Fact]
    public void NativeMenuInjector_DocumentsPauseMenuSupport()
    {
        SourceText.Value.Should().Contain("pause menu",
            because: "SPEC-002 manual AC #7: pause menu must be documented as a native injection surface");
    }

    /// <summary>
    /// SPEC-002 manual AC #7: <c>PauseMenu</c> is listed for diagnostics when DINO names the pause canvas.
    /// </summary>
    [Fact]
    public void CanvasCandidateNames_IncludesPauseMenu()
    {
        Regex.IsMatch(SourceText.Value, @"""PauseMenu""")
            .Should().BeTrue(because: "SPEC-002 manual AC #7: PauseMenu must appear in CanvasCandidateNames");
    }

    /// <summary>
    /// SPEC-002 manual AC #7: injection scans every active canvas (not MainMenu-only) so pause menus qualify.
    /// </summary>
    [Fact]
    public void TryInjectMenuButton_ScansAllActiveCanvasesForSettingsAnchor()
    {
        var body = GetMethodBodyBySignature("internal void TryInjectMenuButton()");
        body.Should().Contain("foreach (Canvas canvas in allCanvases)",
            because: "SPEC-002 manual AC #7: pause menu uses the same Settings/Options anchor scan as main menu");
        body.Should().Contain("FindSettingsButton(canvas)",
            because: "SPEC-002 manual AC #7: each active canvas (main or pause) is probed for Settings/Options");
        body.Should().Contain("Search all active canvases regardless of name",
            because: "SPEC-002 manual AC #7: pause canvas scan is name-agnostic; anchor is button text only");
        body.Should().NotContain("IsCanvasNameMatch(",
            because: "SPEC-002 manual AC #7: injection must not be gated to a fixed canvas name list");
    }

    /// <summary>
    /// SPEC-002 lifecycle: Awake subscribes to scene changes; OnDestroy unsubscribes (DF0105 pair).
    /// </summary>
    [Fact]
    public void Lifecycle_SubscribesAndUnsubscribesSceneChanged()
    {
        var src = SourceText.Value;
        var awakeBody = GetMethodBodyBySignature("private void Awake()");
        awakeBody.Should().Contain("SceneManager.activeSceneChanged += OnActiveSceneChanged");

        var destroyBody = GetMethodBodyBySignature("private void OnDestroy()");
        destroyBody.Should().Contain("SceneManager.activeSceneChanged -= OnActiveSceneChanged");
    }
}
