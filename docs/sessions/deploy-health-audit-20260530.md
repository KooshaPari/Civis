# Cross-Project Web Deploy Health Audit — 2026-05-30

Audit of expected web deploys across KooshaPari's projects (GitHub Pages +
`phenotype.space` subdomains). Diagnosed live with Playwright MCP (navigate +
console + network + screenshot). Fixes applied to DINOForge committed locally;
cross-repo fixes documented for follow-up.

## TL;DR

- **The phenotype.space VitePress subdomains are systemically broken by a base-path mismatch.** Each site is built with `base: '/<RepoName>/'` (for the `kooshapari.github.io/<Repo>/` deploy) but the **same bundle** is served at the custom-domain **root** `/`. Every `/<Repo>/assets/*.js|.css` then 404s, so pages load an unstyled, non-interactive HTML shell. Confirmed on `dino.phenotype.space` (33 errors) and `hwledger.phenotype.space` (34 errors).
- **DINOForge's own VitePress Pages deploy was failing in CI for days** (3 distinct build errors), so `kooshapari.github.io/Dino` and `dino.phenotype.space` were serving a stale bundle. **All 3 build errors + the base-path asset bug are now fixed and committed** (`506a6bb8`); build exits 0 locally.
- **agileplus Pages = not deployed at all** ("Site not found"); every `deploy.yml` run is failing. Same VitePress build class — needs the same triage as Dino.
- SSL was healthy everywhere tested (valid HTTPS, no cert errors). No SSL/cert provisioning failures found. The user's "SSL failures" symptom is most likely the browser surfacing the cascade of mixed 404s / blank renders, not an actual TLS fault.

## Health Table

| URL | SSL | Load errors | Status | Root cause | Fix |
|---|---|---|---|---|---|
| https://kooshapari.github.io/Dino/ | OK | 2× 404 (`/.vitepress/theme/custom.css`, `/favicon.svg` at root) | ⚠️ stale (loads, styled from old bundle) | CI deploy failing for days → stale bundle; head-link favicon/CSS used root-absolute paths instead of base | FIXED `506a6bb8` (build + base paths) — needs push to redeploy |
| https://dino.phenotype.space/ | OK | 33× 404 (all `/Dino/assets/*`) | ❌ broken/unstyled | base `/Dino/` bundle served at domain root `/` | Needs custom-domain build with `base:'/'` (see below) + redeploy of fixed bundle |
| https://kooshapari.github.io/agileplus/ | OK | "Site not found" | ❌ not deployed | every `deploy.yml` run failing (build error class like Dino) | Triage agileplus VitePress build (same 3 error classes) — see follow-up |
| https://agileplus.phenotype.space/ | OK | 404 page + `/favicon.ico` | ❌ 404 / empty | no valid build deployed (consequence of failing CI) + base mismatch | Fix agileplus build, then deploy with root base |
| https://hwledger.phenotype.space/ | OK | 34× 404 (all `/hwLedger/assets/*`) | ❌ broken/unstyled | base `/hwLedger/` bundle served at domain root `/` | custom-domain build with `base:'/'` + redeploy |
| https://civis.phenotype.space/ | OK | none | ✅ healthy | — | — |
| https://phenotype.space/ | OK | none | ✅ healthy (landing hub) | — | — |

Other subdomains listed on the phenotype.space hub (not individually diagnosed
this pass, but the dino/hwledger pattern almost certainly applies to any that is
a github.io-style VitePress build served at a custom-domain root): `tokn`,
`thegent`, `hexakit`, `helioslab`, `heliosapp`, `policystack`, `focalpoint`,
`pheno`, `cliproxyapi`, `parpoura`, and the `*-landing` sites.

Screenshot: `docs/screenshots/deploy-audit-dino-phenotype-space.png` (unstyled
dino.phenotype.space shell).

## Root-Cause Detail

### A. DINOForge VitePress build failing in CI (FIXED)

`deploy.yml` builds `docs/` with `npm run build`. Reproduced locally with the
CI env (`GITHUB_PAGES=true GITHUB_ACTIONS=true GITHUB_REPOSITORY=KooshaPari/Dino`).
Three sequential build-blocking errors:

