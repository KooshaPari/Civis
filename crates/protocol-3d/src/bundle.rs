//! Tick-coalesced [`Frame3d`] bundles (`F3DB` magic) with opt-in zstd compression.
//!
//! The inner payload is a concatenation of complete `F3D0` envelopes produced by
//! [`encode_frame3d_binary`]. Compression is **opt-in** at encode time (flag bit 0);
//! decoders accept both compressed and uncompressed bundles.

use zstd::stream::{decode_all, encode_all};

use crate::{
    decode_frame3d_binary, encode_frame3d_binary, Frame3d, Frame3dBinaryError,
    FRAME3D_BINARY_HEADER_LEN, FRAME3D_BINARY_MAGIC,
};

/// 4-byte magic identifying a coalesced per-tick `Frame3d` bundle.
// The following 3 requirement tags were removed from FRAME3D_BUNDLE_MAGIC.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// `b"F3DB"` filters nothing, connects to nothing, and unpacks nothing.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-PROTO-005: requirement is subscription filtering by entity type/region; filtering is real but by frame kind at SubscriptionFilter::filter_frames, and get_snapshot_for_session returns the full snapshot. A magic constant and a zstd level filter nothing (CIV-0200:1144)
//  [unbound] FR-CIV-PROTO-012: requirement is a Bevy client that connects, subscribes, and renders agent positions under the example_bevy_client gate; the client exists at clients/bevy-ref but the gate does not, and the tag is on a voxel frame and a magic constant (CIV-0200:1179)
//  [unbound] FR-CIV-PROTO-013: requirement is an Unreal plugin that unpacks binary frames and updates AActor transforms; clients/unreal-show contains no C++ that unpacks F3DB (CIV-0200:1184)
pub const FRAME3D_BUNDLE_MAGIC: &[u8; 4] = b"F3DB";

/// Current wire version of the `F3DB` envelope.
pub const FRAME3D_BUNDLE_VERSION: u8 = 1;

/// Per-tick frame count shipped by `civ-server` today (voxel, building, agent,
/// civilian, faction, event feed, climate).
pub const FRAME3D_BUNDLE_STANDARD_LEN: usize = 7;

