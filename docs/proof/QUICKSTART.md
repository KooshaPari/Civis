# Journey Viewer - Quick Start Guide

Get up and running with the Journey Viewer in 5 minutes.

## Installation (Already Done!)

The Journey Viewer is already integrated into the DINOForge docs:

✅ Component: `docs/.vitepress/theme/components/JourneyViewer.vue`  
✅ Registration: `docs/.vitepress/theme/index.ts`  
✅ Config: `docs/.vitepress/config.mts`  

## 1. Create a Journey Manifest

Create `docs/proof/journeys/manifests/my-feature/manifest.json`:

```json
{
  "id": "my-feature-id",
  "intent": "What the user is trying to do",
  "keyframe_count": 3,
  "passed": true,
  "steps": [
    {
      "index": 0,
      "slug": "step-1",
      "intent": "What happens in step 1",
      "screenshot_path": "/proof/screenshots/my-feature/01.png",
      "assertions": {
        "must_contain": ["visible text"],
        "must_not_contain": ["error"]
      }
    },
    {
      "index": 1,
      "slug": "step-2",
      "intent": "What happens in step 2",
      "screenshot_path": "/proof/screenshots/my-feature/02.png",
      "assertions": {
        "must_contain": ["next step text"],
        "must_not_contain": []
      }
    },
    {
      "index": 2,
      "slug": "step-3",
      "intent": "Final step",
      "screenshot_path": "/proof/screenshots/my-feature/03.png",
      "assertions": {
        "must_contain": ["completion text"],
        "must_not_contain": []
      }
    }
  ]
}
```

## 2. Add Screenshots

Place your screenshots at:
```
docs/proof/screenshots/my-feature/
├── 01.png
├── 02.png
└── 03.png
```

**Requirements:**
- Format: PNG (preferred) or JPG
- Resolution: 1920×1080 (widescreen)
- Compressed with ImageOptim or TinyPNG
- Clear, well-lit images

## 3. Create Annotations (Optional)

Create `docs/proof/journeys/manifests/my-feature/annotations.json`:

```json
{
  "0": [
    {
      "bbox": { "x": 100, "y": 100, "width": 200, "height": 150 },
      "label": "Button to Click",
      "type": "passed"
    }
  ],
  "1": [
    {
      "bbox": { "x": 500, "y": 300, "width": 400, "height": 200 },
      "label": "Main Content",
      "type": "info"
    }
  ]
}
```

**Annotation Types:**
- `"passed"` → Green (success)
- `"failed"` → Red (error)
- `"info"` → Blue (info)

## 4. Add to Documentation

In your markdown file, add:

```vue
<script setup>
import manifest from './journeys/manifests/my-feature/manifest.json'
import annotations from './journeys/manifests/my-feature/annotations.json'
</script>

<JourneyViewer 
  :journey="manifest"
  title="My Feature Journey"
  :annotations="annotations"
/>
```

Or inline:

```vue
<script setup>
const myJourney = {
  id: 'my-feature',
  intent: 'User feature',
  keyframe_count: 2,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'step-1',
      intent: 'First step',
      screenshot_path: '/proof/screenshots/my-feature/01.png',
      assertions: { must_contain: ['text'] }
    }
  ]
}
</script>

<JourneyViewer :journey="myJourney" title="My Feature" />
```

## 5. Test Locally

```bash
# Install dependencies (if not already done)
npm install

# Start dev server
npm run dev

# Visit http://localhost:5173 and navigate to your page
```

## Directory Structure

```
docs/
├── .vitepress/
│   └── theme/
│       ├── components/
│       │   └── JourneyViewer.vue      ← Component
│       ├── index.ts                   ← Registration
│       ├── types.ts                   ← TypeScript types
│       └── custom.css
├── proof/
│   ├── README.md                      ← Overview
│   ├── QUICKSTART.md                  ← This file
│   ├── JOURNEY_VIEWER.md              ← Full documentation
│   ├── example-usage.md               ← Complete examples
│   ├── journey-generator.ts           ← Utility functions
│   ├── journeys/
│   │   └── manifests/
│   │       └── my-feature/
│   │           ├── manifest.json
│   │           └── annotations.json
│   ├── screenshots/
│   │   └── my-feature/
│   │       ├── 01.png
│   │       └── 02.png
│   └── journey-examples/
│       ├── example-manifest.json
│       └── shot-annotations.json
```

## Common Tasks

### View Demo

Open `docs/proof/example-usage.md` in the browser - it has interactive examples.

### Load Journey from File

```typescript
// manifest.json - place in journeys/manifests/
{
  "id": "feature-id",
  "intent": "Feature description",
  "keyframe_count": 5,
  "passed": true,
  "steps": [...]
}
```

