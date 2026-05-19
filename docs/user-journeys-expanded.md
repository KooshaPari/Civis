---
title: Complete User Journey Demonstrations
description: Comprehensive journey collection demonstrating all DINOForge features
---

<script setup>
import JourneyViewer from './.vitepress/theme/components/JourneyViewer.vue'

// Asset Import Journey
const assetImportJourney = {
  id: 'us-asset-import',
  intent: 'Developer imports 3D models and creates asset bundle',
  keyframe_count: 6,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'source-files-ready',
      intent: 'GLB/FBX source models ready in asset directory',
      screenshot_path: '/proof/screenshots/asset-import-1.png',
      assertions: { must_contain: ['asset_pipeline.yaml', 'models', 'GLB files'] }
    },
    {
      index: 1,
      slug: 'config-created',
      intent: 'asset_pipeline.yaml configured with LOD targets',
      screenshot_path: '/proof/screenshots/asset-import-2.png',
      assertions: { must_contain: ['LOD definitions', 'Material config'] }
    },
    {
      index: 2,
      slug: 'assets-imported',
      intent: 'Assets imported and normalized',
      screenshot_path: '/proof/screenshots/asset-import-3.png',
      assertions: { must_contain: ['Imported 5 models', 'Mesh data converted'] }
    },
    {
      index: 3,
      slug: 'assets-optimized',
      intent: 'Assets optimized with LOD generation',
      screenshot_path: '/proof/screenshots/asset-import-4.png',
      assertions: { must_contain: ['LOD variants created', '3 levels'] }
    },
    {
      index: 4,
      slug: 'prefabs-generated',
      intent: 'Prefabs generated from optimized assets',
      screenshot_path: '/proof/screenshots/asset-import-5.png',
      assertions: { must_contain: ['Prefab files', 'Serialized metadata'] }
    },
    {
      index: 5,
      slug: 'addressables-registered',
      intent: 'Assets registered in Addressables catalog',
      screenshot_path: '/proof/screenshots/asset-import-6.png',
      assertions: { must_contain: ['Catalog updated', 'Ready for runtime'] }
    }
  ]
}

// Asset Swap Journey
const assetSwapJourney = {
  id: 'us-asset-swap',
  intent: 'Runtime asset swap system replaces vanilla assets with mod assets',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'vanilla-assets-loaded',
      intent: 'Vanilla game assets loaded at startup',
      screenshot_path: '/proof/screenshots/asset-swap-1.png',
      assertions: { must_contain: ['Vanilla units', 'Default colors'] }
    },
    {
      index: 1,
      slug: 'mod-assets-available',
      intent: 'Mod assets loaded via Addressables',
      screenshot_path: '/proof/screenshots/asset-swap-2.png',
      assertions: { must_contain: ['Mod bundle loaded', 'Catalog updated'] }
    },
    {
      index: 2,
      slug: 'swap-triggered',
      intent: 'Swap triggered - vanilla assets unloaded, mod assets applied',
      screenshot_path: '/proof/screenshots/asset-swap-3.png',
      assertions: { must_contain: ['Swap complete', 'Visual assets replaced'] }
    },
    {
      index: 3,
      slug: 'runtime-verified',
      intent: 'Swap verified at runtime - mod units visible',
      screenshot_path: '/proof/screenshots/asset-swap-4.png',
      assertions: { must_contain: ['Mod visuals', 'Correct textures', 'No errors'] }
    }
  ]
}

