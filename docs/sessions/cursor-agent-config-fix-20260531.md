# cursor-agent config fix 2026-05-31

## Root cause
`cursor-agent` in this environment no longer reads `.cursor/cli.json` for project config. It reads the legacy global file `%USERPROFILE%\.cursor\cli-config.json`.

The repo-scoped `.cursor/cli.json` had already been cleaned, but `cli-config.json` still contained:

- `"approvalMode": "unrestricted"`

That caused the fatal startup error:

- `Invalid project config at <worktree>\.cursor\cli.json: Unrecognized key approvalMode`

(while that file itself was already valid).

## Fix applied
Removed `approvalMode` from:

- `%USERPROFILE%\.cursor\cli-config.json`

Validated the key is gone:

- `rg --line-number "approvalMode" $env:USERPROFILE\\.cursor -g "*.json"` returns only the expected no-match state after edit.

## Verification invocation
Command run:

```powershell
cursor-agent.cmd -p --force --model composer-2.5 "say hi"
```

Observed result in this worktree:

- `PROCESS_EXIT:terminated-after-30s`
- STDOUT: `<empty>`
- STDERR: `<empty>`

This indicates the invocation is not returning a completion result within 30s in this environment; no explicit parser error about `approvalMode` was observed after removing it.

## Final result
`composer-2.5` now RUN: **NO** (command did not complete/produce output within the captured 30s timeout).
