# RND-016: SVG Procedural Generation Pipeline — resvg Validation and DOM Edit Tooling

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

**resvg** (pure Rust, no system dependencies) is the recommended SVG rendering library for CivLab's procedural icon/UI generation pipeline. It supports gradients, patterns, filters (including feGaussianBlur, feColorMatrix), text with embedded fonts, and clipping/masking — sufficient for game UI icons and procedural badge/emblem generation. For SVG parsing, use **roxmltree** (read-only, fastest) for analysis/inspection and **xmltree** or direct string templating for SVG mutation (attribute changes, element insertion). The alternative **librsvg** offers marginally better SVG spec coverage but introduces a C dependency (cairo, glib) that complicates cross-platform builds. Decision: **resvg + roxmltree (read) + string templating (write) for the SVG mutation pipeline**.

---

## Research Findings

### 1. resvg — Pure Rust SVG Renderer

#### Overview

resvg is an SVG rendering library written entirely in Rust. It uses tiny-skia for software rasterization and usvg for SVG parsing/simplification. The library aims to support the static SVG subset (no animations, scripting, or interactive elements).

- **Repository:** https://github.com/linebender/resvg
- **Current version:** 0.45.x (as of Feb 2026)
- **License:** MPL-2.0
- **Rendering backend:** tiny-skia (pure Rust software rasterizer)
- **SVG parser:** usvg (converts SVG to simplified render tree)

#### Architecture

resvg separates SVG processing into two distinct phases:

```
SVG File
   │
   ▼
┌──────────┐
│   usvg   │  Parse SVG → Simplified render tree
│          │  - Resolves styles/attributes
│          │  - Converts shapes to paths
│          │  - Removes invisible elements
│          │  - Resolves <use> references
│          │  - Handles CSS cascading
└────┬─────┘
     │
     ▼
┌──────────┐
│  resvg   │  Render tree → Pixel buffer
│          │  - Rasterization via tiny-skia
│          │  - Filter effects
│          │  - Compositing
│          │  - Anti-aliasing
└──────────┘
     │
     ▼
  PNG/RGBA buffer
```

This separation means:
1. usvg can be used standalone for SVG analysis/transformation without rendering
2. resvg receives a clean, simplified tree — no ambiguity in rendering
3. Cross-platform reproducibility: same SVG produces identical pixels on x86 Windows, ARM macOS, and Linux

#### SVG Feature Support Matrix

| SVG Feature | Supported | Notes |
|-------------|-----------|-------|
| **Basic shapes** (rect, circle, ellipse, line, polyline, polygon) | YES | Converted to paths by usvg |
| **Paths** (d attribute, all commands) | YES | Full path data support |
| **Gradients** (linearGradient, radialGradient) | YES | Including spreadMethod, gradientTransform |
| **Patterns** | YES | Pattern fills and strokes |
| **Clipping** (clipPath) | YES | |
| **Masking** (mask) | YES | Luminance and alpha masks |
| **Opacity** | YES | Element and group opacity |
| **Transforms** | YES | All transform types |
| **Text** | YES | With embedded font support (not system fonts) |
| **Filters** | YES | See filter details below |
| **Markers** | YES | |
| **Symbols** | YES | Resolved by usvg |
| **use** (local refs) | YES | Resolved by usvg |
| **use** (external refs) | NO | External SVG file references not supported |
| **Images** (embedded) | YES | Base64 PNG/JPEG in SVG |
| **Images** (external) | YES | File path references |
| **CSS** (inline, style element) | YES | Cascade resolution in usvg |
| **CSS** (external stylesheets) | NO | Not supported |
| **Color fonts** (Emoji) | YES | COLRv0, COLRv1 (mostly), sbix, CBDT, SVG tables |
| **Viewport/viewBox** | YES | |
| **preserveAspectRatio** | YES | |

#### Filter Support Details

