//! Subscription filter for the CIV-0200 WebSocket tick-broadcast path.
//!
//! Implements FR-PROTO-005: a client connects with an optional
//! `?region=minx,miny,minz,maxx,maxy,maxz` query parameter and/or an
//! optional `?frame_kinds=voxel,building,agent,climate` list. The server
//! uses the resulting [`SubscriptionFilter`] to drop frames (or entities
//! inside frames) that the client did not subscribe to.
//!
//! Behaviour:
//!
//! * `region` uses an inclusive axis-aligned bounding box in fixed-point
//!   world coordinates (`phenotype_voxel::WorldCoord`).
//! * `frame_kinds` enumerates which `Frame3d` variants the client wants to
//!   receive; unselected variants are dropped wholesale.
//! * `region` and `frame_kinds` are mutually exclusive on the same
//!   connection (a client that wants both can open two sockets).
//! * Empty / unset fields preserve the existing "broadcast everything"
//!   behaviour, so existing clients are unaffected.

use std::collections::HashMap;

use civ_protocol_3d::{AgentAppearanceFrame, Frame3d, VoxelDeltaFrame};
use civ_voxel::{ChunkId, WorldCoord, FIXED_SCALE};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A closed, axis-aligned region in fixed-point world coordinates.
///
/// `min` and `max` are both inclusive. A degenerate region (`min == max`)
/// retains only entities at that exact world position. An empty region
/// (`min` greater than `max` on any axis) keeps nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Lower bound (inclusive).
    pub min: WorldCoord,
    /// Upper bound (inclusive).
    pub max: WorldCoord,
}

impl Region {
    /// Build a region from raw integer world units (multiplied by
    /// [`FIXED_SCALE`] internally). The values are in the same integer
    /// units used in the `?region=` connect query string.
    #[must_use]
    pub fn from_raw(min: [i64; 3], max: [i64; 3]) -> Self {
        Self {
            min: WorldCoord {
                x: min[0].saturating_mul(FIXED_SCALE),
                y: min[1].saturating_mul(FIXED_SCALE),
                z: min[2].saturating_mul(FIXED_SCALE),
            },
            max: WorldCoord {
                x: max[0].saturating_mul(FIXED_SCALE),
                y: max[1].saturating_mul(FIXED_SCALE),
                z: max[2].saturating_mul(FIXED_SCALE),
            },
        }
    }

    /// Whether the region is "empty" in the sense that no world point can
    /// satisfy all three axis ranges simultaneously. This is *not* the
    /// "no region set" case — that is represented by
    /// [`SubscriptionFilter::region`] being [`None`].
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Whether `coord` lies inside this region (inclusive on both ends).
    #[must_use]
    pub fn contains(self, coord: WorldCoord) -> bool {
        coord.x >= self.min.x
            && coord.x <= self.max.x
            && coord.y >= self.min.y
            && coord.y <= self.max.y
            && coord.z >= self.min.z
            && coord.z <= self.max.z
    }
}

/// Which [`Frame3d`] variant a subscription wants to keep.
///
/// Mirrors `civ_protocol_3d::Frame3d` one-for-one so the wire name on the
/// `?frame_kinds=` query parameter can be parsed deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameKind {
    /// [`Frame3d::VoxelDelta`].
    Voxel,
    /// [`Frame3d::BuildingDiff`].
    Building,
    /// [`Frame3d::AgentAppearance`].
    Agent,
    /// [`Frame3d::Climate`].
    Climate,
}

impl FrameKind {
    /// Wire name for this kind, matching the `frame_kinds=` token in the
    /// connect query string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Voxel => "voxel",
            Self::Building => "building",
            Self::Agent => "agent",
            Self::Climate => "climate",
        }
    }

    /// Parse a wire token. Returns [`None`] for unknown kinds.
    #[must_use]
    pub fn parse_token(token: &str) -> Option<Self> {
        match token {
            "voxel" => Some(Self::Voxel),
            "building" => Some(Self::Building),
            "agent" => Some(Self::Agent),
            "climate" => Some(Self::Climate),
            _ => None,
        }
    }

    /// Whether this kind matches the given [`Frame3d`] variant.
    #[must_use]
    pub fn matches(self, frame: &Frame3d) -> bool {
        matches!(
            (self, frame),
            (Self::Voxel, Frame3d::VoxelDelta(_))
                | (Self::Building, Frame3d::BuildingDiff(_))
                | (Self::Agent, Frame3d::AgentAppearance(_))
                | (Self::Climate, Frame3d::Climate(_))
        )
    }
}

