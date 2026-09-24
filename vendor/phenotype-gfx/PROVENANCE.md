# Provenance: vendored `phenotype-gfx` / `phenotype-voxel`

## Why this is here

`civ-voxel` depended on a pinned git revision:

```toml
phenotype-voxel = { git = "https://github.com/KooshaPari/phenotype-gfx.git",
                    rev = "7ed27211554f5ba25d3a79c54c73262db6e160b1",
                    package = "phenotype-voxel" }
```

As of 2026-09-24 that remote no longer resolves:

```
$ git ls-remote https://github.com/KooshaPari/phenotype-gfx.git
remote: Repository not found.
fatal: repository 'https://github.com/KooshaPari/phenotype-gfx.git/' not found
```

Because `crates/voxel` is a workspace member, this broke `cargo metadata` for the
**entire workspace**, which in turn reddened every cargo-based CI workflow
(`compile-gate`, `cargo-deny`, `Security Guard (Hooks)`, `Mutation`,
`Playthrough Validate`). It had been failing for 8+ consecutive runs before this
fix. Local builds kept working only because `~/.cargo/git/` still held a stale
checkout; a clean clone and CI could not build the project at all.

## What was vendored

| Field | Value |
|-------|-------|
| Upstream repo | `https://github.com/KooshaPari/phenotype-gfx.git` |
| Revision | `7ed27211554f5ba25d3a79c54c73262db6e160b1` (`7ed2721`) |
| Source of copy | `~/.cargo/git/checkouts/phenotype-gfx-8c434bb1d9734182/7ed2721/` |
| Date copied | 2026-09-24 |
| Packages | `phenotype-gfx` v0.2.0 (kernel), `phenotype-voxel` v0.2.0 (re-export shim) |
| Size | 242 files, ~1.3 MB |
| License | MIT OR Apache-2.0 (unchanged from upstream) |

The revision is byte-identical to what the old pin resolved to, so this is a
**relocation, not an upgrade**. No source was modified.

## How it is wired

- `crates/voxel/Cargo.toml` uses a path dependency:
  `phenotype-voxel = { path = "../../vendor/phenotype-gfx/crates/phenotype-voxel", package = "phenotype-voxel" }`
- The root `Cargo.toml` lists `vendor/phenotype-gfx` under `[workspace] exclude`,
  because the vendored tree is its own workspace (`members = [".", "crates/phenotype-voxel"]`).

## Removing this

If a reachable upstream is restored (or the kernel is published to a registry),
switch `crates/voxel/Cargo.toml` back to a git or registry dependency, drop the
`vendor/phenotype-gfx` entry from the workspace `exclude` list, delete this
directory, and refresh `Cargo.lock`.
