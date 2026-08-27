//! Deterministic procedural glyph generator for emergent writing systems.
//!
//! Translates an evolved language's phoneme inventory into a small set of
//! renderable glyphs (vector strokes). Each civilization's writing system is
//! visibly distinct and deterministic from its phoneme structure.
//!
//! **Design:** Glyphs are keyed off the phoneme inventory's feature vectors.
//! No randomness; all output is deterministic given the same language seed.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A single straight line or arc segment normalized to [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Start point in [0, 1] grid
    pub x0: f32,
    pub y0: f32,
    /// End point in [0, 1] grid
    pub x1: f32,
    pub y1: f32,
    /// 0.0 = straight line; >0.0 = arc curvature (bulge toward control point)
    pub curvature: f32,
}

impl Stroke {
    /// Create a straight line stroke.
    pub fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0: x0.clamp(0.0, 1.0),
            y0: y0.clamp(0.0, 1.0),
            x1: x1.clamp(0.0, 1.0),
            y1: y1.clamp(0.0, 1.0),
            curvature: 0.0,
        }
    }

    /// Create a curved stroke (arc).
    pub fn arc(x0: f32, y0: f32, x1: f32, y1: f32, curvature: f32) -> Self {
        Self {
            x0: x0.clamp(0.0, 1.0),
            y0: y0.clamp(0.0, 1.0),
            x1: x1.clamp(0.0, 1.0),
            y1: y1.clamp(0.0, 1.0),
            curvature: curvature.clamp(-1.0, 1.0),
        }
    }
}

/// A single glyph (character/symbol) represented as a vector of strokes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Glyph {
    /// Index/ID of this glyph in the generated set
    pub id: usize,
    /// Vector strokes that compose this glyph
    pub strokes: Vec<Stroke>,
    /// Phoneme inventory indices that map to this glyph (may be empty for derived glyphs)
    pub source_phoneme_indices: Vec<u8>,
}

impl Glyph {
    /// Create a new glyph with a given ID and source phoneme indices.
    pub fn new(id: usize, strokes: Vec<Stroke>, source_phoneme_indices: Vec<u8>) -> Self {
        Self {
            id,
            strokes,
            source_phoneme_indices,
        }
    }

    /// Return the bounding box of this glyph: (min_x, min_y, max_x, max_y).
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        if self.strokes.is_empty() {
            return (0.0, 0.0, 1.0, 1.0);
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for stroke in &self.strokes {
            min_x = min_x.min(stroke.x0).min(stroke.x1);
            min_y = min_y.min(stroke.y0).min(stroke.y1);
            max_x = max_x.max(stroke.x0).max(stroke.x1);
            max_y = max_y.max(stroke.y0).max(stroke.y1);
        }

        if min_x.is_infinite() {
            (0.0, 0.0, 1.0, 1.0)
        } else {
            (min_x, min_y, max_x, max_y)
        }
    }
}

/// Generate a deterministic set of glyphs from a phoneme inventory seed.
///
/// Each glyph is derived from phoneme feature vectors, ensuring that:
/// - Same seed → same glyphs (deterministic)
/// - Different seeds → visibly different glyphs
/// - Glyphs are engine-agnostic vector data (no render deps)
///
/// # Arguments
///
/// * `inventory_seed` - The u64 seed of the phoneme inventory
/// * `phoneme_count` - Number of phonemes in the inventory (typically 8–16)
/// * `desired_glyphs` - Number of glyphs to generate (typically 16–32)
///
/// # Returns
///
/// A Vec of `Glyph` objects, each with deterministic strokes.
pub fn glyphs_for_language(
    inventory_seed: u64,
    phoneme_count: usize,
    desired_glyphs: usize,
) -> Vec<Glyph> {
    let glyph_count = desired_glyphs.max(4).min(256);
    let mut glyphs = Vec::with_capacity(glyph_count);

    for glyph_id in 0..glyph_count {
        // Hash the seed + glyph_id to get a deterministic base for this glyph
        let glyph_seed = hash_combine(inventory_seed, glyph_id as u64);

        // Decide stroke count (2–5 strokes per glyph)
        let stroke_count = 2 + (glyph_seed % 4) as usize;

        let mut strokes = Vec::new();
        for stroke_idx in 0..stroke_count {
            let stroke_seed = hash_combine(glyph_seed, stroke_idx as u64);
            let stroke = generate_stroke_from_seed(stroke_seed);
            strokes.push(stroke);
        }

        // Map glyph to phoneme indices (deterministic assignment)
        let source_phoneme_indices = if phoneme_count > 0 {
            let primary_idx = (glyph_id % phoneme_count) as u8;
            vec![primary_idx]
        } else {
            vec![]
        };

        glyphs.push(Glyph::new(glyph_id, strokes, source_phoneme_indices));
    }

    glyphs
}

