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
pub const FRAME3D_BUNDLE_MAGIC: &[u8; 4] = b"F3DB";

/// Current wire version of the `F3DB` envelope.
pub const FRAME3D_BUNDLE_VERSION: u8 = 1;

/// Per-tick frame count shipped by `civ-server` today (voxel, building, agent,
/// civilian, faction, event feed, climate).
pub const FRAME3D_BUNDLE_STANDARD_LEN: usize = 7;

/// Default zstd level for tick bundles (fast decompress; CIV-0500 §8.4).
pub const DEFAULT_FRAME3D_BUNDLE_ZSTD_LEVEL: i32 = 1;

const FRAME3D_BUNDLE_HEADER_LEN: usize = 23;

/// Capability / compression flag bits in the `F3DB` header.
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
pub fn is_frame3d_bundle(bytes: &[u8]) -> bool {
    bytes.len() >= FRAME3D_BUNDLE_MAGIC.len()
        && &bytes[..FRAME3D_BUNDLE_MAGIC.len()] == FRAME3D_BUNDLE_MAGIC
}

/// Encode a slice of [`Frame3d`] values into one `F3DB` WebSocket binary blob.
///
/// When [`Frame3dBundleEncodeOptions::compress`] is `false` (the default), the
/// on-wire layout matches an uncompressed bundle and existing decoders that only
/// understand `F3D0` can keep skipping `F3DB` via magic mismatch.
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
