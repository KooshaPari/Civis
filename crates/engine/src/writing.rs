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
use std::collections::HashMap;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptType {
    Alphabetic,
    Logographic,
    Syllabic,
    Cuneiform,
    Hieroglyphic,
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

// ---------------------------------------------------------------------------
// Script Properties
// ---------------------------------------------------------------------------

/// Properties of a script type describing its characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptProperties {
    pub script_type: ScriptType,
    pub char_count: u32,
    pub learning_difficulty: f32,
    pub expressiveness: f32,
    pub prestige: f32,
}

/// Return the properties associated with a given script type.
///
/// Each script type has canonical values for character count, learning difficulty,
/// expressiveness, and prestige.
#[must_use]
pub fn script_properties(script_type: ScriptType) -> ScriptProperties {
    match script_type {
        ScriptType::Cuneiform => ScriptProperties {
            script_type,
            char_count: 600,
            learning_difficulty: 0.9,
            expressiveness: 0.4,
            prestige: 0.7,
        },
        ScriptType::Hieroglyphic => ScriptProperties {
            script_type,
            char_count: 700,
            learning_difficulty: 0.85,
            expressiveness: 0.6,
            prestige: 0.8,
        },
        ScriptType::Syllabic => ScriptProperties {
            script_type,
            char_count: 80,
            learning_difficulty: 0.6,
            expressiveness: 0.7,
            prestige: 0.6,
        },
        ScriptType::Alphabetic => ScriptProperties {
            script_type,
            char_count: 26,
            learning_difficulty: 0.3,
            expressiveness: 0.9,
            prestige: 0.5,
        },
        ScriptType::Logographic => ScriptProperties {
            script_type,
            char_count: 5000,
            learning_difficulty: 0.95,
            expressiveness: 0.8,
            prestige: 0.9,
        },
    }
}

/// Return the historical evolution order of script types.
///
/// Scripts historically evolve from more complex to simpler representations:
/// Hieroglyphic → Cuneiform → Logographic → Syllabic → Alphabetic.
#[must_use]
pub fn script_evolution_order() -> Vec<ScriptType> {
    vec![
        ScriptType::Hieroglyphic,
        ScriptType::Cuneiform,
        ScriptType::Logographic,
        ScriptType::Syllabic,
        ScriptType::Alphabetic,
    ]
}

// ---------------------------------------------------------------------------
// Literacy Tracker
// ---------------------------------------------------------------------------

/// Tracks per-faction literacy rates over time with teaching and decay mechanics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteracyTracker {
    faction_literacy: HashMap<u32, f32>,
    teaching_rate: f32,
    decay_rate: f32,
}

impl LiteracyTracker {
    /// Create a new literacy tracker with the given base teaching rate.
    ///
    /// Decay rate is set to 10% of the teaching rate by default.
    pub fn new(teaching_rate: f32) -> Self {
        Self {
            faction_literacy: HashMap::new(),
            teaching_rate,
            decay_rate: teaching_rate * 0.1,
        }
    }

    /// Update the literacy for a faction based on population and scholar count.
    ///
    /// Scholars contribute to literacy growth; non-scholars contribute to
    /// literacy decay (knowledge erosion). Literacy is clamped to [0.0, 1.0].
    pub fn update(&mut self, faction_id: u32, population: u32, scholars: u32) {
        let current = self
            .faction_literacy
            .get(&faction_id)
            .copied()
            .unwrap_or(0.0);

        if population == 0 {
            self.faction_literacy.insert(faction_id, 0.0);
            return;
        }

        let scholar_ratio = scholars as f32 / population as f32;
        let growth = self.teaching_rate * scholar_ratio;
        let non_scholar_ratio = 1.0 - scholar_ratio;
        let decay = self.decay_rate * non_scholar_ratio;

        let new_literacy = (current + growth - decay).clamp(0.0, 1.0);
        self.faction_literacy.insert(faction_id, new_literacy);
    }

    /// Get the current literacy rate for a faction.
    ///
    /// Returns 0.0 if the faction has no recorded literacy.
    #[must_use]
    pub fn get_literacy(&self, faction_id: u32) -> f32 {
        self.faction_literacy
            .get(&faction_id)
            .copied()
            .unwrap_or(0.0)
    }

