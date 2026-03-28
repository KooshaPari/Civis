# RND-004: Web Renderer Decision — Pixi.js v8 + React 19 for CivLab Web Client

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

Pixi.js v8 is the recommended 2D web renderer for CivLab. It provides WebGPU-first rendering with automatic WebGL2 fallback, native React 19 integration via `@pixi/react`, GPU-instanced particle rendering capable of 1M+ sprites at 60fps, and a mature TypeScript-first API. For the Phase 2 3D upgrade path, Babylon.js can coexist on the same page via shared WebGL context or separate canvas layering. The decision is Pixi.js v8 + React 19 for 2D, with a clean `IRenderer` interface contract enabling future Babylon.js 3D substitution.

---

## Research Findings

### 1. Pixi.js v8 Architecture and Performance

#### WebGPU-First with WebGL Fallback

Pixi.js v8 represents a ground-up rewrite that targets WebGPU as the primary rendering backend, with automatic fallback to WebGL2 on browsers that do not yet support WebGPU. This is configured via the renderer preference system:

```typescript
import { Application, autoDetectRenderer } from 'pixi.js';

// Auto-detect best available renderer
const app = new Application();
await app.init({
    preference: 'webgpu',       // Try WebGPU first
    // Falls back to WebGL2 automatically
    width: 1920,
    height: 1080,
    antialias: true,
    backgroundColor: 0x1a1a2e,
});
```

Key architectural changes in v8:
- **Reactive render loop**: v8 only updates what has changed, dramatically reducing CPU overhead for static scenes
- **Unified shader system**: Single shader language compiles to both WebGPU (WGSL) and WebGL (GLSL)
- **Scene graph optimizations**: Dirty-flag propagation means unchanged subtrees cost near-zero CPU

#### Benchmark Results (Bunnymark)

| Scenario | v7 CPU Time | v8 CPU Time | Improvement |
|----------|-------------|-------------|-------------|
| 100k sprites, all moving | ~50ms | ~15ms | **3.3x faster** |
| 100k sprites, static | ~21ms | ~0.12ms | **175x faster** |

The static scene optimization is critical for CivLab: hex map terrain tiles are largely static between turns, meaning the renderer will spend near-zero CPU time on the map background during animations and UI interactions.

#### ParticleContainer — GPU Instancing for Mass Sprites

The v8 `ParticleContainer` is completely rewritten to use GPU instancing, achieving dramatically higher throughput than v7:

| Configuration | M3 MacBook Pro @ 60fps |
|---------------|------------------------|
| Sprites in Container | 200,000 |
| Particles in ParticleContainer | **1,000,000** |

ParticleContainer uses lightweight `Particle` objects (not full `Sprite`) with a static/dynamic property split:
- **Dynamic properties** (position, rotation): uploaded to GPU every frame
- **Static properties** (texture, anchor, tint): uploaded only on explicit `update()` call

This maps well to CivLab's needs:
- Unit tokens on the map: dynamic position, static texture — use ParticleContainer
- Terrain hexes: fully static — use regular Container with reactive updates
- UI overlays: mixed — use standard Sprite/Container hierarchy

#### TilingSprite for Hex Terrain

Each hex tile can be rendered as a `TilingSprite` with the appropriate hex texture. For large maps, `@pixi/tilemap` provides optimized batch rendering:

```typescript
import { TilingSprite, Texture } from 'pixi.js';

const hexTile = new TilingSprite({
    texture: Texture.from('hex-grassland.png'),
    width: 64,
    height: 74, // Hex height for pointy-top
});
hexTile.position.set(hexToPixelX(q, r), hexToPixelY(q, r));
```

### 2. @pixi/react — React 19 Integration

#### Rebuilt for React 19

`@pixi/react` v8 was rebuilt from scratch for React 19. It uses React 19's improved reconciler and the new `extend` API pattern:

