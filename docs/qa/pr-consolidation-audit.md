# PR Consolidation Audit

Read-only inventory from `gh pr list --state open --json number,title,headRefName,baseRefName,author,createdAt,mergeable,isDraft,commits,changedFiles,body` plus `gh pr checks <number>` for each open PR.

| PR | Title | Head | Base | Author | Created | Mergeable | Checks | Commits | Files | Summary | Recommendation |
|---|---|---|---|---|---|---|---|---:|---:|---|---|
| #230 | chore(deps): bump uuid from 11.1.0 to 14.0.0 in the npm_and_yarn group across 1 directory | `dependabot/npm_and_yarn/npm_and_yarn-e9ce4f7be9` | `main` | `app/dependabot` | 2026-05-29T03:41:27Z | MERGEABLE | Green overall; required checks showed passes | 1 | 1 | Updates the indirect `uuid` dependency in the repo root npm/yarn lock context to the latest major release Dependabot proposed. | READY-TO-MERGE |
| #218 | [Snyk] Security upgrade idna from 3.10 to 3.15 | `snyk-fix-80168d908625f6d971bf41969dd61351` | `main` | `KooshaPari` | 2026-05-26T08:38:54Z | CONFLICTING | Conflicting with base; rebase needed before checks can be trusted | 1 | 1 | Pins `idna>=3.15` in `src/Tests/e2e/requirements.txt` to remove a known vulnerability in e2e test installs. | NEEDS-REBASE |
| #217 | [Snyk] Security upgrade @remotion/cli from 4.0.257 to 4.0.464 | `snyk-fix-7389a3c591b5d9eb5726479c717e9955` | `main` | `KooshaPari` | 2026-05-26T08:37:44Z | CONFLICTING | Conflicting with base; rebase needed before checks can be trusted | 1 | 1 | Security bump for the `@remotion/cli` dependency used by the video tooling path. | NEEDS-REBASE |
| #216 | [Snyk] Security upgrade mermaid from 10.9.5 to 10.9.6 | `snyk-fix-9edfd94ce8b34257abbe93fcae9e822c` | `main` | `KooshaPari` | 2026-05-26T08:37:27Z | CONFLICTING | Conflicting with base; rebase needed before checks can be trusted | 1 | 1 | Security bump for `mermaid` to the patched 10.9.6 release. | NEEDS-REBASE |
| #215 | chore(deps): bump @remotion/cli from 4.0.257 to 4.0.468 in /scripts/video | `dependabot/npm_and_yarn/scripts/video/remotion/cli-4.0.467` | `main` | `app/dependabot` | 2026-05-26T08:34:30Z | MERGEABLE | Green overall; required checks showed passes | 1 | 1 | Updates the `scripts/video` copy of `@remotion/cli` to a newer patch release. | READY-TO-MERGE |
| #209 | chore(deps-dev): bump playwright from 1.59.1 to 1.60.0 | `dependabot/npm_and_yarn/playwright-1.60.0` | `main` | `app/dependabot` | 2026-05-26T08:33:51Z | MERGEABLE | Green overall; required checks showed passes | 1 | 2 | Dev-only Playwright upgrade for the browser automation test toolchain. | READY-TO-MERGE |
| #206 | chore(deps): bump nalgebra from 0.33.3 to 0.35.0 in /src/Tools/AssetPipelineRust | `dependabot/cargo/src/Tools/AssetPipelineRust/nalgebra-0.35.0` | `main` | `app/dependabot` | 2026-05-26T08:33:48Z | MERGEABLE | Green overall; required checks showed passes | 1 | 2 | Rust asset pipeline dependency bump for `nalgebra` in `src/Tools/AssetPipelineRust`. | READY-TO-MERGE |
| #193 | chore(deps): bump ndarray from 0.15.6 to 0.17.2 in /src/Tools/AssetPipelineRust | `dependabot/cargo/src/Tools/AssetPipelineRust/ndarray-0.17.2` | `main` | `app/dependabot` | 2026-05-26T08:33:31Z | MERGEABLE | Green overall; required checks showed passes | 1 | 2 | Rust asset pipeline dependency bump for `ndarray` in `src/Tools/AssetPipelineRust`. | READY-TO-MERGE |
| #189 | docs(ci): bootstrap workflows on main | `docs/ci-workflow-bootstrap` | `main` | `KooshaPari` | 2026-05-25T06:20:10Z | CONFLICTING | Conflicting with base; rebase needed before checks can be trusted | 1 | 2 | Adds a docs note and a minimal `ci.yml` bootstrap stub so `main` can regain a named CI workflow before the full restore lands. | NEEDS-REBASE |

## Recommendation Counts

- READY-TO-MERGE: 5
- NEEDS-REBASE: 4
- NEEDS-CI-FIX: 0
- DRAFT-WAIT: 0
- SUPERSEDED: 0

## Overlap Note

No open PR head branch matched `feat/unityexplorer-devtools-20260528` in the current `gh pr list` inventory, so no direct overlap was observed from the open-PR metadata alone.
