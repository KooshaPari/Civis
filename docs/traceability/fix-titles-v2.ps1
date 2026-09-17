# fix-titles-v2.ps1 — Simpler title extraction
$base = "C:\Users\koosh\Civis-clone\docs\traceability"
$dirs = Get-ChildItem -Path $base -Directory -Filter "fr-*" | Sort-Object Name
$total = $dirs.Count
$fixed = 0
$count = 0

foreach ($d in $dirs) {
    $count++
    if ($count % 200 -eq 0) { Write-Output "Progress: $count / $total" }

    $dirName = $d.Name
    $specFile = (Get-ChildItem -Path $d.FullName -File -Name -ErrorAction SilentlyContinue | Where-Object { $_ -match '-spec\.md$' } | Select-Object -First 1)
    if (-not $specFile) { continue }

    $specContent = Get-Content -Path (Join-Path $d.FullName $specFile) -Raw -ErrorAction SilentlyContinue

    $frId = ""
    $frTitle = ""
    foreach ($line in ($specContent -split "`n")) {
        if ($line -match '^#\s+(FR-[\w-]+)\s') {
            $frId = $Matches[1]
            # Use simple replace to strip prefix
            $rest = $line.Replace("# $frId ", "").Replace("# $frId  ", "")
            # Strip any non-ASCII prefix chars (mojibake em-dash etc)
            $rest = $rest.TrimStart()
            while ($rest.Length -gt 0 -and [int][char]$rest[0] -gt 127) {
                $rest = $rest.Substring(1)
            }
            $rest = $rest.TrimStart('-').TrimStart(' ').TrimStart('-').TrimStart(' ')
            $frTitle = $rest.Trim()
            break
        }
    }

    if (-not $frId) { $frId = $dirName.ToUpper() }
    if (-not $frTitle) { $frTitle = ($dirName -replace '^fr-', '' -replace '-', ' ') }

    $epic = ""
    if ($specContent -match 'Epic:\s+(FR-[\w-]+)') { $epic = $Matches[1] }

    # Crate mapping
    $crateGuess = "engine"
    $pairs = @(
        @('fr-civ-species','species'), @('fr-civ-tactics','tactics'), @('fr-civ-war','tactics'),
        @('fr-civ-asset','asset-pipeline'), @('fr-civ-ai','ai'), @('fr-civ-voxel','voxel'),
        @('fr-civ-geo','planet'), @('fr-civ-planet','planet'), @('fr-civ-diffusion','diffusion'),
        @('fr-civ-diplo','diplomacy'), @('fr-civ-econ','economy'), @('fr-civ-emerg','emergence-oracle'),
        @('fr-civ-emergence','emergence-oracle'), @('fr-civ-genetics','genetics'),
        @('fr-civ-climate','climate'), @('fr-civ-hud','hud'), @('fr-civ-lang','i18n'),
        @('fr-civ-infra','infra'), @('fr-civ-laws','laws'), @('fr-civ-legends','legends'),
        @('fr-civ-life','species'), @('fr-civ-mod','mod-host'), @('fr-civ-psyche','needs'),
        @('fr-civ-polity','diplomacy'), @('fr-civ-proto3d','protocol-3d'), @('fr-civ-proto','protocol-3d'),
        @('fr-civ-research','research'), @('fr-civ-save','save-db'), @('fr-civ-server','server'),
        @('fr-civ-social','civ-institutions'), @('fr-civ-build','physics-substrate'),
        @('fr-civ-vehicle','civ-traffic'), @('fr-civ-traffic','civ-traffic'), @('fr-civ-tech','research'),
        @('fr-civ-market','economy'), @('fr-civ-audio','audio'), @('fr-civ-mcp','civis-mcp'),
        @('fr-civ-llm','ai'), @('fr-civ-brush','voxel'), @('fr-civ-web','server'),
        @('fr-civ-scale','engine'), @('fr-civ-arch','engine'), @('fr-civ-render','engine'),
        @('fr-civ-perf','engine'), @('fr-civ-ux','hud'), @('fr-civ-ui','hud'),
        @('fr-civ-3d','protocol-3d'), @('fr-civ-godot','protocol-3d'), @('fr-civ-rts','protocol-3d'),
        @('fr-civ-verify','build'), @('fr-civ-pbr','engine'), @('fr-civ-lod','voxel'),
        @('fr-civ-det','engine'), @('fr-civ-fog','engine'), @('fr-civ-terrain','planet'),
        @('fr-civ-res','economy'), @('fr-civ-ca','ai'), @('fr-civ-bio','species'),
        @('fr-civ-cult','civ-institutions'), @('fr-civ-gov','civ-institutions'),
        @('fr-civ-qol','hud'), @('fr-civ-infoview','hud'), @('fr-civ-inspect','hud'),
        @('fr-civ-notify','hud'), @('fr-civ-road','civ-traffic'), @('fr-civ-metrics','observability'),
        @('fr-civ-engine-int','engine'), @('fr-civ-engine-replay','engine'),
        @('fr-civ-act-','engine'), @('fr-civ-actor','species'), @('fr-civ-agents','ai'),
        @('fr-civ-godtool','protocol-3d'), @('fr-civ-client','protocol-3d'),
        @('fr-ai-','ai'), @('fr-api-','server'), @('fr-asset-','asset-pipeline'),
        @('fr-aud-','build'), @('fr-client-','protocol-3d'), @('fr-clim-','climate'),
        @('fr-core-','engine'), @('fr-det-','engine'), @('fr-dipl-','diplomacy'),
        @('fr-doc-','build'), @('fr-eco-','economy'), @('fr-econ-','economy'),
        @('fr-guard-','engine'), @('fr-inst-','civ-institutions'), @('fr-int-','engine'),
        @('fr-met-','observability'), @('fr-metrics-','observability'), @('fr-mod-','mod-host'),
        @('fr-net-','server'), @('fr-perf-','engine'), @('fr-prot-','protocol-3d'),
        @('fr-proto-','protocol-3d'), @('fr-rep-','engine'), @('fr-replay-','engine'),
        @('fr-save-','save-db'), @('fr-sess-','server'), @('fr-session-','server'),
        @('fr-soc-','civ-institutions'), @('fr-soci-','civ-institutions'), @('fr-stor-','save-db'),
        @('fr-test-','build'), @('fr-thry-','engine'), @('fr-ux-','hud'), @('fr-val-','build')
    )
    foreach ($pair in $pairs) {
        if ($dirName -match [regex]::Escape($pair[0])) { $crateGuess = $pair[1]; break }
    }

    $crateRef = "``crates/$crateGuess/src/``"

    # Build refs
    $implCode = @(); $inImpl = $false
    foreach ($line in ($specContent -split "`n")) {
        if ($line -match '## Implementing Code') { $inImpl = $true; continue }
        if ($inImpl -and $line -match '^## ') { break }
        if ($inImpl -and $line -match '^\s*-\s+`(.+?)`') { $implCode += $Matches[1] }
    }
    $testCode = @(); $inTest = $false
    foreach ($line in ($specContent -split "`n")) {
        if ($line -match '## Test Coverage') { $inTest = $true; continue }
        if ($inTest -and $line -match '^## ') { break }
        if ($inTest -and $line -match '^\s*-\s+`(.+?)`') { $testCode += $Matches[1] }
    }
    $implRefs = if ($implCode.Count -gt 0) { ($implCode | ForEach-Object { "- ``$_``" }) -join "`n" } else { "> _To be implemented._" }
    $testRefs = if ($testCode.Count -gt 0) { ($testCode | ForEach-Object { "- ``$_``" }) -join "`n" } else { "> _No test coverage yet._" }

    # ADR
    $adrContent = "# ADR: $frId -- $frTitle`n`n> Status: Proposed`n> Date: 2026-09-17`n> Deciders: CivLab`n> Relates to: $frId`n> Epic: $epic`n`n## Context`n`n$frId is part of the $epic epic. This functional requirement captures: $frTitle.`n`nImplementing crate: $crateRef`n`n### Referenced Source`n$implRefs`n`n### Test Coverage`n$testRefs`n`n## Decision`n`nTBD -- The architectural decision for $frId needs to be finalized based on implementation exploration.`n`n### Key Considerations`n- Integration with the Bevy ECS engine architecture`n- Consistency with existing patterns in ``crates/$crateGuess/```n- Performance implications for tick-based simulation`n`n## Consequences`n`n### Positive`n- Fulfills the $frTitle requirement in the simulation`n`n### Negative`n- Adds complexity to the $crateGuess crate`n`n### Risks`n- Implementation may surface unforeseen coupling with other FRs`n`n## Alternatives Considered`n`n1. **Option A**: Direct implementation in ``crates/$crateGuess/```n2. **Option B**: Extract into a dedicated sub-crate`n"
    [System.IO.File]::WriteAllText((Join-Path $d.FullName "$dirName-adr.md"), $adrContent, [System.Text.UTF8Encoding]::new($false))

    # Research
    $resContent = "# Research: $frId -- $frTitle`n`n> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)`n> FR: $frId`n> Epic: $epic`n`n## Research Question`n`nWhat is the best approach to implement $frTitle within the Civis simulation engine?`n`n## Background`n`nThis FR belongs to the $epic epic and is expected to be implemented in $crateRef.`n`n### Existing Code References`n$implRefs`n`n### Test References`n$testRefs`n`n## Findings`n`n### Codebase Analysis`n- The ``crates/$crateGuess/`` crate is the primary implementation target`n- Existing patterns in this crate should be followed for consistency`n`n### Feasibility`n- Implementation feasibility: high (patterns exist in the codebase)`n- Estimated complexity: medium`n`n## Recommendations`n`n1. Follow existing patterns in ``crates/$crateGuess/src/```n2. Add integration tests in ``crates/$crateGuess/tests/```n3. Update this research doc once implementation begins`n`n## References`n`n- ``docs/AGILE_WORKSTREAM.md```n- ``crates/$crateGuess/`` crate documentation`n"
    [System.IO.File]::WriteAllText((Join-Path $d.FullName "$dirName-research.md"), $resContent, [System.Text.UTF8Encoding]::new($false))

    # Plan
    $planContent = "# Plan: $frId -- $frTitle`n`n> Date: 2026-09-17`n> FR: $frId`n> Epic: $epic`n> Status: DRAFT`n`n## Implementation Steps`n`n1. **Research and Design** -- Review existing code in $crateRef and finalize the ADR`n2. **Core Implementation** -- Implement the $frTitle logic`n3. **Integration** -- Wire into the Bevy ECS tick system and existing systems`n4. **Testing** -- Add unit tests and integration tests`n5. **Documentation** -- Update spec, ADR, and this plan with final decisions`n`n### Referenced Source Files`n$implRefs`n`n### Test Coverage`n$testRefs`n`n## Dependencies`n`n- Epic: $epic`n- Implementing crate: $crateRef`n- Engine core: ``crates/engine/src/```n`n## Verification`n`n1. ``cargo check -p $crateGuess```n2. ``cargo test -p $crateGuess```n3. ``cargo clippy -p $crateGuess```n4. Manual verification in the simulation runtime`n`n## Estimated Effort`n`n- Implementation: TBD`n- Testing: TBD`n- Total: TBD`n"
    [System.IO.File]::WriteAllText((Join-Path $d.FullName "$dirName-plan.md"), $planContent, [System.Text.UTF8Encoding]::new($false))

    # Intent
    $intentContent = "# Intent: $frId -- $frTitle`n`n> Date: 2026-09-17`n> FR: $frId`n> Epic: $epic`n`n## User Intent`n`nThe product owner requires $frTitle as part of the $epic epic for the Civis civilisation simulation.`n`n### What This FR Achieves`nThis functional requirement ensures that $frTitle is properly specified, implemented, and testable within the simulation engine.`n`n### Product Context`nCivis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. $frId contributes to the overall simulation capability by addressing: $frTitle.`n`n## Acceptance Signal`n`n### Definition of Done`n- [ ] Implementation in ``crates/$crateGuess/`` compiles and passes all checks`n- [ ] Unit tests pass for the new functionality`n- [ ] Integration with the simulation tick system works correctly`n- [ ] No regressions in existing FRs`n- [ ] ADR is finalized and accepted`n- [ ] This intent document is updated with final decisions`n`n### How We Know This FR Is Satisfied`n1. ``cargo test -p $crateGuess`` passes`n2. The simulation runs without errors related to $frTitle`n3. The feature is observable in the simulation output`n`n## Traceability`n`n| Artifact | Path |`n|----------|------|`n| Spec | ``$specFile`` |`n| ADR | ``$dirName-adr.md`` |`n| Research | ``$dirName-research.md`` |`n| Plan | ``$dirName-plan.md`` |`n| Implementing crate | $crateRef |`n"
    [System.IO.File]::WriteAllText((Join-Path $d.FullName "$dirName-intent.md"), $intentContent, [System.Text.UTF8Encoding]::new($false))

    $fixed++
}

Write-Output "=== COMPLETE ==="
Write-Output "Total: $total directories, $fixed files regenerated"