```typescript
import { Application, extend } from '@pixi/react';
import { Container, Sprite, Graphics, Text } from 'pixi.js';

// Register only the Pixi components you use
extend({ Container, Sprite, Graphics, Text });

function GameMap({ hexes }: { hexes: HexTile[] }) {
    return (
        <Application width={1920} height={1080} background={0x1a1a2e}>
            <container>
                {hexes.map(hex => (
                    <sprite
                        key={hex.id}
                        texture={hex.texture}
                        x={hex.pixelX}
                        y={hex.pixelY}
                    />
                ))}
            </container>
        </Application>
    );
}
```

Key characteristics:
- **Tree-shakeable**: Only imported Pixi components are bundled
- **TypeScript-first**: Full generic typing for Container children types
- **React 19 exclusive**: Uses React 19 internals; does not support React 18
- **Declarative**: Pixi scene graph maps to JSX component tree

#### Installation

```bash
npm install pixi.js@^8.2.6 @pixi/react
```

Or scaffold a new project:
```bash
npm create pixi.js@latest --template framework-react
```

### 3. TypeScript Strict Mode Compatibility

Pixi.js v8 is written in TypeScript and ships with full type definitions. Specific TypeScript features:

- **Generic Container typing** (v8.1.0+): `Container\<Sprite\>` enforces child types
- **DTS bundles**: Single definition file with all exports under `PIXI` namespace
- **Strict mode**: v8 is developed with `strict: true` in its own tsconfig; consumer projects using `strict: true` will not encounter type errors from Pixi's definitions

No known issues with TypeScript strict mode in v8. The `@pixi/react` package also ships strict-compatible types.

### 4. Tilemap Plugin for Hex Maps

#### @pixi/tilemap (Official)

- **v5.0.1+** supports Pixi.js v8 via the extension system
- Optimized for rectangular/orthogonal tile grids
- Does NOT have native hexagonal tile support
- Good for background terrain rendering with GPU-optimized batching

#### pixi-tiledmap (Community)

- Supports Tiled editor `.tmx` format including **hexagonal orientation**
- Uses the modern Assets/LoadParser extension system for v8
- Can import hex maps designed in the Tiled map editor

#### Recommended Approach for CivLab

Use a custom hex-coordinate-to-pixel mapping layer (axial coordinates) on top of standard Pixi.js Sprites/Containers. The hex math is straightforward:

```typescript
// Pointy-top hex layout
const HEX_SIZE = 32;
const SQRT3 = Math.sqrt(3);

function hexToPixel(q: number, r: number): { x: number; y: number } {
    return {
        x: HEX_SIZE * (SQRT3 * q + (SQRT3 / 2) * r),
        y: HEX_SIZE * (3 / 2) * r,
    };
}
```

This avoids depending on a tilemap plugin for hex layout, while still allowing `@pixi/tilemap` for batch-optimized rendering of the tile textures themselves.

### 5. 2D-to-3D Upgrade Path: Pixi.js + Babylon.js Coexistence

#### Architecture for Coexistence

Pixi.js (2D) and Babylon.js (3D) can coexist on the same page using two approaches:

**Approach A: Separate Canvases (Recommended)**
- Pixi renders to one `\<canvas\>` for 2D UI, minimaps, HUD
- Babylon renders to another `\<canvas\>` for the 3D game world
- CSS layering (`z-index`) composites them visually
- Each renderer owns its own WebGL/WebGPU context independently
- Simpler, avoids shared-state bugs

**Approach B: Shared WebGL Context**
- Both renderers share a single WebGL context
- Requires careful state management (save/restore GL state between renderers)
- Babylon.js has official community documentation on this pattern
- More complex, but avoids multiple-canvas compositing overhead
- Known issue: PBRMaterial in Babylon can cause rendering conflicts with Pixi

**Recommendation**: Use Approach A (separate canvases). CivLab Phase 1 is 2D-only. When Phase 2 adds 3D, the 3D canvas replaces the 2D game-world canvas while Pixi continues handling 2D UI overlays.

#### Official Integration Resources

