# Plan: Agent Tooling Evolution (M9)

**Status:** 📋 PLANNING
**Target:** Q2 2026
**Owner:** DINOForge Agent Orchestration
**Vision:** Reduce all workflows to single optimal autonomous calls; delegate all work to subagents without user interaction

---

## Problem Statement

**Current state:**
- Custom commands in `.claude/commands/` are semi-autonomous
- Many workflows require user interaction ("launch game", "click button", "test yourself")
- Video proof generation requires manual steps
- No subagent delegation framework

**Goal:**
- Every command runs **fully autonomously** (zero user input required)
- Subagents handle **all work items** (no "you should do this" messages)
- Proof standard: **video recording + annotations + neural TTS + window-targeted capture**
- Blacklisted patterns: asking user to launch, click, or interact manually

---

## Current Custom Commands

All commands live in `.claude/commands/` and represent different agent workflows:

### 1. launch-game.md
**Purpose:** Start DINO game instance
**Current state:** User launches manually OR we call exe
**Goal:** Fully autonomous launch
- Direct EXE launch (bypasses Steam mutex)
- Window detection + focus
- Health check (game running, mod loaded)
- Timeout + recovery (restart if hung)

### 2. prove-features.md
**Purpose:** Autonomous video proof of features
**Current state:** Manual recording steps
**Goal:** Fully autonomous end-to-end proof
- Launch game (via `launch-game.md`)
- Execute scenario (gameplay steps)
- Record window with gdigrab offset targeting
- Generate annotations (text overlays)
- Neural TTS voiceover (edge-tts, Microsoft Aria voice)
- Save video + metadata

### 3. check-game.md
**Purpose:** Verify game health
**Current state:** Status checks only
**Goal:** Autonomous diagnosis + recovery
- Check if game running
- Health check (mod loaded, systems active)
- If hung: collect diagnostics, restart
- Report status + recommendations

### 4. game-test.md
**Purpose:** Execute gameplay scenario
**Current state:** Manual test steps
**Goal:** Fully autonomous scenario execution
- Launch game
- Execute unit tests (spawn unit, move, attack)
- Collect results (damage, movement, timing)
- Shutdown game
- Report pass/fail

### 5. entity-dump.md
**Purpose:** Analyze entity/component state
**Current state:** Manual dump + analysis
**Goal:** Autonomous full analysis
- Launch game
- Trigger entity dump (via GameClient)
- Parse dump output
- Generate report (entity count, component distribution)
- Identify anomalies (duplicate IDs, missing components)
- Shutdown game

### 6. asset-create.md
**Purpose:** Create new asset (model import workflow)
**Current state:** Manual steps
**Goal:** Fully autonomous asset sourcing + import
- Query Sketchfab API
- Download candidate models
- Normalize in Blender (headless)
- Validate against schema
- Generate preview
- Add to pack manifest

### 7. pack-deploy.md
**Purpose:** Package + deploy mod pack
**Current state:** Manual build steps
**Goal:** Fully autonomous deployment
- Validate pack manifest
- Build asset bundles
- Run pack validator
- Generate checksums
- Create release artifact
- Upload to CDN/S3
- Update pack registry

### 8. test-swap.md
**Purpose:** Test asset swap integration
**Current state:** Manual gameplay test
**Goal:** Fully autonomous swap validation
- Launch game
- Trigger asset swap (via GameClient)
- Capture before/after screenshots
- Measure swap timing
- Verify visual correctness
- Shutdown game
- Generate report

---

## Autonomy Framework

### Level 1: Script Execution (Current)
- Command files document steps
- Agent reads steps, executes manually
- User sees agent making decisions in chat

### Level 2: Direct Subagent Delegation (Next)
- Each command maps to subagent task
- Parent agent orchestrates workflow
- Subagents never ask user; they report completion
- No "you should do X" messages in chat

### Level 3: Autonomous Proof Generation (Target)
- Video proof runs end-to-end
- Proof includes: gameplay recording + annotations + TTS voiceover
- Proof automatically published to GitHub (video URL in PR)
- Zero manual proof generation

---

## Subagent Delegation Pattern

All commands should follow this pattern:

```
Parent Agent
├─ Subagent: Game Launch
│  └─ Launch exe, detect window, health check
├─ Subagent: Scenario Execution
│  └─ Execute gameplay steps, collect results
├─ Subagent: Proof Generation
│  └─ Record video, add annotations, TTS, save
└─ Subagent: Reporting
   └─ Generate report, upload artifacts, post links
```

### Example: prove-features.md Evolution

**Current:** Manual recording steps
**Evolved:**

```markdown
# /prove-features

## Subagent Workflow

1. **Game Launch** (subagent: launch-game)
   - Direct EXE launch
   - Window detection
   - Health check

2. **Scenario Setup** (subagent: game-test)
   - Execute test scenario
   - Collect gameplay data

3. **Video Recording** (subagent: video-recorder)
   - Record game window via gdigrab offset
   - Capture at 1080p, 30fps
   - Duration: scenario execution + 5s buffer

4. **Annotation Generation** (subagent: video-annotator)
   - Parse gameplay events from test results
   - Generate text overlays (timestamps, event descriptions)
   - Apply overlays to video

5. **TTS Voiceover** (subagent: tts-narrator)
   - Generate narration script (feature name + description + test result)
   - Render via edge-tts (Microsoft Aria neural voice)
   - Mix audio into video

6. **Output + Publishing** (subagent: artifact-publisher)
   - Save final video to `/proof-videos/`
   - Generate metadata JSON (feature, test, timestamp)
   - Post video URL to chat
   - Commit to repo if requested

## Blacklist

- "Click the button" — use GameClient instead
- "Launch the game yourself" — we launch it
- "Watch the video" — we generate it
- "Test it yourself" — we automate it
```

