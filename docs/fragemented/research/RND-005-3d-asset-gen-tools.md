# RND-005: Agentic 3D Asset Generation — Tool Comparison and Pipeline Recommendation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

For CivLab's 3D asset needs (~200 assets across units, buildings, terrain, and resources), a hybrid pipeline is recommended: **Meshy.ai** for hero units and high-visibility assets where quality and PBR textures matter most, and **Tripo3D** for bulk terrain/building assets where throughput and cost efficiency dominate. InstantMesh serves as a local fallback for rapid prototyping and cases where API latency or cost is unacceptable. Wonder3D is not recommended due to low resolution (256x256) and limited view coverage. Estimated total cost for 200 assets: **$160-$400** depending on quality tier distribution.

---

## Research Findings

### 1. Meshy.ai

#### Overview

Meshy.ai is a commercial AI-powered 3D model generation platform offering both text-to-3D and image-to-3D pipelines. As of 2025-2026, it is one of the most mature and widely-used 3D generation APIs for game asset creation.

#### API Documentation

**Text-to-3D Endpoint:**
```
POST https://api.meshy.ai/v2/text-to-3d
```

Key parameters:
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `prompt` | string | required | Object description (max 600 chars) |
| `art_style` | string | `"realistic"` | Style: realistic, cartoon, low-poly, sculpture, pbr |
| `topology` | string | `"triangle"` | triangle or quad |
| `target_polycount` | int | 30,000 | Range: 100-300,000 |
| `enable_pbr` | bool | false | Generate metallic, roughness, normal maps |

**Image-to-3D Endpoint:**
```
POST https://api.meshy.ai/v2/image-to-3d
```

Key parameters:
| Parameter | Type | Description |
|-----------|------|-------------|
| `image_url` | string | Input reference image |
| `enable_pbr` | bool | PBR map generation |
| `texture_prompt` | string | Text guidance for texturing (max 600 chars) |
| `texture_image_url` | string | 2D image to guide texturing |
| `should_remesh` | bool | Auto-remesh output |

**Output formats:** GLB, FBX, OBJ (+MTL), USDZ

**Remesh API:**
```
POST https://api.meshy.ai/v2/remesh
```
Converts to clean quad-dominant mesh topology, useful for game-engine import.

#### Quality Assessment

| Asset Type | Quality | Fidelity | Notes |
|------------|---------|----------|-------|
| Hard-surface props (weapons, buildings) | High | 80-90% | Best-in-class for architectural/mechanical |
| Terrain objects (rocks, trees) | High | 85-90% | Good organic shapes |
| Characters/creatures | Medium | 70-80% | Needs manual cleanup for animation-ready rigs |
| Stylized/low-poly | High | 85-95% | Excellent when `art_style: "low-poly"` |

**Polygon count control:**
- `target_polycount` parameter accepts 100-300,000
- Actual output varies based on prompt complexity
- Complex prompts may yield 50k+ polys requiring manual decimation
- For game assets, target 3,000-10,000 polys per asset

**Retopology:**
- Built-in remesh API produces quad-dominant meshes
- Suitable for direct import to game engines
- Not sufficient for character animation rigging (manual work needed)

#### Pricing

| Plan | Monthly Credits | Cost | API Access |
|------|-----------------|------|------------|
| Free | 100 | $0 | No |
| Pro | 1,000 | ~$20/mo | Yes |
| Studio | 4,000/seat | ~$60/mo/seat | Yes |

Via third-party platforms (fal.ai): ~$0.80 per text-to-3D generation.

**Cost estimate for 200 assets via Meshy:**
- At $0.80/generation, assuming 1.5 attempts per asset average: 200 * 1.5 * $0.80 = **$240**
- With Pro plan credits: ~$60-100 depending on credit efficiency

#### Style Consistency

Meshy supports an `art_style` parameter and text prompts can enforce style consistency. However, achieving truly consistent style across 50+ variants requires:
1. Using the same `art_style` setting for all assets
2. Including specific style keywords in every prompt (e.g., "low-poly stylized, flat shading, vibrant colors")
3. Using image-to-3D with a reference image from an established style sheet
4. Post-processing with the Text-to-Texture API to re-skin assets with a unified style

**Verdict:** Good for hero assets. Style consistency achievable with disciplined prompting but not guaranteed out-of-the-box for 50+ variants.

---

### 2. Tripo3D

#### Overview

Tripo3D is a newer entrant (2024-2025) that has rapidly improved quality, particularly for hard-surface assets. It supports text-to-3D and image-to-3D (including multi-image input as of v2.5).

#### API Documentation

**Authentication:** Bearer token via `Authorization: Bearer <token>` header.

