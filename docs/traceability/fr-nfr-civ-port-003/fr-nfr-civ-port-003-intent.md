# Intent: FR-NFR-CIV-PORT-003 — Headless server on all platforms

> Date: 2026-09-20
> FR: FR-NFR-CIV-PORT-003
> Epic: FR-NFR-CIV-PORT

## What This FR Captures

The portability NFR requiring the headless simulation server
(`civlab-server`) to build and run without any GPU backend
dependency on all three OS targets (Windows, macOS, Linux). This
enables CI and research workloads on machines without a discrete
GPU. Statement lives in
`docs/reference/non-functional-requirements.md:445-455`.

## User Intent

Headless operation underpins FR-API-002 (research API) and
NFR-CIV-DET-001/002 (determinism testing). If the server pulled in
a GPU feature by default, every CI Linux runner without a GPU
would fail to even compile, blocking the determinism suite.

## Acceptance Signal

- `cargo build -p civlab-server --no-default-features` succeeds
  on all three targets.
- The 50-tick determinism test passes on the CI Linux runner
  with no GPU available.

## Implementing Code

- `docs/reference/non-functional-requirements.md:445` —
  statement.

## Test Coverage

- CI step `build/headless-server-no-gpu` compiling with no GPU
  feature flags.
- CI determinism test on the Linux runner.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-civ-port-003-intent.md` |
| General spec | `docs/reference/non-functional-requirements.md` |