- Pixi.js docs: "Mixing PixiJS and Three.js" guide (same principles apply to Babylon.js)
- Babylon.js docs: "Babylon.js and Pixi.js" community extension page
- Both confirm the separate-canvas approach works reliably

### 6. Comparison with Alternatives

#### Phaser 3

| Factor | Pixi.js v8 | Phaser 3 |
|--------|------------|----------|
| Rendering backend | WebGPU + WebGL2 | WebGL1 (WebGPU experimental) |
| React integration | Official `@pixi/react` | None (imperative only) |
| TypeScript | Native, strict-compatible | Bolted-on types |
| Scene management | Bring your own | Opinionated (Scenes, GameObjects) |
| Bundle size | Tree-shakeable | Monolithic (~1MB) |
| 100k sprite perf | 1M particles @ 60fps | ~50k before dropping frames |
| Hex map support | Manual + tilemap plugins | Built-in tilemap (rectangular only) |

Pixi.js v8 wins on performance, React compatibility, and TypeScript quality. Phaser's opinionated scene system adds unnecessary overhead for CivLab, which has its own ECS and game state management.

#### Raw WebGL/WebGPU

Manual WebGL/WebGPU would provide maximum control but requires implementing:
- Sprite batching
- Texture atlas management
- Scene graph and culling
- Text rendering
- Interaction/hit testing

This is thousands of lines of rendering infrastructure that Pixi provides out of the box. Not justified for CivLab.

---

## Decision

**Pixi.js v8 + React 19** via `@pixi/react` for the CivLab web client.

Rationale:
1. **Performance**: 175x improvement for static scenes; 1M particle capacity at 60fps far exceeds CivLab's hex-map rendering needs
2. **React 19**: Official first-party React 19 integration with declarative JSX scene graph
3. **TypeScript**: Strict-mode compatible, generic-typed containers
4. **WebGPU future-proofing**: WebGPU-first with automatic WebGL2 fallback
5. **3D upgrade path**: Clean separation via IRenderer interface; Babylon.js slots in for Phase 2 via separate canvas
6. **Ecosystem**: Active maintenance, 42k+ GitHub stars, professional backing

---

## Implementation Contract

### IRenderer Interface

Both the 2D (Pixi) and future 3D (Babylon) renderers must implement this interface:

