//! FR-AI-003 — Each AI leader SHALL have a personality profile affecting
//! utility weights.

#[test]
fn weights_differ_per_profile() {
    use civ_ai::personality::{PersonalityKind, PersonalityProfile};
    use civ_ai::utility::UtilityWeights;

    let base = UtilityWeights::default();

    let expansionist = PersonalityProfile::from_kind(PersonalityKind::Expansionist);
    let diplomat = PersonalityProfile::from_kind(PersonalityKind::Diplomat);
    let militant = PersonalityProfile::from_kind(PersonalityKind::Militant);
    let isolationist = PersonalityProfile::from_kind(PersonalityKind::Isolationist);

    let w_exp = expansionist.apply_to(&base);
    let w_dip = diplomat.apply_to(&base);
    let w_mil = militant.apply_to(&base);
    let w_iso = isolationist.apply_to(&base);

    // Expansionist should boost resource over diplomat.
    assert!(w_exp.resource_value > w_dip.resource_value,
        "expansionist resource ({}) > diplomat resource ({})",
        w_exp.resource_value, w_dip.resource_value);

    // Diplomat should boost diplomatic over expansionist.
    assert!(w_dip.diplomatic_value > w_exp.diplomatic_value,
        "diplomat diplomatic ({}) > expansionist diplomatic ({})",
        w_dip.diplomatic_value, w_exp.diplomatic_value);

    // Militant should boost strategic over isolationist.
    assert!(w_mil.strategic_value > w_iso.strategic_value,
        "militant strategic ({}) > isolationist strategic ({})",
        w_mil.strategic_value, w_iso.strategic_value);

    // Isolationist should boost safety over militant.
    assert!(w_iso.safety_value > w_mil.safety_value,
        "isolationist safety ({}) > militant safety ({})",
        w_iso.safety_value, w_mil.safety_value);
}

#[test]
fn personality_changes_move_ranking() {
    use civ_ai::personality::{PersonalityKind, PersonalityProfile};
    use civ_ai::utility::{Move, UtilityScorer, UtilityWeights};

    let moves = vec
![
        Move {
            id: "trade_route".into(),
            resource_value: 0.6,
            strategic_value: 0.2,
            safety_value: 0.3,
            diplomatic_value: 0.9,
        },
        Move {
            id: "military_buildup".into(),
            resource_value: 0.2,
            strategic_value: 0.9,
            safety_value: 0.4,
            diplomatic_value: 0.1,
        },
    ];

    let base = UtilityWeights::default();

    // Diplomat personality should prefer trade_route.
    let dip_profile = PersonalityProfile::from_kind(PersonalityKind::Diplomat);
    let dip_weights = dip_profile.apply_to(&base);
    let dip_scorer = UtilityScorer::new(dip_weights);
    let dip_ranked = dip_scorer.score_all(&moves);
    assert_eq!(dip_ranked[0].move_id, "trade_route");

    // Militant personality should prefer military_buildup.
    let mil_profile = PersonalityProfile::from_kind(PersonalityKind::Militant);
    let mil_weights = mil_profile.apply_to(&base);
    let mil_scorer = UtilityScorer::new(mil_weights);
    let mil_ranked = mil_scorer.score_all(&moves);
    assert_eq!(mil_ranked[0].move_id, "military_buildup");
}