Then in markdown:
```vue
<script setup>
import manifest from './journeys/manifests/feature-id/manifest.json'
</script>

<JourneyViewer :journey="manifest" />
```

### Generate from Tests

```typescript
import { createJourney, validateJourney } from './journey-generator'

const journey = createJourney({
  id: 'test-feature',
  intent: 'Feature workflow',
  steps: [
    {
      intent: 'Step 1',
      screenshot: 'screenshots/01.png',
      assertions: { must_contain: ['text'] }
    }
  ]
})

// Validate
const result = validateJourney(journey)
if (result.valid) {
  // Use journey
}
```

### Export Journey

```typescript
import { 
  exportJourneyJSON, 
  exportJourneyYAML, 
  generateJourneyMarkdown 
} from './journey-generator'

// As JSON
const json = exportJourneyJSON(journey)

// As YAML
const yaml = exportJourneyYAML(journey)

// As Markdown
const md = generateJourneyMarkdown(journey, { includeImages: true })
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `→` | Next frame |
| `←` | Previous frame |
| `Space` | Play/Pause |

## Troubleshooting

### Component Not Found

**Error**: `<JourneyViewer>` shows as plain text

**Fix**: Ensure `docs/.vitepress/theme/index.ts` has:
```typescript
app.component('JourneyViewer', JourneyViewer)
```

### Images Not Loading

**Error**: Broken image icons

**Fix**: 
1. Check path starts with `/proof/` or uses absolute URL
2. For GitHub Pages, use `/Dino/proof/screenshots/...`
3. Verify image file exists
4. Use smaller resolution first (test with 800×450)

### Annotations Don't Show

**Error**: SVG overlays missing

**Fix**:
1. Verify `annotations` prop is passed
2. Check `bbox` coordinates: x/y should be 0-1920, width/height realistic
3. Ensure screenshot path is correct
4. Test in DevTools: `inspect` the SVG element

### JSON Parse Error

**Error**: "JSON parse error in manifest"

**Fix**:
1. Validate JSON at jsonlint.com
2. Check file has `.json` extension
3. Ensure all quotes are straight (not curly)
4. Check for trailing commas

## Performance Tips

1. **Optimize Images**
   ```bash
   # macOS
   brew install imageoptim
   # Then drag images into ImageOptim
   ```

2. **Or use online tools**
   - https://tinypng.com - Great compression
   - https://imageoptim.com/online - Free online

3. **Use responsive sizes**
   - 1920×1080 for full-size shots
   - Compress to <100KB per image

## Best Practices

✓ **Do**
- Keep journeys 3-7 frames
- Write specific assertions
- Use meaningful step slugs
- Optimize images before uploading
- Group related journeys together

✗ **Don't**
- Use vague assertions like "should work"
- Create 15+ frame journeys (split into smaller ones)
- Use uncompressed full-resolution images
- Forget to test locally before committing

## Examples

### Game Launch Journey

```json
{
  "id": "us-f1-1-game-launch",
  "intent": "User can launch game with DINOForge mod",
  "keyframe_count": 3,
  "passed": true,
  "steps": [
    {
      "index": 0,
      "slug": "launch-game",
      "intent": "Click Play in Steam",
      "screenshot_path": "/proof/screenshots/game-launch/01.png",
      "assertions": {
        "must_contain": ["Game launching..."],
        "must_not_contain": ["Error"]
      }
    },
    {
      "index": 1,
      "slug": "game-loaded",
      "intent": "Game loads main menu",
      "screenshot_path": "/proof/screenshots/game-launch/02.png",
      "assertions": {
        "must_contain": ["Main Menu", "New Game"],
        "must_not_contain": []
      }
    },
    {
      "index": 2,
      "slug": "mod-overlay",
      "intent": "Press F10 to show mod overlay",
      "screenshot_path": "/proof/screenshots/game-launch/03.png",
      "assertions": {
        "must_contain": ["DINOForge", "Loaded Packs"],
        "must_not_contain": []
      }
    }
  ]
}
```

## Next Steps

1. ✅ Read this guide
2. → Create your first journey manifest
3. → Add 3-5 screenshots
4. → Embed in documentation
5. → Test locally with `npm run dev`
6. → Commit to git
7. → Share with team

## Getting Help

- **Full Docs**: [JOURNEY_VIEWER.md](./JOURNEY_VIEWER.md)
- **Examples**: [example-usage.md](./example-usage.md)
- **Generator Utils**: [journey-generator.ts](./journey-generator.ts)
- **VitePress Help**: https://vitepress.dev/guide/markdown#vue

---

**Ready?** Start with the [example-usage.md](./example-usage.md) page to see live examples!
