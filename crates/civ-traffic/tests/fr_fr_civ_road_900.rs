//! Tests for FR-CIV-ROAD-900 - Road Kind Speed Multiplier
//!
//! Epic: FR-CIV-FRAME
//! RoadKind SHALL provide speed multipliers for pathing cost model.

#[cfg(test)]
mod fr_fr_civ_road_900 {
    /// FR-CIV-ROAD-900: Each road kind has a distinct speed multiplier.
    #[test]
    fn road_kind_speed_multipliers() {
        use civ_traffic::RoadKind;
        assert_eq!(RoadKind::None.speed_multiplier(), 1.0, "bare ground = baseline");
        assert_eq!(RoadKind::Trail.speed_multiplier(), 1.25);
        assert_eq!(RoadKind::Road.speed_multiplier(), 1.8);
        assert_eq!(RoadKind::Highway.speed_multiplier(), 2.5);
        assert_eq!(RoadKind::Bridge.speed_multiplier(), 1.8);
    }

    /// FR-CIV-ROAD-900: Higher road kinds are faster.
    #[test]
    fn higher_road_kinds_are_faster() {
        use civ_traffic::RoadKind;
        assert!(RoadKind::Trail.speed_multiplier() > RoadKind::None.speed_multiplier());
        assert!(RoadKind::Road.speed_multiplier() > RoadKind::Trail.speed_multiplier());
        assert!(RoadKind::Highway.speed_multiplier() > RoadKind::Road.speed_multiplier());
    }
}