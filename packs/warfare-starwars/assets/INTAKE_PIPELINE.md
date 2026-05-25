# Star Wars Asset Intake Pipeline

Complete asset intake pipeline directory structure and JSON schema for DINOForge Star Wars mod asset management.

## Overview

This pipeline provides a comprehensive system for discovering, acquiring, assessing, and managing 3D assets for the DINOForge Star Wars Clone Wars mod pack. It includes:

- **JSON Schema** for standardized asset metadata
- **Intake Rules** for acquisition and validation policies
- **Risk Assessment Framework** for IP/licensing evaluation
- **Registry System** for tracking assets and provenance
- **Template Manifests** for three key Star Wars assets

## Directory Structure

```
packs/warfare-starwars/assets/
├── raw/                              # Raw asset input directory
│   ├── sw_clone_trooper_sketchfab_001/
│   │   └── asset_manifest.json       # Template manifest (placeholders)
│   ├── sw_stormtrooper_sketchfab_001/
│   │   ├── asset_manifest.json       # Detailed metadata (Sketchfab verified)
│   │   ├── metadata.json             # Additional context
│   │   └── DOWNLOAD_GUIDE.md         # Download instructions
│   └── sw_vader_hero_sketchfab_001/
│       └── asset_manifest.json       # Template manifest (placeholders)
│
├── working/                          # Processing workspace (empty)
│
├── registry/                         # Asset registries and indices
│   ├── asset_index.json             # Central registry of all assets
│   └── provenance_index.json        # Detailed IP/provenance tracking
│
├── policies/                         # Intake and risk assessment rules
│   ├── intake_rules.yaml            # Acquisition, validation, processing rules
│   └── risk_rules.yaml              # IP risk classification framework
│
└── asset_manifest.schema.json       # JSON Schema for asset manifests
```

## Key Files

### 1. `asset_manifest.schema.json`

JSON Schema (draft-07) defining the structure for all asset manifest files.

**Key Fields:**
- `asset_id` - Unique identifier (required)
- `category` - Asset type (unit_model, vehicle_model, building_model, etc., required)
- `source_url` - Source asset page (required)
- `original_format` - File format (glb, gltf, obj, fbx, blend, required)
- `technical_status` - Processing stage (discovered → ready_for_prototype)
- `ip_status` - IP classification (generic_safe → high_risk_do_not_ship)
- `license_label`, `license_url` - License information
- `author_name` - Original creator name
- `provenance_confidence` - 0-1 confidence score for IP assessment
- `polycount_estimate`, `texture_sets`, `rigged`, `animated` - Technical details
- `sha256`, `download_url` - For integrity and distribution
- `notes` - Array of processing/warning notes

### 2. `policies/intake_rules.yaml`

Policy file controlling asset acquisition and processing.

**Sections:**
- `allowed_sources` - Approved source platforms (sketchfab, blendswap, moddb)
- `preferred_acquisition_order` - Acquisition method priority (api, direct_download, scrape, browser_automation)
- `reject_conditions` - Automatic rejection triggers (missing metadata, corrupted files, etc.)
- `validation_requirements` - Technical checks before acceptance
- `risk_labels` - IP classification labels and release policies
- `processing_pipeline` - Stage definitions (discovery → release_approval)
- `provenance_assessment` - Confidence thresholds and assessment factors
- `category_guidelines` - Technical specs per asset type

### 3. `policies/risk_rules.yaml`

Framework for intellectual property risk assessment and classification.

**Risk Categories:**
- `generic_safe` - Generic non-franchised models (CC BY 4.0, etc.)
- `fan_star_wars_private_only` - Fan content (private/dev only, not distributable)
- `official_game_mod_tool_reference_only` - Official assets (reference architecture only)
- `unknown_provenance` - Insufficient licensing/source information
- `high_risk_do_not_ship` - Known IP violations or licensing conflicts

**Includes:**
- Confidence scoring rules and adjustments
- Source authority evaluation (high/medium/low trust)
- License compatibility matrix
- Assessment workflow (intake → verification → scoring → decision)
- Documentation requirements by risk level