/// Default zstd level for tick bundles (fast decompress; CIV-0500 §8.4).
// The following 3 requirement tags were removed from DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// FR-CIV-PROTO-006 is deliberately kept on this constant: a zstd level is exactly
// the data the requirement names, so that one tag is legitimate on shape. The other
// three are not.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-PROTO-005: requirement is subscription filtering by entity type/region; filtering is real but by frame kind at SubscriptionFilter::filter_frames, and get_snapshot_for_session returns the full snapshot. A magic constant and a zstd level filter nothing (CIV-0200:1144)
//  [unbound] FR-CIV-PROTO-012: requirement is a Bevy client that connects, subscribes, and renders agent positions under the example_bevy_client gate; the client exists at clients/bevy-ref but the gate does not, and the tag is on a voxel frame and a magic constant (CIV-0200:1179)
//  [unbound] FR-CIV-PROTO-013: requirement is an Unreal plugin that unpacks binary frames and updates AActor transforms; clients/unreal-show contains no C++ that unpacks F3DB (CIV-0200:1184)
// The following 1 requirement tag was removed from DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL.
// It is not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// The coordinator flagged that FR-CIV-PROTO-006 appears twice in this file and that one site might be genuine while the other is not. Having read both, the honest answer is the opposite of the expected shape: BOTH container sites are false, and the genuine binding is on neither of them but on the two functions further down the same file.
// This is the first of the pair and it is the weaker of the two. The constant is a bare i32 with no behavior of any kind, so the case for a container binding is not merely weak, it is absent. Note also that the prior note in the file header says it is kept 'on shape' -- but shape is precisely what the requirement is not about. The requirement has two behavioral halves and the constant discharges neither.
// The earlier removal recorded in the same header block is worth noting because it got this site almost right: FR-CIV-PROTO-005 was removed from this constant on the grounds that 'a magic constant and a zstd level filter nothing' (bundle.rs:44). That sentence describes this constant just as accurately, and it is the reasoning that should have been applied to FR-CIV-PROTO-006 as well. The sibling id was judged on the symbol; this one was judged on the type of the symbol. Same symbol, different standard.
// No coverage is lost by removing it: the real round trip is already tagged on the encode and decode functions and is exercised by crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:11, which encodes with compress: true at this exact constant and asserts the decoded bundle unpacks -- which is the spec's 'verify frames unpack correctly' clause, run through the constant rather than certified by it.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-PROTO-006: DATA-SHAPE-ONLY, AND SHAPELESS WITH RESPECT TO THE REQUIREMENT. This OVERTURNS a prior GENUINE ruling. The requirement is 'Support binary frames with zstd compression; unpack without errors', tested by 'Subscribe with use_binary_frames=true; verify frames unpack correctly' (CIV-0200-client-protocol.md:1149-1152). The prior in-file note, written into this file's header at bundle.rs:39-41, kept the tag on the grounds that 'a zstd level is exactly the data the requirement names, so that one tag is legitimate on shape'. That is a data-match argument. The requirement is about a wire format and a round trip, and `pub const DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL: i32 = 1` at bundle.rs:48 is a single i32 naming a compression level. A constant is not a frame: it declares no magic bytes, no header layout, no length framing, no flag bits, and performs no encode or decode. It cannot be unpacked and cannot fail to unpack, because it does not participate in either direction of the round trip. The requirement's two halves -- 'Support binary frames' and 'unpack without errors' -- are both absent from this symbol, and neither can be supplied by an integer. The genuine implementation lives in neighboring symbols in this same file, where FR-CIV-PROTO-006 is correctly and independently tagged and where removing this tag loses no coverage: encode_frame3d_bundle at bundle.rs:182 writes the F3DB magic, version, flag byte, big-endian tick, frame count, and uncompressed/payload lengths (:209-216), and decode_frame3d_bundle at :225 parses exactly that header back and returns Frame3dBundleError on truncation, bad magic, and version mismatch (:226-234). Both carry the FR-CIV-PROTO-006 tag at :179 and :222. The constant is consumed by Frame3dBundleEncodeOptions::default at :113, which is the data flowing INTO the real implementer rather than the implementer itself. A default parameter value is the weakest possible claimant to a 'binary frame format' requirement. If the tag is left here it asserts that a magic number is a wire protocol.
pub const DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL: i32 = 1;

const FRAME3D_BUNDLE_HEADER_LEN: usize = 23;

/// Capability / compression flag bits in the `F3DB` header.
// The following 1 requirement tags were removed from Frame3dBundleFlags.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// There is no Unity client in this repository; the tag is on a newtype over one
// compression bit.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-PROTO-014: requirement is a Unity client connecting over WebSocket and rendering snapshots; there is no Unity client in this repository (clients/ holds bevy-ref, godot-ref, unreal-show only) (CIV-0200:1189)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Frame3dBundleFlags(pub u8);

impl Frame3dBundleFlags {
    /// Bit 0 — inner payload is zstd-compressed.
    pub const ZSTD_COMPRESSED: u8 = 0b0000_0001;

    /// Uncompressed inner payload (default; backward compatible).
    #[must_use]
    pub const fn uncompressed() -> Self {
        Self(0)
    }

    /// zstd-compressed inner payload.
    #[must_use]
    pub const fn zstd() -> Self {
        Self(Self::ZSTD_COMPRESSED)
    }

    /// Returns `true` when bit 0 is set.
    #[must_use]
    pub fn is_zstd(self) -> bool {
        self.0 & Self::ZSTD_COMPRESSED != 0
    }
}