**Endpoints:**
```
POST https://api.tripo3d.ai/v2/openapi/task
```

Task types: `text_to_model`, `image_to_model`, `multiview_to_model`, `refine_model`

**Quality tiers:**
| Tier | Output | Credit Cost |
|------|--------|-------------|
| Draft | Base mesh only, no texture | 10 credits |
| Standard | Baked texture + PBR model | 20-25 credits |
| HD | High-res baked texture + PBR | 40-50 credits |

**Quad mesh:** Optional, +5 credits. Useful for clean topology.

**Output formats:** GLB, FBX, OBJ, USDZ

#### Quality Assessment

| Asset Type | Quality | Notes |
|------------|---------|-------|
| Hard-surface (buildings, props) | High | Improved geometry in v2.5 |
| Organic (trees, terrain) | Medium-High | Multi-image input helps |
| Characters | Medium | Better than v1 but still needs cleanup |
| Consistent textures | Medium-High | PBR output with baked textures |

**Polygon counts:**
- Tripo does not expose a direct `target_polycount` parameter
- Output typically 10k-50k triangles depending on complexity
- Quad remesh available for cleaner topology
- Post-processing decimation needed for low-poly game assets

#### Pricing

**Credit-based model:**
- Standard text-to-3D with style: 25 credits
- HD image-to-3D with PBR + quad: 50 credits
- Third-party API (fal.ai): $0.20-$0.40 per model

**Cost estimate for 200 assets via Tripo:**
- At $0.30/model average: 200 * 1.3 attempts * $0.30 = **$78**
- Significantly cheaper than Meshy for bulk generation

#### Style Consistency

Tripo v2.5's multi-image input mode helps maintain consistency by allowing style reference images alongside the target prompt. However, the system lacks a formal "style lock" feature. For 50+ consistent variants:
1. Use multi-image input with style reference
2. Apply consistent prompting templates
3. Re-texture inconsistent outputs with a separate texturing pass

**Verdict:** Best cost-to-quality ratio for bulk assets. Multi-image input is a significant advantage for consistency.

---

### 3. InstantMesh (TencentARC)

#### Overview

InstantMesh is an open-source single-image-to-3D reconstruction model from TencentARC. It uses sparse-view large reconstruction models to generate meshes from a single input image in ~10 seconds.

#### GitHub Repository

- **URL:** https://github.com/TencentARC/InstantMesh
- **License:** Apache 2.0
- **Stars:** 3k+ (as of 2025)
- **Last updated:** Active development

#### Local Inference Requirements

| Requirement | Specification |
|-------------|---------------|
| Python | >= 3.10 |
| PyTorch | >= 2.1.0 |
| CUDA | >= 12.1 |
| GPU VRAM | ~8-12GB (for large variant) |
| xformers | 0.0.22.post7 |
| Inference time | ~10s per model on A100 |

**Model variants:** 4 sparse-view reconstruction variants + customized Zero123++ UNet

**Setup:**
```bash
conda create -n instantmesh python=3.10
conda activate instantmesh
pip install torch==2.1.0 torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
pip install xformers==0.0.22.post7
pip install -r requirements.txt
```

Models auto-download on first run (~2-4GB total).

#### Quality Assessment

| Factor | Rating | Notes |
|--------|--------|-------|
| Geometry quality | Medium | Good for simple objects, struggles with thin features |
| Texture quality | Low-Medium | Single-view inference limits texture coverage |
| Polygon count | Uncontrolled | Output is raw mesh, needs decimation |
| Back-side quality | Poor | Single image means back is hallucinated |
| Processing speed | Fast | ~10s/model on A100 |
| Consistency | Low | No style control mechanism |

#### Cost

- **Infrastructure cost only**: No per-model API fee
- A100 GPU rental: ~$1-3/hour
- At 360 models/hour throughput: ~$0.003-$0.008 per model
- M3 Max (local): MPS backend may work but significantly slower (~30-60s/model)

**Verdict:** Best for rapid prototyping and bulk generation where quality requirements are low. Not suitable for final hero assets without significant manual post-processing.

---

### 4. Wonder3D (CVPR 2024)

#### Overview

Wonder3D generates textured meshes from a single image using cross-domain diffusion for consistent multi-view normal maps and color images, followed by normal fusion for 3D reconstruction.

#### GitHub Repository

- **URL:** https://github.com/xxlong0/Wonder3D
- **License:** Research/Academic
- **Published:** CVPR 2024
- **Recent update:** Wonder3D++ (Dec 2024) extends the base model

#### Pipeline