### 4. `registry/asset_index.json`

Central registry listing all discovered and ingested assets.

**Contents:**
- Asset metadata summary for each asset
- Status tracking (discovered, metadata_fetched, downloaded, etc.)
- Manifest file paths
- Registration and check timestamps
- Aggregate statistics by status, IP category, asset type

**Current State (3 assets):**
- `sw_clone_trooper_sketchfab_001` - Clone Trooper (Republic faction, needs metadata)
- `sw_stormtrooper_sketchfab_001` - Stormtrooper (CIS faction, Sketchfab verified)
- `sw_vader_hero_sketchfab_001` - Vader Hero (CIS faction, needs metadata)

### 5. `registry/provenance_index.json`

Detailed provenance tracking and intellectual property assessment records.

**Per-Asset Records Include:**
- `ip_classification` - Risk category with confidence score
- `source_authority` - Trust level assessment
- `license_assessment` - License terms, verification status, release compatibility
- `author_assessment` - Creator reputation, contact status
- `derivative_work_assessment` - IP ownership analysis
- `risk_factors` - Specific risk indicators present
- `release_policy` - Usage restrictions and required actions
- `acquisition_metadata` - Download details, format, date, polycount

**Current Status:**
- All three assets classified as `fan_star_wars_private_only`
- Suitable for internal prototyping and design validation
- Require replacement with generic/original models for public release

## Asset Manifests

Three template manifest files in `raw/` directories:

### `sw_clone_trooper_sketchfab_001/asset_manifest.json`
- **Status**: Template with placeholders
- **Category**: unit_model
- **Faction**: republic
- **IP Status**: fan_star_wars_private_only
- **Notes**: Requires user to fill in Sketchfab URL, author info, and license

### `sw_stormtrooper_sketchfab_001/asset_manifest.json`
- **Status**: Sketchfab verified with full metadata
- **Category**: unit_model
- **Faction**: cis
- **Author**: Oscar RP (@oscarrep)
- **License**: CC-BY 4.0
- **Polycount**: 3,222 tris (optimized for game)
- **Source URL**: https://sketchfab.com/3d-models/star-wars-low-poly-stormtrooper-7d55b6ca7935440aa59961197ea742ff
- **IP Status**: fan_star_wars_private_only
- **Download Method**: Manual Sketchfab download button (requires free account)

### `sw_vader_hero_sketchfab_001/asset_manifest.json`
- **Status**: Template with placeholders
- **Category**: unit_model (hero)
- **Faction**: cis
- **IP Status**: fan_star_wars_private_only
- **Notes**: Higher polygon budget than infantry units. Requires user acquisition and metadata.

## Workflow

### 1. Asset Discovery
1. Identify asset source (Sketchfab, BlendSwap, etc.)
2. Create manifest file in `raw/<asset_id>/asset_manifest.json`
3. Fill in required fields (asset_id, source_url, original_format, category)
4. Add to `registry/asset_index.json`

### 2. Metadata Acquisition
1. Visit source URL
2. Verify author information
3. Read and document license terms
4. Assess IP status and provenance
5. Download asset file (if permitted)
6. Calculate SHA256 checksum
7. Update manifest with all metadata
8. Update `registry/provenance_index.json` with detailed assessment

### 3. Validation
1. Verify manifest against `asset_manifest.schema.json`
2. Check against `policies/intake_rules.yaml` requirements
3. Apply risk assessment from `policies/risk_rules.yaml`
4. Document any issues or warnings in manifest notes

### 4. Processing
1. Move to `working/` directory for normalization
2. Convert to target format (GLB preferred)
3. Validate mesh structure and materials
4. Generate technical metadata (polycount, texture sets, etc.)
5. Store normalized version

### 5. Release Approval
1. Check `ip_status` in provenance index
2. If `generic_safe` - approved for public release
3. If `fan_star_wars_private_only` - private development only
4. If other risk categories - require explicit action/approval

## IP Status Reference

### fan_star_wars_private_only
**All three current assets are classified this way**

