# DINOForge Feature Proof & Journey Viewer System

This directory contains the **Feature Proof** system for DINOForge—visual documentation of implemented features through interactive journeys and screenshots.

## Quick Start

### Viewing Journeys

1. Navigate to any page with `<JourneyViewer>` component
2. Click **Play** to auto-advance through frames
3. Use **← Previous / Next →** buttons or arrow keys to manually navigate
4. Click a thumbnail to jump to that frame
5. Press **Space** to toggle playback

### Creating a Journey

```vue
<script setup>
const myJourney = {
  id: 'my-feature',
  intent: 'Feature description',
  keyframe_count: 3,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'step-1',
      intent: 'What happens here',
      screenshot_path: '/path/to/image.png',
      assertions: {
        must_contain: ['visible text'],
        must_not_contain: ['error']
      }
    }
    // ... more steps
  ]
}
</script>

<JourneyViewer :journey="myJourney" title="My Feature" />
```

## Directory Structure

```
proof/
├── README.md                          # This file
├── JOURNEY_VIEWER.md                  # Component documentation
├── journey-viewer-demo.md             # Interactive demo with examples
├── journey-generator.ts               # Utility functions for test automation
├── journeys/
│   └── manifests/
│       ├── us-f1-1-game-launch/
│       │   ├── manifest.json          # Journey definition
│       │   └── annotations.json       # Visual overlays
│       └── us-f1-2-pack-loading/
│           └── manifest.json
├── journey-examples/                  # Example files
│   ├── example-manifest.json
│   └── shot-annotations.json
└── screenshots/
    ├── placeholder-1.png
    ├── placeholder-2.png
    └── ... (actual test screenshots)
```

## Journey Manifest Format

A journey manifest is a JSON file describing a sequence of screenshots:

```json
{
  "id": "unique-journey-id",
  "intent": "What the user is trying to accomplish",
  "keyframe_count": 5,
  "passed": true,
  "steps": [
    {
      "index": 0,
      "slug": "unique-step-id",
      "intent": "What happens in this frame",
      "screenshot_path": "/path/to/screenshot.png",
      "assertions": {
        "must_contain": ["text that should be visible"],
        "must_not_contain": ["error messages"]
      }
    }
    // ... more steps
  ]
}
```

## Annotation Overlays

Annotations highlight regions of interest on screenshots:

```json
{
  "0": [
    {
      "bbox": { "x": 100, "y": 100, "width": 200, "height": 150 },
      "label": "Button Label",
      "type": "passed"
    }
  ]
}
```