1. Input single image
2. CLIP text embedding + camera parameters
3. Cross-domain diffusion generates 6 consistent views (normal + color)
4. Normal fusion algorithm reconstructs 3D geometry
5. Output: Textured mesh (2-3 minutes)

#### Quality Assessment

| Factor | Rating | Notes |
|--------|--------|-------|
| Geometry quality | Medium-High | Leading-level geometric detail vs prior work |
| Texture quality | Medium | 256x256 resolution limit |
| View coverage | Limited | 6 views only; occluded areas poorly reconstructed |
| Front-facing bias | Strong | Front-facing images produce best results |
| Processing speed | Slow | 2-3 minutes per model |
| Resolution | Low | 256x256 view resolution |

#### Limitations

- **256x256 resolution**: Major limitation for game assets requiring detail
- **6-view limitation**: Cannot cover full 360-degree object
- **Occlusion sensitivity**: Images with occlusions produce worse results
- **Research license**: Not clearly permissive for commercial use
- **Bug history**: Cross-domain attention CFG bug (fixed Aug 2024) caused misalignment

**Verdict:** Not recommended for CivLab production. Resolution too low, view coverage insufficient, and licensing unclear. Wonder3D++ may address some issues but not yet evaluated.

---

### 5. Comparative Analysis

#### Quality Comparison Matrix

| Factor | Meshy | Tripo3D | InstantMesh | Wonder3D |
|--------|-------|---------|-------------|----------|
| Geometry quality | 8/10 | 7/10 | 5/10 | 6/10 |
| Texture/PBR quality | 9/10 | 7/10 | 3/10 | 5/10 |
| Polygon control | Yes (100-300k) | Limited | None | None |
| Style consistency | 7/10 | 6/10 | 3/10 | 4/10 |
| Output formats | GLB/FBX/OBJ/USDZ | GLB/FBX/OBJ/USDZ | OBJ/GLB | OBJ |
| API maturity | Production | Production | Self-hosted | Research |
| Speed per asset | ~60s | ~30-60s | ~10s | ~150s |
| Cost per asset | $0.50-0.80 | $0.20-0.40 | ~$0.005 | Free (self-hosted) |
| Commercial license | Yes | Yes | Apache 2.0 | Research |

#### Cost Estimate for 200 Assets

| Scenario | Meshy Only | Tripo Only | Hybrid (Recommended) | InstantMesh Only |
|----------|------------|------------|----------------------|------------------|
| 50 hero units | $60 | $26 | Meshy: $60 | $0.75 |
| 50 buildings | $60 | $26 | Tripo: $26 | $0.75 |
| 50 terrain objects | $60 | $26 | Tripo: $26 | $0.75 |
| 50 resources/props | $60 | $26 | Tripo: $26 | $0.75 |
| **Total (1.5x retries)** | **$360** | **$156** | **$207** | **$4.50 + GPU** |

The hybrid approach allocates Meshy's superior quality to high-visibility hero assets while using Tripo's cost efficiency for bulk assets that appear smaller on screen.

---

## Decision

**Hybrid pipeline: Meshy.ai (hero assets) + Tripo3D (bulk assets) + InstantMesh (prototyping)**

### Asset Tier Classification

| Tier | Tool | Assets | Quality Target | Poly Budget |
|------|------|--------|----------------|-------------|
| **S-Tier** (hero units, wonders) | Meshy.ai | ~30 | High PBR, hand-reviewed | 5k-10k tris |
| **A-Tier** (standard buildings, units) | Tripo3D | ~80 | Standard PBR | 3k-5k tris |
| **B-Tier** (terrain, resources, props) | Tripo3D | ~70 | Standard, batch-generated | 1k-3k tris |
| **Prototype** (iteration, concept art) | InstantMesh | as-needed | Draft quality | unconstrained |

### Pipeline Workflow

```
1. Concept Art (2D)
   ├── SD XL generates reference images (see RND-006)
   └── Art director approves style sheet

2. 3D Generation
   ├── S-Tier: Meshy image-to-3D with PBR + style prompt
   ├── A-Tier: Tripo multi-image-to-3D with style reference
   └── B-Tier: Tripo text-to-3D batch generation

3. Post-Processing
   ├── Polygon decimation to budget (Blender CLI or meshoptimizer)
   ├── UV unwrap validation
   ├── PBR texture baking consistency check
   └── Quality gate: screenshot comparison vs style sheet

4. Export
   ├── GLB for web client (Pixi textures or Babylon meshes)
   ├── FBX for asset archive
   └── Thumbnail renders for asset browser
```

---

## Implementation Contract

### Asset Generation Service Interface

