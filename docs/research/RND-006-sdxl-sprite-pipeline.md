# RND-006: Stable Diffusion XL + ControlNet Setup for Consistent 2D Game Sprites

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

The recommended pipeline for generating consistent 2D game sprites is **Automatic1111 WebUI API** (`/sdapi/v1/txt2img` and `/sdapi/v1/img2img`) with **ControlNet OpenPose** for consistent character poses across 8 directional facings, combined with a **custom LoRA** trained on 20-50 reference images to lock the art style. SDXL provides seed-based determinism when the same model, seed, prompt, and deterministic scheduler (Euler, DDIM) are used. ComfyUI is a viable alternative with more flexibility for complex workflows but higher integration complexity. The full pipeline: A1111 API + ControlNet OpenPose + Custom LoRA + seed determinism.

---

## Research Findings

### 1. Automatic1111 (A1111) API

#### Overview

Automatic1111's stable-diffusion-webui exposes a REST API at `/sdapi/v1/*` when launched with the `--api` flag. This is the simplest integration path for programmatic sprite generation.

#### Setup

```bash
# Launch with API enabled
python launch.py --api --listen --port 7860
```

The API documentation is auto-generated and available at `http://localhost:7860/docs` (Swagger UI).

#### Core Endpoints

**Text-to-Image:**
```
POST /sdapi/v1/txt2img
```

```json
{
    "prompt": "game character warrior, pixel art style, front-facing, white background, <lora:civlab_style:0.8>",
    "negative_prompt": "blurry, low quality, deformed, watermark, text",
    "width": 1024,
    "height": 1024,
    "steps": 30,
    "cfg_scale": 7.0,
    "sampler_name": "Euler",
    "scheduler": "Normal",
    "seed": 42,
    "batch_size": 1,
    "n_iter": 1,
    "restore_faces": false,
    "enable_hr": false,
    "alwayson_scripts": {}
}
```

**Image-to-Image:**
```
POST /sdapi/v1/img2img
```

Same parameters as txt2img plus:
- `init_images`: Array of base64-encoded input images
- `denoising_strength`: 0.0-1.0 (lower = more faithful to input)

**Response format:**
```json
{
    "images": ["base64_encoded_png_data"],
    "parameters": { ... },
    "info": "generation_info_json_string"
}
```

#### ControlNet Integration via A1111 API

ControlNet is passed through the `alwayson_scripts` parameter:

```json
{
    "prompt": "game character warrior, side view, walking pose, <lora:civlab_style:0.8>",
    "width": 1024,
    "height": 1024,
    "steps": 30,
    "cfg_scale": 7.0,
    "sampler_name": "Euler",
    "seed": 42,
    "alwayson_scripts": {
        "controlnet": {
            "args": [
                {
                    "enabled": true,
                    "module": "openpose_full",
                    "model": "control_v11p_sd15_openpose",
                    "weight": 1.0,
                    "image": "<base64_encoded_openpose_skeleton>",
                    "resize_mode": "Crop and Resize",
                    "lowvram": false,
                    "processor_res": 512,
                    "guidance_start": 0.0,
                    "guidance_end": 1.0,
                    "control_mode": "Balanced"
                }
            ]
        }
    }
}
```

#### Other Useful Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /sdapi/v1/sd-models` | List available checkpoint models |
| `GET /sdapi/v1/loras` | List available LoRAs |
| `GET /sdapi/v1/samplers` | List available samplers |
| `POST /sdapi/v1/options` | Set/get runtime options (model swap, etc.) |
| `GET /sdapi/v1/progress` | Get generation progress |
| `POST /sdapi/v1/interrupt` | Cancel current generation |

---

### 2. ComfyUI API (Alternative)

#### Overview

ComfyUI provides a node-based workflow system with an HTTP API. Workflows are defined as JSON graphs where each node has an ID, type, and connections to other nodes.

#### API Endpoint

```
POST http://localhost:8188/prompt
```