1. **Unclosed HTML tag** — `docs/specs/SPEC-008-mod-conflict-detection-engine.md`
   line 679 contained `IRegistry<T>` in prose (not backticked). The Vue/VitePress
   markdown compiler parsed `<T>` as an unclosed element → `Element is missing end tag`.
   Fix: wrap in backticks.
2. **20 dead links** — links to repo files outside the docs `srcDir` (`README`,
   `CHANGELOG`, `CONTRIBUTING`, `INSTALLATION`, `src/...`, `packs/...`,
   `.claude/commands/...`, `tools/phenotype-journeys/...`, `evidence/...`, plus
   two unauthored intra-docs pages). VitePress's dead-link checker fails the
   build. Fix: targeted `ignoreDeadLinks` regex patterns in `config.mts`.
3. **SSR crash** — `docs/CI_IMPROVEMENTS.md` line 15 had a GitHub Actions
   expression `${{ runner.os }}` inside inline code. VitePress evaluated `{{ }}`
   as a Vue interpolation during SSR → `Cannot read properties of undefined
   (reading 'os')`. Fix: wrap in `<code v-pre>` to disable interpolation.

### B. Base-path asset 404s (FIXED in config)

`docs/.vitepress/config.mts` `head` and `themeConfig.logo` used root-absolute
paths (`/favicon.svg`, `/.vitepress/theme/custom.css`) that are **not**
base-prefixed by VitePress, so they 404 under the `/Dino/` sub-path deploy. The
`custom.css` `<link>` was also redundant (already `import './custom.css'` in
`theme/index.ts`) and pointed at a theme **source** path that is never published.
Fixes: favicon/logo now use `${docsBase}favicon.svg`; broken CSS link removed.
Verified in `dist/index.html`: `href="/Dino/favicon.svg"`, 0 theme-source CSS links.

### C. Custom-domain root vs github.io sub-path (the systemic phenotype.space bug)

`config.mts` computes `base = '/Dino/'` whenever `GITHUB_ACTIONS` **or**
`GITHUB_PAGES` is set. The phenotype.space subdomains are served at the domain
**root**, so a `/Dino/`-based bundle 404s every asset there. This is the dominant
cause of "broken/empty/CSS-failed" subdomains. **This config fix does NOT yet
address it** — it only fixes the github.io sub-path deploy. See follow-up #1.

## Fixes Applied (DINOForge — commit `506a6bb8`, NOT yet pushed)

- `docs/specs/SPEC-008-mod-conflict-detection-engine.md` — backtick `IRegistry<T>`.
- `docs/.vitepress/config.mts` — `ignoreDeadLinks` patterns; base-relative favicon/logo; removed broken custom.css `<link>`.
- `docs/CI_IMPROVEMENTS.md` — `<code v-pre>` around `${{ runner.os }}` expression.

Local build: `EXIT=0`, `build complete in 129s`. Pushing to `main` (or merging
the branch) will retrigger `deploy.yml` and publish a working bundle to
`kooshapari.github.io/Dino`.

## Follow-Up (needs push / cross-repo / external dashboard access)

