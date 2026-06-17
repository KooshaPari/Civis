//! Opt-in per-connection tick broadcast filtering for bandwidth optimization.
//!
//! Inactive by default (full [`Frame3d`] bundle every tick). Clients opt in via
//! WebSocket query (`sub_filter`, `frame_kinds`, `tick_stride`) or JSON-RPC
//! (`sim.subscribe`, `sim.update_subscription`). [`sim.unsubscribe`] clears the filter.

use std::collections::HashSet;

use civ_protocol_3d::Frame3d;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::jsonrpc::{error_code, JsonRpcError};

/// One variant of the per-tick [`Frame3d`] bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameKind {
    /// `Frame3d::VoxelDelta`
    VoxelDelta,
    /// `Frame3d::BuildingDiff`
    BuildingDiff,
    /// `Frame3d::AgentAppearance`
    AgentAppearance,
    /// `Frame3d::CivilianState`
    CivilianState,
    /// `Frame3d::FactionState`
    FactionState,
    /// `Frame3d::EventFeed`
    EventFeed,
    /// `Frame3d::Climate`
    Climate,
}

impl FrameKind {
    /// All bundle kinds (seven per tick).
    pub const ALL: [Self; 7] = [
        Self::VoxelDelta,
        Self::BuildingDiff,
        Self::AgentAppearance,
        Self::CivilianState,
        Self::FactionState,
        Self::EventFeed,
        Self::Climate,
    ];

    /// Stable wire name for subscribe responses.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::VoxelDelta => "voxel_delta",
            Self::BuildingDiff => "building_diff",
            Self::AgentAppearance => "agent_appearance",
            Self::CivilianState => "civilian_state",
            Self::FactionState => "faction_state",
            Self::EventFeed => "event_feed",
            Self::Climate => "climate",
        }
    }

    fn from_frame(frame: &Frame3d) -> Self {
        match frame {
            Frame3d::VoxelDelta(_) => Self::VoxelDelta,
            Frame3d::BuildingDiff(_) => Self::BuildingDiff,
            Frame3d::AgentAppearance(_) => Self::AgentAppearance,
            Frame3d::CivilianState(_) => Self::CivilianState,
            Frame3d::FactionState(_) => Self::FactionState,
            Frame3d::EventFeed(_) => Self::EventFeed,
            Frame3d::Climate(_) => Self::Climate,
        }
    }
}

/// Parsed WebSocket connect query knobs for subscription filtering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct WsConnectQuery {
    /// Comma-separated frame kind names (`sub_filter=voxel,climate`).
    pub sub_filter: Option<String>,
    /// Alias for `sub_filter`.
    pub frame_kinds: Option<String>,
    /// Deliver every Nth simulation tick (`tick_stride=5`).
    pub tick_stride: Option<String>,
}

/// Opt-in per-connection filter stored on the WebSocket handler task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionFilter {
    active: bool,
    kinds: HashSet<FrameKind>,
    tick_stride: u32,
    subscription_id: Option<String>,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self {
            active: false,
            kinds: HashSet::new(),
            tick_stride: 1,
            subscription_id: None,
        }
    }
}

impl SubscriptionFilter {
    /// Build from WS connect query params. Absent params → inactive (full broadcast).
    pub fn from_connect_query(query: &WsConnectQuery) -> Self {
        let kinds_source = query
            .sub_filter
            .as_deref()
            .or(query.frame_kinds.as_deref());
        let stride = query
            .tick_stride
            .as_deref()
            .and_then(parse_tick_stride);
        build_filter(kinds_source, stride, None)
    }

