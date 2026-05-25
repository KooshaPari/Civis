# DINOForge v0.24.0 Roadmap

**Target Release**: 2026-05-15 | **Current Version**: 0.23.0 | **Status**: Ready for next development cycle

---

## Vision

v0.24.0 shifts focus from **finalization & infrastructure** (v0.23.0) to **ecosystem integration & scale** via PhenoCompose, NuGet publishing, and advanced game automation capabilities. By end of v0.24.0, DINOForge mods will be testable at 100+ concurrent instances with GPU-accelerated visual validation.

---

## Tier 1: Core Features (Weeks 1-2)

### Feature 1.1: PhenoCompose CLI Integration (Sprint 2.1)

**Goal**: Evaluate phenocompose as external CLI tool for parallel game testing

**Work Items**:
- [ ] **Spike**: Test phenocompose installation and `nanovms game test` workflow
  - Clone https://github.com/KooshaPari/phenocompose
  - Build locally: `cargo build --release`
  - Verify `nanovms` binary availability
  - Document system requirements (Go 1.23+, Firecracker, IOMMU/KVM support)

- [ ] **Integration**: Create `scripts/game/Invoke-PhenoCompose.ps1` wrapper
  - Detect phenocompose availability (binary path, version check)
  - Wrap game fleet launch with parameter mapping: `game_launch_fleet(count=4)` → `nanovms game test --count 4`
  - Implement snapshot baseline creation and restore
  - Add fallback to current CreateDesktop-based fleet if phenocompose unavailable

- [ ] **Documentation**: Update ASSET_PIPELINE_CLI.md
  - Add phenocompose workflow section (6-phase setup guide)
  - Document GPU passthrough requirements and fallback behavior
  - Add troubleshooting guide

- [ ] **Testing**: Create `src/Tests/Integration/PhenoComposeTests.cs`
  - Test availability detection
  - Mock phenocompose responses for CI runs
  - Validate parameter mapping

**Effort**: 1 sprint (5 days) | **Owner**: Haiku subagent

---

### Feature 1.2: Bridge Package Publishing (Sprint 2.2)

**Goal**: Publish Bridge.Protocol and Bridge.Client to NuGet.org

**Work Items**:
- [ ] **Verify NuGet infrastructure**:
  - ✅ Bridge.Protocol: IsPackable=true, metadata complete (0.22.0)
  - ✅ Bridge.Client: IsPackable=true, metadata complete (0.22.0)
  - Verify release.yml workflow publishes on tag
  - Test publishing pipeline locally (dry-run)

- [ ] **Prepare release**:
  - Bump Bridge.Protocol and Bridge.Client to 0.24.0
  - Add release notes to CHANGELOG.md
  - Create git tag: `v0.24.0-bridge` for pre-release testing

- [ ] **Publish**:
  - Push tag to trigger release.yml
  - Verify packages appear on nuget.org
  - Test installation: `dotnet add package DINOForge.Bridge.Protocol --version 0.24.0`

- [ ] **Documentation**:
  - Update README.md with NuGet package links
  - Add developer guide for consuming Bridge packages

**Effort**: 0.5 sprint (2-3 days) | **Owner**: Haiku subagent

---

## Tier 2: Optimization & Polish (Weeks 2-3)

### Feature 2.1: Multi-Tier Isolation Backend Selection

**Goal**: Smart backend selection (VDD > CreateDesktop > playCUA > mock) with visual indicators

**Work Items**:
- [ ] **Enhancement**: Update `isolation_layer.py` to detect VDD availability
  - Check for Virtual Display Driver via registry (Windows)
  - Populate tier 0 in fallback chain: VDD (0ms) → CreateDesktop (~2s) → playCUA (~100ms) → mock

- [ ] **MCP tool**: Add `game_backend_info()` tool
  - Returns available backends, latencies, capabilities
  - Shows why each tier is/isn't available
  - Helps users optimize setup

- [ ] **GUI enhancement** (DesktopCompanion):
  - Add "Backend Status" panel to main window
  - Visual indicators (✅✅✅ for all available, ⚠️ for degraded)
  - One-click backend override if multiple available

**Effort**: 0.5 sprint | **Owner**: Haiku subagent

---

### Feature 2.2: CLIP-Based Visual Regression Testing

**Goal**: Automated visual validation of game mods via zero-shot image classification

**Work Items**:
- [ ] **Integration**: Wire CLIP model into `game_analyze_screen`
  - Use `openai/clip-vit-base-patch32` (~420MB, download on first use)
  - Accept text prompts: `["overlay visible", "overlay hidden", "health bar red", "menu open"]`
  - Return confidence scores for each prompt
  - Fallback to pHash if CLIP unavailable (GPU/model not installed)

- [ ] **Testing**: Add visual regression test suite
  - Compare mod vs vanilla via CLIP prompts
  - Store golden baseline images in `docs/proof/golden/`
  - CI gates: CLIP confidence > 0.75 → pass

- [ ] **Documentation**: Update game automation docs
  - Document CLIP tier (Tier 2: medium, ~200ms per screenshot)
  - Show usage: `game_analyze_screen("mod-pack-name", prompts=["unit colors correct", "overlay present"])`

