---
title: Advanced Feature Journeys (Tier 2)
description: Journeys for scenario system, UI customization, installer, and infrastructure
---

<script setup>
import JourneyViewer from '../.vitepress/theme/components/JourneyViewer.vue'

// Scenario Domain Journey
const scenarioDomainJourney = {
  id: 'us-scenario-domain',
  intent: 'Scenario domain configures quests, victory conditions, and scripted events',
  keyframe_count: 6,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'map-loaded',
      intent: 'Custom scenario map loaded with initial state',
      screenshot_path: '/proof/screenshots/scenario-1.png',
      assertions: { must_contain: ['Scenario loaded', 'Victory conditions', 'Objectives'] }
    },
    {
      index: 1,
      slug: 'quests-available',
      intent: 'Quest log populated with mission objectives',
      screenshot_path: '/proof/screenshots/scenario-2.png',
      assertions: { must_contain: ['Primary objective', 'Secondary quests', 'Rewards'] }
    },
    {
      index: 2,
      slug: 'conditions-tracked',
      intent: 'Victory and defeat conditions actively tracked',
      screenshot_path: '/proof/screenshots/scenario-3.png',
      assertions: { must_contain: ['Condition monitor', 'Progress tracking'] }
    },
    {
      index: 3,
      slug: 'event-triggered',
      intent: 'Scripted event triggered by gameplay condition',
      screenshot_path: '/proof/screenshots/scenario-4.png',
      assertions: { must_contain: ['Event fired', 'Story progression'] }
    },
    {
      index: 4,
      slug: 'difficulty-scaled',
      intent: 'Difficulty scaled by DifficultyScaler',
      screenshot_path: '/proof/screenshots/scenario-5.png',
      assertions: { must_contain: ['Difficulty level', 'Enemy scaling'] }
    },
    {
      index: 5,
      slug: 'victory-achieved',
      intent: 'Scenario victory achieved - objectives complete',
      screenshot_path: '/proof/screenshots/scenario-6.png',
      assertions: { must_contain: ['Victory achieved', 'Mission complete', 'Score'] }
    }
  ]
}

// UI Domain Journey
const uiDomainJourney = {
  id: 'us-ui-domain',
  intent: 'UI domain customizes HUD elements, menus, and themes',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'hud-elements-defined',
      intent: 'HUD elements defined via registry (health bars, resources, minimap)',
      screenshot_path: '/proof/screenshots/ui-1.png',
      assertions: { must_contain: ['HUD definition', 'Element registry'] }
    },
    {
      index: 1,
      slug: 'theme-loaded',
      intent: 'Custom theme loaded (colors, fonts, positioning)',
      screenshot_path: '/proof/screenshots/ui-2.png',
      assertions: { must_contain: ['Theme config', 'Color scheme', 'Layout'] }
    },
    {
      index: 2,
      slug: 'hud-rendered',
      intent: 'HUD elements rendered with custom theme',
      screenshot_path: '/proof/screenshots/ui-3.png',
      assertions: { must_contain: ['HUD visible', 'Custom colors', 'No errors'] }
    },
    {
      index: 3,
      slug: 'menu-customized',
      intent: 'In-game menu customized with mod-specific options',
      screenshot_path: '/proof/screenshots/ui-4.png',
      assertions: { must_contain: ['Mod menu', 'Custom options', 'Settings'] }
    },
    {
      index: 4,
      slug: 'interaction-verified',
      intent: 'UI interactions verified (clicks, selections)',
      screenshot_path: '/proof/screenshots/ui-5.png',
      assertions: { must_contain: ['Interaction working', 'State updated'] }
    }
  ]
}