---

## Proof Video Standard

All proof videos MUST include:

1. **Recording**
   - Game window captured via gdigrab with offset targeting
   - 1080p @ 30fps
   - Duration: scenario execution + 3s fade-out
   - Multi-monitor aware (offset from primary monitor to target window)

2. **Annotations**
   - Feature name (top-left corner)
   - Event descriptions (centered, time-synced)
   - Test result (green checkmark or red X, bottom-right)
   - Timestamps (bottom-left, MM:SS format)

3. **Voiceover**
   - Neural TTS (Microsoft Aria voice via edge-tts)
   - Script: Feature name → description → test result
   - Duration: matches video (auto-generated timing)
   - Language: English

4. **Metadata**
   - JSON file alongside video with:
     - Feature name
     - Test scenario
     - Pass/fail result
     - Timestamp
     - Command that generated it

### Example Metadata

```json
{
  "feature": "Asset Swap System",
  "scenario": "Swap Clone Trooper visual asset in-game",
  "result": "PASS",
  "timestamp": "2026-03-24T12:34:56Z",
  "command": "/prove-features asset-swap",
  "video_path": "/proof-videos/asset-swap-2026-03-24-120000.mp4",
  "duration_seconds": 45,
  "voiceover_text": "Asset Swap System test: Successfully swapped Clone Trooper model from vanilla to Star Wars asset. Swap completed in 250ms. Test PASSED."
}
```

---

## Command Evolution Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Extract subagent delegation pattern
- [ ] Build `launch-game` subagent (exe launch, window detection)
- [ ] Build `game-test` subagent (scenario execution)
- [ ] Update `.claude/commands/` with subagent references

### Phase 2: Proof Generation (Week 3-4)
- [ ] Build `video-recorder` subagent (gdigrab offset targeting)
- [ ] Build `video-annotator` subagent (text overlays)
- [ ] Build `tts-narrator` subagent (edge-tts neural voice)
- [ ] Integrate into `prove-features.md`

### Phase 3: Full Autonomy (Week 5-6)
- [ ] Audit all commands for user-interaction points
- [ ] Delegate remaining workflows to subagents
- [ ] Test end-to-end autonomy (no user prompts)
- [ ] Document proof standards + expectations

### Phase 4: Testing + Documentation (Week 7-8)
- [ ] Integration tests for each subagent
- [ ] Proof video validation (ffmpeg checks)
- [ ] Update CLAUDE.md governance
- [ ] Publish agent tooling guide

---

## Blacklist: Patterns to Eliminate

These patterns MUST NEVER appear in agent responses:

| Pattern | Why | Replace with |
|---------|-----|--------------|
| "Please launch the game" | User interaction required | Autonomous exe launch via `launch-game` |
| "Click the X button" | Manual interaction | GameClient button click API |
| "Test it yourself" | No proof generated | Autonomous gameplay test + video proof |
| "Watch the video" | Implies manual recording | Generate proof video autonomously |
| "You should do X" | Doesn't delegate | Create subagent for X |
| "Let me know if it works" | Requires user feedback | Auto-check via GameClient health check |
| "Run this command manually" | User must act | Automate the command |

---

## Integration Points

### GameClient Extensions

Subagents will use GameClient methods:

```csharp
// Launches game process
public async Task<ProcessHandle> LaunchGameAsync(string exePath, TimeSpan timeout);

// Health checks
public async Task<GameState> GetGameStateAsync();
public async Task<bool> IsModLoadedAsync();

// Gameplay automation
public async Task ExecuteScenarioAsync(Scenario scenario);
public async Task<GameplayResults> GetResultsAsync();

// Window detection
public async Task<IntPtr> GetGameWindowHandleAsync();
public async Task<Rect> GetGameWindowBoundsAsync();
```

### Video Recording Extensions

Subagents will use FFmpeg/gdigrab:

```bash
# Record game window with offset targeting
ffmpeg -f gdigrab \
  -offset_x <x> -offset_y <y> \
  -video_size 1920x1080 \
  -framerate 30 \
  -i desktop \
  -vf "drawtext=..." \  # overlays
  -c:v libx264 \
  output.mp4

# Mix audio + video
ffmpeg -i output.mp4 \
  -i voiceover.mp3 \
  -c:v copy \
  -c:a aac \
  final.mp4
```

### TTS Integration

Subagents will use edge-tts:

```bash
# Generate neural TTS audio
edge-tts \
  --voice en-US-AriaNeural \
  --text "Asset Swap System test successful" \
  --write-media voiceover.mp3
```

---

## Success Criteria

✅ **All commands fully autonomous:**
- Zero user interaction points
- Every command runs to completion without prompts
- Subagents never ask user questions

✅ **Proof standard consistently applied:**
- All proof videos include annotations + TTS
- Metadata JSON generated alongside each video
- Video URLs auto-posted to chat

✅ **Subagent delegation framework:**
- 8+ subagents available
- Clear handoff patterns between agents
- No repeated code across commands

✅ **Testing + documentation:**
- Integration tests for each subagent
- Proof video validation (ffmpeg checks)
- CLAUDE.md governance updated
- Agent onboarding guide published

---

## See Also

- **Current Custom Commands:** `.claude/commands/`
- **Proof Video Spec:** `/docs/specs/SPEC-prove-features-video-pipeline.md`
- **Prove Features Skill:** `/docs/specs/SPEC-003-prove-features-skill.md`
- **GameClient Bridge:** `src/Bridge/Protocol/IGameBridge.cs`
- **M5 Example Packs:** `/docs/milestones/MILESTONE-M5-example-packs.md`

---

**Plan Owner:** DINOForge Agent Orchestration
**Last Updated:** 2026-03-24
**Next Phase Start:** Week of 2026-03-31
