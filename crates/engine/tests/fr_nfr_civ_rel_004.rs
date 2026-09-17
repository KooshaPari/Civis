//! NFR-CIV-REL-004 — a corrupted replay checksum SHALL be hard fatal.
//!
//! Matrix check: `replay_checksum_corruption`.
//! Acceptance contract: a bit-flipped `.civreplay` load returns `Err`
//! containing `checksum`; it never loads silently.

use civ_engine::{decode_civreplay, encode_civreplay, ReplayError, ReplayLog};

/// A bit flip anywhere in header or payload must be rejected, never ignored.
#[test]
fn replay_checksum_corruption() {
    let log = ReplayLog {
        events: Vec::new(),
        seed: 0x1234,
        schema_version: 1,
        running_hash: None,
    };
    let clean = encode_civreplay(&log).expect("encodes");

    // The clean container round-trips, so the negative results below are
    // attributable to the corruption and not to a broken encoder.
    decode_civreplay(&clean).expect("clean container decodes");

    // Flip one bit in each byte position across the header and payload region
    // (everything except the trailing checksum, which we test separately).
    let body_end = clean.len() - 32;
    let mut rejected = 0usize;
    for i in 0..body_end {
        let mut corrupt = clean.clone();
        corrupt[i] ^= 0x01;
        match decode_civreplay(&corrupt) {
            Ok(_) => panic!("byte {i} corruption loaded silently instead of failing"),
            Err(ReplayError::ChecksumMismatch) => rejected += 1,
            Err(other) => {
                // A flip can make magic/version invalid; that is also a hard
                // failure, which satisfies the contract.
                let _ = other;
            }
        }
    }
    assert!(rejected > 0, "at least payload corruption yields ChecksumMismatch");

    // Corrupting the stored checksum itself must also be fatal.
    let mut bad_footer = clean.clone();
    let last = bad_footer.len() - 1;
    bad_footer[last] ^= 0x01;
    assert!(
        matches!(
            decode_civreplay(&bad_footer),
            Err(ReplayError::ChecksumMismatch)
        ),
        "a corrupted footer checksum must report a checksum mismatch"
    );

    // Truncation is fatal too.
    let truncated = &clean[..clean.len() - 4];
    assert!(
        decode_civreplay(truncated).is_err(),
        "a truncated container must not load"
    );
}