// Installer Journey
const installerJourney = {
  id: 'us-installer-setup',
  intent: 'User installs DINOForge and configures game integration',
  keyframe_count: 6,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'installer-launched',
      intent: 'DINOForge installer launched (GUI or PowerShell)',
      screenshot_path: '/proof/screenshots/installer-1.png',
      assertions: { must_contain: ['Welcome', 'Setup wizard', 'License'] }
    },
    {
      index: 1,
      slug: 'requirements-checked',
      intent: 'System requirements verified (.NET, BepInEx, game path)',
      screenshot_path: '/proof/screenshots/installer-2.png',
      assertions: { must_contain: ['.NET detected', 'Game path found', 'All checks pass'] }
    },
    {
      index: 2,
      slug: 'dependencies-installed',
      intent: 'Dependencies installed (BepInEx, config manager)',
      screenshot_path: '/proof/screenshots/installer-3.png',
      assertions: { must_contain: ['BepInEx installed', 'Plugins configured'] }
    },
    {
      index: 3,
      slug: 'runtime-deployed',
      intent: 'DINOForge runtime deployed to game directory',
      screenshot_path: '/proof/screenshots/installer-4.png',
      assertions: { must_contain: ['DLL deployed', 'Config created'] }
    },
    {
      index: 4,
      slug: 'verification-run',
      intent: 'Installation verified - game launched and mod detected',
      screenshot_path: '/proof/screenshots/installer-5.png',
      assertions: { must_contain: ['Game launched', 'DINOForge loaded'] }
    },
    {
      index: 5,
      slug: 'setup-complete',
      intent: 'Installation complete - ready for pack usage',
      screenshot_path: '/proof/screenshots/installer-6.png',
      assertions: { must_contain: ['Setup complete', 'Ready to use'] }
    }
  ]
}

// NuGet Package Consumption Journey
const nugetConsumptionJourney = {
  id: 'us-nuget-consumption',
  intent: 'External developer consumes Bridge packages from NuGet.org',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'nuget-search',
      intent: 'Developer searches NuGet.org for DINOForge.Bridge packages',
      screenshot_path: '/proof/screenshots/nuget-1.png',
      assertions: { must_contain: ['NuGet.org', 'DINOForge.Bridge', 'Package found'] }
    },
    {
      index: 1,
      slug: 'package-installed',
      intent: 'Bridge.Client NuGet package installed via dotnet CLI',
      screenshot_path: '/proof/screenshots/nuget-2.png',
      assertions: { must_contain: ['$ dotnet add package', 'Successfully installed'] }
    },
    {
      index: 2,
      slug: 'reference-added',
      intent: 'Package reference added to project file',
      screenshot_path: '/proof/screenshots/nuget-3.png',
      assertions: { must_contain: ['PackageReference', 'DINOForge.Bridge.Client'] }
    },
    {
      index: 3,
      slug: 'code-integrated',
      intent: 'Developer uses GameClient API in their code',
      screenshot_path: '/proof/screenshots/nuget-4.png',
      assertions: { must_contain: ['using DINOForge.Bridge', 'GameClient client'] }
    },
    {
      index: 4,
      slug: 'app-running',
      intent: 'External application running with DINOForge Bridge integration',
      screenshot_path: '/proof/screenshots/nuget-5.png',
      assertions: { must_contain: ['Connected to game', 'Bridge working'] }
    }
  ]
}

// MCP Tools Journey
const mcpToolsJourney = {
  id: 'us-mcp-tools',
  intent: 'Developer uses MCP server tools for game automation and testing',
  keyframe_count: 6,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'mcp-running',
      intent: 'MCP server started and health check passes',
      screenshot_path: '/proof/screenshots/mcp-1.png',
      assertions: { must_contain: ['MCP server running', 'Health check OK', 'Port 8765'] }
    },
    {
      index: 1,
      slug: 'game-launched',
      intent: 'game_launch tool used to launch game instance',
      screenshot_path: '/proof/screenshots/mcp-2.png',
      assertions: { must_contain: ['Game launched', 'Window visible'] }
    },
    {
      index: 2,
      slug: 'screenshot-captured',
      intent: 'game_screenshot tool captures current game state',
      screenshot_path: '/proof/screenshots/mcp-3.png',
      assertions: { must_contain: ['Screenshot captured', 'File saved'] }
    },
    {
      index: 3,
      slug: 'entities-queried',
      intent: 'game_query_entities tool queries ECS world',
      screenshot_path: '/proof/screenshots/mcp-4.png',
      assertions: { must_contain: ['Entities found', '45K+ results'] }
    },
    {
      index: 4,
      slug: 'input-injected',
      intent: 'game_input tool sends keyboard/mouse input to game',
      screenshot_path: '/proof/screenshots/mcp-5.png',
      assertions: { must_contain: ['Input injected', 'Game responded'] }
    },
    {
      index: 5,
      slug: 'test-completed',
      intent: 'Automated test completed with all tools working',
      screenshot_path: '/proof/screenshots/mcp-6.png',
      assertions: { must_contain: ['Test passed', 'All tools functional'] }
    }
  ]
}

