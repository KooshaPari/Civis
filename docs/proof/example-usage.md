# Journey Viewer - Complete Usage Examples

This page demonstrates all the ways to use the Journey Viewer component in your documentation.

## Simple Journey

The most basic usage with just a journey manifest:

<script setup>
import JourneyViewer from '../.vitepress/theme/components/JourneyViewer.vue'

const simpleJourney = {
  id: 'simple-example',
  intent: 'Basic journey showing 3 steps',
  keyframe_count: 3,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'first-step',
      intent: 'This is the first step',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23333" width="1920" height="1080"/><text x="960" y="540" font-size="48" fill="white" text-anchor="middle" dominant-baseline="middle">Step 1</text></svg>',
      assertions: {
        must_contain: ['Step 1'],
        must_not_contain: ['Error']
      }
    },
    {
      index: 1,
      slug: 'second-step',
      intent: 'This is the second step',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23444" width="1920" height="1080"/><text x="960" y="540" font-size="48" fill="white" text-anchor="middle" dominant-baseline="middle">Step 2</text></svg>',
      assertions: {
        must_contain: ['Step 2'],
        must_not_contain: ['Warning']
      }
    },
    {
      index: 2,
      slug: 'third-step',
      intent: 'This is the third step',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23555" width="1920" height="1080"/><text x="960" y="540" font-size="48" fill="white" text-anchor="middle" dominant-baseline="middle">Step 3 - Complete</text></svg>',
      assertions: {
        must_contain: ['Complete']
      }
    }
  ]
}

const annotatedJourney = {
  id: 'annotated-example',
  intent: 'Feature with annotated regions of interest',
  keyframe_count: 2,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'button-region',
      intent: 'Locate and highlight the main button',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23222" width="1920" height="1080"/><rect x="800" y="400" width="320" height="80" fill="%230066ff" opacity="0.3"/><text x="960" y="450" font-size="36" fill="white" text-anchor="middle" dominant-baseline="middle">Click Me</text></svg>',
      assertions: {
        must_contain: ['Click Me button']
      }
    },
    {
      index: 1,
      slug: 'result-view',
      intent: 'Show successful result after clicking',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23222" width="1920" height="1080"/><circle cx="960" cy="540" r="100" fill="%2322c55e" opacity="0.3"/><text x="960" y="540" font-size="48" fill="%2322c55e" text-anchor="middle" dominant-baseline="middle">✓ Success</text></svg>',
      assertions: {
        must_contain: ['Success message'],
        must_not_contain: ['Error']
      }
    }
  ]
}

const annotatedAnnotations = {
  0: [
    {
      bbox: { x: 800, y: 400, width: 320, height: 80 },
      label: 'Click Button',
      type: 'passed'
    }
  ],
  1: [
    {
      bbox: { x: 860, y: 440, width: 200, height: 200 },
      label: 'Success Indicator',
      type: 'passed'
    }
  ]
}

const failedJourney = {
  id: 'failed-example',
  intent: 'User encounters an error during workflow',
  keyframe_count: 2,
  passed: false,
  steps: [
    {
      index: 0,
      slug: 'success-state',
      intent: 'Initial successful state',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23222" width="1920" height="1080"/><text x="960" y="500" font-size="36" fill="%2322c55e" text-anchor="middle">Ready</text><rect x="800" y="550" width="320" height="60" fill="%23444" stroke="%23666" stroke-width="2"/><text x="960" y="585" font-size="24" fill="white" text-anchor="middle" dominant-baseline="middle">Submit</text></svg>',
      assertions: {
        must_contain: ['Ready', 'Submit button']
      }
    },
    {
      index: 1,
      slug: 'error-state',
      intent: 'Error appears after invalid action',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23222" width="1920" height="1080"/><rect x="100" y="100" width="1720" height="200" fill="%23ff4444" opacity="0.2" stroke="%23ef4444" stroke-width="3"/><text x="960" y="200" font-size="36" fill="%23ef4444" text-anchor="middle" dominant-baseline="middle">Error: Invalid input</text></svg>',
      assertions: {
        must_contain: ['Error message'],
        must_not_contain: []
      }
    }
  ]
}

const failedAnnotations = {
  1: [
    {
      bbox: { x: 100, y: 100, width: 1720, height: 200 },
      label: 'Error Alert',
      type: 'failed'
    }
  ]
}

const multiAnnotationJourney = {
  id: 'multi-annotation',
  intent: 'Complex interface with multiple regions of interest',
  keyframe_count: 1,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'complex-ui',
      intent: 'Complex user interface with multiple elements',
      screenshot_path: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23111" width="1920" height="1080"/><rect x="50" y="50" width="300" height="400" fill="%23222" stroke="%23444" stroke-width="2"/><text x="200" y="100" font-size="20" fill="white" text-anchor="middle">Sidebar</text><rect x="400" y="50" width="1470" height="400" fill="%23333" stroke="%23555" stroke-width="2"/><text x="1100" y="300" font-size="28" fill="white" text-anchor="middle">Content Area</text><rect x="400" y="500" width="1470" height="530" fill="%23222" stroke="%23444" stroke-width="2"/><text x="1100" y="760" font-size="28" fill="white" text-anchor="middle">Details Panel</text></svg>',
      assertions: {
        must_contain: ['Sidebar', 'Content Area', 'Details Panel'],
        must_not_contain: ['Loading', 'Error']
      }
    }
  ]
}