- **Contains**: Star Wars IP (characters, brands, designs)
- **Release Policy**: Private/dev-only. Cannot ship in public releases.
- **Legal Rationale**: Star Wars is owned by Lucasfilm/Disney. Fan content is non-transferable.
- **Recommended Action**: Replace with generic sci-fi models or obtain explicit Lucasfilm licensing

### generic_safe
- **Contains**: Original or generic models with clear licensing
- **Release Policy**: Approved for public release with attribution
- **License Examples**: CC BY 4.0, CC0, MIT

### high_risk_do_not_ship
- **Contains**: Known IP violations or licensing conflicts
- **Release Policy**: REJECT. Do not acquire, use, or distribute.
- **Examples**: Pirated paid assets, copyright claims, DMCA violations

## Configuration Files

### `intake_rules.yaml`
```yaml
allowed_sources: [sketchfab, blendswap, moddb]
preferred_acquisition_order: [api, direct_download, scrape, browser_automation]
reject_conditions: [missing_source_url, missing_author_name, missing_license_label, corrupted_archive, unsupported_format]
```

### `risk_rules.yaml`
Defines confidence scoring adjustments, source authority levels, license compatibility matrix, and assessment workflow.

## Using the Pipeline

### Validate a manifest
```bash
# Schema validation
dotnet run --project src/Tools/PackCompiler -- validate-schema \
  packs/warfare-starwars/assets/raw/sw_clone_trooper_sketchfab_001/asset_manifest.json \
  packs/warfare-starwars/assets/asset_manifest.schema.json
```

### Check asset status
```bash
# Query registry
cat packs/warfare-starwars/assets/registry/asset_index.json | jq '.assets[] | {asset_id, technical_status, ip_status}'
```

### Review IP assessment
```bash
# Examine provenance records
cat packs/warfare-starwars/assets/registry/provenance_index.json | jq '.provenance_records[] | {asset_id, ip_classification, ip_confidence, release_policy}'
```

## Next Steps

1. **For Clone Trooper & Vader assets**: User must obtain actual Sketchfab URLs and fill in all placeholders (author_name, license, download_url, sha256)

2. **For all assets**: Ensure full metadata in manifest before technical processing

3. **For release planning**: Plan generic sci-fi model replacements for public distribution

4. **For extended use**: Consider adding more asset sources (vehicles, buildings, weapons, effects) following same pattern

## Schema Fields Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| asset_id | string | ✓ | Unique identifier (pattern: `^[a-z0-9_]+$`) |
| category | string (enum) | ✓ | Asset type (unit_model, vehicle_model, building_model, etc.) |
| source_url | string (uri) | ✓ | URL to source asset page |
| original_format | string (enum) | ✓ | File format (glb, gltf, obj, fbx, blend) |
| canonical_name | string | | Human-readable name |
| franchise_tag | string | | Franchise identifier (e.g., 'star_wars') |
| faction | string | | Faction (republic, cis, etc.) |
| source_platform | string (enum) | | Platform (sketchfab, blendswap, moddb, other) |
| author_name | string | | Original creator name |
| license_label | string | | License (e.g., 'CC BY 4.0') |
| license_url | string (uri) | | License documentation link |
| acquisition_mode | string (enum) | | How acquired (api, download, scrape, browser_automation) |
| acquired_at_utc | string (date-time) | | ISO 8601 acquisition timestamp |
| download_url | string (uri) | | Direct download URL |
| sha256 | string | | SHA256 file hash (64 hex chars) |
| technical_status | string (enum) | | Processing stage |
| ip_status | string (enum) | | IP risk classification |
| provenance_confidence | number (0-1) | | Confidence score |
| polycount_estimate | integer | | Polygon count estimate |
| texture_sets | integer | | Number of texture sets |
| rigged | boolean | | Has skeleton rig |
| animated | boolean | | Has animations |
| notes | array[string] | | Processing notes and warnings |

## Related Documentation

- `asset_manifest.schema.json` - JSON Schema specification
- `policies/intake_rules.yaml` - Acquisition and processing rules
- `policies/risk_rules.yaml` - IP assessment framework
- `registry/asset_index.json` - Asset registry
- `registry/provenance_index.json` - IP assessment records