```json
{
    "client_id": "unique-client-id",
    "prompt": {
        "3": {
            "class_type": "KSampler",
            "inputs": {
                "seed": 42,
                "steps": 30,
                "cfg": 7.0,
                "sampler_name": "euler",
                "scheduler": "normal",
                "denoise": 1.0,
                "model": ["4", 0],
                "positive": ["6", 0],
                "negative": ["7", 0],
                "latent_image": ["5", 0]
            }
        },
        "4": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": "sd_xl_base_1.0.safetensors"
            }
        },
        "5": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width": 1024,
                "height": 1024,
                "batch_size": 1
            }
        },
        "6": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": "game character warrior, pixel art style",
                "clip": ["4", 1]
            }
        },
        "7": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": "blurry, low quality, deformed",
                "clip": ["4", 1]
            }
        },
        "8": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": ["3", 0],
                "vae": ["4", 2]
            }
        },
        "9": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "warrior",
                "images": ["8", 0]
            }
        }
    }
}
```

#### ComfyUI Parameterization

Variables can be injected using handlebars syntax (`{{prompt}}`, `{{seed}}`) when using wrapper frameworks. Direct API usage requires modifying the JSON node values programmatically before submission.

#### WebSocket for Progress

ComfyUI prefers WebSocket connections for real-time progress:
```
ws://localhost:8188/ws?clientId=unique-client-id
```

Messages include `execution_start`, `executing` (per-node progress), and `executed` (completion with output paths).

#### ComfyUI vs A1111 API Comparison

| Factor | A1111 API | ComfyUI API |
|--------|-----------|-------------|
| Simplicity | Simple REST, flat JSON | Complex graph JSON |
| ControlNet | `alwayson_scripts` parameter | Dedicated ControlNet nodes |
| Workflow flexibility | Fixed pipeline | Arbitrary node graphs |
| Progress tracking | Polling (`/progress`) | WebSocket (real-time) |
| Batch operations | Built-in batch_size/n_iter | Manual graph construction |
| Documentation | Swagger auto-docs | Minimal, community-driven |
| Dynamic params | Direct JSON fields | Template injection or graph mutation |
| Setup complexity | `--api` flag | Already API-native |

**Recommendation:** A1111 API for CivLab's sprite pipeline. The use case is straightforward (txt2img + ControlNet + LoRA), and A1111's simpler REST interface reduces integration overhead. ComfyUI's graph-based approach is overkill for this pipeline.

---

### 3. ControlNet OpenPose for 8-Directional Character Sprites

#### The 8-Direction Sprite Sheet Problem

CivLab game units need sprites for 8 facing directions:

```
    N
  NW  NE
W       E
  SW  SE
    S
```

Each direction requires a consistent character in a different pose/orientation. Without ControlNet, SDXL would generate unpredictable poses, making sprite sheets inconsistent.

#### OpenPose Skeleton Control

ControlNet OpenPose uses a stick-figure skeleton to precisely control character pose and orientation. For each of the 8 directions, a pre-defined skeleton is created:

```
Direction skeletons (simplified):

N (back):     NE (3/4 back):  E (side right):  SE (3/4 front):
  O              O               O                  O
 /|\            /|\             /|                  /|\
  |              |              |                    |
 / \            / \            / \                  / \

S (front):    SW (3/4 front): W (side left):  NW (3/4 back):
  O              O                O               O
 /|\            /|\               |\              /|\
  |              |                |                |
 / \            / \              / \              / \
```

Each skeleton is saved as a 512x512 or 1024x1024 PNG with the OpenPose color-coded keypoint format:
- Red: right limbs
- Blue: left limbs
- Yellow: torso/spine
- Green: face points

#### Workflow for 8-Direction Generation