```typescript
interface IAssetGenerator {
    /** Generate a 3D model from a text description. */
    generateFromText(request: TextTo3DRequest): Promise<Asset3DResult>;

    /** Generate a 3D model from one or more reference images. */
    generateFromImages(request: ImageTo3DRequest): Promise<Asset3DResult>;

    /** Check the status of an async generation task. */
    getTaskStatus(taskId: string): Promise<TaskStatus>;

    /** Download the generated asset files. */
    downloadAsset(taskId: string, format: AssetFormat): Promise<Buffer>;
}

interface TextTo3DRequest {
    prompt: string;
    artStyle: 'realistic' | 'cartoon' | 'low-poly' | 'stylized';
    targetPolycount: number;
    enablePBR: boolean;
    tier: 'S' | 'A' | 'B' | 'prototype';
}

interface ImageTo3DRequest {
    imageUrls: string[];         // 1-4 reference images
    texturePrompt?: string;      // Optional texture guidance
    enablePBR: boolean;
    tier: 'S' | 'A' | 'B' | 'prototype';
}

interface Asset3DResult {
    taskId: string;
    status: 'pending' | 'processing' | 'completed' | 'failed';
    downloadUrls?: {
        glb?: string;
        fbx?: string;
        obj?: string;
        usdz?: string;
    };
    metadata?: {
        polycount: number;
        hasPBR: boolean;
        generationTimeMs: number;
        provider: 'meshy' | 'tripo' | 'instantmesh';
    };
}

type AssetFormat = 'glb' | 'fbx' | 'obj' | 'usdz';

interface TaskStatus {
    taskId: string;
    status: 'pending' | 'processing' | 'completed' | 'failed';
    progress: number;      // 0.0-1.0
    estimatedTimeMs?: number;
    errorMessage?: string;
}
```

### Provider Routing Logic

```typescript
function selectProvider(request: TextTo3DRequest | ImageTo3DRequest): 'meshy' | 'tripo' | 'instantmesh' {
    switch (request.tier) {
        case 'S':
            return 'meshy';
        case 'A':
        case 'B':
            return 'tripo';
        case 'prototype':
            return 'instantmesh';
    }
}
```

---

## Open Questions Remaining

1. **Meshy v6 evaluation**: Meshy 6 was announced with WaveSpeedAI integration. Need to evaluate whether quality improvements change the tier allocation.

2. **Tripo v2.5 multi-image consistency**: How many reference images are optimal for style consistency? Need empirical testing with CivLab's art style.

3. **Animation rigging pipeline**: None of the evaluated tools produce animation-ready rigged meshes. Need a separate pipeline for character animation (Mixamo auto-rigging or manual).

4. **Texture atlas consolidation**: Generated assets each have individual textures. For web rendering performance, these need to be consolidated into shared texture atlases. Need to define the atlas packing pipeline.

5. **LOD generation**: For the web client, assets need 2-3 LOD levels (high for close-up, low for zoomed-out map view). Can meshoptimizer or Simplygon handle this automatically?

6. **InstantMesh on Apple Silicon**: MPS backend compatibility with InstantMesh's xformers dependency is uncertain. Need to test local inference on M3 Max.

---

## Sources

- [Meshy.ai Pricing](https://www.meshy.ai/pricing)
- [Meshy API Docs — Text-to-3D](https://docs.meshy.ai/en/api/text-to-3d)
- [Meshy API Docs — Image-to-3D](https://docs.meshy.ai/en/api/image-to-3d)
- [Meshy API Docs — Remesh](https://docs.meshy.ai/en/api/remesh)
- [Tripo3D Pricing](https://www.tripo3d.ai/pricing)
- [Tripo3D API Platform](https://www.tripo3d.ai/api)
- [Tripo3D API Billing](https://platform.tripo3d.ai/docs/billing)
- [Tripo3D v2.5 on fal.ai](https://fal.ai/models/tripo3d/tripo/v2.5/image-to-3d/api)
- [InstantMesh GitHub](https://github.com/TencentARC/InstantMesh)
- [Wonder3D GitHub](https://github.com/xxlong0/Wonder3D)
- [Wonder3D CVPR 2024 Paper](https://openaccess.thecvf.com/content/CVPR2024/papers/Long_Wonder3D_Single_Image_to_3D_using_Cross-Domain_Diffusion_CVPR_2024_paper.pdf)
- [Meshy Review — AIquiks](https://aiquiks.com/ai-tools/meshy)
- [Tripo AI Review 2025 — Skywork](https://skywork.ai/blog/tripo-ai-review-2025/)
- [Meshy 6 on WaveSpeedAI](https://wavespeed.ai/models/wavespeed-ai/meshy6/text-to-3d)