```typescript
/**
 * IRenderer — Abstraction over 2D and 3D rendering backends.
 * Phase 1: Pixi2DRenderer implements this.
 * Phase 2: Babylon3DRenderer implements this.
 */
interface IRenderer {
    /** Initialize the renderer and attach to the target DOM element. */
    init(config: RendererConfig): Promise<void>;

    /** Destroy the renderer and release all GPU resources. */
    destroy(): void;

    /** Resize the rendering viewport. */
    resize(width: number, height: number): void;

    /** Set camera position and zoom for the game world view. */
    setCamera(center: WorldCoord, zoom: number): void;

    /** Get the current camera state. */
    getCamera(): CameraState;

    /** Convert screen coordinates to world coordinates. */
    screenToWorld(screenX: number, screenY: number): WorldCoord;

    /** Convert world coordinates to screen coordinates. */
    worldToScreen(worldX: number, worldY: number): ScreenCoord;

    /**
     * Render a complete frame from the given render state.
     * The renderer does NOT own game state — it receives a snapshot each frame.
     */
    renderFrame(state: RenderState): void;

    /** Register a callback for user interaction events on the game world. */
    onWorldInteraction(callback: (event: WorldInteractionEvent) => void): void;

    /** Get performance metrics from the last rendered frame. */
    getFrameMetrics(): FrameMetrics;
}

interface RendererConfig {
    /** Target DOM element to attach the canvas to. */
    container: HTMLElement;

    /** Initial viewport width. */
    width: number;

    /** Initial viewport height. */
    height: number;

    /** Preferred rendering backend. */
    preference: 'webgpu' | 'webgl2' | 'auto';

    /** Device pixel ratio override (default: window.devicePixelRatio). */
    resolution?: number;

    /** Enable antialiasing (default: true). */
    antialias?: boolean;
}

interface CameraState {
    center: WorldCoord;
    zoom: number;
    viewportBounds: { minX: number; minY: number; maxX: number; maxY: number };
}

interface WorldCoord {
    x: number;
    y: number;
}

interface ScreenCoord {
    x: number;
    y: number;
}

interface RenderState {
    /** Hex tiles visible in the current viewport. */
    visibleTiles: TileRenderData[];

    /** Units visible in the current viewport. */
    visibleUnits: UnitRenderData[];

    /** City markers visible in the current viewport. */
    visibleCities: CityRenderData[];

    /** Fog of war overlay state. */
    fogOfWar: FogOfWarData;

    /** Active animations (movement paths, combat effects). */
    animations: AnimationData[];

    /** Selection highlights and hover indicators. */
    selection: SelectionData | null;

    /** Turn counter for animation timing. */
    turnNumber: number;

    /** Frame timestamp in ms for smooth animations. */
    timestamp: number;
}

interface TileRenderData {
    q: number;
    r: number;
    terrain: string;       // texture key: 'grassland', 'desert', 'ocean', etc.
    elevation: number;     // 0-5 for terrain height shading
    resource?: string;     // optional resource overlay texture key
    improvement?: string;  // optional improvement overlay texture key
    owner?: number;        // player ID for border coloring
}

interface UnitRenderData {
    id: string;
    q: number;
    r: number;
    unitType: string;      // texture key: 'warrior', 'settler', etc.
    owner: number;         // player ID for tinting
    health: number;        // 0.0-1.0 for health bar
    facing: number;        // rotation in radians
    isMoving: boolean;
    movePath?: WorldCoord[];  // interpolation path for movement animation
}

interface CityRenderData {
    id: string;
    q: number;
    r: number;
    name: string;
    owner: number;
    population: number;
    hasWalls: boolean;
}

interface FogOfWarData {
    /** Set of "q,r" keys that are fully hidden (unexplored). */
    unexplored: Set<string>;

    /** Set of "q,r" keys that are in fog (explored but not visible). */
    fogged: Set<string>;
}

interface AnimationData {
    type: 'unit_move' | 'combat' | 'city_grow' | 'border_expand';
    progress: number;  // 0.0-1.0
    data: unknown;     // type-specific payload
}

interface SelectionData {
    selectedHex: { q: number; r: number } | null;
    selectedUnit: string | null;
    highlightedHexes: { q: number; r: number; color: number }[];
    movementRange: { q: number; r: number }[];
    attackRange: { q: number; r: number }[];
}

interface WorldInteractionEvent {
    type: 'click' | 'rightclick' | 'hover' | 'drag_start' | 'drag_move' | 'drag_end' | 'zoom';
    worldX: number;
    worldY: number;
    hexQ: number;
    hexR: number;
    button?: number;
    delta?: number;  // for zoom events
}

interface FrameMetrics {
    fps: number;
    drawCalls: number;
    triangles: number;
    cpuTimeMs: number;
    gpuTimeMs: number;
    visibleSprites: number;
}
```

### Pixi2DRenderer — Phase 1 Implementation Skeleton

```typescript
import { Application, Container, Sprite, ParticleContainer } from 'pixi.js';

class Pixi2DRenderer implements IRenderer {
    private app: Application;
    private worldContainer: Container;
    private tileLayer: Container;
    private unitLayer: ParticleContainer;
    private overlayLayer: Container;

    async init(config: RendererConfig): Promise<void> {
        this.app = new Application();
        await this.app.init({
            preference: config.preference === 'auto' ? undefined : config.preference,
            width: config.width,
            height: config.height,
            antialias: config.antialias ?? true,
            resolution: config.resolution ?? window.devicePixelRatio,
            autoDensity: true,
        });

        config.container.appendChild(this.app.canvas);

        // Layer hierarchy
        this.worldContainer = new Container();
        this.tileLayer = new Container();
        this.unitLayer = new ParticleContainer({
            dynamicProperties: { position: true, rotation: true },
            staticProperties: { texture: true, tint: true },
        });
        this.overlayLayer = new Container();

        this.worldContainer.addChild(this.tileLayer);
        this.worldContainer.addChild(this.unitLayer);
        this.worldContainer.addChild(this.overlayLayer);
        this.app.stage.addChild(this.worldContainer);
    }

    // ... remaining methods implement IRenderer contract
}
```