**Types:**
- `"passed"` → Green (#22c55e) - Successful regions
- `"failed"` → Red (#ef4444) - Problem areas
- `"info"` → Blue (#3b82f6) - Informational highlights

## Integration with Tests

### Automated Journey Generation

Use the `journey-generator.ts` utilities to create manifests from test automation:

```typescript
import { createJourney, validateJourney } from './journey-generator'

const journey = createJourney({
  id: 'us-f1-1-game-launch',
  intent: 'User launches game with mod loaded',
  steps: [
    {
      intent: 'Launch Steam',
      screenshot: 'screenshots/01.png',
      assertions: { must_contain: ['Play button'] }
    },
    // ... more steps
  ]
})

// Validate manifest
const result = validateJourney(journey)
if (!result.valid) {
  console.error('Validation errors:', result.errors)
}
```

### CI/CD Pipeline

```yaml
# .github/workflows/generate-journeys.yml
on: [push]

jobs:
  generate-proofs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run E2E tests with screenshots
        run: npm run test:e2e -- --screenshots
      
      - name: Generate journey manifests
        run: npm run proof:generate
      
      - name: Validate journeys
        run: npm run proof:validate
      
      - name: Commit to docs
        run: |
          git add docs/proof/journeys/
          git commit -m "chore: update journey proofs from CI" || true
          git push
```

## Component API

### Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `journey` | Journey | Yes | Journey manifest object |
| `title` | String | No | Display title (default: "Journey") |
| `annotations` | Record<number, Annotation[]> | No | Visual overlays by frame index |

### Keyboard Navigation

| Key | Action |
|-----|--------|
| `←` / `→` | Previous / Next frame |
| `Space` | Play / Pause |

## Best Practices

### Journey Design

1. **Focused Scope**: 3-7 frames per journey
2. **Clear Intent**: Each step has a single purpose
3. **Explicit Assertions**: Specific, verifiable statements
4. **Meaningful Slugs**: URL-friendly step identifiers

```json
{
  "index": 0,
  "slug": "steam-library-view",     // ✓ Good
  "intent": "Navigate to DINO in library"
}
```

### Annotations

1. **Highlight Key Elements**: Box main interaction points
2. **Use Consistent Colors**: Green = success, Red = error
3. **Short Labels**: 1-3 words
4. **Avoid Overlap**: Don't stack annotations

### Screenshot Guidelines

- **Resolution**: 1920x1080 (widescreen standard)
- **Format**: PNG (lossless, smaller than JPG)
- **Compression**: Optimize with ImageOptim or similar
- **Consistency**: Uniform aspect ratio and lighting

## Examples

### Example 1: Simple Game Launch Journey

```javascript
const journey = {
  id: 'us-f1-1-game-launch',
  intent: 'User can launch game with DINOForge mod loaded',
  keyframe_count: 3,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'click-play',
      intent: 'Click Play button in Steam library',
      screenshot_path: '/proof/screenshots/game-launch/01-steam.png',
      assertions: {
        must_contain: ['Diplomacy is Not an Option', 'Play button'],
        must_not_contain: ['Error']
      }
    },
    {
      index: 1,
      slug: 'game-loading',
      intent: 'Game initializes with splash screen',
      screenshot_path: '/proof/screenshots/game-launch/02-splash.png',
      assertions: {
        must_contain: ['Loading...'],
        must_not_contain: ['Fatal error', 'Exception']
      }
    },
    {
      index: 2,
      slug: 'mod-overlay-ready',
      intent: 'Press F10 to show DINOForge overlay',
      screenshot_path: '/proof/screenshots/game-launch/03-overlay.png',
      assertions: {
        must_contain: ['DINOForge', 'Loaded Packs'],
        must_not_contain: []
      }
    }
  ]
}
```

### Example 2: Pack Management Journey

```javascript
const journey = {
  id: 'us-f2-1-pack-management',
  intent: 'User can view, enable, and disable mod packs',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'overlay-open',
      intent: 'DINOForge overlay is open',
      screenshot_path: '/proof/screenshots/pack-mgmt/01-overlay.png',
      assertions: {
        must_contain: ['Loaded Packs'],
        must_not_contain: []
      }
    },
    {
      index: 1,
      slug: 'packs-listed',
      intent: 'All installed packs are listed',
      screenshot_path: '/proof/screenshots/pack-mgmt/02-list.png',
      assertions: {
        must_contain: ['example-balance', 'warfare-modern', 'warfare-starwars'],
        must_not_contain: []
      }
    },
    {
      index: 2,
      slug: 'toggle-pack',
      intent: 'Click toggle to disable a pack',
      screenshot_path: '/proof/screenshots/pack-mgmt/03-toggled.png',
      assertions: {
        must_contain: ['warfare-modern (disabled)'],
        must_not_contain: []
      }
    },
    {
      index: 3,
      slug: 'changes-applied',
      intent: 'Changes are applied to game',
      screenshot_path: '/proof/screenshots/pack-mgmt/04-reloaded.png',
      assertions: {
        must_contain: ['Packs reloaded'],
        must_not_contain: ['Error']
      }
    }
  ]
}
```

## Troubleshooting

### Images Not Loading

**Problem**: Broken image icons in viewer

**Solutions**:
1. Check path is relative to `docs/` root
2. Use absolute path for GitHub Pages: `/Dino/proof/screenshots/image.png`
3. Verify image file exists
4. Try smaller resolution images first

### Annotations Not Rendering

**Problem**: SVG overlays don't appear

**Solutions**:
1. Verify `annotations` prop is passed
2. Check bbox coordinates are 0-1920 × 0-1080
3. Inspect SVG element in DevTools
4. Try different annotation type colors

### Journey Not Found

**Problem**: "Journey is undefined" error

**Solutions**:
1. Check manifest file path is correct
2. Verify JSON syntax (use JSONLint)
3. Ensure `keyframe_count` matches `steps.length`
4. Check all required fields are present

## Performance Tips

1. **Optimize Images**: Use ImageOptim, TinyPNG, or similar
2. **Lazy Load**: Only load journeys when viewport visible
3. **Batch Journeys**: Group related journeys together
4. **Cache Assets**: VitePress handles image caching automatically

## Extending the System

### Custom Annotation Types

Extend the `Annotation` type to add metadata:

```typescript
interface AnnotationWithMetadata extends Annotation {
  // Custom fields
  element_id?: string
  interaction?: 'click' | 'hover' | 'scroll'
  duration_ms?: number
}
```

### Journey Plugins

Create custom rendering for specialized journeys:

```vue
<template>
  <JourneyViewer :journey="journey">
    <template #annotation="{ annotation }">
      <CustomAnnotationRenderer :annotation="annotation" />
    </template>
  </JourneyViewer>
</template>
```

### Export Formats

Generate different output formats:

```typescript
import { exportJourneyJSON, exportJourneyYAML, generateJourneyMarkdown }
  from './journey-generator'

// Export as YAML
const yaml = exportJourneyYAML(journey)

// Export as Markdown with images
const md = generateJourneyMarkdown(journey, { includeImages: true })
```

## Contributing

To add a new journey:

1. **Create directory**: `docs/proof/journeys/manifests/us-X-Y-feature-name/`
2. **Add manifest**: `manifest.json` with journey definition
3. **Add annotations** (optional): `annotations.json`
4. **Add screenshots**: `docs/proof/screenshots/us-X-Y-*.png`
5. **Create page**: Add `<JourneyViewer>` in markdown
6. **Update sidebar**: Add link to journey in `config.mts`
7. **Test locally**: `npm run docs:dev` and verify rendering

## References

- **Component Docs**: [JOURNEY_VIEWER.md](./JOURNEY_VIEWER.md)
- **Demo & Examples**: [journey-viewer-demo.md](./journey-viewer-demo.md)
- **Generator Utils**: [journey-generator.ts](./journey-generator.ts)
- **VitePress Docs**: https://vitepress.dev/

## License

DINOForge is released under the MIT License.

---

**Last Updated**: 2026-04-23  
**Version**: 1.0.0
