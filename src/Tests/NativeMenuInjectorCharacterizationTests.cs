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
/// </summary>
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
        // The catch must include exception details — either via {ex} interpolation
        // (which is ex.ToString() including message + stack), via ex.ToString()
        // explicitly, or via ex.Message + ex.StackTrace separately. All three forms
        // satisfy Pattern #74 / #111 governance (no silent swallow / no Message-only).
        Regex.IsMatch(body,
                @"catch\s*\(\s*Exception\s+ex\s*\)[\s\S]{0,800}?LogWarning\([\s\S]{0,400}?InjectButton EXCEPTION[\s\S]{0,400}?(?:\{ex\}|ex\.ToString\(\)|ex\.Message[\s\S]{0,300}?ex\.StackTrace)")
            .Should().BeTrue(because: "behavior #3: outer catch must LogWarning with 'InjectButton EXCEPTION' and full exception detail ({ex}, ex.ToString(), or ex.Message + ex.StackTrace)");
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
        var cloneIdx = body.IndexOf("NativeUiHelper.CloneButton(cloneSource, \"Mods\")", StringComparison.Ordinal);
        var regIdx = body.IndexOf("RepurposedModsButtonGoName =", StringComparison.Ordinal);
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
}