### Babylon3DRenderer — Phase 2 Placeholder

```typescript
import { Engine, Scene } from '@babylonjs/core';

class Babylon3DRenderer implements IRenderer {
    private engine: Engine;
    private scene: Scene;

    async init(config: RendererConfig): Promise<void> {
        const canvas = document.createElement('canvas');
        canvas.width = config.width;
        canvas.height = config.height;
        config.container.appendChild(canvas);

        this.engine = new Engine(canvas, config.antialias ?? true);
        this.scene = new Scene(this.engine);
        // Phase 2: 3D camera, hex terrain mesh, unit models
    }

    // ... remaining methods implement same IRenderer contract
}
```

### RendererFactory

```typescript
type RendererType = '2d' | '3d';

function createRenderer(type: RendererType): IRenderer {
    switch (type) {
        case '2d':
            return new Pixi2DRenderer();
        case '3d':
            return new Babylon3DRenderer();
    }
}
```

---

## Dependency Versions

| Package | Version | Purpose |
|---------|---------|---------|
| `pixi.js` | `^8.2.6` | Core 2D renderer |
| `@pixi/react` | `^8.0.0` | React 19 bindings |
| `@pixi/tilemap` | `^5.0.1` | Optimized tile batch rendering |
| `react` | `^19.0.0` | UI framework |
| `@babylonjs/core` | `^7.0.0` | Phase 2 3D renderer |

---

## Open Questions Remaining

1. **Viewport culling strategy**: Should culling be done at the application level (only passing visible tiles in `RenderState`) or at the renderer level (Pixi culling off-screen sprites)? Likely application level for consistency between 2D/3D.

2. **Texture atlas pipeline**: How will hex tile textures be packed into atlases? Pixi's `Assets` system supports spritesheet loading from TexturePacker/Aseprite output. Need to define the asset pipeline spec.

3. **Fog of war rendering**: Best approach for fog — alpha mask overlay, per-tile tinting, or shader-based? Per-tile tinting is simplest in Pixi; shader-based is more performant for large maps.

4. **WebGPU browser support timeline**: As of 2026, Chrome and Edge ship WebGPU by default. Firefox and Safari support is progressing. The automatic WebGL2 fallback handles this, but worth tracking.

5. **@pixi/tilemap hex integration**: The tilemap plugin optimizes rectangular grids. For hex grids, we need to verify whether its batch renderer still provides benefits over plain Container with Sprites, or if the custom hex layout negates the batching advantage.

---

## Sources

- [PixiJS v8 Launch Blog](https://pixijs.com/blog/pixi-v8-launches)
- [ParticleContainer v8 Blog](https://pixijs.com/blog/particlecontainer-v8)
- [PixiJS React v8 Blog](https://pixijs.com/blog/pixi-react-v8-live)
- [PixiJS Renderers Guide](https://pixijs.com/8.x/guides/components/renderers)
- [PixiJS v8 Migration Guide](https://pixijs.com/8.x/guides/migrations/v8)
- [@pixi/react GitHub](https://github.com/pixijs/pixi-react)
- [@pixi/tilemap npm](https://www.npmjs.com/package/@pixi/tilemap)
- [Babylon.js + Pixi.js Docs](https://doc.babylonjs.com/communityExtensions/Babylon.js+ExternalLibraries/BabylonJS_and_PixiJS/)
- [Mixing PixiJS and Three.js Guide](https://pixijs.com/8.x/guides/third-party/mixing-three-and-pixi)
- [PixiJS React Getting Started](https://react.pixijs.io/getting-started/)
