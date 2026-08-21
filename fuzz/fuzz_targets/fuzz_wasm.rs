#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use civ_mod_host::{invoke_policy_tick, invoke_economy_tick, invoke_military_tick};
use arbitrary::Unstructured;

#[derive(Debug, Arbitrary)]
enum FuzzTarget {
    Policy,
    Economy,
    Military,
}

fuzz_target!(|data: &[u8]| {
    // Fuzz the WASM guest loaders with arbitrary byte streams.
    // We expect wasmtime errors for malformed modules, but no panics.
    if data.is_empty() { return; }
    
    let mut u = Unstructured::new(data);
    let target = FuzzTarget::arbitrary(&mut u).unwrap_or(FuzzTarget::Policy);
    
    let mut guest_memory = Vec::new();
    
    // Use a fixed sim_tick for determinism in fuzzing
    let sim_tick = 0;
    
    let _ = match target {
        FuzzTarget::Policy => invoke_policy_tick(data, sim_tick, &mut guest_memory),
        FuzzTarget::Economy => invoke_economy_tick(data, sim_tick, &mut guest_memory),
        FuzzTarget::Military => invoke_military_tick(data, sim_tick, &mut guest_memory),
    };
});
