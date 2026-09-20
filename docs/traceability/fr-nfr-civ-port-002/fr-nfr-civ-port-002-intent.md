# Intent: FR-NFR-CIV-PORT-002 — Backend selection tradeoff ADR

> Date: 2026-09-20
> FR: FR-NFR-CIV-PORT-002
> Epic: FR-NFR-CIV-PORT

## What This FR Captures

The portability NFR requiring an ADR documenting the
DLSS-requires-Vulkan vs Solari-requires-DX12 tradeoff on Windows.
The selection policy must be explicit so agent-driven work does not
introduce regressions. The ADR must reference the `dlss` and
`solari` Cargo features on `civ-client`. Full statement lives in
`docs/reference/non-functional-requirements.md:431-441`.

## User Intent

DLSS (Bevy upscaling via DLSS plugin) needs Vulkan; Solari
global illumination needs DX12. These are mutually exclusive on
Windows; without a documented selection rule, agents will
arbitrarily pick one and silently disable the other.

## Acceptance Signal

- ADR file `docs/adr/backend-selection-dlss-vs-solari.md`
  exists, is ≥ 200 words, and is referenced from `clients/`
  `Cargo.toml` feature flag comments for `dlss` / `solari`.

## Implementing Code

- `docs/reference/non-functional-requirements.md:431` —
  statement.

## Test Coverage

- CI documentation lint asserting the ADR file exists and that
  `clients/Cargo.toml` references it in the `dlss` / `solari`
  feature definitions.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-civ-port-002-intent.md` |
| General spec | `docs/reference/non-functional-requirements.md` |