```python
import requests
import base64
import json

A1111_URL = "http://localhost:7860"

DIRECTIONS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW']

def load_skeleton(direction: str) -> str:
    """Load pre-made OpenPose skeleton for given direction as base64."""
    with open(f"assets/skeletons/{direction}.png", "rb") as f:
        return base64.b64encode(f.read()).decode()

def generate_sprite(
    unit_type: str,
    direction: str,
    seed: int,
    lora_name: str = "civlab_style",
    lora_weight: float = 0.8,
) -> bytes:
    """Generate a single sprite for a unit type and direction."""

    direction_prompts = {
        'N':  'back view, facing away',
        'NE': 'three-quarter back view, slight right turn',
        'E':  'side view, facing right, profile',
        'SE': 'three-quarter front view, slight right turn',
        'S':  'front view, facing camera',
        'SW': 'three-quarter front view, slight left turn',
        'W':  'side view, facing left, profile',
        'NW': 'three-quarter back view, slight left turn',
    }

    prompt = (
        f"game character {unit_type}, {direction_prompts[direction]}, "
        f"white background, centered, full body, "
        f"<lora:{lora_name}:{lora_weight}>"
    )

    payload = {
        "prompt": prompt,
        "negative_prompt": "blurry, low quality, deformed, watermark, text, cropped, partial body",
        "width": 1024,
        "height": 1024,
        "steps": 30,
        "cfg_scale": 7.0,
        "sampler_name": "Euler",
        "seed": seed,
        "alwayson_scripts": {
            "controlnet": {
                "args": [{
                    "enabled": True,
                    "module": "openpose_full",
                    "model": "control_v11p_sd15_openpose",
                    "weight": 1.0,
                    "image": load_skeleton(direction),
                    "resize_mode": "Crop and Resize",
                    "guidance_start": 0.0,
                    "guidance_end": 1.0,
                    "control_mode": "Balanced",
                }]
            }
        }
    }

    response = requests.post(f"{A1111_URL}/sdapi/v1/txt2img", json=payload)
    result = response.json()
    return base64.b64decode(result["images"][0])

def generate_sprite_sheet(unit_type: str, base_seed: int) -> dict[str, bytes]:
    """Generate all 8 directional sprites for a unit type."""
    sprites = {}
    for i, direction in enumerate(DIRECTIONS):
        # Use base_seed + direction offset for reproducibility
        # while maintaining different compositions per direction
        sprites[direction] = generate_sprite(
            unit_type=unit_type,
            direction=direction,
            seed=base_seed + i,
        )
    return sprites
```

#### OpenPose Model Selection for SDXL

| Model | Base | Notes |
|-------|------|-------|
| `control_v11p_sd15_openpose` | SD 1.5 | Most mature, widest adoption |
| `controlnet-openpose-sdxl-1.0` | SDXL | Native SDXL resolution (1024x1024) |
| `t2i-adapter-openpose-sdxl` | SDXL | T2I-Adapter variant, lighter weight |

For SDXL, use `controlnet-openpose-sdxl-1.0` for native resolution support.

---

### 4. LoRA Training for Art Style Consistency

#### Why LoRA?

Without a trained LoRA, SDXL generates images in a generic style that varies across prompts. A LoRA fine-tunes the model on a specific art style using a small dataset, ensuring all generated sprites share a consistent visual identity.

#### Training Dataset Requirements

| Parameter | Recommended | Notes |
|-----------|-------------|-------|
| Number of reference images | **20-50** | Sweet spot: 20-25 images with diversity |
| Minimum images | 10 | Below this, overfitting risk increases |
| Image resolution | >= 1024x1024 | Match SDXL native resolution |
| Image diversity | High | Different poses, lighting, backgrounds |
| Style consistency | Critical | All images must share target art style |
| Image format | PNG | Lossless, no compression artifacts |

**Dataset preparation:**
1. Collect 20-50 reference images in the target art style
2. Crop/resize to 1024x1024
3. Write captions for each image (auto-captioning via BLIP-2 or manual)
4. Organize as pairs: `image_001.png` + `image_001.txt`

#### Training Parameters

| Parameter | Recommended | Range | Notes |
|-----------|-------------|-------|-------|
| **Network Rank (dim)** | 32 | 8-64 | Higher = more capacity, more VRAM |
| **Network Alpha** | 16 | half of rank | Effective LR = LR * (alpha/rank) |
| **Learning Rate** | 1e-4 | 5e-5 to 2e-4 | Use Adafactor optimizer |
| **Training Steps** | 1,500-2,000 | 1,000-3,000 | More steps risk overfitting |
| **Batch Size** | 1-4 | depends on VRAM | Larger = smoother gradients |
| **Optimizer** | Adafactor | or AdamW8bit | Adafactor is memory-efficient |
| **LR Scheduler** | constant | or cosine | Constant with 0 warmup works well |
| **Loss Type** | smooth_l1 | or mse | `huber_schedule: "snr"` |
| **Resolution** | 1024x1024 | | Match SDXL native |

#### Training Infrastructure

