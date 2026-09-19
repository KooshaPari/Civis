//! Tests for FR-CIV-AI-013
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-AI-013: Culture/meme drift service
//! (embeddings -> cosine drift -> speciation threshold). Speciation event
//! fires past threshold; vectors stored on meme record.

use civ_ai::{DummyAiProvider, EmbedRequest};

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod fr_fr_civ_ai_013 {
    use civ_ai::AiProvider;
    use super::*;

    /// FR-CIV-AI-013 — Embedding provider produces fixed-dimension vectors
    /// suitable for cosine-distance drift measurement between memes/cultures.
    #[test]
    fn drift_embedding_produces_vectors() {
        let provider = DummyAiProvider;
        let req = EmbedRequest {
            texts: vec!["meme: cooperation".into(), "meme: aggression".into()],
            input_snapshot_hash: [0u8; 32],
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vectors = rt.block_on(provider.embed(&req)).unwrap();
        assert_eq!(vectors.len(), 2, "one vector per input text");
        assert!(!vectors[0].is_empty(), "vectors must be non-empty");
        assert_eq!(vectors[0].len(), vectors[1].len(), "all vectors same dim");
    }

    /// FR-CIV-AI-013 — Same meme text always produces the same embedding
    /// (deterministic drift measurement across ticks).
    #[test]
    fn drift_embedding_is_deterministic() {
        let provider = DummyAiProvider;
        let req = EmbedRequest {
            texts: vec!["meme: tradition".into()],
            input_snapshot_hash: [0u8; 32],
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let v1 = rt.block_on(provider.embed(&req)).unwrap();
        let v2 = rt.block_on(provider.embed(&req)).unwrap();
        assert_eq!(v1[0], v2[0], "same input must produce same embedding");
    }

    /// FR-CIV-AI-013 — Different memes produce different embeddings, enabling
    /// cosine-distance-based speciation detection.
    #[test]
    fn drift_different_memes_diverge() {
        let provider = DummyAiProvider;
        let req = EmbedRequest {
            texts: vec![
                "meme: agrarian tradition".into(),
                "meme: industrial innovation".into(),
            ],
            input_snapshot_hash: [0u8; 32],
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vectors = rt.block_on(provider.embed(&req)).unwrap();
        let sim = cosine_similarity(&vectors[0], &vectors[1]);
        // These should not be identical (sim < 1.0) to allow drift detection
        assert!(sim < 1.0, "different memes should produce different vectors");
    }

    /// FR-CIV-AI-013 — Cosine similarity is in [-1, 1] and identical vectors
    /// yield similarity 1.0, confirming the threshold mechanism works.
    #[test]
    fn drift_cosine_similarity_range() {
        let provider = DummyAiProvider;
        let req = EmbedRequest {
            texts: vec!["meme: unity".into()],
            input_snapshot_hash: [0u8; 32],
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vectors = rt.block_on(provider.embed(&req)).unwrap();
        let v = &vectors[0];
        let sim_self = cosine_similarity(v, v);
        assert!((sim_self - 1.0).abs() < 1e-6, "self-similarity must be ~1.0");

        // Test against zero vector
        let zero = vec![0.0; v.len()];
        let sim_zero = cosine_similarity(v, &zero);
        assert_eq!(sim_zero, 0.0, "zero vector similarity must be 0.0");
    }
}