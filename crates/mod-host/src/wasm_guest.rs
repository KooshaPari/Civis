//! WASM guest invocation via `wasmtime` (CIV-0700 v3 — policy, economy, military exports).

use thiserror::Error;
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store};

/// WASM module filename inside mod directories and `.civmod` archives.
pub const MOD_WASM_NAME: &str = "mod.wasm";

/// Host import namespace for capability stubs (FR-CIV-TACTICS-047).
pub const HOST_IMPORT_MODULE: &str = "civlab";

/// Host imports exposed to guests (FR-CIV-TACTICS-053).
pub const HOST_CAPABILITY_IMPORTS: &[&str] = &[
    "capability_api_version",
    "sim_tick",
    "memory_size",
    "memory_read",
    "memory_write",
];

/// Packed capability API major version returned by host import `capability_api_version`.
pub const HOST_CAPABILITY_API_VERSION: i32 = 1;

/// Maximum guest scratch bytes exposed via host memory imports (FR-CIV-TACTICS-049).
pub const HOST_GUEST_MEMORY_CAP: usize = 65_536;

/// Per-instance host state for capability imports.
#[derive(Debug, Default)]
pub struct HostState {
    guest_memory: Vec<u8>,
    sim_tick: u64,
}

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

fn trim_guest_memory(mem: &mut Vec<u8>) {
    if mem.len() > HOST_GUEST_MEMORY_CAP {
        mem.truncate(HOST_GUEST_MEMORY_CAP);
    }
}

fn link_host_imports(linker: &mut Linker<HostState>) -> Result<(), wasmtime::Error> {
    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "capability_api_version",
        || -> Result<i32, wasmtime::Error> { Ok(HOST_CAPABILITY_API_VERSION) },
    )?;

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "sim_tick",
        |caller: Caller<'_, HostState>| -> Result<i64, wasmtime::Error> {
            Ok(i64::try_from(caller.data().sim_tick).unwrap_or(i64::MAX))
        },
    )?;

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "memory_size",
        |caller: Caller<'_, HostState>| -> Result<i32, wasmtime::Error> {
            Ok(i32::try_from(caller.data().guest_memory.len()).unwrap_or(i32::MAX))
        },
    )?;

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "memory_read",
        |caller: Caller<'_, HostState>, offset: i32| -> Result<i32, wasmtime::Error> {
            let o = offset.max(0) as usize;
            Ok(caller.data().guest_memory.get(o).copied().unwrap_or(0) as i32)
        },
    )?;

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "memory_write",
        |mut caller: Caller<'_, HostState>,
         offset: i32,
         value: i32|
         -> Result<(), wasmtime::Error> {
            let o = offset.max(0) as usize;
            if o >= HOST_GUEST_MEMORY_CAP {
                return Ok(());
            }
            let mem = &mut caller.data_mut().guest_memory;
            if o >= mem.len() {
                mem.resize((o + 1).min(HOST_GUEST_MEMORY_CAP), 0);
            }
            if o < mem.len() {
                mem[o] = (value & 0xFF) as u8;
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn with_guest_instance<R>(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    invoke: impl FnOnce(Instance, &mut Store<HostState>) -> Result<R, WasmGuestError>,
) -> Result<R, WasmGuestError> {
    trim_guest_memory(guest_memory);
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes).map_err(WasmGuestError::Engine)?;
    let mut linker = Linker::new(&engine);
    link_host_imports(&mut linker).map_err(WasmGuestError::Engine)?;
    let mut store = Store::new(
        &engine,
        HostState {
            guest_memory: guest_memory.clone(),
            sim_tick,
        },
    );
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(WasmGuestError::Engine)?;
    let result = invoke(instance, &mut store)?;
    *guest_memory = store.data().guest_memory.clone();
    trim_guest_memory(guest_memory);
    Ok(result)
}

fn invoke_tick_export(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    primary: &str,
    fallback: &str,
) -> Result<i32, WasmGuestError> {
    with_guest_instance(wasm_bytes, sim_tick, guest_memory, |instance, store| {
        if let Ok(func) = instance.get_typed_func::<(), i32>(&mut *store, primary) {
            return func.call(&mut *store, ()).map_err(WasmGuestError::Engine);
        }
        if let Ok(func) = instance.get_typed_func::<(), i32>(&mut *store, fallback) {
            return func.call(&mut *store, ()).map_err(WasmGuestError::Engine);
        }
        Err(WasmGuestError::MissingExport {
            export: primary.to_owned(),
        })
    })
}

fn invoke_tick_with_sim_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    primary: &str,
    fallback: &str,
) -> Result<i32, WasmGuestError> {
    let tick_arg = i64::try_from(sim_tick).unwrap_or(i64::MAX);
    let result = with_guest_instance(wasm_bytes, sim_tick, guest_memory, |instance, store| {
        if let Ok(func) = instance.get_typed_func::<i64, i32>(&mut *store, primary) {
            return func
                .call(&mut *store, tick_arg)
                .map_err(WasmGuestError::Engine);
        }
        if let Ok(func) = instance.get_typed_func::<i64, i32>(&mut *store, fallback) {
            return func
                .call(&mut *store, tick_arg)
                .map_err(WasmGuestError::Engine);
        }
        Err(WasmGuestError::MissingExport {
            export: primary.to_owned(),
        })
    });
    match result {
        Ok(code) => Ok(code),
        Err(WasmGuestError::MissingExport { .. }) => {
            invoke_tick_export(wasm_bytes, sim_tick, guest_memory, primary, fallback)
        }
        Err(err) => Err(err),
    }
}