| Hardware | Training Time | Notes |
|----------|---------------|-------|
| M3 Max (48GB) | 2-4 hours | Via mps backend, slower but usable |
| A100 (80GB) | 30-60 minutes | Fastest, recommended for iteration |
| RTX 4090 (24GB) | 1-2 hours | Good balance of cost/speed |

**Tools:**
- **kohya_ss**: Most popular LoRA trainer for SDXL, supports all parameters above
- **OneTrainer**: Alternative with GUI, good for experimentation
- **ai-toolkit**: Simplified training scripts

#### Training Command (kohya_ss)

```bash
accelerate launch train_network.py \
    --pretrained_model_name_or_path="stabilityai/stable-diffusion-xl-base-1.0" \
    --train_data_dir="./training_data" \
    --output_dir="./output_lora" \
    --output_name="civlab_style" \
    --network_module="networks.lora" \
    --network_dim=32 \
    --network_alpha=16 \
    --learning_rate=1e-4 \
    --lr_scheduler="constant" \
    --lr_warmup_steps=0 \
    --optimizer_type="Adafactor" \
    --max_train_steps=2000 \
    --resolution="1024,1024" \
    --train_batch_size=1 \
    --mixed_precision="bf16" \
    --save_every_n_steps=500 \
    --caption_extension=".txt" \
    --xformers \
    --cache_latents
```

#### LoRA Usage in Generation

Once trained, the LoRA is referenced in prompts:

```
<lora:civlab_style:0.8>
```

The weight (0.8) controls influence strength:
- **0.5-0.7**: Subtle style influence, more prompt flexibility
- **0.7-0.9**: Strong style lock, recommended for consistency
- **0.9-1.0**: Very strong, may reduce prompt adherence

---

### 5. Seed Determinism in SDXL

#### Determinism Guarantee

SDXL IS deterministic given identical:
- Model checkpoint (exact same .safetensors file)
- Seed value
- Prompt and negative prompt
- Sampler/scheduler
- Steps, CFG scale, resolution
- ControlNet inputs (if used)
- LoRA weights (if used)

#### Deterministic Schedulers

| Scheduler | Deterministic | Notes |
|-----------|---------------|-------|
| **Euler** | YES | Converges reliably, robust baseline |
| **DDIM** | YES | Deterministic, good quality |
| **DPM++ 2M** | YES | High quality, deterministic |
| **DPM++ 2M Karras** | YES | Karras noise schedule variant |
| **Euler Ancestral** | NO | Injects random noise per step |
| **DPM++ 2S a** | NO | Stochastic variant |
| **DPM++ SDE** | NO | Stochastic differential equation |

**Recommendation:** Use **Euler** as the default scheduler for CivLab sprite generation. It is deterministic, converges reliably, and produces consistent results.

#### Cross-Platform Caveats

Determinism is NOT guaranteed across:
- Different GPU hardware (NVIDIA A100 vs RTX 4090 may produce subtly different results)
- Different CUDA versions
- CPU vs GPU execution
- Different operating systems (floating-point rounding)

For CivLab, this means the sprite generation pipeline should run on a **fixed, dedicated machine** (or container with pinned CUDA version) to ensure reproducibility.

#### Seed Strategy for Sprite Sheets

```python
# Seed allocation strategy
UNIT_TYPE_SEEDS = {
    'warrior': 1000,
    'archer': 2000,
    'settler': 3000,
    'scout': 4000,
    # ... etc
}

DIRECTION_OFFSETS = {
    'N': 0, 'NE': 1, 'E': 2, 'SE': 3,
    'S': 4, 'SW': 5, 'W': 6, 'NW': 7,
}

def get_seed(unit_type: str, direction: str) -> int:
    return UNIT_TYPE_SEEDS[unit_type] + DIRECTION_OFFSETS[direction]

# warrior facing NE -> seed 1001
# warrior facing S  -> seed 1004
# archer facing E   -> seed 2002
```

This ensures:
- Same unit+direction always produces same output (reproducibility)
- Different directions use different seeds (variety)
- Different unit types use different seed ranges (no collisions)

---

### 6. Full Recommended Workflow