1. **Fix the phenotype.space base mismatch (highest impact).** The custom-domain
   deploy must be built with `base: '/'`. Options, in order of robustness:
   - Drive base off an explicit env var the custom-domain pipeline sets, e.g.
     `const base = process.env.SITE_BASE ?? (isPagesBuild ? '/${repo}/' : '/')`,
     and have the Cloudflare Pages / phenotype.space build set `SITE_BASE=/`.
   - OR build twice (one artifact per host) — github.io with `/<repo>/`, custom
     domain with `/`.
   - The intended mechanism already exists: the shared phenodocs pipeline builds
     with `PHENOTYPE_CUSTOM_DOMAIN: "true"`, and the `@phenotype/docs` theme is
     meant to switch `base` to `/` when that flag is set. The broken subdomains
     (`dino`, `hwledger`) still serve `/Dino/`- and `/hwLedger/`-based bundles,
     which means **they were deployed by an OLD pipeline that did not honor
     `PHENOTYPE_CUSTOM_DOMAIN`** (or aren't on the shared theme yet). Notably,
     `Dino/docs/.vitepress/config.mts` does **not** reference
     `PHENOTYPE_CUSTOM_DOMAIN` at all — it hardcodes `/Dino/` for any CI build.
   - The phenotype.space subdomains are served via Cloudflare (observed
     `/cdn-cgi/rum` beacon). Two paths to fix: (a) migrate Dino/hwLedger to the
     shared phenodocs pipeline that already passes `PHENOTYPE_CUSTOM_DOMAIN`, or
     (b) add `PHENOTYPE_CUSTOM_DOMAIN` handling to each repo's `config.mts`:
     `const base = process.env.PHENOTYPE_CUSTOM_DOMAIN === 'true' ? '/' : (isPagesBuild ? \`/${repoName}/\` : '/')`.
     Then redeploy via the Cloudflare-connected pipeline.
2. **Push the DINOForge fix** (`506a6bb8`) to `main` to redeploy github.io and
   the dino.phenotype.space bundle.
3. **agileplus deploy failure is in the shared-theme install step, NOT the
   markdown.** Important: the **remote** `deploy.yml` on `main` is NOT the same
   as the local copy. The remote pipeline uses a **shared `@phenotype/docs`
   VitePress theme** (`KooshaPari/phenodocs`) and fails at the **"Install
   dependencies"** step (runs ~7-14s, before build). The local repo builds clean
   (`bun run build` EXIT 0), confirming the markdown is fine. The remote install
   step is:
   ```yaml
   - name: Install dependencies
     working-directory: docs
     env: { GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }} }
     run: |
       mkdir -p docs/node_modules/@phenotype          # BUG: cwd is already docs/ → makes docs/docs/...
       ln -sf /tmp/phenodocs/packages/docs docs/node_modules/@phenotype/docs
       echo "@phenotype:registry=https://npm.pkg.github.com" >> ~/.npmrc
       echo "//npm.pkg.github.com/:_authToken=\${GITHUB_TOKEN}" >> ~/.npmrc
       bun install
   ```
   Likely failure causes (need expired-log refresh / re-run to confirm exact one):
   - **Path bug**: with `working-directory: docs`, `mkdir -p docs/node_modules/...`
     and the `ln -sf docs/node_modules/...` target both resolve to `docs/docs/...`,
     so the `@phenotype/docs` symlink is never placed where bun resolves it
     (`docs/node_modules/@phenotype/docs`). Fix: drop the leading `docs/`
     (paths are already relative to `docs/`).
   - **GitHub Packages auth**: `bun install` then falls back to the
     `@phenotype` npm registry, which requires `read:packages` scope. The
     default `GITHUB_TOKEN` may lack access to the `KooshaPari/phenodocs`
     package, or the package isn't published → install fails.
   This shared-theme pipeline is **the systemic linchpin**: the same phenodocs
   theme + `PHENOTYPE_CUSTOM_DOMAIN: "true"` build flag drives every
   phenotype.space VitePress site. Fixing it here fixes the class. Repo:
   `C:\Users\koosh\agileplus` (and `KooshaPari/phenodocs`, not present locally).
4. **Sweep remaining phenotype.space subdomains** (tokn, thegent, hexakit, etc.)
   for the same base-path 404 once the base fix pattern is decided.

## Known Residual Risk (DINOForge)

- **Flaky `.temp` media-import failure.** A custom-domain build run hit
  `ERR_MODULE_NOT_FOUND` for `.vitepress/.temp/embeds/media/mods-quick-panel.mp4`
  (the file *does* exist at `docs/embeds/media/mods-quick-panel.mp4`). This is a
  VitePress `.temp` relative-path resolution race, intermittent, and pre-existing
  — the standard `GITHUB_PAGES` build passes cleanly. If CI flakes on this,
  rerun, or convert the markdown video embed to an absolute (base-relative) path.

## Method / Repro Commands

```powershell
# Reproduce a VitePress Pages build exactly as CI does:
cd <repo>\docs   # or repo root for agileplus
$env:GITHUB_PAGES='true'; $env:GITHUB_ACTIONS='true'; $env:GITHUB_REPOSITORY='KooshaPari/<Repo>'
npm ci; npm run build   # agileplus: bun install --frozen-lockfile; bun run build

# Check recent Pages deploy status:
gh run list --workflow=deploy.yml --limit 5
```