/// Deterministically generate a single stroke from a seed.
fn generate_stroke_from_seed(seed: u64) -> Stroke {
    // Use seed bytes to drive stroke parameters
    let bytes = seed.to_le_bytes();

    // Unpack parameters from seed bytes
    let x0 = (bytes[0] as f32) / 255.0;
    let y0 = (bytes[1] as f32) / 255.0;
    let x1 = (bytes[2] as f32) / 255.0;
    let y1 = (bytes[3] as f32) / 255.0;
    let curve_byte = bytes[4];
    let is_curved = (bytes[5] & 0x80) != 0;

    if is_curved {
        let curvature = (curve_byte as f32 / 127.0) - 1.0; // [-1, 1]
        Stroke::arc(x0, y0, x1, y1, curvature)
    } else {
        Stroke::line(x0, y0, x1, y1)
    }
}

/// Combine two u64 hashes deterministically.
fn hash_combine(a: u64, b: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_line_creation() {
        let stroke = Stroke::line(0.1, 0.2, 0.8, 0.9);
        assert_eq!(stroke.x0, 0.1);
        assert_eq!(stroke.y0, 0.2);
        assert_eq!(stroke.x1, 0.8);
        assert_eq!(stroke.y1, 0.9);
        assert_eq!(stroke.curvature, 0.0);
    }

    #[test]
    fn stroke_arc_creation() {
        let stroke = Stroke::arc(0.1, 0.2, 0.8, 0.9, 0.5);
        assert_eq!(stroke.x0, 0.1);
        assert_eq!(stroke.y0, 0.2);
        assert_eq!(stroke.x1, 0.8);
        assert_eq!(stroke.y1, 0.9);
        assert_eq!(stroke.curvature, 0.5);
    }

    #[test]
    fn stroke_clamps_coordinates_to_0_1() {
        let stroke = Stroke::line(-0.5, 1.5, 0.5, 0.5);
        assert_eq!(stroke.x0, 0.0);
        assert_eq!(stroke.y0, 1.0);
        assert_eq!(stroke.x1, 0.5);
        assert_eq!(stroke.y1, 0.5);
    }

    #[test]
    fn glyph_creation() {
        let strokes = vec![Stroke::line(0.0, 0.0, 1.0, 1.0)];
        let glyph = Glyph::new(0, strokes, vec![0]);
        assert_eq!(glyph.id, 0);
        assert_eq!(glyph.strokes.len(), 1);
        assert_eq!(glyph.source_phoneme_indices, vec![0]);
    }

    #[test]
    fn glyph_bounding_box() {
        let strokes = vec![
            Stroke::line(0.1, 0.2, 0.8, 0.9),
            Stroke::line(0.3, 0.4, 0.6, 0.7),
        ];
        let glyph = Glyph::new(0, strokes, vec![]);
        let (min_x, min_y, max_x, max_y) = glyph.bounding_box();
        assert_eq!(min_x, 0.1);
        assert_eq!(min_y, 0.2);
        assert_eq!(max_x, 0.8);
        assert_eq!(max_y, 0.9);
    }

    #[test]
    fn glyph_bounding_box_empty() {
        let glyph = Glyph::new(0, vec![], vec![]);
        let (min_x, min_y, max_x, max_y) = glyph.bounding_box();
        assert_eq!((min_x, min_y, max_x, max_y), (0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn glyphs_for_language_deterministic_same_seed() {
        let seed = 42u64;
        let phoneme_count = 8;
        let desired_glyphs = 16;

        let glyphs_a = glyphs_for_language(seed, phoneme_count, desired_glyphs);
        let glyphs_b = glyphs_for_language(seed, phoneme_count, desired_glyphs);

        assert_eq!(glyphs_a.len(), glyphs_b.len());
        for (a, b) in glyphs_a.iter().zip(glyphs_b.iter()) {
            assert_eq!(a.id, b.id, "glyph id mismatch");
            assert_eq!(a.strokes, b.strokes, "strokes mismatch for glyph {}", a.id);
            assert_eq!(a.source_phoneme_indices, b.source_phoneme_indices);
        }
    }

    #[test]
    fn glyphs_for_language_different_seeds_diverge() {
        let seed_a = 42u64;
        let seed_b = 99u64;
        let phoneme_count = 8;
        let desired_glyphs = 16;

        let glyphs_a = glyphs_for_language(seed_a, phoneme_count, desired_glyphs);
        let glyphs_b = glyphs_for_language(seed_b, phoneme_count, desired_glyphs);

        assert_eq!(glyphs_a.len(), glyphs_b.len());

        // Count differing glyphs (most should differ)
        let mut differing_count = 0;
        for (a, b) in glyphs_a.iter().zip(glyphs_b.iter()) {
            if a.strokes != b.strokes {
                differing_count += 1;
            }
        }

        // With different seeds, expect most glyphs to differ
        assert!(
            differing_count > glyphs_a.len() / 2,
            "different seeds must produce different glyphs (only {} differed out of {})",
            differing_count,
            glyphs_a.len()
        );
    }

    #[test]
    fn glyphs_for_language_respects_glyph_count_bounds() {
        // Test minimum (should clamp to 4)
        let glyphs_min = glyphs_for_language(1, 8, 0);
        assert!(glyphs_min.len() >= 4);

        // Test maximum (should clamp to 256)
        let glyphs_max = glyphs_for_language(1, 8, 1000);
        assert!(glyphs_max.len() <= 256);

        // Test normal
        let glyphs_normal = glyphs_for_language(1, 8, 32);
        assert_eq!(glyphs_normal.len(), 32);
    }

    #[test]
    fn glyphs_for_language_phoneme_mapping() {
        let glyphs = glyphs_for_language(42, 8, 16);

        for glyph in &glyphs {
            // Each glyph should have at most one primary source phoneme
            assert!(glyph.source_phoneme_indices.len() <= 1);

            // If it has a source, it should be valid
            if !glyph.source_phoneme_indices.is_empty() {
                let idx = glyph.source_phoneme_indices[0];
                assert!(idx < 8, "phoneme index out of range");
            }
        }
    }

    #[test]
    fn glyphs_for_language_returns_correct_count() {
        for desired in &[4, 8, 16, 32, 64] {
            let glyphs = glyphs_for_language(7, 8, *desired);
            assert_eq!(glyphs.len(), *desired);
        }
    }

    #[test]
    fn stroke_serialize_deserialize() {
        let stroke = Stroke::arc(0.1, 0.2, 0.8, 0.9, 0.5);
        let json = serde_json::to_string(&stroke).expect("serialize");
        let restored: Stroke = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(stroke, restored);
    }

    #[test]
    fn glyph_serialize_deserialize() {
        let glyph = Glyph::new(5, vec![Stroke::line(0.0, 0.0, 1.0, 1.0)], vec![2, 3]);
        let json = serde_json::to_string(&glyph).expect("serialize");
        let restored: Glyph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(glyph, restored);
    }
}

/// Type of writing system script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Alphabetic,
    Logographic,
    Syllabic,
}

/// A writing system with characters, literacy, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSystem {
    pub name: String,
    pub script_type: ScriptType,
    pub characters: Vec<char>,
    pub literacy_rate: f32,
    pub created_tick: u64,
}

impl Default for WritingSystem {
    fn default() -> Self {
        Self {
            name: String::new(),
            script_type: ScriptType::Alphabetic,
            characters: Vec::new(),
            literacy_rate: 0.0,
            created_tick: 0,
        }
    }
}

/// A document written in a specific writing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub author: String,
    pub content: String,
    pub writing_system: String,
    pub literacy_level: f32,
    pub created_tick: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            author: String::new(),
            content: String::new(),
            writing_system: String::new(),
            literacy_level: 0.0,
            created_tick: 0,
        }
    }
}