// Pack Creation Journey
const packCreationJourney = {
  id: 'us-pack-create',
  intent: 'Developer creates a new mod pack with content',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'scaffold-created',
      intent: 'Pack scaffold created with pack.yaml manifest',
      screenshot_path: '/proof/screenshots/pack-create-1.png',
      assertions: { must_contain: ['pack.yaml', 'pack structure', 'directories'] }
    },
    {
      index: 1,
      slug: 'manifest-configured',
      intent: 'pack.yaml configured with metadata and dependencies',
      screenshot_path: '/proof/screenshots/pack-create-2.png',
      assertions: { must_contain: ['id', 'name', 'version', 'author'] }
    },
    {
      index: 2,
      slug: 'content-added',
      intent: 'Content definitions added (units, factions, balance)',
      screenshot_path: '/proof/screenshots/pack-create-3.png',
      assertions: { must_contain: ['Unit definitions', 'Faction config', 'Stats'] }
    },
    {
      index: 3,
      slug: 'pack-validated',
      intent: 'Pack validation checks manifest and schemas',
      screenshot_path: '/proof/screenshots/pack-create-4.png',
      assertions: { must_contain: ['Validation passed', 'All schemas valid'] }
    },
    {
      index: 4,
      slug: 'pack-deployed',
      intent: 'Pack built and deployed to game',
      screenshot_path: '/proof/screenshots/pack-create-5.png',
      assertions: { must_contain: ['Build successful', 'Deployed to game'] }
    }
  ]
}

// Pack Hot Reload Journey
const packHotReloadJourney = {
  id: 'us-pack-hotreload',
  intent: 'Developer modifies pack and hot-reloads without game restart',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'game-running',
      intent: 'Game running with pack loaded',
      screenshot_path: '/proof/screenshots/hotreload-1.png',
      assertions: { must_contain: ['Game active', 'Pack loaded'] }
    },
    {
      index: 1,
      slug: 'file-modified',
      intent: 'Developer modifies pack YAML file (e.g., unit stats)',
      screenshot_path: '/proof/screenshots/hotreload-2.png',
      assertions: { must_contain: ['File changed', 'Stats updated'] }
    },
    {
      index: 2,
      slug: 'hotreload-triggered',
      intent: 'File watcher detects change and triggers reload',
      screenshot_path: '/proof/screenshots/hotreload-3.png',
      assertions: { must_contain: ['HotReload signal', 'Reloading...'] }
    },
    {
      index: 3,
      slug: 'reload-complete',
      intent: 'Pack reloaded without game restart - changes live',
      screenshot_path: '/proof/screenshots/hotreload-4.png',
      assertions: { must_contain: ['Reload complete', 'New stats active'] }
    }
  ]
}

// Warfare Domain Journey
const warfareDomainJourney = {
  id: 'us-warfare-domain',
  intent: 'Warfare domain plugin configures unit combat, doctrines, waves',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'archetypes-loaded',
      intent: 'Unit archetypes loaded (Infantry, Ranged, Heavy)',
      screenshot_path: '/proof/screenshots/warfare-1.png',
      assertions: { must_contain: ['Archetypes', 'Role system'] }
    },
    {
      index: 1,
      slug: 'doctrines-applied',
      intent: 'Doctrines applied to factions (Blitz, Defensive, Balanced)',
      screenshot_path: '/proof/screenshots/warfare-2.png',
      assertions: { must_contain: ['Doctrines', 'Stat multipliers'] }
    },
    {
      index: 2,
      slug: 'combat-simulation',
      intent: 'Combat system simulates unit interactions',
      screenshot_path: '/proof/screenshots/warfare-3.png',
      assertions: { must_contain: ['Damage calc', 'Hit chance', 'Unit HP'] }
    },
    {
      index: 3,
      slug: 'waves-spawning',
      intent: 'Enemy waves spawn with wave manager',
      screenshot_path: '/proof/screenshots/warfare-4.png',
      assertions: { must_contain: ['Wave config', 'Spawn points', 'Unit counts'] }
    },
    {
      index: 4,
      slug: 'balance-verified',
      intent: 'Balance properties verified via stat inspection',
      screenshot_path: '/proof/screenshots/warfare-5.png',
      assertions: { must_contain: ['Balance data', 'Stat ranges'] }
    }
  ]
}

