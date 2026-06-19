# Civis Bevy dev loop — fast incremental builds + hot-reload

This guide describes the fast iteration loop for the Civis Bevy desktop client
(`civ-bevy-ref`). It covers incremental compilation, the LLD linker, asset
hot-reload, and the system/code hot-reload status.

> TL;DR
>
> | Command | What it does |
> | --- | --- |
> | `just run` | One-shot launch of the standalone sandbox (incremental). |
> | `just run-voxel` | One-shot launch of the live windowed/voxel client. |
> | `just dev-fast` | Watch + incremental + **asset hot-reload** + `dynamic_linking` (`hot`). |
> | `just dev-fast-voxel` | Same loop for the windowed/voxel client. |
>
> Taskfile parallels exist: `task run`, `task run:voxel`, `task dev:fast`, `task dev:fast-voxel`.
>
> NOTE: the plain `just dev` / `just dev-stop` recipes are reserved for the
> infra stack (`process-compose up`/`down`). The fast Bevy iteration loop is
> therefore exposed as `dev-fast` / `dev-fast-voxel` (and `task dev:fast`).

---

## 1. Incremental builds + fast linker

Configured in `.cargo/config.toml` and the workspace `[profile.dev]`
(`Cargo.toml`). **Release is untouched**, so determinism and release
optimization are unaffected.

- **`incremental = true`** (`[build]`) — reuse codegen across warm rebuilds.
  CI/release set `CARGO_INCREMENTAL=0` for reproducible artifacts.
- **`[profile.dev]`** — `opt-level = 0` (our crates), `debug = 1` (line tables
  only), `split-debuginfo = "packed"` (separate `.pdb`, lighter links).
- **`[profile.dev.package."*"]`** — `opt-level = 3`, `debug = false` for
  third-party deps (compiled once): keeps the running sandbox fast without
  slowing warm rebuilds. This is the standard Bevy fast-dev recommendation.
- **rust-lld linker** (Windows `x86_64-pc-windows-msvc`) — `rust-lld.exe` ships
  *inside* the Rust toolchain (no install). It only changes link time, not
  codegen, so it cannot affect runtime behaviour or determinism. Linux/macOS
  linker stanzas are present but commented out (uncomment if `clang`/`lld`/`mold`
  is installed).
- **cranelift codegen backend** (optional, nightly) — documented in
  `.cargo/config.toml`; not wired because `rust-toolchain.toml` pins stable.

### Measured compile-time deltas (this machine: Ryzen + RTX 3090 Ti, Windows 11)

Target: `cargo build -p civ-bevy-ref --features bevy,egui --bin civ-standalone`,
measured with `Measure-Command` on an otherwise-idle machine (rustc 1.95).

| Scenario | Time (after: rust-lld + dev profile + incremental) |
| --- | --- |
| Cold (clean target, all deps at `opt-level=3`) | **100.2 s** |
| Warm rebuild (no-op) | **38.6 s** |
| Incremental (1-line edit in our bin → relink `civ-bevy-ref` only) | **43.4 s** |

Notes:
- The dominant warm/incremental cost is the **final link** of the standalone
  binary; rust-lld already makes this far cheaper than MSVC `link.exe`. The
  `hot` feature (`dynamic_linking`) removes the need to relink the Bevy engine
  into the binary on each iteration, shrinking the edit→run relink further — use
  `just dev-fast` for that loop.
- An earlier "warm" sample of ~2180 s was discarded: it was taken while a second
  full `cargo` build was saturating all cores on this shared machine. Always
  measure with no competing build running.
- Deps are built once at `opt-level=3`; that inflates the *first* cold build but
  keeps every subsequent warm/incremental build fast and the running sandbox
  performant.

---

## 2. Asset hot-reload (live)

Enabled by the **`dev`** Cargo feature, which turns on Bevy's `file_watcher` and
forces `AssetPlugin { watch_for_changes_override: Some(true) }` via
`native_backend::dev_asset_plugin()` (wired into both `civ-standalone` and
`civ-bevy-window`). Desktop-only; never enabled for release/CI.

With `just dev-fast` running, edit any asset under `assets/` — SVG-derived PNGs,
`.glb` meshes, or WGSL shaders — and Bevy reloads it **into the running process
without a rebuild or restart**. Watch the console for
`Reloading <path> because it changed`.

Without the `dev`/`hot` feature, `dev_asset_plugin()` is a plain
`AssetPlugin::default()` (watcher off).

---

## 3. System / code hot-reload (HMR) status

**`dexterous_developer` is NOT Bevy-0.18-ready.** As of May 2026 the latest
`bevy_dexterous_developer` is `0.4.0-alpha.3`, targeting Bevy 0.14. Integrating
it would require either downgrading Bevy (unacceptable) or forking the crate
across four major Bevy versions (0.14 -> 0.18), which is out of scope and high
risk. **Blocker: upstream crate has not tracked Bevy past 0.14.**

**Delivered fallback (the supported subsecond loop):**

- **`hot` feature** = `dev` + `bevy/dynamic_linking`. Bevy is linked as a shared
  library, so editing one of *our* systems only relinks our small crate, not the
  whole engine — the dominant warm-rebuild cost disappears.
- **`cargo watch`** drives the rebuild-on-save loop (`just dev-fast` / `just dev-fast-voxel`).
- Net effect: save a Rust system -> cargo-watch triggers an incremental rebuild
  that relinks only our crate -> the app relaunches. Asset edits skip the
  rebuild entirely (section 2).

`bevy_dylib 0.18.1` is published and matches our pinned `bevy 0.18.1`, so
`dynamic_linking` resolves cleanly (the earlier 0.18.0-only gap, bevy issue
#22654, is fixed in 0.18.1).

When `dexterous_developer` ships Bevy 0.18 support, wire it behind the existing
`hot` feature (replace the cargo-watch relaunch with in-process system reload).

---

## 4. The loop, end to end

```
# fastest iteration (watch + asset hot-reload + dynamic_linking)
just dev-fast            # standalone sandbox
just dev-fast-voxel      # live windowed/voxel client

# one-shot launches (incremental, no watcher)
just run
just run-voxel
```

`just dev-fast` first runs `just dev-tools`, which installs `cargo-watch` if missing
(idempotent). Taskfile equivalents: `task dev:fast`, `task dev:fast-voxel`, `task run`,
`task run:voxel`.

---

## Measurement log

Reproduce on this machine:

```powershell
# cold
cargo clean -p civ-bevy-ref
Measure-Command { cargo build -p civ-bevy-ref --features bevy,egui --bin civ-standalone }

# warm (no-op)
Measure-Command { cargo build -p civ-bevy-ref --features bevy,egui --bin civ-standalone }

# incremental: touch one of our source files, rebuild
(Get-Item clients/bevy-ref/src/bin/standalone.rs).LastWriteTime = Get-Date
Measure-Command { cargo build -p civ-bevy-ref --features bevy,egui --bin civ-standalone }
```

Raw numbers captured during setup are in section 1's table.