/// Per-connection subscription filter parsed from the connect query
/// string. `region` and `frame_kinds` are mutually exclusive (validated by
/// [`from_connect_query`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubscriptionFilter {
    /// Optional set of frame kinds to keep. `None` means "all kinds".
    pub frame_kinds: Option<Vec<FrameKind>>,
    /// Optional spatial region. `None` means "no region filter".
    pub region: Option<Region>,
}

impl SubscriptionFilter {
    /// Construct an empty filter that lets every frame through.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// True when both `frame_kinds` and `region` are unset, i.e. the
    /// subscription does not narrow the broadcast at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frame_kinds.is_none() && self.region.is_none()
    }

    /// Parse a connect query string (with or without a leading `?`) into a
    /// [`SubscriptionFilter`].
    ///
    /// Recognised keys:
    ///
    /// * `region=minx,miny,minz,maxx,maxy,maxz` — six signed integers in
    ///   raw world units (multiplied by [`FIXED_SCALE`] internally).
    /// * `frame_kinds=voxel,building,agent,climate` — comma-separated list
    ///   of [`FrameKind`] wire names.
    ///
    /// Setting both keys on the same query is rejected with
    /// [`SubscriptionError::RegionAndFrameKindsConflict`].
    pub fn from_connect_query(query: &str) -> Result<Self, SubscriptionError> {
        let trimmed = query.strip_prefix('?').unwrap_or(query);
        let mut region: Option<Region> = None;
        let mut frame_kinds: Option<Vec<FrameKind>> = None;

        for (key, value) in split_query_pairs(trimmed) {
            match key {
                "region" => {
                    if region.is_some() {
                        return Err(SubscriptionError::DuplicateKey("region".to_owned()));
                    }
                    region = Some(parse_region_value(&value)?);
                }
                "frame_kinds" => {
                    if frame_kinds.is_some() {
                        return Err(SubscriptionError::DuplicateKey(
                            "frame_kinds".to_owned(),
                        ));
                    }
                    frame_kinds = Some(parse_frame_kinds_value(&value)?);
                }
                _ => {
                    return Err(SubscriptionError::UnknownKey(key.to_owned()));
                }
            }
        }

        if region.is_some() && frame_kinds.is_some() {
            return Err(SubscriptionError::RegionAndFrameKindsConflict);
        }

        Ok(Self { frame_kinds, region })
    }
}

/// Snapshot of per-agent world positions used by [`filter_frames`] when
/// applying a region filter to [`Frame3d::AgentAppearance`] frames.
///
/// The bridge populates this map from the ECS world once per tick; the
/// subscription filter never mutates it.
pub type AgentPositionMap = HashMap<u64, WorldCoord>;

/// Errors produced by the subscription filter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SubscriptionError {
    /// The `region` query value could not be parsed.
    #[error("invalid region `{raw}`: {reason}")]
    InvalidRegion {
        /// The raw value supplied on the connect query.
        raw: String,
        /// Why the value was rejected.
        reason: String,
    },
    /// The `frame_kinds` query value contained an unknown token.
    #[error("unknown frame kind `{token}` (expected one of: voxel, building, agent, climate)")]
    UnknownFrameKind {
        /// The unknown token.
        token: String,
    },
    /// A query key was supplied more than once.
    #[error("duplicate query key `{0}`")]
    DuplicateKey(String),
    /// An unrecognised query key was supplied.
    #[error("unknown query key `{0}`")]
    UnknownKey(String),
    /// Both `region` and `frame_kinds` were set on the same connection.
    #[error("region and frame_kinds filters are mutually exclusive on a single connection")]
    RegionAndFrameKindsConflict,
}

/// Apply `filter` to a batch of frames, dropping any frames (or entities
/// within frames) that the subscription did not opt into.
///
/// `agent_positions` is consulted only when `filter.region` is `Some` and
/// the frame kind is [`FrameKind::Agent`]; agents without a known position
/// are kept (the bridge prefers false positives to silently dropping
/// stateful updates when the world snapshot is incomplete).
#[must_use]
pub fn filter_frames(
    frames: Vec<Frame3d>,
    filter: &SubscriptionFilter,
    agent_positions: &AgentPositionMap,
) -> Vec<Frame3d> {
    if filter.is_empty() {
        return frames;
    }

    frames
        .into_iter()
        .filter_map(|frame| filter_one_frame(frame, filter, agent_positions))
        .collect()
}