/// Invoke the policy-phase export from a WASM guest (`civlab_policy_tick`, else `policy_tick`).
pub fn invoke_policy_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(wasm_bytes, sim_tick, guest_memory, "civlab_policy_tick", "policy_tick")
}

/// Invoke the economy-phase export (`civlab_economy_tick`, else `economy_tick`).
pub fn invoke_economy_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(
        wasm_bytes,
        sim_tick,
        guest_memory,
        "civlab_economy_tick",
        "economy_tick",
    )
}

/// Invoke the military-phase export (`civlab_military_tick`, else `military_tick`).
pub fn invoke_military_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(
        wasm_bytes,
        sim_tick,
        guest_memory,
        "civlab_military_tick",
        "military_tick",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_memory_import_read_write() {
        const WAT: &str = r#"
            (module
              (import "civlab" "memory_write" (func $write (param i32 i32)))
              (import "civlab" "memory_read" (func $read (param i32) (result i32)))
              (func (export "civlab_economy_tick") (param i64) (result i32)
                (i32.const 7)
                (i32.const 42)
                (call $write)
                (i32.const 7)
                (call $read))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(invoke_economy_tick(&wasm, 0, &mut mem).expect("invoke"), 42);
        assert_eq!(mem.get(7).copied(), Some(42));
    }

    #[test]
    fn guest_memory_persists_across_invocations() {
        const WAT: &str = r#"
            (module
              (import "civlab" "memory_read" (func $read (param i32) (result i32)))
              (import "civlab" "memory_write" (func $write (param i32 i32)))
              (func (export "civlab_economy_tick") (param i64) (result i32)
                (i32.const 0)
                (call $read)
                (if (result i32)
                  (i32.eqz)
                  (then
                    (i32.const 0)
                    (i32.const 99)
                    (call $write)
                    (i32.const 99))
                  (else
                    (i32.const 0)
                    (call $read))))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(invoke_economy_tick(&wasm, 1, &mut mem).expect("first"), 99);
        assert_eq!(invoke_economy_tick(&wasm, 2, &mut mem).expect("second"), 99);
    }

    #[test]
    fn sim_tick_host_import_visible_to_guest() {
        const WAT: &str = r#"
            (module
              (import "civlab" "sim_tick" (func $tick (result i64)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (call $tick)
                i32.wrap_i64)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(
            invoke_policy_tick(&wasm, 17, &mut mem).expect("invoke"),
            17
        );
    }
}
