# Intent: FR-CIV-CLIENT-011 — Six-step tutorial hint system

> Date: 2026-09-19
> FR: FR-CIV-CLIENT-011
> Epic: FR-CIV-CLIENT

## User Intent

The Bevy reference client displays a six-step tutorial hint panel
bottom-centre during the InGame state. Enter / click advances to the next
step; `H` replays the sequence. The panel is intentionally local-only and
does not communicate with the server over JSON-RPC.

## Acceptance Signal

- `clients/bevy-ref/src/tutorial.rs` exposes the static `HINTS` array and
  panel system gated by `cfg(all(feature = "bevy", feature = "egui"))`.
- `in_playing_state` gate ensures the panel only renders during `InGame`.
- `// Covers: FR-CIV-CLIENT-011` on the panel module header.
