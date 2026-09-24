//! NFR-CIV-MAINT-004 — doc-comment coverage for the audio substrate.
//!
//! The crate denies `missing_docs`, so compiling `civ-audio` at all proves
//! every publicly reachable item carries a rustdoc comment. These tests pin
//! the enforcement attributes on the crate root and independently scan the
//! substrate sources for public items whose doc comment is missing.

const LIB_RS: &str = include_str!("../src/lib.rs");

const MODULE_SOURCES: &[(&str, &str)] = &[
    ("ambient", include_str!("../src/ambient.rs")),
    ("bus", include_str!("../src/bus.rs")),
    ("ducking", include_str!("../src/ducking.rs")),
    ("mix", include_str!("../src/mix.rs")),
    ("mood", include_str!("../src/mood.rs")),
    ("placeholder", include_str!("../src/placeholder.rs")),
    ("sfx", include_str!("../src/sfx.rs")),
    ("triggers", include_str!("../src/triggers.rs")),
    ("ui_sound", include_str!("../src/ui_sound.rs")),
];

// NFR-CIV-MAINT-004 — the crate root denies missing_docs (and unsafe_code), so
// `cargo test -p civ-audio` cannot even link unless every public item is
// documented, and the ID stays discoverable for the traceability audit.
#[test]
fn nfr_civ_maint_004_lib_root_denies_missing_docs() {
    assert!(
        LIB_RS.contains("#![deny(missing_docs)]"),
        "crates/audio/src/lib.rs must keep #![deny(missing_docs)] (NFR-CIV-MAINT-004)"
    );
    assert!(
        LIB_RS.contains("#![deny(unsafe_code)]"),
        "crates/audio/src/lib.rs must keep #![deny(unsafe_code)]"
    );
    assert!(
        LIB_RS.contains("NFR-CIV-MAINT-004"),
        "the crate root must document the NFR-CIV-MAINT-004 doc-coverage contract"
    );
}

// NFR-CIV-MAINT-004 — every public module opens with `//!` module docs and
// every public item in the substrate sources carries a doc comment directly
// above it (attributes in between do not break the association).
#[test]
fn nfr_civ_maint_004_public_items_carry_rustdoc() {
    assert!(
        LIB_RS
            .lines()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.starts_with("//!")),
        "lib.rs must open with a //! module doc"
    );

    let mut undocumented = Vec::new();
    for (module, source) in MODULE_SOURCES {
        assert!(
            source
                .lines()
                .find(|line| !line.trim().is_empty())
                .is_some_and(|line| line.starts_with("//!")),
            "src/{module}.rs must open with a //! module doc"
        );
        undocumented.extend(undocumented_pub_items(module, source));
    }
    undocumented.extend(undocumented_pub_items("lib", LIB_RS));
    assert!(
        undocumented.is_empty(),
        "public items missing rustdoc comments (NFR-CIV-MAINT-004): {undocumented:?}"
    );
}

/// Public item declarations (`pub fn` / `pub struct` / ...) with no doc
/// comment attached. Walks each line keeping a `doc_seen` flag that survives
/// attribute lines, blank lines, and plain `//` comments, and resets it on any
/// other code line — mirroring how rustdoc associates `///` with the next item.
fn undocumented_pub_items(module: &str, source: &str) -> Vec<String> {
    const ITEM_PREFIXES: &[&str] = &[
        "pub fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub const ",
        "pub type ",
        "pub static ",
    ];

    let mut undocumented = Vec::new();
    let mut doc_seen = false;
    let mut attr_depth: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Attributes (outer `#[..]` or inner `#![..]`) may wrap across lines
        // and never break the doc → item association.
        if attr_depth > 0 || trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            attr_depth += trimmed.matches('[').count() as i32;
            attr_depth -= trimmed.matches(']').count() as i32;
            if attr_depth <= 0 {
                attr_depth = 0;
            }
            continue;
        }
        if trimmed.starts_with("///") || trimmed.starts_with("#[doc") {
            doc_seen = true;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("pub mod ") && trimmed.ends_with(';') {
            // Module docs live as `//!` inside the module file (checked above).
            doc_seen = false;
            continue;
        }
        if trimmed.starts_with("pub use ") {
            doc_seen = false;
            continue;
        }

        let is_pub_item = ITEM_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix));
        if is_pub_item {
            if !doc_seen {
                undocumented.push(format!("{module}.rs: {trimmed}"));
            }
            doc_seen = false;
            continue;
        }
        // Any other code line ends the current doc association.
        doc_seen = false;
    }
    undocumented
}