    /// Compute the average literacy across all tracked factions.
    ///
    /// Returns 0.0 if no factions are tracked.
    #[must_use]
    pub fn average_literacy(&self) -> f32 {
        if self.faction_literacy.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.faction_literacy.values().sum();
        sum / self.faction_literacy.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Literature Generation
// ---------------------------------------------------------------------------

/// Genre of a literary work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteratureGenre {
    Epic,
    LawCode,
    Philosophy,
    History,
    Poetry,
}

/// A literary work produced by a faction's scholars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteraryWork {
    pub title: String,
    pub genre: LiteratureGenre,
    pub author: String,
    pub faction_id: u32,
    pub literacy_requirement: f32,
    pub prestige_value: f32,
    pub created_tick: u64,
}

/// Generate a literary work deterministically based on genre, faction, literacy, and seed.
///
/// The title and author are derived deterministically from the seed. Higher literacy
/// levels unlock higher-prestige works.
#[must_use]
pub fn generate_literary_work(
    genre: LiteratureGenre,
    faction_id: u32,
    literacy_level: f32,
    tick: u64,
    seed: u64,
) -> LiteraryWork {
    let combined = hash_combine(hash_combine(seed, faction_id.into()), tick);

    // Literacy requirement varies by genre
    let (literacy_req, base_prestige) = match genre {
        LiteratureGenre::Epic => (0.3, 0.8),
        LiteratureGenre::LawCode => (0.5, 0.9),
        LiteratureGenre::Philosophy => (0.7, 0.85),
        LiteratureGenre::History => (0.4, 0.6),
        LiteratureGenre::Poetry => (0.2, 0.5),
    };

    // Prestige scales with literacy level above requirement
    let prestige = if literacy_level >= literacy_req {
        let excess = (literacy_level - literacy_req).clamp(0.0, 1.0 - literacy_req);
        base_prestige + excess * 0.3
    } else {
        base_prestige * (literacy_level / literacy_req).clamp(0.0, 1.0)
    };

    // Deterministic title generation
    let title_prefix = match genre {
        LiteratureGenre::Epic => "The Epic of",
        LiteratureGenre::LawCode => "Code of",
        LiteratureGenre::Philosophy => "Meditations on",
        LiteratureGenre::History => "Chronicles of",
        LiteratureGenre::Poetry => "Odes of",
    };

    let suffix_index = (combined % 20) as usize;
    let suffixes = [
        "Kings",
        "Stars",
        "Rivers",
        "Mountains",
        "Thunder",
        "Dawn",
        "Fate",
        "Memory",
        "Fire",
        "Wisdom",
        "Valor",
        "Silence",
        "Harvest",
        "Tides",
        "Shadows",
        "Light",
        "Stone",
        "Wind",
        "Earth",
        "Sky",
    ];

    let faction_label = match faction_id % 5 {
        0 => "the First",
        1 => "the Ancient",
        2 => "the Eternal",
        3 => "the Wise",
        _ => "the Mighty",
    };

    let title = format!(
        "{} {} {}",
        title_prefix, faction_label, suffixes[suffix_index]
    );

    // Deterministic author name
    let author_names = [
        "Hammurabi",
        "Sappho",
        "Confucius",
        "Herodotus",
        "Virgil",
        "Hesiod",
        "Plato",
        "Socrates",
        "Sophocles",
        "Cicero",
        "Laozi",
        "Sun Tzu",
        "Vishnu",
        "Odin",
        "Isis",
    ];
    let author_index = (combined >> 8) % author_names.len() as u64;
    let author = author_names[author_index as usize].to_string();

    LiteraryWork {
        title,
        genre,
        author,
        faction_id,
        literacy_requirement: literacy_req,
        prestige_value: prestige.clamp(0.0, 1.0),
        created_tick: tick,
    }
}

/// Check whether a reader with the given literacy level can access a literary work.
///
/// Returns true if the reader's literacy meets or exceeds the work's requirement.
#[must_use]
pub fn can_access_work(work: &LiteraryWork, reader_literacy: f32) -> bool {
    reader_literacy >= work.literacy_requirement
}

// ---------------------------------------------------------------------------
// Codification Events
// ---------------------------------------------------------------------------

/// An event representing a law being written down in a specific writing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodificationEvent {
    pub law_name: String,
    pub writing_system: String,
    pub faction_id: u32,
    pub tick: u64,
    pub enforceability_boost: f32,
}

/// Codify (write down) a law, creating a codification event.
///
/// The enforceability boost scales with the current literacy level. Laws written
/// in systems with higher literacy are more enforceable.
#[must_use]
pub fn codify_law(
    law_name: &str,
    faction_id: u32,
    writing_system: &str,
    current_literacy: f32,
    tick: u64,
) -> CodificationEvent {
    // Base enforceability is 0.1; literacy adds up to 0.4 more
    let enforceability_boost = (0.1 + current_literacy * 0.4).clamp(0.0, 0.5);

    CodificationEvent {
        law_name: law_name.to_string(),
        writing_system: writing_system.to_string(),
        faction_id,
        tick,
        enforceability_boost,
    }
}