fn filter_one_frame(
    frame: Frame3d,
    filter: &SubscriptionFilter,
    agent_positions: &AgentPositionMap,
) -> Option<Frame3d> {
    if let Some(kinds) = &filter.frame_kinds {
        if !kinds.iter().any(|k| k.matches(&frame)) {
            return None;
        }
    }

    if let Some(region) = filter.region {
        if region.is_empty() {
            // No world point can be inside this region, so any spatially
            // filtered entity is dropped. `BuildingDiff` and `Climate` do
            // not carry per-entity coords, so they pass through.
            return match frame {
                Frame3d::VoxelDelta(_) | Frame3d::AgentAppearance(_) => None,
                other => Some(other),
            };
        }
        return Some(match frame {
            Frame3d::VoxelDelta(v) => Frame3d::VoxelDelta(filter_voxel_delta(v, region)),
            Frame3d::AgentAppearance(a) => {
                Frame3d::AgentAppearance(filter_agent_appearance(a, region, agent_positions))
            }
            // Building diffs and climate frames do not carry per-entity
            // world coordinates in this protocol revision, so a region
            // filter cannot narrow them. They pass through unchanged.
            Frame3d::BuildingDiff(b) => Frame3d::BuildingDiff(b),
            Frame3d::Climate(c) => Frame3d::Climate(c),
        });
    }

    Some(frame)
}

fn filter_voxel_delta(frame: VoxelDeltaFrame, region: Region) -> VoxelDeltaFrame {
    let edge_world = 16i64.saturating_mul(FIXED_SCALE);
    let deltas = frame
        .deltas
        .into_iter()
        .filter(|delta| {
            let (cx, cy, cz) = chunk_id_to_components(delta.event.chunk_id);
            // Inclusive chunk bounds in world coords.
            let chunk_min = WorldCoord {
                x: i64::from(cx).saturating_mul(edge_world),
                y: i64::from(cy).saturating_mul(edge_world),
                z: i64::from(cz).saturating_mul(edge_world),
            };
            let chunk_max = WorldCoord {
                x: chunk_min.x.saturating_add(edge_world.saturating_sub(1)),
                y: chunk_min.y.saturating_add(edge_world.saturating_sub(1)),
                z: chunk_min.z.saturating_add(edge_world.saturating_sub(1)),
            };
            // Keep the chunk if it overlaps the region on every axis.
            chunk_max.x >= region.min.x
                && chunk_min.x <= region.max.x
                && chunk_max.y >= region.min.y
                && chunk_min.y <= region.max.y
                && chunk_max.z >= region.min.z
                && chunk_min.z <= region.max.z
        })
        .collect();
    VoxelDeltaFrame {
        tick: frame.tick,
        deltas,
    }
}

fn filter_agent_appearance(
    frame: AgentAppearanceFrame,
    region: Region,
    agent_positions: &AgentPositionMap,
) -> AgentAppearanceFrame {
    let updates = frame
        .updates
        .into_iter()
        .filter(|update| match agent_positions.get(&update.agent_id) {
            // No known position → keep (we cannot prove it is outside).
            None => true,
            Some(coord) => region.contains(*coord),
        })
        .collect();
    AgentAppearanceFrame {
        tick: frame.tick,
        updates,
    }
}

fn chunk_id_to_components(id: ChunkId) -> (i32, i32, i32) {
    // Mirror of `ChunkCoord::chunk_id` from `phenotype-voxel::coord`
    // (crates/voxel/src/lib.rs re-exported from phenotype-voxel/src/coord.rs):
    //
    //     (cx as u32 as u64) << 40
    //   | (cy as u32 as u64) << 16
    //   | (cz as u32 as u64) & 0xFFFF
    //
    // The high 8 bits of each 32-bit component would overlap, so the canonical
    // packing is effectively 24 / 24 / 16 bits. Decoding uses the matching
    // masks; using 32-bit masks on every component would bleed `cy` into `cz`
    // and silently drop chunks that straddle small x offsets.
    let raw = id.0;
    let cx = ((raw >> 40) & 0x00FF_FFFF) as u32 as i32;
    let cy = ((raw >> 16) & 0x00FF_FFFF) as u32 as i32;
    let cz = (raw & 0xFFFF) as u32 as i32;
    (cx, cy, cz)
}

