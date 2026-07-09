# Security / cargo-audit triage — 2026-07-08

**Scope:** Open RUSTSEC issues blocking Cargo Audit on `main` (#491, #1371–#1378) and PR #1379 (audit report, no patches).  
**Policy:** Document escape hatches so Wave 1+ product work is not falsely blocked; apply upgrades when available.

## Open advisories

| Issue | Advisory | Package | Severity | Escape hatch |
|-------|----------|---------|----------|--------------|
| #1371 | RUSTSEC-2026-0194 | `quick-xml` 0.39.4 | HIGH | **H1:** bump to patched release when published; until then `deny.toml` ignore with justification (quadratic on duplicate attrs — Civis XML surface is mod-manifest only, untrusted mods already sandboxed) |
| #1372 | RUSTSEC-2026-0195 | `quick-xml` NsReader | HIGH | **H1** (same bump / ignore) |
| #1373 | RUSTSEC-2023-0071 | `rsa` 0.9.10 Marvin | HIGH | **H2:** no upstream fix — keep ignore; rsa is transitive (TLS/signing path). Prefer `ed25519` for first-party mod signing (already Live). Revisit when rsa 0.10+ lands |
| #1376–#1378 | RUSTSEC-2026-0098/99/0104 | `rustls-webpki` 0.101.7 | HIGH/MED | **H3:** bump `rustls`/`webpki` stack via `cargo update -p rustls-webpki`; if locked by Bevy/deps, ignore with note that Civis server is local-first WS (not public TLS terminator) |
| #1375 | unmaintained/yanked | bincode, paste, ttf-parser, num-bigint | LOW | **H4:** hygiene — track in Dependabot; no ship blocker |
| #491 | Cargo Audit failing on main | — | CI | **H5:** weekly audit workflow stays red until H1–H3 applied; do not gate product PRs on audit alone — `cargo-deny` on PR remains the license/advisory gate |

## Recommended next PR (deps-only)

1. Land or close #1379 after copying its report into `docs/reports/SECURITY-AUDIT-2026-07-05.md` (if missing on main).  
2. `cargo update -p quick-xml -p rustls-webpki` (or workspace-compatible versions).  
3. Add explicit `[advisories] ignore` entries in `deny.toml` only for H2 (rsa Marvin) with expiry comment.  
4. Re-run Cargo Audit workflow; close #491 when green.

## Non-goals

- Do not block PHASE_ORDER / god-tool / MCP product merges on unresolved transitive RUSTSECs with documented hatches.  
- Do not absorb org PolicyStack into Civis for this triage.