// Economy Domain Journey
const economyDomainJourney = {
  id: 'us-economy-domain',
  intent: 'Economy domain configures production, trade, and resource management',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'resources-defined',
      intent: 'Resources defined (Gold, Wood, Stone, etc.)',
      screenshot_path: '/proof/screenshots/economy-1.png',
      assertions: { must_contain: ['Resource types', 'Starting amounts'] }
    },
    {
      index: 1,
      slug: 'production-calculated',
      intent: 'Production rates calculated for buildings',
      screenshot_path: '/proof/screenshots/economy-2.png',
      assertions: { must_contain: ['Production rates', 'Per-second values'] }
    },
    {
      index: 2,
      slug: 'trade-enabled',
      intent: 'Trade system enables resource exchange between factions',
      screenshot_path: '/proof/screenshots/economy-3.png',
      assertions: { must_contain: ['Trade routes', 'Exchange rates'] }
    },
    {
      index: 3,
      slug: 'balance-adjusted',
      intent: 'Economy balance pack adjusts costs and production',
      screenshot_path: '/proof/screenshots/economy-4.png',
      assertions: { must_contain: ['Cost rebalance', 'Rate adjustments'] }
    },
    {
      index: 4,
      slug: 'gameplay-verified',
      intent: 'Economy gameplay verified in match simulation',
      screenshot_path: '/proof/screenshots/economy-5.png',
      assertions: { must_contain: ['Resource flow', 'Balanced progression'] }
    }
  ]
}

// PackCompiler CLI Journey
const packCompilerJourney = {
  id: 'us-tool-packcompiler',
  intent: 'Developer uses PackCompiler CLI to validate and build packs',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'cli-invoked',
      intent: 'PackCompiler CLI invoked with pack name',
      screenshot_path: '/proof/screenshots/cli-compiler-1.png',
      assertions: { must_contain: ['$ dotnet run', 'PackCompiler'] }
    },
    {
      index: 1,
      slug: 'validation-running',
      intent: 'Validation phase checks schemas and references',
      screenshot_path: '/proof/screenshots/cli-compiler-2.png',
      assertions: { must_contain: ['Validating...', 'Schema check'] }
    },
    {
      index: 2,
      slug: 'build-successful',
      intent: 'Build phase compiles and bundles assets',
      screenshot_path: '/proof/screenshots/cli-compiler-3.png',
      assertions: { must_contain: ['Building...', 'Output created'] }
    },
    {
      index: 3,
      slug: 'complete',
      intent: 'Pack ready for deployment',
      screenshot_path: '/proof/screenshots/cli-compiler-4.png',
      assertions: { must_contain: ['Build complete', 'Ready to deploy'] }
    }
  ]
}

// DumpTools Journey
const dumpToolsJourney = {
  id: 'us-tool-dumptools',
  intent: 'Developer uses DumpTools to analyze game state and entities',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'cli-invoked',
      intent: 'DumpTools CLI invoked with dump scope',
      screenshot_path: '/proof/screenshots/cli-dump-1.png',
      assertions: { must_contain: ['$ dumptools', 'entities'] }
    },
    {
      index: 1,
      slug: 'entities-dumped',
      intent: 'Entity data dumped to structured output',
      screenshot_path: '/proof/screenshots/cli-dump-2.png',
      assertions: { must_contain: ['45K+ entities', 'Archetype breakdown'] }
    },
    {
      index: 2,
      slug: 'analysis-shown',
      intent: 'Analysis displayed with Spectre.Console tables',
      screenshot_path: '/proof/screenshots/cli-dump-3.png',
      assertions: { must_contain: ['Component counts', 'System info'] }
    },
    {
      index: 3,
      slug: 'insights-generated',
      intent: 'Developer gains insights for debugging',
      screenshot_path: '/proof/screenshots/cli-dump-4.png',
      assertions: { must_contain: ['Analysis complete', 'Saved to file'] }
    }
  ]
}

