//! Tests for FR-CIV-ROAD-901 - Road Promotion Ladder
//!
//! Epic: FR-CIV-FRAME
//! RoadKind SHALL support emergent promotion along desire-path ladder.

#[cfg(test)]
mod fr_fr_civ_road_901 {
    /// FR-CIV-ROAD-901: Road promotion follows the ladder.
    #[test]
    fn road_promotion_ladder() {
        use civ_traffic::RoadKind;
        assert_eq!(RoadKind::None.promoted(), RoadKind::Trail, "None promotes to Trail");
        assert_eq!(RoadKind::Trail.promoted(), RoadKind::Road, "Trail promotes to Road");
        assert_eq!(RoadKind::Road.promoted(), RoadKind::Highway, "Road promotes to Highway");
        // Highway and Bridge are terminal
        assert_eq!(RoadKind::Highway.promoted(), RoadKind::Highway, "Highway is terminal");
        assert_eq!(RoadKind::Bridge.promoted(), RoadKind::Bridge, "Bridge is terminal");
    }
}