const multiAnnotations = {
  0: [
    {
      bbox: { x: 50, y: 50, width: 300, height: 400 },
      label: 'Navigation',
      type: 'passed'
    },
    {
      bbox: { x: 400, y: 50, width: 1470, height: 400 },
      label: 'Main Content',
      type: 'info'
    },
    {
      bbox: { x: 400, y: 500, width: 1470, height: 530 },
      label: 'Details',
      type: 'passed'
    }
  ]
}

const speedDemoJourney = {
  id: 'speed-demo',
  intent: 'Demonstrate different playback speeds',
  keyframe_count: 5,
  passed: true,
  steps: Array.from({ length: 5 }, (_, i) => ({
    index: i,
    slug: `frame-${i}`,
    intent: `Frame ${i + 1} of 5`,
    screenshot_path: `data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080"><rect fill="%23${200 + i * 50}" width="1920" height="1080"/><text x="960" y="540" font-size="64" fill="white" text-anchor="middle" dominant-baseline="middle">Frame ${i + 1}</text></svg>`,
    assertions: { must_contain: [`Frame ${i + 1}`] }
  }))
}
</script>

<JourneyViewer :journey="simpleJourney" title="Simple Example" />

Try it:
- Click **Play** to auto-advance
- Use arrow keys to navigate
- Press **Space** to toggle playback
- Click a thumbnail to jump

---

## Journey with Annotations

Annotations highlight important regions on screenshots:

<JourneyViewer 
  :journey="annotatedJourney" 
  title="With Annotations"
  :annotations="annotatedAnnotations"
/>

Notice:
- Green boxes highlight successful regions
- Annotations are only shown on relevant frames
- Labels appear above the bounding boxes

---

## Failed Journey

A journey where assertions fail:

<JourneyViewer 
  :journey="failedJourney"
  title="Failed Journey - Error Handling"
  :annotations="failedAnnotations"
/>

Notice:
- Status badge shows "✗ Failed"
- Error boxes are highlighted in red
- Can still use to document error states and recovery paths

---

## Multi-Type Annotations

Using different annotation types (passed, failed, info):

<JourneyViewer 
  :journey="multiAnnotationJourney"
  title="Multiple Annotation Types"
  :annotations="multiAnnotations"
/>

Color guide:
- **Green** (passed) - Working features
- **Blue** (info) - Informational highlights
- **Red** (failed) - Problem areas

---

## Playback Speed Demonstration

The same journey at different speeds:

<JourneyViewer 
  :journey="speedDemoJourney"
  title="Playback Speed Options"
/>

**Try these:**
1. Click **Play** (Normal speed - 1s per frame)
2. Change to **Slow** (2s per frame) and play again
3. Change to **Fast** (500ms per frame) and play again

---

## Code Integration Examples

### From TypeScript Test Suite

```typescript
import { createJourney } from './journey-generator'

const testResults = {
  id: 'e2e-test-game-launch',
  intent: 'E2E test: Game launch with mod loaded',
  steps: [
    {
      name: 'Launch Steam',
      screenshot: 'e2e-results/01.png',
      expectedText: ['Steam library', 'Play button'],
      success: true
    },
    // ... more test steps
  ]
}

const journey = createJourney({
  id: testResults.id,
  intent: testResults.intent,
  steps: testResults.steps.map(step => ({
    intent: step.name,
    screenshot: step.screenshot,
    assertions: {
      must_contain: step.expectedText,
      must_not_contain: step.forbiddenText
    }
  }))
})
```

### From JSON Manifest File

```vue
<script setup>
import manifest from './journeys/us-f1-1/manifest.json'
import annotations from './journeys/us-f1-1/annotations.json'
</script>

<JourneyViewer :journey="manifest" :annotations="annotations" />
```

### Dynamically From API

```typescript
async function loadJourney(journeyId: string) {
  const response = await fetch(`/api/journeys/${journeyId}`)
  const journey = await response.json()
  return journey
}

// In component:
const journey = await loadJourney('us-f1-1-game-launch')
```

---

## Best Practices Demonstrated

### Do ✓

```json
{
  "id": "feature-id-step-name",
  "steps": [
    {
      "slug": "meaningful-step-identifier",
      "intent": "User clicks the Start button",
      "assertions": {
        "must_contain": ["Game loaded", "Ready to play"],
        "must_not_contain": ["Error loading", "Fatal exception"]
      }
    }
  ]
}
```

### Don't ✗

```json
{
  "id": "test1",
  "steps": [
    {
      "slug": "step1",
      "intent": "Do stuff",
      "assertions": {
        "must_contain": ["ok"]
      }
    }
  ]
}
```

---

## Next Steps

1. **Create a Journey**: Follow the manifest format above
2. **Add Screenshots**: Save to `docs/proof/screenshots/`
3. **Write Assertions**: Be specific and verifiable
4. **Add Annotations** (optional): Highlight key regions
5. **Embed in Docs**: Use `<JourneyViewer>` in markdown
6. **Share & Review**: Get feedback from team

See [JOURNEY_VIEWER.md](./JOURNEY_VIEWER.md) for full documentation.