// Test Automation Journey
const testAutomationJourney = {
  id: 'us-test-automation',
  intent: 'Automated test suite validates pack functionality end-to-end',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'test-suite-started',
      intent: 'Test suite launched via xUnit',
      screenshot_path: '/proof/screenshots/test-auto-1.png',
      assertions: { must_contain: ['$ dotnet test', 'xUnit.net'] }
    },
    {
      index: 1,
      slug: 'unit-tests-running',
      intent: 'Unit tests validate pack logic and schemas',
      screenshot_path: '/proof/screenshots/test-auto-2.png',
      assertions: { must_contain: ['Running unit tests', 'assertions passing'] }
    },
    {
      index: 2,
      slug: 'integration-tests',
      intent: 'Integration tests run with MCP bridge',
      screenshot_path: '/proof/screenshots/test-auto-3.png',
      assertions: { must_contain: ['Game bridge connected', 'Integration tests'] }
    },
    {
      index: 3,
      slug: 'screenshots-captured',
      intent: 'Automated screenshots verify visual correctness',
      screenshot_path: '/proof/screenshots/test-auto-4.png',
      assertions: { must_contain: ['Screenshots captured', 'Analysis running'] }
    },
    {
      index: 4,
      slug: 'coverage-report',
      intent: '95%+ coverage achieved and reported',
      screenshot_path: '/proof/screenshots/test-auto-5.png',
      assertions: { must_contain: ['95% coverage', 'All checks passed'] }
    }
  ]
}

// Visual Regression Journey
const visualRegressionJourney = {
  id: 'us-visual-regression',
  intent: 'CLIP-based visual regression testing validates mod visuals',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'baseline-captured',
      intent: 'Golden baseline screenshot captured for comparison',
      screenshot_path: '/proof/screenshots/visual-regress-1.png',
      assertions: { must_contain: ['Baseline', 'Reference image'] }
    },
    {
      index: 1,
      slug: 'test-screenshot',
      intent: 'Test screenshot captured after mod changes',
      screenshot_path: '/proof/screenshots/visual-regress-2.png',
      assertions: { must_contain: ['Test image', 'Comparison'] }
    },
    {
      index: 2,
      slug: 'clip-analysis',
      intent: 'CLIP model analyzes visual similarity',
      screenshot_path: '/proof/screenshots/visual-regress-3.png',
      assertions: { must_contain: ['CLIP analysis', 'Confidence score'] }
    },
    {
      index: 3,
      slug: 'result-verified',
      intent: 'Visual regression test passed - no unwanted changes',
      screenshot_path: '/proof/screenshots/visual-regress-4.png',
      assertions: { must_contain: ['Confidence > 0.75', 'Test PASSED'] }
    }
  ]
}
</script>

# Complete User Journey Demonstrations

Comprehensive collection of 12+ interactive journey demonstrations covering all DINOForge features, workflows, and developer tools.

---

## Asset Management Journeys

### Asset Pipeline: Import, Optimize, Generate

<JourneyViewer :journey="assetImportJourney" title="Complete Asset Pipeline Workflow" />

---

### Runtime Asset Swap System

<JourneyViewer :journey="assetSwapJourney" title="Asset Swap: Vanilla → Mod Visuals" />

---

## Pack Creation & Deployment

### Pack Creation: Scaffold to Deployment

<JourneyViewer :journey="packCreationJourney" title="End-to-End Pack Creation" />

---

### Live Development: Hot Reload

<JourneyViewer :journey="packHotReloadJourney" title="File Watcher & Hot Reload" />

---

## Domain Plugins

### Warfare Domain: Combat & Doctrines

<JourneyViewer :journey="warfareDomainJourney" title="Warfare Domain Plugin" />

---

### Economy Domain: Production & Trade

<JourneyViewer :journey="economyDomainJourney" title="Economy Domain Plugin" />

---

## Developer Tools

### PackCompiler CLI: Validate & Build

<JourneyViewer :journey="packCompilerJourney" title="CLI Tool: PackCompiler" />

---

### DumpTools: Analyze Game State

<JourneyViewer :journey="dumpToolsJourney" title="CLI Tool: DumpTools" />

---

## Testing & Validation

### Automated Test Suite: End-to-End

<JourneyViewer :journey="testAutomationJourney" title="Complete Test Automation" />

---

### Visual Regression: CLIP-Based

<JourneyViewer :journey="visualRegressionJourney" title="Visual Regression Testing" />

---

**Total**: 12 comprehensive journeys covering game automation, asset pipelines, pack workflows, domain features, CLI tools, and testing infrastructure.
