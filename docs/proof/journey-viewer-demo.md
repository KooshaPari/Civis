# Journey Viewer - Interactive Demo

The **Journey Viewer** is an interactive component for visualizing user journeys through the DINOForge mod platform. It displays step-by-step screenshots with annotations, assertions, and playback controls.

## Features

- **Frame Navigation**: Click thumbnails or use Previous/Next buttons
- **Playback Controls**: Play through all frames automatically with adjustable speed
- **Annotations**: Visual overlays showing regions of interest with color-coded labels
- **Assertions**: See what each frame must contain or avoid
- **Keyboard Shortcuts**:
  - `←/→` arrow keys: Navigate frames
  - `Space`: Play/Pause
- **Responsive Design**: Works on desktop, tablet, and mobile

## Example Journey: Game Launch

This example demonstrates the full lifecycle of launching DINO with DINOForge mod loaded:

<script setup>
import JourneyViewer from '../.vitepress/theme/components/JourneyViewer.vue'

const exampleJourney = {
  id: 'us-f1-1-game-launch',
  intent: 'User launches the game and verifies DINOForge mod is loaded',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'steam-launch',
      intent: 'Steam window with game in library - verify game is available',
      screenshot_path: '/proof/screenshots/placeholder-1.png',
      assertions: {
        must_contain: ['Diplomacy is Not an Option', 'Play button'],
        must_not_contain: ['Error', 'Fatal']
      }
    },
    {
      index: 1,
      slug: 'game-launching',
      intent: 'Game splash screen during startup - verify initialization',
      screenshot_path: '/proof/screenshots/placeholder-2.png',
      assertions: {
        must_contain: ['Unity logo', 'Loading bar'],
        must_not_contain: ['Error dialog']
      }
    },
    {
      index: 2,
      slug: 'game-main-menu',
      intent: 'Game main menu loaded successfully - verify game is responsive',
      screenshot_path: '/proof/screenshots/placeholder-3.png',
      assertions: {
        must_contain: ['New Game', 'Load Game', 'Settings'],
        must_not_contain: ['Fatal error', 'Exception']
      }
    },
    {
      index: 3,
      slug: 'mod-overlay-visible',
      intent: 'DINOForge mod overlay appears via F10 key - verify mod is active',
      screenshot_path: '/proof/screenshots/placeholder-4.png',
      assertions: {
        must_contain: ['DINOForge', 'Loaded Packs', 'Settings'],
        must_not_contain: []
      }
    },
    {
      index: 4,
      slug: 'game-started',
      intent: 'Game session started with mod active - verify full integration',
      screenshot_path: '/proof/screenshots/placeholder-5.png',
      assertions: {
        must_contain: ['Game board', 'Units', 'Resources'],
        must_not_contain: ['Unhandled exception', 'Stack trace']
      }
    }
  ]
}

const annotations = {
  0: [
    {
      bbox: { x: 1650, y: 50, width: 200, height: 100 },
      label: 'Play Button',
      type: 'passed'
    }
  ],
  1: [
    {
      bbox: { x: 850, y: 450, width: 200, height: 30 },
      label: 'Loading Bar',
      type: 'info'
    }
  ],
  2: [
    {
      bbox: { x: 750, y: 300, width: 400, height: 60 },
      label: 'Main Menu',
      type: 'passed'
    },
    {
      bbox: { x: 800, y: 400, width: 300, height: 40 },
      label: 'New Game Button',
      type: 'passed'
    }
  ],
  3: [
    {
      bbox: { x: 50, y: 50, width: 300, height: 200 },
      label: 'DINOForge Overlay',
      type: 'passed'
    }
  ],
  4: [
    {
      bbox: { x: 100, y: 100, width: 150, height: 100 },
      label: 'Game Board',
      type: 'passed'
    }
  ]
}
</script>

<JourneyViewer 
  :journey="exampleJourney"
  title="US-F1.1: Game Launch Journey"
  :annotations="annotations"
/>

## How to Use Journey Viewer

### Basic Implementation

```vue
<script setup>
import JourneyViewer from './.vitepress/theme/components/JourneyViewer.vue'

const myJourney = {
  id: 'my-journey',
  intent: 'Journey description',
  keyframe_count: 3,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'step-1',
      intent: 'First step intent',
      screenshot_path: '/path/to/image.png',
      assertions: {
        must_contain: ['text1', 'text2'],
        must_not_contain: ['error']
      }
    },
    // ... more steps
  ]
}
</script>

<JourneyViewer 
  :journey="myJourney"
  title="My Journey Title"
/>
```

### With Annotations

Add visual overlays to highlight important regions:

```vue
const annotations = {
  0: [
    {
      bbox: { x: 100, y: 100, width: 200, height: 150 },
      label: 'Button to Click',
      type: 'passed'  // or 'failed', 'info'
    }
  ]
}

<JourneyViewer 
  :journey="myJourney"
  title="Annotated Journey"
  :annotations="annotations"
/>
```

## Props Reference

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `journey` | Journey | Yes | Journey manifest with steps and metadata |
| `title` | String | No | Display title for the viewer |
| `annotations` | Record<number, Annotation[]> | No | Frame annotations keyed by step index |

### Journey Interface

```typescript
interface Journey {
  id: string
  intent: string
  keyframe_count: number
  passed: boolean
  steps: JourneyStep[]
}

interface JourneyStep {
  index: number
  slug: string
  intent: string
  screenshot_path: string
  assertions?: {
    must_contain?: string[]
    must_not_contain?: string[]
  }
}

interface Annotation {
  bbox: { x: number; y: number; width: number; height: number }
  label: string
  type: 'passed' | 'failed' | 'info'
}
```

## Color Scheme

The component uses VitePress theme colors with semantic highlights:

- **Green (Passed)**: `#22c55e` - Successful assertions and valid regions
- **Red (Failed)**: `#ef4444` - Errors or failed assertions
- **Blue (Info)**: `#3b82f6` - Informational highlights
- **Dark backgrounds**: Follow VitePress dark mode preferences

## Tips for Creating Journeys

1. **Keep frames focused**: Each frame should represent one major action or transition
2. **Use clear intent descriptions**: Describe what the user is trying to accomplish
3. **Be specific with assertions**: Use concrete, verifiable statements
4. **Annotate key elements**: Help readers understand where to look
5. **Test playback**: Verify the journey flows logically at different speeds

## Integration with CI/CD

Journey Viewer integrates seamlessly with automated testing:

- Store journey manifests in version control
- Generate from test automation frameworks
- Update assertions based on test results
- Use as living documentation for feature validation

---

**Next Steps:**
- Create journeys for your features
- Store manifests in `docs/proof/journeys/manifests/`
- Reference in documentation via `<JourneyViewer>`
- Share with stakeholders for visual validation