/// Compute the enforceability multiplier from a codification event.
///
/// Returns a value in [1.0, 1.5] representing the multiplier applied to
/// law enforcement in the faction after codification.
#[must_use]
pub fn codification_effect(event: &CodificationEvent) -> f32 {
    1.0 + event.enforceability_boost
}

// ---------------------------------------------------------------------------
// Printing Press
// ---------------------------------------------------------------------------

/// A printing press invention that boosts literacy through mass production of texts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintingPress {
    pub invention_tick: u64,
    pub production_rate: f32,
    pub literacy_multiplier: f32,
    pub distribution_range: u32,
}

/// Invent a printing press at a given tick with a base production rate.
///
/// The literacy multiplier is derived from the production rate (higher production
/// means more books, more readers). Distribution range is based on production rate
/// in abstract distance units.
#[must_use]
pub fn invent_printing_press(invention_tick: u64, base_production: f32) -> PrintingPress {
    let literacy_multiplier = 1.0 + (base_production * 0.5).clamp(0.0, 2.0);
    let distribution_range = (base_production * 100.0) as u32;

    PrintingPress {
        invention_tick,
        production_rate: base_production.clamp(0.01, 10.0),
        literacy_multiplier,
        distribution_range,
    }
}

/// Compute the literacy boost provided by a printing press.
///
/// The boost scales with population (more people = more potential readers) and
/// the press's literacy multiplier. Returns the absolute literacy points to add.
#[must_use]
pub fn press_literacy_boost(press: &PrintingPress, base_literacy: f32, population: u32) -> f32 {
    if population == 0 {
        return 0.0;
    }
    // Diminishing returns: boost decreases as base literacy increases
    let diminishing = (1.0 - base_literacy).clamp(0.0, 1.0);
    let population_factor = (population as f32).sqrt() / 1000.0;
    let raw_boost = press.literacy_multiplier * diminishing * population_factor;
    raw_boost.clamp(0.0, 0.1) // Cap at 0.1 per tick
}