fn parse_region_value(value: &str) -> Result<Region, SubscriptionError> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != 6 {
        return Err(SubscriptionError::InvalidRegion {
            raw: value.to_owned(),
            reason: format!("expected 6 numbers, got {}", parts.len()),
        });
    }
    let mut nums = [0i64; 6];
    for (idx, part) in parts.iter().enumerate() {
        nums[idx] = part.parse::<i64>().map_err(|err| SubscriptionError::InvalidRegion {
            raw: value.to_owned(),
            reason: format!("part {} (`{part}`) is not an integer: {err}", idx + 1),
        })?;
    }
    Ok(Region::from_raw(
        [nums[0], nums[1], nums[2]],
        [nums[3], nums[4], nums[5]],
    ))
}

fn parse_frame_kinds_value(value: &str) -> Result<Vec<FrameKind>, SubscriptionError> {
    let tokens: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if tokens.is_empty() {
        return Err(SubscriptionError::InvalidRegion {
            raw: value.to_owned(),
            reason: "frame_kinds value is empty".to_owned(),
        });
    }
    tokens
        .into_iter()
        .map(|token| {
            FrameKind::parse_token(token).ok_or_else(|| SubscriptionError::UnknownFrameKind {
                token: token.to_owned(),
            })
        })
        .collect()
}

