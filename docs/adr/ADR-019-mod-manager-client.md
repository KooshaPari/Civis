# ADR-019: Local Mod Manager Client (M12)

**Status:** Accepted  
**Date:** 2026-03-29  
**Supersedes:** N/A  
**Related:** ADR-011 (Desktop Companion App)

## Context

DINOForge currently lacks a local mod management interface within the Desktop Companion application. Users need a way to:

1. **Browse** available mods from local folders or remote HTTP catalog endpoints
2. **Check for updates** by comparing installed pack versions against catalog versions
3. **Detect conflicts** between active packs, including missing dependencies and incompatible versions

The Desktop Companion (WinUI 3 app) already provides Dashboard, Pack List, Debug Panel, and Settings views. Extending it with mod management features provides a unified, familiar experience without introducing a separate application.

## Decision

We will extend the Desktop Companion application with three new pages and supporting services:

### New Views

| Page | Purpose |
|------|---------|
| **Browse** | Display available packs from a catalog (local folder or HTTP URL), with search and type filtering |
| **Updates** | Compare installed versions against catalog versions, show available updates |
| **Conflicts** | Run dependency resolution, display conflicts, missing dependencies, and computed load order |

### New Services

| Service | Responsibility |
|---------|----------------|
| `IModCatalogService` / `ModCatalogService` | Load pack metadata from file:// or https:// catalog sources |
| `UpdateCheckService` | Semantic version comparison between installed and catalog packs |
| `ConflictDetectionService` | Use SDK's `PackDependencyResolver` to detect conflicts and compute load order |

### New ViewModels

| ViewModel | Responsibility |
|-----------|----------------|
| `BrowseViewModel` | Catalog loading, search/filter, install action |
| `UpdateViewModel` | Update detection and download launch |
| `ConflictViewModel` | Conflict analysis, dependency tree visualization |

### Architecture Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Catalog Source │────▶│  ModCatalogSvc  │────▶│  BrowseViewModel│
│  (file:// or    │     │  (IModCatalog)  │     │                 │
│   https://)     │     └─────────────────┘     └────────┬────────┘
└─────────────────┘                                       │
                                                          ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Installed Packs│────▶│ UpdateCheckSvc  │────▶│  UpdateViewModel│
│  (PackViewModel)│     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                          │
┌─────────────────┐     ┌─────────────────┐              │
│  PackDependency │◀────│ConflictDetection│◀─────────────┘
│  Resolver (SDK) │     │  Service        │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  Load Order    │     │  ConflictReport │
│  (DependencyRes)│     │  - Conflicts    │
└─────────────────┘     │  - MissingDeps  │
                       │  - LoadOrderErrs │
                       └─────────────────┘
```

## User Stories

### Browse Packs
- **Given** a catalog URL or folder path
- **When** user enters the source and clicks Load
- **Then** the application displays all packs with name, author, version, type, and description
- **And** installed packs are marked with an "Installed" badge

### Search and Filter
- **Given** a loaded catalog
- **When** user types in the search box or selects a type filter
- **Then** the list updates to show only matching packs

### Check for Updates
- **Given** installed packs and a catalog with newer versions
- **When** user clicks "Check for Updates"
- **Then** the application shows which packs have updates available
- **And** displays current → available version

### Detect Conflicts
- **Given** multiple active packs with dependencies and conflicts
- **When** user navigates to the Conflicts page and clicks Analyze
- **Then** the application shows:
  - Packs declaring conflicts with other active packs
  - Packs with missing dependencies
  - The computed load order

### Dependency Tree
- **Given** loaded packs
- **When** user selects a pack from the load order list
- **Then** the dependency tree for that pack is displayed

## Consequences

### Positive
- Unified experience within Desktop Companion
- Clear visibility into pack conflicts before game launch
- Support for both local and remote catalog sources
- Semantic version comparison for accurate update detection

### Negative
- Increases Desktop Companion complexity
- Requires users to configure catalog sources

### Risks
- Remote catalog endpoints may be unavailable or slow
- Complex dependency graphs may produce large load order lists

## Implementation Notes

### Services
- All services use `ILogger<T>` for diagnostic logging
- Services are registered in `App.xaml.cs` DI container
- ViewModels use `ObservableObject` + `RelayCommand` pattern (CommunityToolkit.Mvvm)

### Views
- All XAML views use existing DINOForge theme styles
- Views get their ViewModels from DI via `App.Services.GetRequiredService<T>()`
- Code-behind handles navigation events only

### SDK Integration
- `ConflictDetectionService` wraps `PackDependencyResolver` from `DINOForge.SDK.Dependencies`
- Provides higher-level UI-focused reporting on top of SDK primitives