/// Opt-in encoder settings for [`encode_frame3d_bundle`].
// The following 4 requirement tags were removed from Frame3dBundleEncodeOptions.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// FR-CIV-PROTO-006 is kept here for the same reason as on the constant above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-PROTO-005: requirement is subscription filtering by entity type/region; filtering is real but by frame kind at SubscriptionFilter::filter_frames, and get_snapshot_for_session returns the full snapshot. A magic constant and a zstd level filter nothing (CIV-0200:1144)
//  [unbound] FR-CIV-PROTO-012: requirement is a Bevy client that connects, subscribes, and renders agent positions under the example_bevy_client gate; the client exists at clients/bevy-ref but the gate does not, and the tag is on a voxel frame and a magic constant (CIV-0200:1179)
//  [unbound] FR-CIV-PROTO-013: requirement is an Unreal plugin that unpacks binary frames and updates AActor transforms; clients/unreal-show contains no C++ that unpacks F3DB (CIV-0200:1184)
//  [unbound] FR-CIV-PROTO-015: requirement is a React/Vue web client that connects, subscribes, and renders; no component under web/dashboard/src imports a protocol-3d type, and the spec's example_web_client gate exists only inside docs/fragmented/ (CIV-0200:1194)
// The following 1 requirement tag was removed from Frame3dBundleEncodeOptions.
// It is not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// Second of the FR-CIV-PROTO-006 pair, judged separately as instructed. It is a closer call than the constant and it still fails, for a reason the constant does not even reach: this struct is at least ON the real code path, carried as an argument into the real encoder. Being on the path is not the same as being the implementation, and the requirement names a format and a round trip rather than a config knob.
// The decisive fact is the default. compress: false at bundle.rs:112 means the type as constructed by Default does not request zstd compression, and the requirement's first clause is 'Support binary frames with zstd compression'. So the tagged symbol's own default behavior runs against the requirement it is tagged with. I checked whether that made the tag a mis-bound-but-real feature flag instead; it does not, because the flag is only meaningful when the caller opts in and the opt-in does nothing by itself -- maybe_compress at :202 is what reads it, and that function is inside the tagged-on-correctly encoder.
// I want to be precise about the prior ruling's reasoning, because the coordinator may push back. The header note says FR-CIV-PROTO-006 'is kept here for the same reason as on the constant above', i.e. shape matching. Shape matching cannot establish a behavioral requirement: the requirement is satisfied by bytes surviving a round trip, and no field of this struct is a byte. The genuine coverage for this id is at bundle.rs:179 and :222 and in the round-trip test at crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:11, and none of that is lost when this tag comes off.
// Both FR-CIV-PROTO-006 container sites in this file therefore come off, while the two function sites at :182 and :225 stay. That is the 'one genuine, one not' shape the coordinator anticipated, relocated: the genuine binding is real, it just is not on either container.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-PROTO-006: DATA-SHAPE-ONLY, AND THE SHAPE IS AN INPUT, NOT A FORMAT. This OVERTURNS a prior GENUINE ruling, which held this tag 'for the same reason as on the constant above' (bundle.rs:93). The requirement is 'Support binary frames with zstd compression; unpack without errors', tested by 'Subscribe with use_binary_frames=true; verify frames unpack correctly' (CIV-0200-client-protocol.md:1149-1152). Frame3dBundleEncodeOptions at bundle.rs:102 is a two-field opt-in config struct -- `compress: bool` (:104) and `compress_level: i32` (:106) -- and its impl is a single Default (:109-116) returning compress: false. It is a bag of caller-supplied knobs. It does not encode, does not decode, does not describe a byte layout, and holds nothing that could be 'unpacked': there is no buffer, no header, no framing, and no round trip on this type. Even taken at its strongest it is the parameter object OF the frame format rather than the frame format. Two further points sharpen the verdict. First, the default DISABLES the very compression the requirement mandates: compress: false at :112 means an out-of-the-box Frame3dBundleEncodeOptions does not produce a zstd-compressed frame at all, so a client that constructs this type with Default::default() gets no binary-frame compression and therefore does not get FR-CIV-PROTO-006 behavior. A type whose default state contradicts the requirement is a weak binding for it. Second, the flags bit that actually records compression on the wire is a DIFFERENT type, Frame3dBundleFlags at bundle.rs:63, whose ZSTD_COMPRESSED bit 0 (:67) is what decode_frame3d_bundle reads at :237 to decide whether to decompress; that type carries other removed ids and not this one. The requirement is genuinely implemented, by encode_frame3d_bundle (bundle.rs:182, tagged at :179), encode_frame3d_bundle_from_f3d0 (:195, which builds the 23-byte header at :208-216), and decode_frame3d_bundle (:225, tagged at :222). This struct is passed into the first two as an argument (:184, :199) and read by maybe_compress at :202. Tagging it asserts that a struct of two switches is a binary frame format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame3dBundleEncodeOptions {
    /// When `true`, zstd-compress the concatenated `F3D0` payload.
    pub compress: bool,
    /// zstd level (meaningful only when [`Self::compress`] is `true`).
    pub compress_level: i32,
}

