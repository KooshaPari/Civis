# mod-host

> CivLab mod manifest loader and WASM tick-hook host for the Civis mod runtime.

## Overview

The civ-mod-host crate is the sandboxed runtime that loads, validates, and executes Civis mods. Mods are distributed as signed WASM bundles. The host verifies Ed25519 signatures, parses TOML manifests, and invokes mod tick hooks through Wasmtime with deterministic-guarantee features.

## Features

- Ed25519 signature verification for mod authenticity
- TOML manifest parsing with schema validation
- Wasmtime-based WASM execution sandbox
- ZIP bundle extraction for mod packages
- Tick-hook dispatch with deterministic execution guarantees
- civ-mod-sign CLI tool for signing mod bundles

## Usage

```rust
use civ_mod_host::{ModHost, ModManifest};

let host = ModHost::new();
let manifest = ModManifest::load("my-mod.toml")?;
host.tick_hook(&manifest)?;
```

## Architecture

Mod bundles are ZIP archives containing a WASM binary, a TOML manifest, and an Ed25519 signature. The host extracts the bundle, verifies the signature, loads the manifest, and instantiates a Wasmtime engine. Tick hooks are called each simulation step in a deterministic sandbox.

## License

Part of the Civis project.