// Backend Selection Journey
const backendSelectionJourney = {
  id: 'us-backend-selection',
  intent: 'System auto-detects and selects optimal isolation backend',
  keyframe_count: 4,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'detection-started',
      intent: 'Backend detection initiated on game launch',
      screenshot_path: '/proof/screenshots/backend-1.png',
      assertions: { must_contain: ['Detecting backends', 'Checking availability'] }
    },
    {
      index: 1,
      slug: 'backends-checked',
      intent: 'All backend tiers checked (VDD > CreateDesktop > PlayCUA > Mock)',
      screenshot_path: '/proof/screenshots/backend-2.png',
      assertions: { must_contain: ['Backend status', 'Available: CreateDesktop, PlayCUA'] }
    },
    {
      index: 2,
      slug: 'optimal-selected',
      intent: 'Optimal backend selected and UI updated',
      screenshot_path: '/proof/screenshots/backend-3.png',
      assertions: { must_contain: ['Selected: PlayCUA', 'Status: Active'] }
    },
    {
      index: 3,
      slug: 'feature-verified',
      intent: 'Backend features working (screenshot, input, isolation)',
      screenshot_path: '/proof/screenshots/backend-4.png',
      assertions: { must_contain: ['Features working', 'Latency measured'] }
    }
  ]
}

// PhenoCompose Integration Journey
const phenocomposeJourney = {
  id: 'us-phenocompose-fleet',
  intent: 'PhenoCompose launches parallel game fleet for scale testing',
  keyframe_count: 5,
  passed: true,
  steps: [
    {
      index: 0,
      slug: 'phenocompose-available',
      intent: 'PhenoCompose binary detected and verified',
      screenshot_path: '/proof/screenshots/phenocompose-1.png',
      assertions: { must_contain: ['nanovms binary found', 'Version check passed'] }
    },
    {
      index: 1,
      slug: 'fleet-config',
      intent: 'Fleet configuration created (4 instances, snapshot baseline)',
      screenshot_path: '/proof/screenshots/phenocompose-2.png',
      assertions: { must_contain: ['Fleet config', '4 instances', 'Baseline snapshot'] }
    },
    {
      index: 2,
      slug: 'fleet-launched',
      intent: 'Parallel game fleet launched via nanovms CLI',
      screenshot_path: '/proof/screenshots/phenocompose-3.png',
      assertions: { must_contain: ['4 instances running', 'All healthy'] }
    },
    {
      index: 3,
      slug: 'testing-parallel',
      intent: 'Tests run in parallel across all 4 instances',
      screenshot_path: '/proof/screenshots/phenocompose-4.png',
      assertions: { must_contain: ['Parallel execution', '100% CPU utilization'] }
    },
    {
      index: 4,
      slug: 'results-collected',
      intent: 'Test results collected and aggregated from all instances',
      screenshot_path: '/proof/screenshots/phenocompose-5.png',
      assertions: { must_contain: ['Results aggregated', 'All tests complete'] }
    }
  ]
}
</script>

# Advanced Feature Journeys (Tier 2)

Journeys covering scenario system, UI customization, installer workflows, NuGet integration, MCP tools, backend selection, and PhenoCompose scaling.

---

## Domain Systems

### Scenario Domain: Quests & Victory Conditions

<JourneyViewer :journey="scenarioDomainJourney" title="Scenario Domain Plugin" />

---

### UI Domain: HUD Customization & Themes

<JourneyViewer :journey="uiDomainJourney" title="UI Domain Plugin" />

---

## Installation & Configuration

### DINOForge Installer: Setup Wizard

<JourneyViewer :journey="installerJourney" title="Complete Installation Workflow" />

---

## Package Distribution

### NuGet Consumption: External Integration

<JourneyViewer :journey="nugetConsumptionJourney" title="Bridge Package Integration" />

---

## Automation & Infrastructure

### MCP Server Tools: Game Automation

<JourneyViewer :journey="mcpToolsJourney" title="MCP Tool Integration" />

---

### Backend Selection: Auto-Detection

<JourneyViewer :journey="backendSelectionJourney" title="Isolation Backend Selection" />

---

### PhenoCompose Fleet: Parallel Testing

<JourneyViewer :journey="phenocomposeJourney" title="Parallel Game Fleet Testing" />

---

**Total**: 7 advanced journeys covering all Tier 2 features and infrastructure components.