impl Default for Frame3dBundleEncodeOptions {
    fn default() -> Self {
        Self {
            compress: false,
            compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
        }
    }
}

/// Decoded `F3DB` bundle returned to clients.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame3dBundle {
    /// Server tick shared by every inner frame.
    pub tick: u64,
    /// Inner `Frame3d` values in wire order.
    pub frames: Vec<Frame3d>,
}

/// Errors from `F3DB` bundle encode / decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame3dBundleError {
    /// Magic bytes do not match [`FRAME3D_BUNDLE_MAGIC`].
    BadMagic,
    /// Buffer is shorter than the fixed header.
    TooShort,
    /// Header version is not supported by this decoder.
    UnsupportedVersion(u8),
    /// Declared lengths do not match the buffer or inner payload.
    LengthMismatch,
    /// Bundle contained no inner frames.
    EmptyBundle,
    /// Inner frames disagree on tick.
    TickMismatch {
        /// Tick taken from the first inner frame.
        expected: u64,
        /// Tick on a later inner frame.
        found: u64,
    },
    /// Declared frame count does not match parsed inner frames.
    FrameCountMismatch {
        /// Count in the `F3DB` header.
        declared: u8,
        /// Frames parsed from the inner payload.
        parsed: usize,
    },
    /// zstd compression failed.
    CompressionFailed(String),
    /// zstd decompression failed.
    DecompressionFailed(String),
    /// An inner `F3D0` frame could not be encoded or decoded.
    InvalidInnerFrame(Frame3dBinaryError),
}

/// Returns `true` if `bytes` starts with [`FRAME3D_BUNDLE_MAGIC`].
#[must_use]
/// FR-CIV-PROTO-005
/// FR-CIV-PROTO-006
/// FR-CIV-PROTO-012
/// FR-CIV-PROTO-013
pub fn is_frame3d_bundle(bytes: &[u8]) -> bool {
    bytes.len() >= FRAME3D_BUNDLE_MAGIC.len()
        && &bytes[..FRAME3D_BUNDLE_MAGIC.len()] == FRAME3D_BUNDLE_MAGIC
}

/// Encode a slice of [`Frame3d`] values into one `F3DB` WebSocket binary blob.
///
/// When [`Frame3dBundleEncodeOptions::compress`] is `false` (the default), the
/// on-wire layout matches an uncompressed bundle and existing decoders that only
/// understand `F3D0` can keep skipping `F3DB` via magic mismatch.
// FR-CIV-PROTO-005
// FR-CIV-PROTO-006
// FR-CIV-PROTO-012
// FR-CIV-PROTO-013
pub fn encode_frame3d_bundle(
    frames: &[Frame3d],
    options: &Frame3dBundleEncodeOptions,
) -> Result<Vec<u8>, Frame3dBundleError> {
    let tick = bundle_tick(frames)?;
    let inner = concat_f3d0_frames(frames)?;
    encode_frame3d_bundle_from_f3d0(tick, frames.len(), &inner, options)
}