```
┌─────────────────────────────────────────────────────┐
│                 SPRITE GENERATION PIPELINE           │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. STYLE DEFINITION                                │
│     ├── Collect 20-50 reference images              │
│     ├── Train SDXL LoRA (rank=32, 2000 steps)       │
│     └── Validate: generate 10 test images           │
│                                                     │
│  2. POSE PREPARATION                                │
│     ├── Create 8 OpenPose skeletons (one per dir)   │
│     ├── Validate: overlay on reference images        │
│     └── Store as 1024x1024 PNGs                     │
│                                                     │
│  3. SPRITE GENERATION                               │
│     ├── A1111 API with --api flag                   │
│     ├── ControlNet OpenPose (SDXL model)            │
│     ├── LoRA: <lora:civlab_style:0.8>               │
│     ├── Scheduler: Euler (deterministic)            │
│     ├── Seed: unit_base + direction_offset          │
│     └── Resolution: 1024x1024                       │
│                                                     │
│  4. POST-PROCESSING                                 │
│     ├── Background removal (rembg or SAM)           │
│     ├── Resize to game resolution (64x64, 128x128)  │
│     ├── Pack into sprite sheet atlas                │
│     └── Generate metadata JSON                      │
│                                                     │
│  5. QUALITY GATE                                    │
│     ├── Visual consistency check across 8 dirs      │
│     ├── Silhouette comparison (shape consistency)   │
│     ├── Color palette validation                    │
│     └── Approve or regenerate with adjusted seed    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## Decision

**A1111 API + ControlNet OpenPose (SDXL) + Custom LoRA + Euler scheduler with seed determinism.**

This pipeline provides:
1. **Pose consistency**: ControlNet OpenPose enforces character orientation across 8 directions
2. **Style consistency**: Custom LoRA locks the art style across all unit types
3. **Reproducibility**: Deterministic seed + Euler scheduler = identical output for identical inputs
4. **Simplicity**: A1111 REST API is straightforward to integrate vs ComfyUI's graph-based approach
5. **Flexibility**: LoRA weight and ControlNet strength provide fine-tuning knobs

---

## Implementation Contract

### SpritePipeline Interface

```typescript
interface ISpritePipeline {
    /** Check A1111 server health and model availability. */
    healthCheck(): Promise<PipelineHealth>;

    /** Generate a single sprite for a unit type and direction. */
    generateSprite(request: SpriteRequest): Promise<SpriteResult>;

    /** Generate a complete 8-direction sprite sheet for a unit type. */
    generateSpriteSheet(request: SpriteSheetRequest): Promise<SpriteSheetResult>;

    /** Get generation progress for an active request. */
    getProgress(): Promise<GenerationProgress>;
}

interface SpriteRequest {
    unitType: string;
    direction: 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW';
    seed: number;
    loraName: string;
    loraWeight: number;          // 0.0-1.0, recommended 0.8
    controlnetWeight: number;    // 0.0-1.0, recommended 1.0
    steps: number;               // recommended 30
    cfgScale: number;            // recommended 7.0
    width: number;               // recommended 1024
    height: number;              // recommended 1024
}

interface SpriteSheetRequest {
    unitType: string;
    baseSeed: number;
    loraName: string;
    loraWeight: number;
    outputSize: { width: number; height: number };  // final sprite size (e.g., 64x64)
}

interface SpriteResult {
    imageData: Buffer;       // PNG bytes
    seed: number;            // actual seed used
    generationTimeMs: number;
    metadata: {
        prompt: string;
        negativPrompt: string;
        sampler: string;
        steps: number;
        cfgScale: number;
    };
}

interface SpriteSheetResult {
    sprites: Record<string, SpriteResult>;  // direction -> sprite
    atlasImage: Buffer;                      // packed sprite sheet PNG
    atlasMetadata: {                         // for game engine consumption
        frameWidth: number;
        frameHeight: number;
        frames: Record<string, { x: number; y: number; w: number; h: number }>;
    };
}

interface PipelineHealth {
    serverReachable: boolean;
    modelLoaded: string;
    lorasAvailable: string[];
    controlnetModelsAvailable: string[];
    gpuName: string;
    gpuVramMb: number;
}

interface GenerationProgress {
    progress: number;       // 0.0-1.0
    etaSeconds: number;
    currentStep: number;
    totalSteps: number;
    currentImage?: Buffer;  // preview of current generation
}
```

### Seed Registry

```typescript
/**
 * Deterministic seed allocation for reproducible sprite generation.
 * Each unit type gets a 1000-seed range. Directions use offsets 0-7.
 * Variants (armor upgrades, etc.) use offsets 100-199.
 */