    /// Apply `sim.subscribe` / `sim.update_subscription` params.
    pub fn apply_subscribe_params(
        &mut self,
        params: Option<&Value>,
        current_tick: u64,
    ) -> Result<Value, JsonRpcError> {
        let kinds = parse_kind_sources(params)?;
        let stride = params
            .and_then(|p| p.get("tick_stride"))
            .and_then(parse_tick_stride_value)
            .or_else(|| {
                params
                    .and_then(|p| p.get("max_framerate_hz"))
                    .and_then(|v| v.as_u64())
                    .and_then(tick_stride_from_max_framerate)
            });
        let subscription_id = params
            .and_then(|p| p.get("subscription_id"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        *self = build_filter(kinds.as_deref(), stride, Some(subscription_id));
        Ok(self.subscribe_result(current_tick))
    }

    /// Clear opt-in filtering (`sim.unsubscribe`).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Whether this connection requested filtered/decimated delivery.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Whether a tick broadcast should be delivered on this connection.
    #[must_use]
    pub fn should_deliver_tick(&self, tick: u64) -> bool {
        if !self.active || self.tick_stride <= 1 {
            return true;
        }
        tick.is_multiple_of(u64::from(self.tick_stride))
    }

    /// Return the subset of frames allowed by this filter (cloned).
    #[must_use]
    pub fn filter_frames(&self, frames: &[Frame3d]) -> Vec<Frame3d> {
        if !self.active {
            return frames.to_vec();
        }
        frames
            .iter()
            .filter(|frame| self.kinds.contains(&FrameKind::from_frame(frame)))
            .cloned()
            .collect()
    }

    fn subscribe_result(&self, current_tick: u64) -> Value {
        let kinds: Vec<&str> = if self.active {
            let mut names: Vec<&str> = self.kinds.iter().map(FrameKind::wire_name).collect();
            names.sort_unstable();
            names
        } else {
            Vec::new()
        };
        json!({
            "subscribed": true,
            "subscription_id": self.subscription_id,
            "filter_active": self.active,
            "frame_kinds": kinds,
            "tick_stride": self.tick_stride,
            "current_tick": current_tick,
        })
    }
}

fn build_filter(
    kinds_source: Option<&str>,
    tick_stride: Option<u32>,
    subscription_id: Option<String>,
) -> SubscriptionFilter {
    let stride = tick_stride.unwrap_or(1);
    let Some(source) = kinds_source.filter(|s| !s.trim().is_empty()) else {
        if stride > 1 {
            return SubscriptionFilter {
                active: true,
                kinds: FrameKind::ALL.into_iter().collect(),
                tick_stride: stride,
                subscription_id,
            };
        }
        let mut filter = SubscriptionFilter::default();
        filter.subscription_id = subscription_id;
        return filter;
    };

    let kinds = parse_kind_csv(source).unwrap_or_default();
    let all_kinds = kinds.is_empty() || kinds.len() == FrameKind::ALL.len();
    SubscriptionFilter {
        active: true,
        kinds: if all_kinds {
            FrameKind::ALL.into_iter().collect()
        } else {
            kinds
        },
        tick_stride: stride.max(1),
        subscription_id,
    }
}

fn parse_kind_sources(params: Option<&Value>) -> Result<Option<String>, JsonRpcError> {
    if let Some(array) = params
        .and_then(|p| p.get("frame_kinds"))
        .or_else(|| params.and_then(|p| p.get("filter")))
        .or_else(|| params.and_then(|p| p.get("filter_types")))
        .and_then(|v| v.as_array())
    {
        let mut kinds = HashSet::new();
        for entry in array {
            let Some(name) = entry.as_str() else {
                return Err(invalid_subscribe_params(
                    "frame kind entries must be strings",
                ));
            };
            extend_kind_aliases(name, &mut kinds);
        }
        if kinds.is_empty() {
            return Ok(None);
        }
        let csv = kinds
            .iter()
            .map(|kind| kind.wire_name())
            .collect::<Vec<_>>()
            .join(",");
        return Ok(Some(csv));
    }

    Ok(params
        .and_then(|p| p.get("frame_kinds"))
        .or_else(|| params.and_then(|p| p.get("sub_filter")))
        .and_then(|v| v.as_str())
        .map(str::to_owned))
}

fn parse_kind_csv(source: &str) -> Option<HashSet<FrameKind>> {
    let mut kinds = HashSet::new();
    for part in source.split(',') {
        extend_kind_aliases(part.trim(), &mut kinds);
    }
    if kinds.is_empty() {
        None
    } else {
        Some(kinds)
    }
}

fn extend_kind_aliases(name: &str, kinds: &mut HashSet<FrameKind>) {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "*" || normalized == "all" {
        kinds.extend(FrameKind::ALL);
        return;
    }
    if let Some(kind) = parse_frame_kind_alias(&normalized) {
        kinds.insert(kind);
        return;
    }
    if normalized.starts_with("entities.agents") || normalized == "entities.agent" {
        kinds.insert(FrameKind::AgentAppearance);
        kinds.insert(FrameKind::CivilianState);
        return;
    }
    if normalized.starts_with("entities.buildings") || normalized == "entities.building" {
        kinds.insert(FrameKind::BuildingDiff);
        return;
    }
    if normalized.starts_with("events.") {
        kinds.insert(FrameKind::EventFeed);
        return;
    }
    if normalized.starts_with("metrics.") {
        kinds.insert(FrameKind::FactionState);
    }
}

fn parse_frame_kind_alias(normalized: &str) -> Option<FrameKind> {
    match normalized {
        "voxel" | "voxel_delta" | "voxeldelta" => Some(FrameKind::VoxelDelta),
        "building" | "building_diff" | "buildingdiff" => Some(FrameKind::BuildingDiff),
        "agent" | "agent_appearance" | "agentappearance" => Some(FrameKind::AgentAppearance),
        "civilian" | "civilian_state" | "civilianstate" => Some(FrameKind::CivilianState),
        "faction" | "faction_state" | "factionstate" => Some(FrameKind::FactionState),
        "event" | "event_feed" | "eventfeed" | "events" => Some(FrameKind::EventFeed),
        "climate" => Some(FrameKind::Climate),
        _ => None,
    }
}

fn parse_tick_stride(raw: &str) -> Option<u32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let value = trimmed.parse::<u32>().ok()?;
    (value >= 1).then_some(value)
}

fn parse_tick_stride_value(value: &Value) -> Option<u32> {
    value
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .filter(|v| *v >= 1)
}

fn tick_stride_from_max_framerate(hz: u64) -> Option<u32> {
    if hz == 0 {
        return None;
    }
    let stride = 10_u64.div_ceil(hz);
    u32::try_from(stride).ok()
}

fn invalid_subscribe_params(message: &str) -> JsonRpcError {
    JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message: format!("Invalid subscribe params: {message}"),
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Simulation;
    use civ_protocol_3d::{
        AgentAppearanceFrame, BuildingDiffFrame, BuildingProvenance, CivilianStateFrame,
        ClimateFrame, EventFeedFrame, FactionStateFrame, VoxelDeltaFrame,
    };

    fn sample_frames() -> [Frame3d; 3] {
        [
            Frame3d::VoxelDelta(VoxelDeltaFrame {
                tick: 4,
                deltas: Vec::new(),
            }),
            Frame3d::Climate(ClimateFrame {
                tick: 4,
                climate: *Simulation::with_seed(1).climate(),
                weather: Vec::new(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 4,
                events: Vec::new(),
            }),
        ]
    }

    #[test]
    fn default_filter_is_inactive_and_passes_through() {
        let filter = SubscriptionFilter::default();
        assert!(!filter.is_active());
        let frames = sample_frames();
        assert_eq!(filter.filter_frames(&frames).len(), frames.len());
        assert!(filter.should_deliver_tick(7));
    }

    #[test]
    fn connect_query_parses_sub_filter() {
        let filter = SubscriptionFilter::from_connect_query(&WsConnectQuery {
            sub_filter: Some("climate,voxel".into()),
            ..Default::default()
        });
        assert!(filter.is_active());
        let filtered = filter.filter_frames(&sample_frames());
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn tick_stride_skips_intermediate_ticks() {
        let filter = SubscriptionFilter::from_connect_query(&WsConnectQuery {
            tick_stride: Some("2".into()),
            ..Default::default()
        });
        assert!(filter.is_active());
        assert!(filter.should_deliver_tick(2));
        assert!(!filter.should_deliver_tick(3));
    }

    #[test]
    fn subscribe_params_accept_legacy_filter_strings() {
        let mut filter = SubscriptionFilter::default();
        let params = json!({
            "filter": ["entities.agents", "events.all"],
            "tick_stride": 1
        });
        let result = filter
            .apply_subscribe_params(Some(&params), 12)
            .expect("subscribe");
        assert_eq!(result.get("subscribed"), Some(&json!(true)));
        let frames = [
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 1,
                provenance: BuildingProvenance::Procedural,
                buildings: Vec::new(),
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 1,
                updates: Vec::new(),
            }),
            Frame3d::CivilianState(CivilianStateFrame {
                tick: 1,
                civilians: Vec::new(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 1,
                events: Vec::new(),
            }),
        ];
        let filtered = filter.filter_frames(&frames);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn clear_restores_full_broadcast() {
        let mut filter = SubscriptionFilter::from_connect_query(&WsConnectQuery {
            sub_filter: Some("climate".into()),
            ..Default::default()
        });
        filter.clear();
        assert!(!filter.is_active());
        assert_eq!(filter.filter_frames(&sample_frames()).len(), 3);
    }

    #[test]
    fn faction_alias_maps_metrics_filter() {
        let mut kinds = HashSet::new();
        extend_kind_aliases("metrics.gdp", &mut kinds);
        assert!(kinds.contains(&FrameKind::FactionState));
    }
}