fn split_query_pairs(query: &str) -> impl Iterator<Item = (&str, &str)> {
    query.split('&').filter_map(|pair| {
        let pair = pair.trim();
        if pair.is_empty() {
            return None;
        }
        let (key, value) = pair.split_once('=')?;
        Some((key.trim(), value.trim()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Climate;
    use civ_protocol_3d::{
        AgentAppearanceFrame, AgentAppearanceUpdate, BuildingDiffFrame, BuildingProvenance,
        ClimateFrame, Frame3d, VoxelChunkDelta, VoxelDeltaFrame,
    };
    use civ_voxel::{DirtyChunkEvent, MaterialId, WriteSeq};

    fn raw(x: i64, y: i64, z: i64) -> WorldCoord {
        WorldCoord {
            x: x.saturating_mul(FIXED_SCALE),
            y: y.saturating_mul(FIXED_SCALE),
            z: z.saturating_mul(FIXED_SCALE),
        }
    }

    fn agent_frame_with(agent_ids: &[u64]) -> AgentAppearanceFrame {
        AgentAppearanceFrame {
            tick: 1,
            updates: agent_ids
                .iter()
                .map(|id| AgentAppearanceUpdate {
                    agent_id: *id,
                    era: 0,
                    wardrobe: MaterialId(0),
                    tools: MaterialId(0),
                    scale: 1.0,
                })
                .collect(),
        }
    }

    fn chunk_id(cx: i32, cy: i32, cz: i32) -> ChunkId {
        let cx_u = (cx as u32) as u64;
        let cy_u = (cy as u32) as u64;
        let cz_u = (cz as u32) as u64;
        ChunkId((cx_u << 40) | (cy_u << 16) | (cz_u & 0xFFFF))
    }

    fn voxel_delta_with(chunks: &[(ChunkId, WriteSeq)]) -> VoxelDeltaFrame {
        VoxelDeltaFrame {
            tick: 1,
            deltas: chunks
                .iter()
                .map(|(id, seq)| VoxelChunkDelta {
                    event: DirtyChunkEvent {
                        chunk_id: *id,
                        write_seq: *seq,
                    },
                    voxels: Vec::new(),
                })
                .collect(),
        }
    }

    fn building_frame() -> BuildingDiffFrame {
        BuildingDiffFrame {
            tick: 1,
            provenance: BuildingProvenance::Procedural,
        }
    }

    fn climate_frame() -> ClimateFrame {
        ClimateFrame {
            tick: 1,
            climate: Climate {
                tick: 1,
                day_phase: 0.0,
                year_phase: 0.0,
                moon_phase: 0.0,
                tide_offset: 0.0,
            },
            weather_grid: Vec::new(),
        }
    }

    fn pos(x: i64, y: i64, z: i64) -> WorldCoord {
        raw(x, y, z)
    }

    #[test]
    fn region_inside_box_keeps_agent() {
        let region = Region::from_raw([0, 0, 0], [100, 100, 100]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let mut positions = AgentPositionMap::new();
        positions.insert(7, pos(10, 0, 20));
        let frame = Frame3d::AgentAppearance(agent_frame_with(&[7]));
        let out = filter_frames(vec![frame], &filter, &positions);
        assert_eq!(out.len(), 1, "agent inside the box must be kept");
        if let Frame3d::AgentAppearance(agent_frame) = &out[0] {
            assert_eq!(agent_frame.updates.len(), 1);
            assert_eq!(agent_frame.updates[0].agent_id, 7);
        } else {
            panic!("expected AgentAppearance frame");
        }
    }

    #[test]
    fn region_outside_box_drops_agent() {
        let region = Region::from_raw([50, 0, 50], [100, 100, 100]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let mut positions = AgentPositionMap::new();
        positions.insert(7, pos(10, 0, 20));
        let frame = Frame3d::AgentAppearance(agent_frame_with(&[7]));
        let out = filter_frames(vec![frame], &filter, &positions);
        assert_eq!(out.len(), 1, "frame wrapper must remain");
        if let Frame3d::AgentAppearance(agent_frame) = &out[0] {
            assert!(agent_frame.updates.is_empty(), "agent outside must be dropped");
        } else {
            panic!("expected AgentAppearance frame");
        }
    }

    #[test]
    fn region_degenerate_keeps_only_exact_match() {
        let region = Region::from_raw([10, 0, 20], [10, 0, 20]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let mut positions = AgentPositionMap::new();
        positions.insert(1, pos(10, 0, 20));
        positions.insert(2, pos(10, 0, 21));
        positions.insert(3, pos(11, 0, 20));
        let frame = Frame3d::AgentAppearance(agent_frame_with(&[1, 2, 3]));
        let out = filter_frames(vec![frame], &filter, &positions);
        assert_eq!(out.len(), 1);
        if let Frame3d::AgentAppearance(agent_frame) = &out[0] {
            assert_eq!(agent_frame.updates.len(), 1, "only the exact-match agent is kept");
            assert_eq!(agent_frame.updates[0].agent_id, 1);
        } else {
            panic!("expected AgentAppearance frame");
        }
    }

    #[test]
    fn empty_region_is_no_op() {
        // No region, no frame_kinds: the existing "broadcast everything"
        // behaviour is preserved.
        let filter = SubscriptionFilter::empty();
        let positions = AgentPositionMap::new();
        let frames = vec![
            Frame3d::AgentAppearance(agent_frame_with(&[1, 2, 3])),
            Frame3d::BuildingDiff(building_frame()),
            Frame3d::Climate(climate_frame()),
        ];
        let out = filter_frames(frames.clone(), &filter, &positions);
        assert_eq!(out.len(), frames.len());
    }

    #[test]
    fn region_and_frame_kinds_set_returns_error_on_parse() {
        let err = SubscriptionFilter::from_connect_query(
            "region=0,0,0,100,100,100&frame_kinds=voxel",
        )
        .expect_err("mutually-exclusive conflict must be rejected");
        assert_eq!(err, SubscriptionError::RegionAndFrameKindsConflict);
    }

    #[test]
    fn from_connect_query_parses_region() {
        let filter =
            SubscriptionFilter::from_connect_query("?region=-10,0,5,50,60,70").expect("parse");
        let region = filter.region.expect("region should be set");
        assert_eq!(region.min, raw(-10, 0, 5));
        assert_eq!(region.max, raw(50, 60, 70));
        assert!(filter.frame_kinds.is_none());
    }

    #[test]
    fn from_connect_query_parses_frame_kinds() {
        let filter = SubscriptionFilter::from_connect_query("?frame_kinds=voxel,agent")
            .expect("parse");
        let kinds = filter.frame_kinds.expect("frame_kinds should be set");
        assert_eq!(kinds, vec![FrameKind::Voxel, FrameKind::Agent]);
        assert!(filter.region.is_none());
    }

    #[test]
    fn from_connect_query_rejects_unknown_frame_kind() {
        let err = SubscriptionFilter::from_connect_query("?frame_kinds=voxel,alien")
            .expect_err("unknown frame kind must be rejected");
        match err {
            SubscriptionError::UnknownFrameKind { token } => assert_eq!(token, "alien"),
            other => panic!("expected UnknownFrameKind, got {other:?}"),
        }
    }

    #[test]
    fn from_connect_query_rejects_malformed_region() {
        let err = SubscriptionFilter::from_connect_query("?region=0,0,0,100")
            .expect_err("short region must be rejected");
        assert!(matches!(err, SubscriptionError::InvalidRegion { .. }));
    }

    #[test]
    fn from_connect_query_rejects_unknown_key() {
        let err = SubscriptionFilter::from_connect_query("?bogus=1").expect_err("reject");
        assert_eq!(err, SubscriptionError::UnknownKey("bogus".to_owned()));
    }

    #[test]
    fn frame_kinds_filter_drops_unselected_variants() {
        let filter = SubscriptionFilter {
            frame_kinds: Some(vec![FrameKind::Agent]),
            region: None,
        };
        let positions = AgentPositionMap::new();
        let frames = vec![
            Frame3d::AgentAppearance(agent_frame_with(&[1])),
            Frame3d::BuildingDiff(building_frame()),
            Frame3d::Climate(climate_frame()),
        ];
        let out = filter_frames(frames, &filter, &positions);
        assert_eq!(out.len(), 1, "only Agent frames should remain");
        assert!(matches!(&out[0], Frame3d::AgentAppearance(_)));
    }

    #[test]
    fn region_drops_voxel_deltas_outside_box() {
        // Region covers chunks (0,0,0) and (1,0,0); chunk (5,0,0) is dropped.
        let region = Region::from_raw([0, 0, 0], [31, 0, 0]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let positions = AgentPositionMap::new();
        let frame = Frame3d::VoxelDelta(voxel_delta_with(&[
            (chunk_id(0, 0, 0), WriteSeq(1)),
            (chunk_id(1, 0, 0), WriteSeq(2)),
            (chunk_id(5, 0, 0), WriteSeq(3)),
        ]));
        let out = filter_frames(vec![frame], &filter, &positions);
        assert_eq!(out.len(), 1);
        if let Frame3d::VoxelDelta(v) = &out[0] {
            assert_eq!(v.deltas.len(), 2, "only the two inside-region chunks remain");
            let chunk_ids: Vec<ChunkId> = v.deltas.iter().map(|d| d.event.chunk_id).collect();
            assert!(chunk_ids.contains(&chunk_id(0, 0, 0)));
            assert!(chunk_ids.contains(&chunk_id(1, 0, 0)));
        } else {
            panic!("expected VoxelDelta frame");
        }
    }

    #[test]
    fn region_passes_through_building_and_climate() {
        let region = Region::from_raw([0, 0, 0], [10, 10, 10]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let positions = AgentPositionMap::new();
        let frames = vec![
            Frame3d::BuildingDiff(building_frame()),
            Frame3d::Climate(climate_frame()),
        ];
        let out = filter_frames(frames, &filter, &positions);
        assert_eq!(out.len(), 2, "non-entity frames pass through");
    }

    #[test]
    fn region_keeps_agents_with_unknown_position() {
        // Defensive default: an agent whose position is missing from the
        // snapshot is kept so a transient world-snapshot gap cannot drop
        // stateful updates.
        let region = Region::from_raw([100, 100, 100], [200, 200, 200]);
        let filter = SubscriptionFilter {
            region: Some(region),
            frame_kinds: None,
        };
        let positions = AgentPositionMap::new();
        let frame = Frame3d::AgentAppearance(agent_frame_with(&[1, 2, 3]));
        let out = filter_frames(vec![frame], &filter, &positions);
        assert_eq!(out.len(), 1);
        if let Frame3d::AgentAppearance(agent_frame) = &out[0] {
            assert_eq!(
                agent_frame.updates.len(),
                3,
                "all agents with unknown positions are kept"
            );
        } else {
            panic!("expected AgentAppearance frame");
        }
    }

    #[test]
    fn region_is_empty_means_nothing_inside() {
        // min > max on x axis: the region cannot contain any world point.
        let region = Region {
            min: raw(50, 0, 0),
            max: raw(10, 100, 100),
        };
        assert!(region.is_empty());
    }
}
