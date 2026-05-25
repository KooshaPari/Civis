//! WASM guest invocation via `wasmtime` (CIV-0700 v3 — policy tick export).

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
    #[error("missing export civlab_policy_tick (or policy_tick)")]
    MissingExport,
}

/// Invoke the policy-phase export from a WASM guest (`civlab_policy_tick`, else `policy_tick`).
///
/// Returns the i32 the guest exported (convention: `0` = no-op success).
pub fn invoke_policy_tick(wasm_bytes: &[u8]) -> Result<i32, WasmGuestError> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wasm_bytes)?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

    if let Ok(func) = instance.get_typed_func::<(), i32>(&mut store, "civlab_policy_tick") {
        return Ok(func.call(&mut store, ())?);
    }
    if let Ok(func) = instance.get_typed_func::<(), i32>(&mut store, "policy_tick") {
        return Ok(func.call(&mut store, ())?);
    }
    Err(WasmGuestError::MissingExport)
}
