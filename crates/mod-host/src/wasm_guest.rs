//! WASM guest invocation via `wasmtime` (CIV-0700 v3 — policy, economy, military exports).

use crate::capability::{ModCapabilitySet, ModEnforcementCtx, WorldDomain, ERR_PERMISSION_DENIED};
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
    "world_read",
    "action_emit",
];

/// Packed capability API major version returned by host import `capability_api_version`.
pub const HOST_CAPABILITY_API_VERSION: i32 = 1;

/// Maximum guest scratch bytes exposed via host memory imports (FR-CIV-TACTICS-049).
pub const HOST_GUEST_MEMORY_CAP: usize = 65_536;

/// Per-instance host state for capability imports.
#[derive(Debug)]
pub struct HostState {
    guest_memory: Vec<u8>,
    sim_tick: u64,
    capabilities: ModCapabilitySet,
    enforcement: ModEnforcementCtx,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            guest_memory: Vec::new(),
            sim_tick: 0,
            capabilities: ModCapabilitySet::allow_all(),
            enforcement: ModEnforcementCtx::default(),
        }
    }
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

fn record_permission_denial(state: &mut HostState, call: &str, domain: Option<WorldDomain>) {
    state.enforcement.record_denial(call, domain);
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

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "world_read",
        |mut caller: Caller<'_, HostState>, domain: i32| -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if state.enforcement.suspended {
                record_permission_denial(state, "world_read", None);
                return Ok(ERR_PERMISSION_DENIED);
            }
            let Some(domain) = WorldDomain::from_i32(domain) else {
                record_permission_denial(state, "world_read", None);
                return Ok(ERR_PERMISSION_DENIED);
            };
            if state.capabilities.can_read_domain(domain) {
                Ok(1)
            } else {
                record_permission_denial(state, "world_read", Some(domain));
                Ok(ERR_PERMISSION_DENIED)
            }
        },
    )?;

    linker.func_wrap(
        HOST_IMPORT_MODULE,
        "action_emit",
        |mut caller: Caller<'_, HostState>,
         action_type: i64,
         _payload_ptr: i32,
         _payload_len: i32|
         -> Result<i32, wasmtime::Error> {
            let state = caller.data_mut();
            if state.enforcement.suspended {
                record_permission_denial(state, "action_emit", None);
                return Ok(ERR_PERMISSION_DENIED);
            }
            let action_type = u32::try_from(action_type).unwrap_or(u32::MAX);
            if state.capabilities.can_emit_action(action_type) {
                Ok(0)
            } else {
                record_permission_denial(state, "action_emit", None);
                Ok(ERR_PERMISSION_DENIED)
            }
        },
    )?;
    Ok(())
}

fn with_guest_instance<R>(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
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
            capabilities,
            enforcement: ModEnforcementCtx {
                violations: enforcement.violations,
                suspended: enforcement.suspended,
                last_call: enforcement.last_call.clone(),
                last_domain: enforcement.last_domain,
            },
        },
    );
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(WasmGuestError::Engine)?;
    let result = invoke(instance, &mut store)?;
    *guest_memory = store.data().guest_memory.clone();
    enforcement.violations = store.data().enforcement.violations;
    enforcement.suspended = store.data().enforcement.suspended;
    enforcement.last_call = store.data().enforcement.last_call.clone();
    enforcement.last_domain = store.data().enforcement.last_domain;
    trim_guest_memory(guest_memory);
    Ok(result)
}

fn invoke_tick_export(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
    primary: &str,
    fallback: &str,
) -> Result<i32, WasmGuestError> {
    with_guest_instance(
        wasm_bytes,
        sim_tick,
        guest_memory,
        capabilities,
        enforcement,
        |instance, store| {
            if let Ok(func) = instance.get_typed_func::<(), i32>(&mut *store, primary) {
                return func.call(&mut *store, ()).map_err(WasmGuestError::Engine);
            }
            if let Ok(func) = instance.get_typed_func::<(), i32>(&mut *store, fallback) {
                return func.call(&mut *store, ()).map_err(WasmGuestError::Engine);
            }
            Err(WasmGuestError::MissingExport {
                export: primary.to_owned(),
            })
        },
    )
}

fn invoke_tick_with_sim_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
    primary: &str,
    fallback: &str,
) -> Result<i32, WasmGuestError> {
    let tick_arg = i64::try_from(sim_tick).unwrap_or(i64::MAX);
    let result = with_guest_instance(
        wasm_bytes,
        sim_tick,
        guest_memory,
        capabilities.clone(),
        enforcement,
        |instance, store| {
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
        },
    );
    match result {
        Ok(code) => Ok(code),
        Err(WasmGuestError::MissingExport { .. }) => invoke_tick_export(
            wasm_bytes,
            sim_tick,
            guest_memory,
            capabilities,
            enforcement,
            primary,
            fallback,
        ),
        Err(err) => Err(err),
    }
}