**Effort**: 1 sprint | **Owner**: Haiku subagent

---

### Feature 2.3: Performance Optimization Pass

**Goal**: Reduce test execution time by 20% through parallelization and caching

**Work Items**:
- [ ] **Asset loading cache**:
  - Cache compiled Addressables catalog in memory across test runs
  - Reduce DLL load time in GameBridgeServer
  - Measure impact on test suite execution time

- [ ] **Test parallelization**:
  - Enable test parallel execution in xUnit (by category)
  - Profile test dependencies to identify safe parallelization boundaries
  - Target: 50% reduction in unit test suite time

- [ ] **Build optimization**:
  - Cache polyglot artifacts in GitHub Actions
  - Reduce build time by 30% via incremental compilation

**Effort**: 0.5 sprint | **Owner**: Haiku subagent

---

## Tier 3: Documentation & Community (Weeks 3-4)

### Feature 3.1: Contributor Onboarding Guide

**Goal**: Enable new contributors to productively work on DINOForge

**Work Items**:
- [ ] Create `CONTRIBUTING.md`:
  - Development environment setup (VS Code, .NET 11, WSL2 for Windows)
  - Branching strategy and PR workflow
  - Test coverage requirements (95%+)
  - Code style guide (C# 12+, nullable reference types)
  - Agent rules (wrap, don't handroll; use registries)

- [ ] Create `docs/DEVELOPER_GUIDE.md`:
  - Architecture overview (layers, hexagonal pattern)
  - Adding a new pack (step-by-step with examples)
  - Adding a domain plugin (Warfare → Economy blueprint)
  - Testing game logic (MCP server + game automation)

- [ ] Create tutorial videos (Remotion compositions):
  - "Creating your first mod pack" (5 min)
  - "Running automated tests" (3 min)

**Effort**: 0.5 sprint | **Owner**: Haiku subagent

---

### Feature 3.2: Roadmap Transparency

**Goal**: Public visibility into DINOForge development priorities

**Work Items**:
- [ ] Publish ROADMAP.md to GitHub (this document + versions beyond v0.24.0)
- [ ] Create GitHub Project board linked to milestones
- [ ] Monthly status updates in discussions forum

**Effort**: 0.25 sprint | **Owner**: Local (documentation)

---

## Success Criteria

### Performance
- [ ] Test suite completes in <5 minutes (down from ~8 min in v0.23.0)
- [ ] PhenoCompose fleet launches 4 parallel game instances in <60s
- [ ] CLIP-based analysis completes in <500ms per screenshot

### Quality
- [ ] Coverage remains ≥95%
- [ ] All 20 CI workflows pass
- [ ] Bridge packages published to NuGet.org

### Adoption
- [ ] First external PR using Bridge.Protocol package
- [ ] Documentation improvements enable 1+ new contributors

---

## Known Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| PhenoCompose unavailable on CI | Test fleet can't run | Mock nanovms responses; fallback to CreateDesktop |
| CLIP model too large for CI | Pipeline timeout | Lazy-load model; skip if not available; gate behind feature flag |
| GPU binding conflicts (VFIO) | Host rendering breaks | Require separate GPU; document IOMMU setup; provide detection script |
| NuGet publishing fails | CI/CD broken | Dry-run before merge; have manual fallback |

---

## Version Roadmap: v0.24.0 → v0.27.0

| Version | Focus | Timeline |
|---------|-------|----------|
| **v0.24.0** | PhenoCompose + NuGet + CLIP | 2026-05-15 |
| **v0.25.0** | Phenocompose MCP server wrapper + GPU detection | 2026-06-15 |
| **v0.26.0** | VDD driver integration + Docker backend | 2026-07-15 |
| **v0.27.0** | Advanced observability (Grafana dashboards, Prometheus metrics) | 2026-08-15 |

---

## Implementation Order

```
Week 1:
  - PhenoCompose CLI spike (Tier 1.1)
  - Bridge package publishing (Tier 1.2)
  - Multi-tier backend selection (Tier 2.1)

Week 2:
  - CLIP visual regression (Tier 2.2)
  - Performance optimization (Tier 2.3)
  - Contributor guide (Tier 3.1)

Week 3:
  - Final testing & bug fixes
  - Documentation review
  - Roadmap transparency (Tier 3.2)

Week 4:
  - Release v0.24.0
  - Plan v0.25.0 (PhenoCompose MCP wrapper)
```

---

## Notes for Future Sessions

- **PhenoCompose learning curve**: Requires IOMMU/KVM knowledge; may need spikes to understand on developer machines
- **CLIP resource requirements**: Model is 420MB; consider CloudStorage caching for CI to avoid repeated downloads
- **NuGet publishing**: Requires NUGET_API_KEY secret in GitHub Actions (verify it's set)
- **Contributor onboarding**: Priority is CLAUDE.md + agent rules clarity; these alone will unlock external contributions

---

## Sign-Off

**Plan created**: 2026-04-20 | **By**: Claude Haiku (finalization sprint completion) | **Approved**: Pending v0.24.0 kick-off
