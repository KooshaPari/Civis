//! FR-AI-001 — AI civilizations SHALL select actions using a utility scoring
//! function over available moves.

#[test]
fn scores_all_moves() {
    use civ_ai::utility::{Move, UtilityScorer};

    let scorer = UtilityScorer::default_scorer();
    let moves = vec![
        Move {
            id: "expand".into(),
            resource_value: 0.9,
            strategic_value: 0.7,
            safety_value: 0.3,
            diplomatic_value: 0.2,
        },
        Move {
            id: "trade".into(),
            resource_value: 0.4,
            strategic_value: 0.3,
            safety_value: 0.8,
            diplomatic_value: 0.9,
        },
        Move {
            id: "fortify".into(),
            resource_value: 0.1,
            strategic_value: 0.5,
            safety_value: 0.95,
            diplomatic_value: 0.1,
        },
    ];

    let scored = scorer.score_all(&moves);
    // All three moves must be scored.
    assert_eq!(scored.len(), 3);
    // Scores must be in descending order.
    assert!(scored[0].score >= scored[1].score);
    assert!(scored[1].score >= scored[2].score);
    // Each scored move references a valid id.
    let ids: Vec<&str> = scored.iter().map(|s| s.move_id.as_str()).collect();
    assert!(ids.contains(&"expand"));
    assert!(ids.contains(&"trade"));
    assert!(ids.contains(&"fortify"));
}

#[test]
fn custom_weights_change_ranking() {
    use civ_ai::utility::{Move, UtilityScorer, UtilityWeights};

    let moves = vec![
        Move {
            id: "aggressive".into(),
            resource_value: 0.5,
            strategic_value: 0.9,
            safety_value: 0.1,
            diplomatic_value: 0.1,
        },
        Move {
            id: "peaceful".into(),
            resource_value: 0.5,
            strategic_value: 0.1,
            safety_value: 0.9,
            diplomatic_value: 0.9,
        },
    ];

    // Safety + diplomacy heavy weights -> peaceful wins.
    let safety_weights = UtilityWeights {
        resource_value: 0.1,
        strategic_value: 0.1,
        safety_value: 2.0,
        diplomatic_value: 2.0,
    };
    let scorer = UtilityScorer::new(safety_weights);
    let scored = scorer.score_all(&moves);
    assert_eq!(scored[0].move_id, "peaceful");

    // Strategy-heavy weights -> aggressive wins.
    let strat_weights = UtilityWeights {
        resource_value: 0.1,
        strategic_value: 3.0,
        safety_value: 0.0,
        diplomatic_value: 0.0,
    };
    let scorer2 = UtilityScorer::new(strat_weights);
    let scored2 = scorer2.score_all(&moves);
    assert_eq!(scored2[0].move_id, "aggressive");
}

#[test]
fn empty_moves_returns_empty() {
    use civ_ai::utility::UtilityScorer;

    let scorer = UtilityScorer::default_scorer();
    let scored: Vec<_> = scorer.score_all(&[]);
    assert!(scored.is_empty());
}