/// Compute the cultural diffusion effect of a printing press at a given distance.
///
/// Effect decreases exponentially with distance from the press's origin.
/// Returns a multiplier in (0.0, 1.0] representing cultural influence.
#[must_use]
pub fn press_cultural_effect(press: &PrintingPress, distance: f32) -> f32 {
    if distance <= 0.0 {
        return press.literacy_multiplier.min(1.0);
    }
    let range = press.distribution_range as f32;
    if range <= 0.0 {
        return 0.0;
    }
    let normalized_distance = distance / range;
    // Exponential decay
    (-normalized_distance).exp().clamp(0.0, 1.0)
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

// ---------------------------------------------------------------------------
// Additional Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod script_properties_tests {
    use super::*;

    #[test]
    fn cuneiform_properties() {
        let props = script_properties(ScriptType::Cuneiform);
        assert_eq!(props.script_type, ScriptType::Cuneiform);
        assert_eq!(props.char_count, 600);
        assert!((props.learning_difficulty - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn hieroglyphic_properties() {
        let props = script_properties(ScriptType::Hieroglyphic);
        assert_eq!(props.script_type, ScriptType::Hieroglyphic);
        assert_eq!(props.char_count, 700);
        assert!((props.expressiveness - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn alphabetic_properties() {
        let props = script_properties(ScriptType::Alphabetic);
        assert_eq!(props.char_count, 26);
        assert!((props.learning_difficulty - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn syllabic_properties() {
        let props = script_properties(ScriptType::Syllabic);
        assert_eq!(props.char_count, 80);
        assert!((props.learning_difficulty - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn logographic_properties() {
        let props = script_properties(ScriptType::Logographic);
        assert_eq!(props.char_count, 5000);
        assert!((props.prestige - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn evolution_order_length() {
        let order = script_evolution_order();
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn evolution_order_starts_with_hieroglyphic() {
        let order = script_evolution_order();
        assert_eq!(order.first(), Some(&ScriptType::Hieroglyphic));
    }

    #[test]
    fn evolution_order_ends_with_alphabetic() {
        let order = script_evolution_order();
        assert_eq!(order.last(), Some(&ScriptType::Alphabetic));
    }

    #[test]
    fn evolution_order_no_duplicates() {
        let order = script_evolution_order();
        let mut unique = order.clone();
        unique.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        unique.dedup();
        assert_eq!(order.len(), unique.len());
    }
}

#[cfg(test)]
mod literacy_tracker_tests {
    use super::*;

    #[test]
    fn tracker_new_starts_empty() {
        let tracker = LiteracyTracker::new(0.1);
        assert_eq!(tracker.get_literacy(1), 0.0);
        assert_eq!(tracker.average_literacy(), 0.0);
    }

    #[test]
    fn tracker_update_increases_literacy() {
        let mut tracker = LiteracyTracker::new(0.2);
        tracker.update(1, 100, 50);
        let literacy = tracker.get_literacy(1);
        assert!(literacy > 0.0, "literacy should increase after update");
    }

    #[test]
    fn tracker_update_zero_population() {
        let mut tracker = LiteracyTracker::new(0.2);
        tracker.update(1, 0, 0);
        assert_eq!(tracker.get_literacy(1), 0.0);
    }

    #[test]
    fn tracker_average_multiple_factions() {
        let mut tracker = LiteracyTracker::new(0.3);
        tracker.update(1, 100, 50);
        tracker.update(2, 200, 100);
        let avg = tracker.average_literacy();
        assert!(
            avg > 0.0,
            "average should be positive with multiple factions"
        );
    }

    #[test]
    fn tracker_per_faction_isolation() {
        let mut tracker = LiteracyTracker::new(0.2);
        tracker.update(1, 100, 100);
        // Faction 2 should be unaffected
        assert_eq!(tracker.get_literacy(2), 0.0);
        // Faction 1 should have literacy
        assert!(tracker.get_literacy(1) > 0.0);
    }

    #[test]
    fn tracker_literacy_clamped() {
        let mut tracker = LiteracyTracker::new(1.0);
        for _ in 0..100 {
            tracker.update(1, 100, 100);
        }
        let lit = tracker.get_literacy(1);
        assert!(lit <= 1.0, "literacy must not exceed 1.0, got {}", lit);
    }

    #[test]
    fn tracker_unknown_faction_returns_zero() {
        let tracker = LiteracyTracker::new(0.1);
        assert_eq!(tracker.get_literacy(999), 0.0);
    }
}

#[cfg(test)]
mod literature_tests {
    use super::*;

    #[test]
    fn generate_epic_work() {
        let work = generate_literary_work(LiteratureGenre::Epic, 1, 0.5, 100, 42);
        assert_eq!(work.genre, LiteratureGenre::Epic);
        assert_eq!(work.faction_id, 1);
        assert_eq!(work.created_tick, 100);
        assert!(!work.title.is_empty());
        assert!(!work.author.is_empty());
    }

    #[test]
    fn generate_law_code_work() {
        let work = generate_literary_work(LiteratureGenre::LawCode, 2, 0.6, 200, 7);
        assert_eq!(work.genre, LiteratureGenre::LawCode);
        assert!((work.literacy_requirement - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn generate_philosophy_work() {
        let work = generate_literary_work(LiteratureGenre::Philosophy, 3, 0.8, 300, 99);
        assert_eq!(work.genre, LiteratureGenre::Philosophy);
        assert!((work.literacy_requirement - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn generate_history_work() {
        let work = generate_literary_work(LiteratureGenre::History, 4, 0.5, 400, 12);
        assert_eq!(work.genre, LiteratureGenre::History);
    }

    #[test]
    fn generate_poetry_work() {
        let work = generate_literary_work(LiteratureGenre::Poetry, 5, 0.3, 500, 33);
        assert_eq!(work.genre, LiteratureGenre::Poetry);
        assert!((work.literacy_requirement - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn can_access_work_sufficient_literacy() {
        let work = generate_literary_work(LiteratureGenre::Epic, 1, 0.5, 100, 42);
        assert!(can_access_work(&work, 0.5));
        assert!(can_access_work(&work, 0.8));
    }

    #[test]
    fn can_access_work_insufficient_literacy() {
        let work = generate_literary_work(LiteratureGenre::Philosophy, 1, 0.8, 100, 42);
        assert!(!can_access_work(&work, 0.3));
        assert!(!can_access_work(&work, 0.0));
    }

    #[test]
    fn generate_literary_work_deterministic() {
        let a = generate_literary_work(LiteratureGenre::Epic, 1, 0.5, 100, 42);
        let b = generate_literary_work(LiteratureGenre::Epic, 1, 0.5, 100, 42);
        assert_eq!(a.title, b.title);
        assert_eq!(a.author, b.author);
        assert_eq!(a.prestige_value, b.prestige_value);
    }

    #[test]
    fn literary_work_prestige_scales_with_literacy() {
        let low = generate_literary_work(LiteratureGenre::Epic, 1, 0.3, 100, 42);
        let high = generate_literary_work(LiteratureGenre::Epic, 1, 0.9, 100, 42);
        assert!(
            high.prestige_value >= low.prestige_value,
            "higher literacy should yield higher prestige"
        );
    }
}

#[cfg(test)]
mod codification_tests {
    use super::*;

    #[test]
    fn codify_law_creates_event() {
        let event = codify_law("The Twelve Tables", 1, "Cuneiform", 0.4, 500);
        assert_eq!(event.law_name, "The Twelve Tables");
        assert_eq!(event.faction_id, 1);
        assert_eq!(event.tick, 500);
        assert!(event.enforceability_boost > 0.0);
    }

    #[test]
    fn codify_law_enforceability_scales_with_literacy() {
        let low = codify_law("Law A", 1, "Greek", 0.1, 100);
        let high = codify_law("Law A", 1, "Greek", 0.9, 100);
        assert!(
            high.enforceability_boost > low.enforceability_boost,
            "higher literacy should yield higher enforceability"
        );
    }

    #[test]
    fn codification_effect_multiplier() {
        let event = codify_law("Test Law", 1, "Greek", 0.5, 100);
        let effect = codification_effect(&event);
        assert!((effect - (1.0 + event.enforceability_boost)).abs() < f32::EPSILON);
        assert!(effect >= 1.0);
        assert!(effect <= 1.5);
    }

    #[test]
    fn codification_effect_always_at_least_one() {
        let event = codify_law("Test Law", 1, "Greek", 0.0, 100);
        let effect = codification_effect(&event);
        assert!(effect >= 1.0);
    }
}

#[cfg(test)]
mod printing_press_tests {
    use super::*;

    #[test]
    fn invent_press_creates_valid_press() {
        let press = invent_printing_press(1000, 2.0);
        assert_eq!(press.invention_tick, 1000);
        assert!((press.production_rate - 2.0).abs() < f32::EPSILON);
        assert!(press.literacy_multiplier >= 1.0);
        assert!(press.distribution_range > 0);
    }

    #[test]
    fn press_literacy_boost_zero_population() {
        let press = invent_printing_press(0, 1.0);
        let boost = press_literacy_boost(&press, 0.5, 0);
        assert_eq!(boost, 0.0);
    }

    #[test]
    fn press_literacy_boost_positive() {
        let press = invent_printing_press(0, 2.0);
        let boost = press_literacy_boost(&press, 0.3, 1000);
        assert!(
            boost > 0.0,
            "boost should be positive for non-zero population"
        );
    }

    #[test]
    fn press_literacy_boost_capped() {
        let press = invent_printing_press(0, 10.0);
        let boost = press_literacy_boost(&press, 0.0, 100_000);
        assert!(boost <= 0.1, "boost should be capped at 0.1, got {}", boost);
    }

    #[test]
    fn press_literacy_boost_diminishes_with_high_literacy() {
        let press = invent_printing_press(0, 2.0);
        let low_lit = press_literacy_boost(&press, 0.1, 1000);
        let high_lit = press_literacy_boost(&press, 0.9, 1000);
        assert!(
            low_lit >= high_lit,
            "boost should decrease as base literacy increases"
        );
    }

    #[test]
    fn press_cultural_effect_at_origin() {
        let press = invent_printing_press(0, 2.0);
        let effect = press_cultural_effect(&press, 0.0);
        assert!(effect > 0.0);
        assert!(effect <= 1.0);
    }

    #[test]
    fn press_cultural_effect_decreases_with_distance() {
        let press = invent_printing_press(0, 2.0);
        let near = press_cultural_effect(&press, 10.0);
        let far = press_cultural_effect(&press, 500.0);
        assert!(near > far, "effect should decrease with distance");
    }

    #[test]
    fn press_cultural_effect_at_max_distance() {
        let press = invent_printing_press(0, 1.0);
        let range = press.distribution_range as f32;
        let effect = press_cultural_effect(&press, range * 10.0);
        assert!(
            (effect - 0.0).abs() < 0.01,
            "effect should be near zero at far distance"
        );
    }

    #[test]
    fn press_production_rate_clamped() {
        let press = invent_printing_press(0, 100.0);
        assert!(press.production_rate <= 10.0);
        let press2 = invent_printing_press(0, -5.0);
        assert!(press2.production_rate >= 0.01);
    }
}
