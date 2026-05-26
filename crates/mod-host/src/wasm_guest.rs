//! WASM guest invocation via `wasmtime` (CIV-0700 v3 — policy + military tick exports).

use thiserror::Error;

/// WASM module filename inside mod directories and `.civmod` archives.
pub const MOD_WASM_NAME: &str = "mod.wasm";

/// Errors from guest instantiation or export calls.
#[derive(Debug, Error)]
pub enum WasmGuestError {
    /// Engine or module compilation failure.
    #[error("wasm engine: {0}")]
    Engine(#[from] wasmtime::Error),
    /// Required export missing from the guest module.
    #[error("missing export {export}")]
    MissingExport {
        /// Export name the host expected.
        export: String,
    },
}

fn invoke_tick_export(
    wasm_bytes: &[u8],
    primary: &str,
    fallback: &str,
) -> Result<i32, WasmGuestError> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wasm_bytes)?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

    if let Ok(func) = instance.get_typed_func::<(), i32>(&mut store, primary) {
        return Ok(func.call(&mut store, ())?);
    }
    if let Ok(func) = instance.get_typed_func::<(), i32>(&mut store, fallback) {
        return Ok(func.call(&mut store, ())?);
    }
    Err(WasmGuestError::MissingExport {
        export: primary.to_owned(),
    })
}

/// Invoke the policy-phase export from a WASM guest (`civlab_policy_tick`, else `policy_tick`).
///
/// Returns the i32 the guest exported (convention: `0` = no-op success).
pub fn invoke_policy_tick(wasm_bytes: &[u8]) -> Result<i32, WasmGuestError> {
    invoke_tick_export(wasm_bytes, "civlab_policy_tick", "policy_tick")
}

/// Invoke the military-phase export (`civlab_military_tick`, else `military_tick`).
///
/// Prefers `(i64) -> i32` exports that receive the simulation tick (FR-CIV-TACTICS-040);
/// falls back to legacy zero-arg exports when absent.
pub fn invoke_military_tick(wasm_bytes: &[u8], sim_tick: u64) -> Result<i32, WasmGuestError> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wasm_bytes)?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[])?;
    let tick_arg = i64::try_from(sim_tick).unwrap_or(i64::MAX);

    if let Ok(func) = instance.get_typed_func::<i64, i32>(&mut store, "civlab_military_tick") {
        return Ok(func.call(&mut store, tick_arg)?);
    }
    if let Ok(func) = instance.get_typed_func::<i64, i32>(&mut store, "military_tick") {
        return Ok(func.call(&mut store, tick_arg)?);
    }
    invoke_tick_export(wasm_bytes, "civlab_military_tick", "military_tick")
}
