# CodeRabbit main config orchestration report

Generated: 2026-05-27 (agent subagent; shell blocked in Cursor)

## 1. Config comparison (origin/main vs origin/followup/post-pr188-followups)

| Setting | `main` | `followup/post-pr188-followups` |
|---------|--------|--------------------------------|
| `request_changes_workflow` | **missing** | `true` |
| `auto_approve.enabled` | **missing** | `true` |
| `auto_incremental_review` | **missing** | `true` |

`main` still has only basic `auto_review`; followup branch has the bot-approve settings CodeRabbit needs on **default branch** for public repos.

## 2. Execution status

| Step | Status |
|------|--------|
| Branch `agent/coderabbit-main-config` from `origin/main` | **NOT pushed** (Cursor Shell tool broken: `MIC_LD_LIBRARY_PATH` truncates `ps-script-*.ps1`) |
| PR "chore: enable CodeRabbit bot approve on main" | **NOT created** |
| CodeRabbit `@coderabbitai review` / `approve` on config PR | **NOT run** |
| Poll / merge config PR | **NOT run** |
| PR #221 approve / merge | **NOT run** |
| `git checkout main && git pull` in Dino | **NOT run** |

## 3. Unblock and run (one-time)

**Fix shell (outside Cursor):** run `C:\Users\koosh\.cursor\fix-cursor-shell.cmd` then restart Cursor.

**Or run orchestration without Cursor:**

```text
wscript C:\Users\koosh\Dino\scripts\run-coderabbit-orchestration.vbs
```

Log: `C:\Users\koosh\Dino\scripts\coderabbit-orchestration.log`

**Alternative (direct to main, no config PR):**

```powershell
powershell -NoProfile -File C:\Users\koosh\Dino\scripts\push-coderabbit-main.ps1
```

## 4. If config PR never gets `coderabbitai[bot]` APPROVED

- One-time **human approve** the tiny `.coderabbit.yaml`-only PR, **or**
- Add repo secret **`AGENT_MERGE_PAT`** and use `gh pr merge` / `agent-merge-on-bot-approve` workflow_dispatch.

## 5. Return payload (current remote state)

| Item | Value |
|------|-------|
| **Config PR number** | *(none — branch not on origin)* |
| **Config PR head SHA** | *(n/a)* |
| **PR #221** | [Open](https://github.com/KooshaPari/Dino/pull/221) |
| **PR #221 head SHA** | `eece6acb3034fd7e59a25ff86f9a3f4d0186712e` |
| **PR #221 merged?** | **No** |
| **PR #221 base (main) SHA** | `15ba282c02c35de752f5d60964c7f9ecaaf64fec` |
| **origin/main HEAD** | `15ba282c02c35de752f5d60964c7f9ecaaf64fec` |
| **coderabbitai[bot] APPROVED on #221?** | **No** (only COMMENTED bot reviews) |

Prepared locally:

- `scripts/coderabbit-main-target.yaml` — target config for main
- `scripts/coderabbit-main-config.bat` — full git/gh/poll/merge pipeline
- `scripts/run-coderabbit-orchestration.vbs` — runs fix + bat, writes log
