#![no_main]

use libfuzzer_sys::fuzz_target;

use civ_engine::replay_format::decode_civreplay;

fuzz_target!(|data: &[u8]| {
    // Fuzz the .civreplay binary container decoder.
    // We expect ReplayError variants for malformed input, but no panics.
    let _ = decode_civreplay(data);
});