interface ISeedRegistry {
    /** Get the deterministic seed for a specific unit + direction + variant. */
    getSeed(unitType: string, direction: string, variant?: string): number;

    /** Register a new unit type with a base seed. */
    registerUnitType(unitType: string, baseSeed: number): void;

    /** Export the full seed map for reproducibility audit. */
    exportSeedMap(): Record<string, number>;
}
```

---

## Open Questions Remaining

1. **SDXL ControlNet model maturity**: The `controlnet-openpose-sdxl-1.0` model is less battle-tested than the SD 1.5 variant. Need to validate quality at SDXL resolution for game sprite use cases.

2. **LoRA + ControlNet interaction**: High LoRA weights (>0.8) combined with strong ControlNet guidance may conflict. Need empirical testing to find the optimal balance.

3. **Background removal pipeline**: rembg vs SAM (Segment Anything Model) for clean background removal. rembg is simpler; SAM is more accurate for complex silhouettes.

4. **Animation frames**: Beyond 8-direction static sprites, CivLab may need animated sprites (walk cycle, attack, idle). This requires ControlNet temporal consistency or AnimateDiff integration.

5. **M3 Max performance**: SDXL inference on Apple Silicon via MPS is slower than CUDA. Need to benchmark: at 1024x1024 with 30 steps, how many sprites/hour? Estimate: ~2-4 sprites/minute.

6. **LoRA overfitting detection**: How to detect overfitting during training? Monitor validation loss; generate test images at each checkpoint (every 500 steps) and visually inspect for mode collapse.

7. **Sprite resolution pipeline**: Generate at 1024x1024, downscale to 64x64 or 128x128. Which downscaling algorithm preserves pixel-art crispness? Lanczos for smooth art; nearest-neighbor for pixel art.

---

## Sources

- [A1111 API Wiki](https://github.com/AUTOMATIC1111/stable-diffusion-webui/wiki/API)
- [A1111 API Discussion #3734](https://github.com/AUTOMATIC1111/stable-diffusion-webui/discussions/3734)
- [A1111 txt2img Guide](https://randombits.dev/articles/stable-diffusion/txt2img)
- [A1111 API Guide](https://randombits.dev/articles/stable-diffusion/api)
- [ComfyUI Workflow JSON Spec](https://docs.comfy.org/specs/workflow_json)
- [Hosting ComfyUI via API — 9elements](https://9elements.com/blog/hosting-a-comfyui-workflow-via-api/)
- [Building Production-Ready ComfyUI API — ViewComfy](https://www.viewcomfy.com/blog/building-a-production-ready-comfyui-api)
- [ComfyUI API Deep Wiki](https://deepwiki.com/Comfy-Org/ComfyUI/7-api-and-programmatic-usage)
- [LoRA Training 2025 Ultimate Guide — sanj.dev](https://sanj.dev/post/lora-training-2025-ultimate-guide)
- [Detailed LoRA Training Guide — ViewComfy](https://www.viewcomfy.com/blog/detailed-LoRA-training-guide-for-Stable-Diffusion)
- [SDXL LoRA Training — froehlichundfrei](https://www.froehlichundfrei.de/blog/2024-01-22-stable-diffusion-xl-lora-training/)
- [Perfect LoRA Parameters — HuggingFace](https://discuss.huggingface.co/t/perfect-lora-training-parameters-human-character/147211)
- [Ultimate SDXL LoRA Training — lilys.ai](https://lilys.ai/en/notes/training-lora-20260208/ultimate-sdxl-lora-training)
- [Reproducible Pipelines — HuggingFace Diffusers](https://huggingface.co/docs/diffusers/using-diffusers/reusing_seeds)
- [SDXL Settings Guide — Replicate](https://sdxl.replicate.dev/)
- [Sampler/Scheduler Reference — CivitAI](https://civitai.com/articles/16231/sampler-and-scheduler-reference-for-hi-dream-flux-sdxl-illustrious-and-pony)
- [OpenPose ControlNet Tutorial — NextDiffusion](https://www.nextdiffusion.ai/tutorials/how-to-use-open-pose-controlnet-in-stable-diffusion)