/// Invoke the policy-phase export from a WASM guest (`civlab_policy_tick`, else `policy_tick`).
pub fn invoke_policy_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
) -> Result<i32, WasmGuestError> {
    invoke_policy_tick_with_capabilities(
        wasm_bytes,
        sim_tick,
        guest_memory,
        ModCapabilitySet::allow_all(),
        &mut ModEnforcementCtx::default(),
    )
}

/// Invoke policy tick with manifest-derived capabilities and violation tracking.
pub fn invoke_policy_tick_with_capabilities(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(
        wasm_bytes,
        sim_tick,
        guest_memory,
        capabilities,
        enforcement,
        "civlab_policy_tick",
        "policy_tick",
    )
}

/// Invoke the economy-phase export (`civlab_economy_tick`, else `economy_tick`).
pub fn invoke_economy_tick(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
) -> Result<i32, WasmGuestError> {
    invoke_economy_tick_with_capabilities(
        wasm_bytes,
        sim_tick,
        guest_memory,
        ModCapabilitySet::allow_all(),
        &mut ModEnforcementCtx::default(),
    )
}

/// Invoke economy tick with manifest-derived capabilities and violation tracking.
pub fn invoke_economy_tick_with_capabilities(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(
        wasm_bytes,
        sim_tick,
        guest_memory,
        capabilities,
        enforcement,
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
    invoke_military_tick_with_capabilities(
        wasm_bytes,
        sim_tick,
        guest_memory,
        ModCapabilitySet::allow_all(),
        &mut ModEnforcementCtx::default(),
    )
}

/// Invoke military tick with manifest-derived capabilities and violation tracking.
pub fn invoke_military_tick_with_capabilities(
    wasm_bytes: &[u8],
    sim_tick: u64,
    guest_memory: &mut Vec<u8>,
    capabilities: ModCapabilitySet,
    enforcement: &mut ModEnforcementCtx,
) -> Result<i32, WasmGuestError> {
    invoke_tick_with_sim_tick(
        wasm_bytes,
        sim_tick,
        guest_memory,
        capabilities,
        enforcement,
        "civlab_military_tick",
        "military_tick",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::ERR_PERMISSION_DENIED;

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
        assert_eq!(invoke_policy_tick(&wasm, 17, &mut mem).expect("invoke"), 17);
    }

    #[test]
    fn denied_world_read_returns_permission_denied() {
        const WAT: &str = r#"
            (module
              (import "civlab" "world_read" (func $read (param i32) (result i32)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (i32.const 2)
                (call $read))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        let caps = ModCapabilitySet::from_permissions(&crate::ModPermissions::default());
        let mut enforcement = ModEnforcementCtx::default();
        assert_eq!(
            invoke_policy_tick_with_capabilities(&wasm, 0, &mut mem, caps, &mut enforcement)
                .expect("invoke"),
            ERR_PERMISSION_DENIED
        );
        assert_eq!(enforcement.violations, 1);
    }

    #[test]
    fn allowed_write_policy_permits_action_emit_type_one() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func $emit (param i64 i32 i32) (result i32)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (i64.const 1)
                (i32.const 0)
                (i32.const 0)
                (call $emit))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        let caps = ModCapabilitySet::from_permissions(&crate::ModPermissions {
            write_policy: true,
            ..crate::ModPermissions::default()
        });
        let mut enforcement = ModEnforcementCtx::default();
        assert_eq!(
            invoke_policy_tick_with_capabilities(&wasm, 0, &mut mem, caps, &mut enforcement)
                .expect("invoke"),
            0
        );
        assert_eq!(enforcement.violations, 0);
    }

    #[test]
    fn denied_action_emit_returns_permission_denied() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func $emit (param i64 i32 i32) (result i32)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (i64.const 5)
                (i32.const 0)
                (i32.const 0)
                (call $emit))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        let caps = ModCapabilitySet::from_permissions(&crate::ModPermissions {
            write_policy: true,
            ..crate::ModPermissions::default()
        });
        let mut enforcement = ModEnforcementCtx::default();
        assert_eq!(
            invoke_policy_tick_with_capabilities(&wasm, 0, &mut mem, caps, &mut enforcement)
                .expect("invoke"),
            ERR_PERMISSION_DENIED
        );
        assert_eq!(enforcement.violations, 1);
        assert_eq!(enforcement.last_call.as_deref(), Some("action_emit"));
    }

    #[test]
    fn allowed_world_read_returns_one() {
        const WAT: &str = r#"
            (module
              (import "civlab" "world_read" (func $read (param i32) (result i32)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (i32.const 0)
                (call $read))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        let caps = ModCapabilitySet::from_permissions(&crate::ModPermissions {
            read_economy: true,
            ..crate::ModPermissions::default()
        });
        let mut enforcement = ModEnforcementCtx::default();
        assert_eq!(
            invoke_policy_tick_with_capabilities(&wasm, 0, &mut mem, caps, &mut enforcement)
                .expect("invoke"),
            1
        );
    }
}