| Filter Primitive | Supported | Notes |
|------------------|-----------|-------|
| **feGaussianBlur** | YES | Single-threaded IIR blur (slower than librsvg's box blur) |
| **feColorMatrix** | YES | All matrix types |
| **feComponentTransfer** | YES | |
| **feComposite** | YES | All operators |
| **feMerge** | YES | |
| **feOffset** | YES | |
| **feFlood** | YES | |
| **feBlend** | YES | All blend modes |
| **feMorphology** | YES | Erode and dilate |
| **feDisplacementMap** | YES | |
| **feTurbulence** | YES | Perlin and fractal noise |
| **feDiffuseLighting** | YES | |
| **feSpecularLighting** | YES | |
| **feImage** | YES | |
| **feTile** | YES | |
| **feConvolveMatrix** | YES | |
| **feDropShadow** | YES | SVG 2 feature |

This filter coverage is sufficient for game UI effects like:
- Drop shadows on icons (feDropShadow or feGaussianBlur + feOffset)
- Color tinting for faction-specific icons (feColorMatrix)
- Glow effects (feGaussianBlur + feComposite)
- Emboss/bevel on badges (feDiffuseLighting + feSpecularLighting)
- Noise textures for procedural backgrounds (feTurbulence)

#### Unsupported Features

The following are explicitly **not supported** in resvg:

**Elements:**
- `altGlyph`, `altGlyphDef`, `altGlyphItem` (deprecated font elements)
- `font`, `font-face`, `glyph`, `missing-glyph` (SVG fonts — use TrueType/OpenType instead)
- `color-profile` (deprecated)
- `use` with external SVG file references

**Attributes:**
- `clip` (deprecated in SVG 2)
- `color-interpolation`, `color-profile`, `color-rendering`
- `direction`, `unicode-bidi` (complex text layout)
- `font-size-adjust`, `font-stretch`
- `glyph-orientation-horizontal/vertical` (removed/deprecated in SVG 2)
- `kerning` (removed in SVG 2)

**Interactive/Dynamic:**
- Animations (SMIL `<animate>`, `<animateTransform>`, etc.)
- Scripting (`<script>`)
- Events (onclick, onload, etc.)
- Cursor (`<cursor>`)
- Links (`<a>`)

None of these unsupported features are relevant to CivLab's static icon/UI generation use case.

#### Performance

**General characteristics:**
- Pure Rust, single-threaded software rasterization (tiny-skia)
- No system library dependencies — fully self-contained
- Cross-platform identical output (bit-for-bit reproducibility)
- ~1600 regression tests in the test suite

**Benchmark data:**

From resvg-js (NAPI bindings to resvg):
- ~39.6 ops/s for SVG-to-PNG conversion (general SVG files)
- 3.6x faster than sharp for the same task

For the paris-30k.svg benchmark (30,000 layers):
- Before optimization: ~33,760ms
- After layer bounding box optimization: ~290ms (115x faster)

**Estimated performance for CivLab 64x64 icons on M3:**

resvg's bottleneck is filter-heavy SVGs (especially Gaussian blur). For simple icons at 64x64:
- Without filters: estimated **500-2000 renders/sec** (based on tiny-skia throughput for small rasters)
- With feGaussianBlur: estimated **100-500 renders/sec** (IIR blur is the bottleneck)
- With complex filter chains: estimated **50-200 renders/sec**

These are estimates extrapolated from available benchmark data. The 39.6 ops/s figure from resvg-js is for much larger, more complex SVGs. Small 64x64 icons will be orders of magnitude faster.

**Recommendation:** For CivLab's needs (generating hundreds of icons at build time, not real-time), even the conservative estimate of 50 renders/sec means a full set of 500 icons renders in 10 seconds. Performance is not a concern.

#### API Usage (Rust)

```rust
use resvg::usvg::{self, fontdb, TreeParsing, TreeTextToPath};
use resvg::tiny_skia;

fn render_svg_to_png(svg_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    // Set up font database
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    // Parse SVG
    let opt = usvg::Options::default();
    let mut tree = usvg::Tree::from_data(svg_data, &opt).unwrap();
    tree.convert_text(&fontdb);

    // Create pixel buffer
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();

    // Render
    let tree = resvg::Tree::from_usvg(&tree);
    tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Encode to PNG
    pixmap.encode_png().unwrap()
}
```

#### API Usage (Node.js via resvg-js)

```typescript
import { Resvg } from '@resvg/resvg-js';

function renderSvgToPng(svgString: string, width: number): Buffer {
    const resvg = new Resvg(svgString, {
        fitTo: { mode: 'width', value: width },
        font: {
            loadSystemFonts: false,
            fontFiles: ['./assets/fonts/game-font.ttf'],
        },
    });

    const pngData = resvg.render();
    return pngData.asPng();
}
```

---

### 2. SVG Parsing: roxmltree

#### Overview

roxmltree is a high-performance, read-only XML/SVG parser for Rust. It parses XML into an immutable tree structure optimized for fast traversal.

- **Repository:** https://github.com/RazrFalcon/roxmltree
- **License:** MIT/Apache-2.0
- **Key characteristic:** **Read-only** — no mutation of the parsed tree

#### Performance

roxmltree is the fastest XML parser in the Rust ecosystem:
- Backed by xmlparser (many times faster than xml-rs)
- Read-only design enables arena allocation and zero-copy string references
- Parent node access supported (unlike some streaming parsers)

#### Usage for SVG Analysis

```rust
use roxmltree::Document;

fn analyze_svg(svg_data: &str) {
    let doc = Document::parse(svg_data).unwrap();

    // Find all rect elements
    for node in doc.descendants() {
        if node.has_tag_name("rect") {
            let x = node.attribute("x").unwrap_or("0");
            let y = node.attribute("y").unwrap_or("0");
            let width = node.attribute("width").unwrap_or("0");
            let height = node.attribute("height").unwrap_or("0");
            let fill = node.attribute("fill").unwrap_or("none");
            println!("rect at ({x},{y}) size {width}x{height} fill={fill}");
        }
    }

    // Find elements by ID
    if let Some(icon) = doc.descendants().find(|n| n.attribute("id") == Some("icon-base")) {
        println!("Found icon-base element: {:?}", icon.tag_name().name());
    }
}
```

#### Limitations

- **Cannot modify the tree**: No `set_attribute()`, `append_child()`, `remove_child()`
- **Cannot serialize back to XML**: Read-only parsing, no writer
- For mutation, need a different approach (see section 3)

---

### 3. SVG Mutation Strategies

Since roxmltree is read-only, CivLab needs a separate approach for generating and modifying SVG documents. Three options were evaluated:

#### Option A: minidom (Rust XML library with mutation)

**Overview:** minidom is a mutable XML DOM library based on quick-xml.

```rust
use minidom::Element;

fn mutate_svg() {
    let svg: Element = "<svg xmlns='http://www.w3.org/2000/svg' width='64' height='64'>
        <rect id='bg' x='0' y='0' width='64' height='64' fill='#333'/>
    </svg>".parse().unwrap();

    // minidom supports:
    // - Element creation
    // - Attribute setting
    // - Child insertion
    // - Element removal
    // - Serialization back to XML string
}
```

**Pros:**
- Full DOM mutation (setAttribute, appendChild, removeChild)
- Based on quick-xml (faster than xml-rs)
- Serializes back to XML string

**Cons:**
- Designed primarily for XMPP, not SVG
- SVG namespace handling can be awkward
- No SVG-specific validation
- Less performant than roxmltree for read operations

#### Option B: xml-doc (Rust mutable XML tree)

**Overview:** xml-doc provides a mutable XML tree with a cleaner API than minidom.

- **Repository:** https://github.com/BlueGreenMagick/xml-doc
- Supports `set_attribute()`, element insertion, and serialization
- Slower than roxmltree for parsing but provides full mutation

#### Option C: String Templating (Recommended)

For CivLab's use case (procedural SVG generation), the SVG documents are **generated from scratch**, not parsed-and-mutated from existing files. String templating is the simplest and most performant approach:

```rust
fn generate_faction_icon(
    faction_color: &str,
    emblem_path: &str,
    border_style: &str,
    size: u32,
) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{size}" height="{size}" viewBox="0 0 64 64">
  <defs>
    <radialGradient id="bg-grad">
      <stop offset="0%" stop-color="{faction_color}" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="{faction_color}" stop-opacity="0.3"/>
    </radialGradient>
    <filter id="shadow">
      <feDropShadow dx="1" dy="1" stdDeviation="1" flood-opacity="0.5"/>
    </filter>
  </defs>

  <!-- Background -->
  <circle cx="32" cy="32" r="30" fill="url(#bg-grad)"
          stroke="{faction_color}" stroke-width="2"/>

  <!-- Emblem -->
  <path d="{emblem_path}" fill="white" filter="url(#shadow)"
        transform="translate(16,16) scale(0.5)"/>

  <!-- Border decoration -->
  <circle cx="32" cy="32" r="31" fill="none"
          stroke="{border_style}" stroke-width="1" stroke-dasharray="4,2"/>
</svg>"#,
        size = size,
        faction_color = faction_color,
        emblem_path = emblem_path,
        border_style = border_style,
    )
}
```

**Pros:**
- Zero parsing overhead — SVG is generated directly as a string
- No external dependencies beyond std::fmt
- Full control over SVG structure
- Easy to parameterize any attribute or element
- Composable: build SVG fragments as functions, combine them

**Cons:**
- No structural validation (malformed SVG possible if template is wrong)
- Harder to conditionally modify existing SVGs (but CivLab generates fresh SVGs)
- String escaping needed for user-provided content (but CivLab uses known-safe values)

#### Recommendation

**String templating** for SVG generation (primary path) + **roxmltree** for any SVG analysis/inspection needed during build validation.

Rationale: CivLab generates procedural SVGs from templates with parameterized values (faction colors, emblem paths, border styles). This is fundamentally a generation problem, not a mutation problem. String templating is the simplest solution with zero overhead.

---

### 4. librsvg — Alternative Renderer

#### Overview

librsvg is the GNOME project's SVG rendering library. Originally C, it has been progressively rewritten in Rust but still depends on cairo, glib, and other C libraries.

#### Comparison with resvg

| Factor | resvg | librsvg |
|--------|-------|---------|
| Language | Pure Rust | Rust + C (cairo, glib, pango) |
| System dependencies | None | cairo, glib, pango, libxml2 |
| SVG spec coverage | Good (static subset) | Better (more edge cases) |
| Text rendering | Embedded fonts only | System fonts via pango |
| Filter performance | Single-threaded IIR blur | Box blur + multithreading |
| Cross-platform builds | Trivial (cargo build) | Complex (C toolchain + deps) |
| Cross-platform output | Bit-identical | Platform-dependent (different cairo/pango) |
| Package size | ~2MB binary | ~15MB+ with deps |
| Rust API | Native | FFI bindings |
| Maintenance | Active (linebender) | Active (GNOME) |

#### Key Differences

1. **Text rendering**: librsvg uses pango for text layout, supporting system fonts and complex text shaping (Arabic, CJK). resvg uses its own text engine with embedded fonts only. For game UI with a custom font, resvg is sufficient.

2. **Filter performance**: librsvg's Gaussian blur uses box blur approximation with multithreading, making it significantly faster for blur-heavy SVGs. resvg uses single-threaded IIR blur. For small 64x64 icons, this difference is negligible.

3. **Build complexity**: librsvg requires cairo, glib, pango, and libxml2 as build dependencies. On macOS, this means either Homebrew or a complex cross-compilation setup. On Linux, these are common but add container image size. resvg has zero non-Rust dependencies.

4. **Output reproducibility**: resvg produces bit-identical output across platforms. librsvg's output depends on the system's cairo and pango versions, which may differ between macOS and Linux.

#### Verdict

**resvg is preferred** for CivLab because:
- Pure Rust build: no C toolchain needed, trivial cross-compilation
- Bit-identical output: CI and local dev produce same results
- Feature coverage is sufficient for game UI icons
- Performance is adequate for build-time generation
- No system font dependency: game ships its own fonts

librsvg would be preferred only if:
- Complex text layout (bidirectional, CJK) were needed
- Heavy Gaussian blur performance were critical in hot paths
- Broader SVG spec coverage were required for user-supplied SVGs

None of these apply to CivLab's procedural icon pipeline.

---

### 5. SVG Mutation Contract

The following contract defines the interface for CivLab's procedural SVG generation system:

```rust
/// SVG template engine for procedural game UI generation.
/// Generates SVG strings from parameterized templates,
/// then renders them to PNG via resvg.
trait SvgPipeline {
    /// Generate an SVG string from a template and parameters.
    fn generate_svg(&self, template: &SvgTemplate, params: &SvgParams) -> String;

    /// Render an SVG string to a PNG buffer at the specified dimensions.
    fn render_to_png(&self, svg_data: &str, width: u32, height: u32) -> Vec<u8>;

    /// Batch-render multiple SVGs to PNGs.
    fn batch_render(
        &self,
        items: &[(String, u32, u32)],  // (svg_data, width, height)
    ) -> Vec<Vec<u8>>;

    /// Validate an SVG string (parse with usvg, check for errors).
    fn validate_svg(&self, svg_data: &str) -> Result<SvgValidation, SvgError>;
}

/// Template definition for procedural SVG generation.
struct SvgTemplate {
    /// Template name (e.g., "faction_icon", "resource_badge", "unit_health_bar").
    name: String,

    /// SVG template string with {{placeholder}} markers.
    template: String,

    /// Required parameters for this template.
    required_params: Vec<String>,

    /// Default values for optional parameters.
    defaults: HashMap<String, String>,
}

/// Parameters to fill an SVG template.
struct SvgParams {
    /// Key-value pairs replacing {{placeholder}} markers.
    values: HashMap<String, String>,
}

/// Validation result from SVG parsing.
struct SvgValidation {
    /// Whether the SVG is valid and renderable.
    valid: bool,

    /// Parsed dimensions.
    width: f64,
    height: f64,

    /// Number of elements in the simplified tree.
    element_count: usize,

    /// Whether filters are used (affects performance).
    uses_filters: bool,

    /// List of referenced fonts.
    fonts_used: Vec<String>,

    /// Warnings (non-fatal issues).
    warnings: Vec<String>,
}

/// SVG generation error.
enum SvgError {
    /// Template parameter missing.
    MissingParam(String),

    /// SVG parsing failed (malformed XML or unsupported features).
    ParseError(String),

    /// Rendering failed.
    RenderError(String),
}
```

### Template Examples

#### Faction Icon Template

```rust
const FACTION_ICON_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{size}}" height="{{size}}" viewBox="0 0 64 64">
  <defs>
    <radialGradient id="bg">
      <stop offset="0%" stop-color="{{primary_color}}" stop-opacity="0.9"/>
      <stop offset="100%" stop-color="{{secondary_color}}" stop-opacity="0.4"/>
    </radialGradient>
    <filter id="emblem-shadow">
      <feDropShadow dx="0.5" dy="0.5" stdDeviation="0.8" flood-color="#000" flood-opacity="0.4"/>
    </filter>
    <clipPath id="circle-clip">
      <circle cx="32" cy="32" r="29"/>
    </clipPath>
  </defs>

  <!-- Background circle with gradient -->
  <circle cx="32" cy="32" r="30" fill="url(#bg)" stroke="{{border_color}}" stroke-width="2"/>

  <!-- Emblem path (clipped to circle) -->
  <g clip-path="url(#circle-clip)" filter="url(#emblem-shadow)">
    <path d="{{emblem_path}}" fill="{{emblem_color}}"
          transform="translate({{emblem_x}},{{emblem_y}}) scale({{emblem_scale}})"/>
  </g>

  <!-- Decorative border ring -->
  <circle cx="32" cy="32" r="31" fill="none"
          stroke="{{border_color}}" stroke-width="0.5" opacity="0.6"/>
</svg>"#;
```

#### Resource Badge Template

```rust
const RESOURCE_BADGE_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{size}}" height="{{size}}" viewBox="0 0 32 32">
  <defs>
    <linearGradient id="badge-bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{{top_color}}"/>
      <stop offset="100%" stop-color="{{bottom_color}}"/>
    </linearGradient>
  </defs>

  <!-- Rounded rectangle background -->
  <rect x="1" y="1" width="30" height="30" rx="4" ry="4"
        fill="url(#badge-bg)" stroke="{{border_color}}" stroke-width="1"/>

  <!-- Resource icon -->
  <path d="{{icon_path}}" fill="{{icon_color}}"
        transform="translate(4,4) scale(0.75)"/>

  <!-- Quantity text -->
  <text x="28" y="28" font-family="{{font_family}}" font-size="10"
        fill="white" text-anchor="end" font-weight="bold">{{quantity}}</text>
</svg>"#;
```

#### Unit Health Bar Template

```rust
const HEALTH_BAR_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{bar_width}}" height="{{bar_height}}" viewBox="0 0 48 6">
  <!-- Background -->
  <rect x="0" y="0" width="48" height="6" rx="3" fill="#333" opacity="0.8"/>

  <!-- Health fill (width proportional to health %) -->
  <rect x="1" y="1" width="{{fill_width}}" height="4" rx="2" fill="{{health_color}}"/>

  <!-- Segment lines -->
  <line x1="12" y1="0" x2="12" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
  <line x1="24" y1="0" x2="24" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
  <line x1="36" y1="0" x2="36" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
</svg>"#;
```

---

## Decision

**resvg (pure Rust) + roxmltree (read-only analysis) + string templating (SVG generation)**

This combination provides:
1. **Zero system dependencies**: Pure Rust stack, trivial to build on all platforms
2. **Bit-identical output**: Same SVG produces same PNG on macOS, Linux, and CI
3. **Sufficient feature coverage**: Gradients, filters, text, clipping — everything needed for game UI
4. **High performance**: Estimated 500+ icons/sec for simple 64x64 renders on M3
5. **Simple mutation model**: String templating is the right abstraction for procedural generation
6. **Build-time pipeline**: Icons are generated at build/deploy time, not runtime

---

## Implementation Contract

### Rust Crate Dependencies

```toml
[dependencies]
resvg = "0.45"      # SVG rendering
usvg = "0.45"       # SVG parsing/simplification (re-exported by resvg)
tiny-skia = "0.11"  # Pixel buffer (re-exported by resvg)
roxmltree = "0.20"  # Read-only SVG analysis (for validation)
```

### Build Pipeline Integration

```
Source Templates (Rust string constants)
         │
         ▼
┌──────────────────────┐
│  Template Engine      │
│  - Parameter injection│
│  - Faction colors     │
│  - Emblem paths       │
│  - Resource icons     │
└────────┬─────────────┘
         │
         ▼ SVG strings
┌──────────────────────┐
│  Validation (usvg)   │
│  - Parse check        │
│  - Font resolution    │
│  - Element count      │
└────────┬─────────────┘
         │
         ▼ validated SVG
┌──────────────────────┐
│  Rendering (resvg)   │
│  - SVG → PNG @64x64   │
│  - Batch processing   │
│  - Deterministic      │
└────────┬─────────────┘
         │
         ▼ PNG buffers
┌──────────────────────┐
│  Atlas Packing       │
│  - Combine into sheet │
│  - Generate metadata  │
│  - Output atlas.png   │
│  - Output atlas.json  │
└──────────────────────┘
```

### Performance Budget

| Operation | Target | Estimated |
|-----------|--------|-----------|
| Simple icon render (64x64, no filters) | < 2ms | ~0.5-2ms |
| Icon with drop shadow (64x64) | < 10ms | ~2-10ms |
| Icon with complex filters (64x64) | < 20ms | ~5-20ms |
| Full icon set (500 icons) | < 30s | ~5-15s |
| Validation pass (SVG parse only) | < 0.5ms | ~0.1-0.5ms |

These targets are for build-time generation on M3 hardware. Performance is not a runtime concern.

---

## Open Questions Remaining

1. **Font embedding strategy**: resvg requires fonts to be explicitly loaded (no system font fallback). Need to select and bundle a game font (e.g., Inter, Source Sans Pro, or a custom pixel font). Font must be loaded into fontdb before rendering.

2. **SVG template versioning**: As the game evolves, icon templates will change. Need a strategy for versioning templates and invalidating cached renders when templates change. Content hashing of template + params as cache key is the likely approach.

3. **Atlas packing algorithm**: For the final sprite atlas, which packing algorithm? Options: maxrects (most space-efficient), shelf (simplest), or use an existing crate like `texture-packer`. Need to evaluate based on CivLab's icon count and size distribution.

4. **resvg 0.45 vs 0.42 changes**: The task mentioned 0.42.x specifically, but resvg is now at 0.45.x. Need to verify no breaking API changes between these versions. The architecture (usvg + resvg separation) has been stable since 0.28+.

5. **SVG-in-WebGPU alternative**: For runtime rendering (not build-time), could SVGs be rendered directly in the Pixi.js WebGPU pipeline instead of pre-rasterized PNGs? Pixi supports SVG textures via the browser's built-in SVG renderer. This would avoid the build-time pipeline entirely but loses cross-platform determinism.

6. **Color space handling**: resvg supports sRGB. If CivLab's art style uses wide-gamut colors (Display P3), need to verify resvg's color handling. Likely not an issue for stylized game art.

---

## Sources

- [resvg GitHub](https://github.com/linebender/resvg)
- [resvg README](https://github.com/linebender/resvg/blob/main/README.md)
- [resvg Unsupported Features](https://github.com/linebender/resvg/blob/main/docs/unsupported.md)
- [resvg CHANGELOG](https://github.com/linebender/resvg/blob/main/CHANGELOG.md)
- [resvg SVG2 Changelog](https://github.com/linebender/resvg/blob/main/docs/svg2-changelog.md)
- [resvg docs.rs](https://docs.rs/crate/resvg/latest)
- [resvg-js GitHub](https://github.com/thx/resvg-js)
- [roxmltree GitHub](https://github.com/RazrFalcon/roxmltree)
- [roxmltree docs.rs](https://docs.rs/roxmltree/latest/roxmltree/)
- [XML Parsing in Rust — Mainmatter](https://mainmatter.com/blog/2020/12/31/xml-and-rust/)
- [xml-doc GitHub](https://github.com/BlueGreenMagick/xml-doc)
- [librsvg discussion — libvips](https://github.com/libvips/libvips/discussions/2048)
- [resvg Benchmark Issue #185](https://github.com/RazrFalcon/resvg/issues/185)
- [resvg-js sharp comparison — Issue #145](https://github.com/thx/resvg-js/issues/145)
