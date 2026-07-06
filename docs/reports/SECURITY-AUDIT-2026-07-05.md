# Civis Security Audit — 2026-07-05

**Tool:** `cargo audit 0.22.2` (RustSec advisory database, 1156 advisories loaded at scan time)
**Target:** `Cargo.lock` at origin/main `f828a009` (HEAD: `da8c771e` for merge artifact dedup, plus PR #1370 not yet merged into main at scan time)
**Scope:** Full workspace `Cargo.lock` (1110 crate dependencies scanned)
**Outcome:** 6 vulnerabilities (2 HIGH, 4 MEDIUM) + 4 unmaintained/yanked warnings.
**Action this audit:** Documentation only. **No `Cargo.toml` or `Cargo.lock` mutations applied** — all six CVEs are blocked at the dep-graph level and require a user-chosen escape hatch (H1–H5 below).

---

## Findings

### HIGH (2)

| RUSTSEC ID | Crate | Version | Title | Fix Version |
|---|---|---|---|---|
| [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194) | `quick-xml` | 0.39.4 | Quadratic run time when checking a start tag for duplicate attribute names (CVSS 7.5) | `>=0.41.0` |
| [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195) | `quick-xml` | 0.39.4 | Unbounded namespace-declaration allocation in `NsReader` enables memory-exhaustion DoS (CVSS 7.5) | `>=0.41.0` |

Both HIGHs share a single root cause (one transitive dep version) and resolve with one bump.

### MEDIUM (4)

| RUSTSEC ID | Crate | Version | Title | Fix Version |
|---|---|---|---|---|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | `rsa` | 0.9.10 | Marvin Attack: potential key recovery through timing sidechannels (CVSS 5.9) | **No upstream fix available** |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | `rustls-webpki` | 0.101.7 | Name constraints for URI names were incorrectly accepted | `>=0.103.12, <0.104.0-alpha.1` OR `>=0.104.0-alpha.6` |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | `rustls-webpki` | 0.101.7 | Name constraints were accepted for certificates asserting a wildcard name | `>=0.103.12, <0.104.0-alpha.1` OR `>=0.104.0-alpha.6` |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | `rustls-webpki` | 0.101.7 | Reachable panic in certificate revocation list parsing | `>=0.103.13, <0.104.0-alpha.1` OR `>=0.104.0-alpha.7` |

The three `rustls-webpki` MEDIUMs share one transitive dep version. The `rsa` MEDIUM is a special case: no upstream patch, so resolution requires dep replacement or fork+mitigation.

### UNMAINTAINED / YANKED (4)

Not security per se, but flagged by `cargo audit`:

| ID | Crate | Version | Note |
|---|---|---|---|
| RUSTSEC-2025-0141 | `bincode` | 1.3.3 | Unmaintained (2025-12-16). Replacement candidate: `bincode-next` 3.1.x. |
| RUSTSEC-2024-0436 | `paste` | 1.0.15 | Unmaintained (2024-10-07). Compiler built-in `paste!` macro available in recent nightly; otherwise no clean replacement. |
| RUSTSEC-2026-0192 | `ttf-parser` | 0.25.1 | Unmaintained (2026-06-28). Active fork in development upstream. |
| (yanked) | `num-bigint` | 0.4.7 | Yanked from crates.io. Likely superseded by `num-bigint 0.4.8` in lockfile when `cargo update` is run; verify by tracing dependents. |

---

## Dep-Graph Traces (block analysis)

### `quick-xml 0.39.4` — root cause of 2 HIGH

```
civ-bevy-ref (clients/bevy-ref, member of workspace)
└── winit 0.30.13
    └── sct 0.19.2 (= smithay-client-toolkit)
        └── smithay-client-toolkit 0.19.2
            └── wayland-scanner 0.31.10   ← LATEST on crates.io (search 2026-07-05)
                └── quick-xml = "^0.39"   ← hard upper bound
```

`wayland-scanner 0.31.10` is the latest published version. No future release on the registry relaxes `quick-xml = "^0.39"`. The 2.0 line of wayland-scanner (`wayland-scanner 2.x`) exists in some forks but is not on the registry at scan time.

`cargo update -p quick-xml --precise 0.41.0` → **REJECTED**: "candidate versions found which didn't match: 0.41.0" (constraint `^0.39` cannot be widened by `cargo update`).

`[patch.crates-io]` git override pointing at `tafia/quick-xml@v0.41.0` → **REJECTED**: "patch was not used in the crate graph". The constraint `^0.39` is already satisfied by 0.39.4 (the latest `^0.39` resolution), so cargo will not substitute the patched source.

### `rustls-webpki 0.101.7` — root cause of 3 MEDIUM

```
civ-infra (crates/infra, member of workspace)
└── aws-config 1.8.18                       ← LATEST on crates.io (search 2026-07-05)
    └── aws-smithy-runtime 1.11.3
        └── aws-smithy-http-client 1.1.13   ← LATEST in 1.1.x line
            └── legacy-rustls = "^0.21.8"
                └── rustls 0.21.12
                    └── rustls-webpki = "^0.101.7"  ← vulnerable
```

`aws-config 1.8.18` is the latest published version. `aws-smithy-http-client 1.1.13` is the latest in the `1.1.x` series. The `1.x` line of aws-smithy-http-client still binds `legacy-rustls`; the `aws-sdk-rust` team has not yet cut a release that lifts the `rustls` constraint off 0.21. Resolution requires either upstream movement or a workspace fork.

### `rsa 0.9.10` — root cause of 1 MEDIUM (Marvin Attack)

The Marvin Attack advisory was filed 2023-11-22. The Rust `rsa` crate maintainers have **not released a fix** in the 0.9.x line; the project is effectively in maintenance mode with no patch planned. Replacement candidates exist (`dalek` ed25519/x25519 for signing; `ml-kem`/`ml-dsa` for post-quantum) but require application-layer refactor since `rsa` is used for signature verification in our deps.

**Owner must decide:** replace `rsa` with a modern signature scheme, OR pin `rsa` users to time-constant operations via `rsa` crate options (mitigation, not fix).

---

## Escape Hatches (none applied — user decision required)

### H1 — Fork `wayland-scanner`, relax `quick-xml` to `^0.41`
**Size:** small fork (single Cargo.toml change), but distributed via `[patch.crates-io]` from a Civis-owned repo. Re-builds `civ-bevy-ref` against new quick-xml API.
**Side-effects:** `civ-bevy-ref` is currently RED (missing modules `perf_hud`, `tutorial`, `menus` per PR #1370 follow-ups). Fork lands BEFORE that build is green → deferred risk. CI required.
**Status:** feasible.

### H2 — Backport security fix into `quick-xml 0.39.x`
**Size:** small (one crate, one function family). 0.39 line is EOL; we own the backport forever.
**Side-effects:** divergence from upstream `quick-xml`; backport must be re-applied on every new finding. Brittle.
**Status:** feasible but operationally expensive.

### H3 — Bump `winit` to 0.31.0-beta.2
**Size:** moderate. `winit 0.31` rewrote `EventLoop` API; `civ-bevy-ref` uses winit event loop glue.
**Side-effects:** beta API, may shift again before stable. Requires full re-test of civ-bevy-ref's windowing path. Likely pulls newer `sctk`/`smithay-client-toolkit`/`wayland-scanner` with relaxed `quick-xml` constraint → solves H1 problem indirectly.
**Status:** feasible, beta risk.

### H4 — Fork `aws-smithy-http-client`, bump its `rustls` deps
**Size:** large. AWS-SDK-team-grade backport; touches hyper integration code in `aws-smithy-http-client`.
**Side-effects:** we own a fork of a security-critical dep in our infra path. Long-term maintenance burden. The right answer is upstream PR to `aws-sdk-rust`; if accepted, we delete the fork.
**Status:** feasible but should be paired with upstream issue filing per memory `feedback_org_level_fixes.md`.

### H5 — Vendor `quick-xml 0.41` + backport `webpki 0.103` fixes into `webpki 0.101`
**Size:** moderate but very brittle. Two patches to maintain; loses upstream forward-port for both.
**Side-effects:** `quick-xml 0.41` is a vendored fork we ship; `rustls-webpki 0.101` is forked with cherry-picked fixes. Both diverge from upstream.
**Status:** feasible, **NOT RECOMMENDED** unless H1 and H4 are both blocked.

---

## Recommendations

1. **File upstream issues** for both `wayland-scanner` (relax `quick-xml`) and `aws-smithy-http-client` (lift `legacy-rustls` constraint) per memory `feedback_org_level_fixes.md` — root-cause in sibling → upstream PR pattern. This is the cheapest path forward.
2. **H1 is the lowest-cost near-term fix** for the 2 HIGH CVEs. Defer until `civ-bevy-ref` is build-green (currently in PR #1370 follow-up lane).
3. **H4 is the lowest-cost near-term fix** for the 3 MEDIUMs. Pair with upstream PR to `aws-sdk-rust` per memory `feedback_org_level_fixes.md`.
4. **`rsa 0.9.10` Marvin Attack requires a security-decision** — replacement vs. mitigation. Surface to user before any action.
5. **Unmaintained items (bincode, paste, ttf-parser, num-bigint)** are hygiene, not security — defer to next DEP-ROT cycle AFTER the 6 CVE patches land, per memory `feedback_superset_merge_default.md` (don't conflate security with hygiene).

---

## Spec / Issue Tracking

After this audit doc merges, SPEC issues to open (per lead direction):

- **P0** × 2 — one per HIGH CVE (RUSTSEC-2026-0194, RUSTSEC-2026-0195)
- **P0** × 1 — `rsa` replacement/fork decision (no upstream fix; user-decision required)
- **P1** × 3 — one per MEDIUM rustls-webpki CVE (RUSTSEC-2026-0098, RUSTSEC-2026-0099, RUSTSEC-2026-0104)
- **P2** × 1 — collective unmaintained (bincode, paste, ttf-parser, num-bigint)

Total: 7 SPEC issues.

---

## Verification

```
$ cargo audit --version
cargo-audit-audit 0.22.2

$ cargo audit
Loaded 1156 security advisories (from E:\Dev\.cargo\advisory-db)
Updating crates.io index
Scanning Cargo.lock for vulnerabilities (1110 crate dependencies)
... (6 vulnerabilities, 4 unmaintained/yanked warnings — see findings above)
error: 6 vulnerabilities found!
warning: 4 allowed warnings found
```

Re-run at any time to confirm. Repeat on every `Cargo.lock` change in CI per memory `feedback_latest_pkgs_cve_aware.md`.

---

## Changelog

- 2026-07-05 — Initial audit. Document + dep-graph traces only. No `Cargo.toml` or `Cargo.lock` mutations.