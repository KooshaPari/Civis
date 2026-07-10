# Threat Model — CivLab / Civis (seed)

**Date:** 2026-07-08  
**Status:** Seed (pillar L20/L39). Expand with STRIDE workshop before public multiplayer.

## Assets

| Asset | Sensitivity |
|-------|-------------|
| Deterministic sim state + `.civreplay` | Integrity (research/replay) |
| Save slots / save-db | Integrity + availability |
| Mod packages (`.civmod`) + Ed25519 signatures | Integrity; untrusted code |
| JSON-RPC / WS control plane | Integrity of world mutations |
| Operator role (god-tools) | Authorization boundary |

## Trust boundaries

1. **Local operator ↔ civ-server** — default local-first; `require_role` is param-asserted (not AuthN). Spoofable if exposed on a network.  
2. **Mod host ↔ WASM mods** — untrusted; signing + sandbox assumed.  
3. **MCP client ↔ civis-mcp ↔ civ-server** — agent tools inherit operator capability.

## Top threats (STRIDE lite)

| ID | Threat | Mitigation today | Gap |
|----|--------|------------------|-----|
| T1 | Spoofed operator role on WS | Ignore `x-civis-role` header; role in params | Real AuthN (token/mTLS) |
| T2 | Malicious mod | Ed25519 verify + WASM sandbox | Continuous fuzz of mod host |
| T3 | DoS via god-tool / place_voxel flood | `max_clients` | Per-RPC rate limits |
| T4 | Supply-chain CVE | cargo-deny + weekly audit | See SECURITY-AUDIT-TRIAGE |
| T5 | Replay / save tampering | Hash chain + slot metadata | Signed save attestations |

## Out of scope (v1)

Public internet exposure, multi-tenant isolation, payment/PII.