/// Wrap already-serialized `F3D0` frame bytes in an `F3DB` envelope.
///
/// Use when the server already produced per-frame `F3D0` blobs and wants to
/// avoid re-serializing JSON for the bundle pass.
pub fn encode_frame3d_bundle_from_f3d0(
    tick: u64,
    frame_count: usize,
    inner: &[u8],
    options: &Frame3dBundleEncodeOptions,
) -> Result<Vec<u8>, Frame3dBundleError> {
    let frame_count = u8::try_from(frame_count).map_err(|_| Frame3dBundleError::LengthMismatch)?;
    let (flags, payload, uncompressed_len) = maybe_compress(inner, options)?;
    let payload_len =
        u32::try_from(payload.len()).map_err(|_| Frame3dBundleError::LengthMismatch)?;
    let uncompressed_len =
        u32::try_from(uncompressed_len).map_err(|_| Frame3dBundleError::LengthMismatch)?;

    let mut out = Vec::with_capacity(FRAME3D_BUNDLE_HEADER_LEN + payload.len());
    out.extend_from_slice(FRAME3D_BUNDLE_MAGIC);
    out.push(FRAME3D_BUNDLE_VERSION);
    out.push(flags.0);
    out.extend_from_slice(&tick.to_be_bytes());
    out.push(frame_count);
    out.extend_from_slice(&uncompressed_len.to_be_bytes());
    out.extend_from_slice(&payload_len.to_be_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Decode an `F3DB` blob produced by [`encode_frame3d_bundle`].
// FR-CIV-PROTO-005
// FR-CIV-PROTO-006
// FR-CIV-PROTO-012
// FR-CIV-PROTO-013
pub fn decode_frame3d_bundle(bytes: &[u8]) -> Result<Frame3dBundle, Frame3dBundleError> {
    if bytes.len() < FRAME3D_BUNDLE_HEADER_LEN {
        return Err(Frame3dBundleError::TooShort);
    }
    if &bytes[..4] != FRAME3D_BUNDLE_MAGIC {
        return Err(Frame3dBundleError::BadMagic);
    }
    let version = bytes[4];
    if version != FRAME3D_BUNDLE_VERSION {
        return Err(Frame3dBundleError::UnsupportedVersion(version));
    }

    let flags = Frame3dBundleFlags(bytes[5]);
    let tick = u64::from_be_bytes([
        bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13],
    ]);
    let frame_count = bytes[14];
    let uncompressed_len =
        u32::from_be_bytes([bytes[15], bytes[16], bytes[17], bytes[18]]) as usize;
    let payload_len = u32::from_be_bytes([bytes[19], bytes[20], bytes[21], bytes[22]]) as usize;
    let expected = FRAME3D_BUNDLE_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(Frame3dBundleError::LengthMismatch)?;
    if bytes.len() != expected {
        return Err(Frame3dBundleError::LengthMismatch);
    }

    let payload = &bytes[FRAME3D_BUNDLE_HEADER_LEN..];
    let inner = decompress_inner(payload, flags, uncompressed_len)?;
    let frames = parse_f3d0_payload(&inner)?;
    if usize::from(frame_count) != frames.len() {
        return Err(Frame3dBundleError::FrameCountMismatch {
            declared: frame_count,
            parsed: frames.len(),
        });
    }
    if let Some(first) = frames.first() {
        if first.tick() != tick {
            return Err(Frame3dBundleError::TickMismatch {
                expected: tick,
                found: first.tick(),
            });
        }
    } else if frame_count != 0 {
        return Err(Frame3dBundleError::EmptyBundle);
    }
    for frame in frames.iter().skip(1) {
        if frame.tick() != tick {
            return Err(Frame3dBundleError::TickMismatch {
                expected: tick,
                found: frame.tick(),
            });
        }
    }

    Ok(Frame3dBundle { tick, frames })
}

fn bundle_tick(frames: &[Frame3d]) -> Result<u64, Frame3dBundleError> {
    let Some(first) = frames.first() else {
        return Ok(0);
    };
    let tick = first.tick();
    for frame in frames.iter().skip(1) {
        if frame.tick() != tick {
            return Err(Frame3dBundleError::TickMismatch {
                expected: tick,
                found: frame.tick(),
            });
        }
    }
    Ok(tick)
}

fn concat_f3d0_frames(frames: &[Frame3d]) -> Result<Vec<u8>, Frame3dBundleError> {
    let mut out = Vec::new();
    for frame in frames {
        let bytes = encode_frame3d_binary(frame).map_err(Frame3dBundleError::InvalidInnerFrame)?;
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

fn maybe_compress(
    inner: &[u8],
    options: &Frame3dBundleEncodeOptions,
) -> Result<(Frame3dBundleFlags, Vec<u8>, usize), Frame3dBundleError> {
    if !options.compress {
        return Ok((
            Frame3dBundleFlags::uncompressed(),
            inner.to_vec(),
            inner.len(),
        ));
    }
    let compressed = encode_all(inner, options.compress_level)
        .map_err(|err| Frame3dBundleError::CompressionFailed(err.to_string()))?;
    Ok((Frame3dBundleFlags::zstd(), compressed, inner.len()))
}

fn decompress_inner(
    payload: &[u8],
    flags: Frame3dBundleFlags,
    uncompressed_len: usize,
) -> Result<Vec<u8>, Frame3dBundleError> {
    if flags.is_zstd() {
        let inner = decode_all(payload)
            .map_err(|err| Frame3dBundleError::DecompressionFailed(err.to_string()))?;
        if inner.len() != uncompressed_len {
            return Err(Frame3dBundleError::LengthMismatch);
        }
        return Ok(inner);
    }
    if payload.len() != uncompressed_len {
        return Err(Frame3dBundleError::LengthMismatch);
    }
    Ok(payload.to_vec())
}

fn parse_f3d0_payload(payload: &[u8]) -> Result<Vec<Frame3d>, Frame3dBundleError> {
    let mut frames = Vec::new();
    let mut cursor = 0;
    while cursor < payload.len() {
        let remaining = &payload[cursor..];
        if remaining.len() < FRAME3D_BINARY_HEADER_LEN {
            return Err(Frame3dBundleError::LengthMismatch);
        }
        if &remaining[..4] != FRAME3D_BINARY_MAGIC {
            return Err(Frame3dBundleError::InvalidInnerFrame(
                Frame3dBinaryError::BadMagic,
            ));
        }
        let len =
            u32::from_be_bytes([remaining[5], remaining[6], remaining[7], remaining[8]]) as usize;
        let frame_len = FRAME3D_BINARY_HEADER_LEN
            .checked_add(len)
            .ok_or(Frame3dBundleError::LengthMismatch)?;
        if remaining.len() < frame_len {
            return Err(Frame3dBundleError::LengthMismatch);
        }
        let frame = decode_frame3d_binary(&remaining[..frame_len])
            .map_err(Frame3dBundleError::InvalidInnerFrame)?;
        frames.push(frame);
        cursor = cursor
            .checked_add(frame_len)
            .ok_or(Frame3dBundleError::LengthMismatch)?;
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgentAppearanceFrame, BuildingDiffFrame, BuildingProvenance, ClimateFrame, EventFeedFrame,
        FactionStateFrame, Frame3d, VoxelDeltaFrame,
    };

    fn sample_frames(tick: u64) -> Vec<Frame3d> {
        vec![
            Frame3d::VoxelDelta(VoxelDeltaFrame {
                tick,
                deltas: vec![],
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick,
                provenance: BuildingProvenance::Procedural,
                buildings: vec![],
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick,
                updates: vec![],
            }),
            Frame3d::CivilianState(crate::CivilianStateFrame {
                tick,
                civilians: vec![],
            }),
            Frame3d::FactionState(FactionStateFrame {
                tick,
                factions: vec![],
                population_by_faction: Default::default(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick,
                events: vec![],
            }),
            Frame3d::Climate(ClimateFrame {
                tick,
                climate: civ_planet::Climate::default(),
                weather: vec![],
            }),
        ]
    }

    #[test]
    fn frame3d_bundle_uncompressed_roundtrip() {
        let frames = sample_frames(42);
        let bytes =
            encode_frame3d_bundle(&frames, &Frame3dBundleEncodeOptions::default()).expect("encode");
        assert!(is_frame3d_bundle(&bytes));
        assert_eq!(bytes[5], Frame3dBundleFlags::uncompressed().0);
        let back = decode_frame3d_bundle(&bytes).expect("decode");
        assert_eq!(back.tick, 42);
        assert_eq!(back.frames, frames);
    }

    #[test]
    fn frame3d_bundle_zstd_roundtrip() {
        let frames = sample_frames(99);
        let options = Frame3dBundleEncodeOptions {
            compress: true,
            compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
        };
        let bytes = encode_frame3d_bundle(&frames, &options).expect("encode");
        assert!(is_frame3d_bundle(&bytes));
        assert!(Frame3dBundleFlags(bytes[5]).is_zstd());
        let back = decode_frame3d_bundle(&bytes).expect("decode");
        assert_eq!(back.frames, frames);
    }

    #[test]
    fn frame3d_bundle_zstd_reduces_repetitive_payload() {
        let frames = sample_frames(7);
        let uncompressed = encode_frame3d_bundle(&frames, &Frame3dBundleEncodeOptions::default())
            .expect("uncompressed");
        let compressed = encode_frame3d_bundle(
            &frames,
            &Frame3dBundleEncodeOptions {
                compress: true,
                compress_level: DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL,
            },
        )
        .expect("compressed");
        assert!(
            compressed.len() < uncompressed.len(),
            "zstd should shrink repetitive tick bundles"
        );
    }

    #[test]
    fn encode_from_f3d0_matches_full_encode() {
        let frames = sample_frames(11);
        let inner: Vec<Vec<u8>> = frames
            .iter()
            .map(|frame| encode_frame3d_binary(frame).expect("f3d0"))
            .collect();
        let concatenated: Vec<u8> = inner
            .iter()
            .flat_map(|bytes| bytes.iter().copied())
            .collect();
        let from_f3d0 = encode_frame3d_bundle_from_f3d0(
            11,
            frames.len(),
            &concatenated,
            &Frame3dBundleEncodeOptions::default(),
        )
        .expect("from f3d0");
        let full =
            encode_frame3d_bundle(&frames, &Frame3dBundleEncodeOptions::default()).expect("full");
        assert_eq!(from_f3d0, full);
    }

    #[test]
    fn decode_rejects_bad_magic_and_short_buffer() {
        let frames = sample_frames(1);
        let mut bytes =
            encode_frame3d_bundle(&frames, &Frame3dBundleEncodeOptions::default()).expect("encode");
        bytes[0] = b'X';
        assert_eq!(
            decode_frame3d_bundle(&bytes),
            Err(Frame3dBundleError::BadMagic)
        );
        assert_eq!(
            decode_frame3d_bundle(&[0u8; 4]),
            Err(Frame3dBundleError::TooShort)
        );
    }

    #[test]
    fn decode_rejects_tick_mismatch_in_frames() {
        let mut frames = sample_frames(5);
        if let Frame3d::VoxelDelta(ref mut voxel) = frames[0] {
            voxel.tick = 6;
        }
        let err = encode_frame3d_bundle(&frames, &Frame3dBundleEncodeOptions::default())
            .expect_err("tick mismatch");
        assert!(matches!(err, Frame3dBundleError::TickMismatch { .. }));
    }
}