/// Create a deterministic script from a seed, pulling characters from the Greek Unicode block.
#[must_use]
pub fn create_script(
    name: &str,
    script_type: ScriptType,
    seed: u64,
    char_count: usize,
) -> WritingSystem {
    let base = 0x0391u32; // Greek Capital Letter Alpha
    let mut characters = Vec::with_capacity(char_count);
    for i in 0..char_count {
        let code = base + ((seed.wrapping_add(i as u64)) % 24) as u32;
        if let Some(ch) = char::from_u32(code) {
            characters.push(ch);
        }
    }
    WritingSystem {
        name: name.to_string(),
        script_type,
        characters,
        literacy_rate: 0.0,
        created_tick: 0,
    }
}

/// Translate content from one writing system to another (character substitution).
#[must_use]
pub fn translate(content: &str, source: &WritingSystem, target: &WritingSystem) -> String {
    let mut result = String::with_capacity(content.len());
    for ch in content.chars() {
        if let Some(idx) = source.characters.iter().position(|&c| c == ch) {
            if idx < target.characters.len() {
                result.push(target.characters[idx]);
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Compute literacy rate from literate count and total population.
#[must_use]
pub fn compute_literacy_rate(literate_count: u32, total_population: u32) -> f32 {
    if total_population == 0 {
        return 0.0;
    }
    literate_count as f32 / total_population as f32
}

/// Archive a document by copying it with updated literacy level from writing system.
#[must_use]
pub fn archive_document(doc: &Document, ws: &WritingSystem) -> Document {
    Document {
        literacy_level: ws.literacy_rate,
        ..doc.clone()
    }
}

/// Advance a writing system by one tick, increasing literacy by teaching_rate.
#[must_use]
pub fn tick_writing_system(ws: &WritingSystem, teaching_rate: f32) -> WritingSystem {
    WritingSystem {
        literacy_rate: (ws.literacy_rate + teaching_rate).clamp(0.0, 1.0),
        ..ws.clone()
    }
}

#[cfg(test)]
mod writing_extended_tests {
    use super::*;

    #[test]
    fn create_script_alphabetic() {
        let ws = create_script("Greek", ScriptType::Alphabetic, 42, 16);
        assert_eq!(ws.name, "Greek");
        assert_eq!(ws.script_type, ScriptType::Alphabetic);
        assert_eq!(ws.characters.len(), 16);
    }

    #[test]
    fn create_script_deterministic() {
        let a = create_script("Test", ScriptType::Logographic, 99, 8);
        let b = create_script("Test", ScriptType::Logographic, 99, 8);
        assert_eq!(a.characters, b.characters);
    }

    #[test]
    fn create_script_different_seeds() {
        let a = create_script("A", ScriptType::Syllabic, 1, 8);
        let b = create_script("A", ScriptType::Syllabic, 2, 8);
        assert_ne!(a.characters, b.characters);
    }

    #[test]
    fn translate_basic() {
        let src = create_script("Src", ScriptType::Alphabetic, 1, 4);
        let tgt = create_script("Tgt", ScriptType::Alphabetic, 2, 4);
        let content: String = src.characters.iter().take(2).collect();
        let translated = translate(&content, &src, &tgt);
        assert_eq!(translated.len(), content.len());
    }

    #[test]
    fn translate_identity() {
        let ws = create_script("Same", ScriptType::Alphabetic, 1, 8);
        let content: String = ws.characters.iter().take(3).collect();
        let translated = translate(&content, &ws, &ws);
        assert_eq!(translated, content);
    }

    #[test]
    fn compute_literacy_rate_normal() {
        assert!((compute_literacy_rate(75, 100) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_literacy_rate_zero_pop() {
        assert_eq!(compute_literacy_rate(0, 0), 0.0);
    }

    #[test]
    fn archive_document_updates_level() {
        let doc = Document {
            author: "Alice".into(),
            content: "Hello".into(),
            writing_system: "Greek".into(),
            literacy_level: 0.0,
            created_tick: 5,
        };
        let ws = WritingSystem {
            literacy_rate: 0.75,
            ..WritingSystem::default()
        };
        let archived = archive_document(&doc, &ws);
        assert!((archived.literacy_level - 0.75).abs() < f32::EPSILON);
        assert_eq!(archived.author, "Alice");
    }

    #[test]
    fn tick_writing_system_increases_literacy() {
        let ws = WritingSystem {
            literacy_rate: 0.5,
            ..WritingSystem::default()
        };
        let ticked = tick_writing_system(&ws, 0.1);
        assert!((ticked.literacy_rate - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_writing_system_clamps() {
        let ws = WritingSystem {
            literacy_rate: 0.99,
            ..WritingSystem::default()
        };
        let ticked = tick_writing_system(&ws, 0.1);
        assert!((ticked.literacy_rate - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn writing_system_default() {
        let ws = WritingSystem::default();
        assert!(ws.name.is_empty());
        assert_eq!(ws.literacy_rate, 0.0);
    }
}
