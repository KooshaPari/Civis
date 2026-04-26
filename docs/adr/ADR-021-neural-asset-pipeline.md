# ADR-021: Neural Asset Pipeline

**Date**: 2026-04-04
**Status**: Proposed
**Deciders**: DINOForge Research Team

## Context

Creating high-quality assets for total conversion mods requires significant artistic skill and time. Neural generation technologies have matured to the point where they can accelerate asset creation, reduce costs, and enable solo developers to create professional-quality content packs.

Current asset pipeline limitations:
- 3D modeling requires specialized skills
- Texture creation is time-intensive
- Animation requires motion capture or manual rigging
- Sound design requires audio expertise

## Decision Drivers

- **Developer Accessibility**: Lower barrier to entry for mod creators
- **Content Velocity**: Generate assets faster than manual creation
- **Quality Threshold**: Match or exceed manual creation quality
- **Cost Efficiency**: Reduce/eliminate need for contracted artists
- **Iteration Speed**: Rapid prototyping of visual concepts

## Options Considered

### Option A: Manual-Only (Status Quo)

Continue with fully manual asset creation pipeline.

**Pros**:
- Highest quality control
- Artistic consistency
- No generation artifacts

**Cons**:
- Slow iteration (weeks per asset)
- High skill barrier
- Expensive (contract artists)
- Limited scalability

### Option B: AI-Assisted Hybrid (Selected)

Integrate neural generation tools at specific pipeline stages, with human oversight and refinement.

**Pros**:
- 10x speedup for concept→production
- Maintains quality through human refinement
- Scalable to large content packs
- Accessible to non-artists

**Cons**:
- Requires new tooling integration
- Generation quality varies
- Legal/IP considerations for training data
- Computational costs (GPU time)

**Pipeline Integration**:
```
Stage 1: Concept
  Input: Text description
  Tool: Stable Diffusion / DALL-E
  Output: Concept art (2D)

Stage 2: 3D Generation
  Input: Concept art or text
  Tool: Rodin / Shap-E / Wonder3D
  Output: Base mesh (OBJ/GLB)

Stage 3: Refinement
  Input: Base mesh
  Tool: Blender + human artist
  Output: Game-ready mesh (FBX)

Stage 4: Texturing
  Input: Mesh + concept
  Tool: Stable Diffusion (texture) + Materialize
  Output: PBR textures (albedo, normal, metallic)

Stage 5: Integration
  Input: Mesh + textures
  Tool: Unity Editor
  Output: Prefab + Addressables bundle
```

### Option C: Fully Automated Generation

End-to-end neural pipeline with minimal human intervention.

**Pros**:
- Maximum speed (assets in hours)
- True scalability
- No artistic skill required

**Cons**:
- Quality inconsistent
- Limited artistic direction
- Technical artifacts common
- Not viable for production currently

### Option D: Outsourced Asset Production

Contract external studios for asset creation.

**Pros**:
- Professional quality
- No technical investment
- Established workflows

**Cons**:
- Expensive ($500-5000 per asset)
- Slow turnaround (weeks)
- Communication overhead
- IP ownership complexity

## Decision

**Adopt Option B (AI-Assisted Hybrid)** for neural asset generation, with phased integration starting with concept art and texture generation.

### Phase 1: Concept Art & Textures (Immediate)

| Task | Tool | Output | Quality |
|------|------|--------|---------|
| Concept art | Stable Diffusion XL | 1024x1024 PNG | Production-ready |
| Texture generation | SD + ControlNet | 1K-2K textures | Production-ready |
| Normal maps | Materialize / SD | 1K-2K normal | Production-ready |

### Phase 2: 3D Generation (6-12 months)

| Task | Tool | Output | Quality Target |
|------|------|--------|----------------|
| Base meshes | Rodin / Wonder3D | OBJ/GLB | Retopology needed |
| Retopology | InstantMeshes / human | Game-ready | Production-ready |
| LOD generation | Simplygon / Unity | 100%/60%/30% | Automated |

### Phase 3: Animation & Audio (12-24 months)

| Task | Tool | Output | Quality Target |
|------|------|--------|----------------|
| Animation | DeepMotion / Mixamo | FBX | Good |
| Audio | AudioLDM / Stable Audio | WAV | Good |

### Quality Gates

Every neural-generated asset must pass:

| Gate | Method | Threshold |
|------|--------|-----------|
| **Technical** | Automated checks | Poly count, UVs, textures present |
| **Visual** | Human review | Matches concept art |
| **Integration** | Unity import test | No errors, correct materials |
| **Performance** | Profiler | Budget-compliant |
| **Style** | Art director review | Consistent with pack theme |

### Training Data Policy

| Source | Usage | Legal Status |
|--------|-------|--------------|
| **Public datasets** | Base model training | Varies by license |
| **Generated concepts** | Style training | Clean (owned by user) |
| **Licensed assets** | Fine-tuning | License-dependent |
| **Game screenshots** | Never for training | Prohibited |

## Consequences

### Positive

- **Democratization**: Solo developers can create total conversions
- **Speed**: Concept to prototype in hours vs weeks
- **Iteration**: Rapid visual experimentation
- **Cost**: Fraction of outsourced production costs
- **Scale**: Feasible to create 100+ unit content packs

### Negative

- **Quality Variance**: Generation inconsistency requires oversight
- **Legal Uncertainty**: Evolving IP landscape for AI assets
- **Hardware Costs**: GPU requirements for generation
- **Skills Shift**: Artists need AI tool proficiency
- **Community Perception**: Some players dislike AI-generated content

### Neutral

- **Tool Maintenance**: Rapidly evolving ecosystem requires updates
- **Training Investment**: Team needs to learn new workflows

## Implementation Plan

### Infrastructure
- [ ] GPU compute environment (local or cloud)
- [ ] Stable Diffusion WebUI / ComfyUI setup
- [ ] Rodin / Wonder3D API access
- [ ] Materialize integration

### Pipeline Integration
- [ ] Unity editor extensions for AI import
- [ ] Automated texture pipeline
- [ ] Concept art → 3D workflow
- [ ] Quality gate automation

### Documentation
- [ ] Neural asset creation guides
- [ ] Style consistency templates
- [ ] Legal/IP guidance for creators

## Related ADRs

- ADR-010: Asset Intake Pipeline (foundation)
- ADR-017: Neural TTS (related AI technology)

## References

- Stable Diffusion: https://github.com/Stability-AI/stablediffusion
- Rodin (3D Generation): https://rodin.genie.studio/
- Wonder3D: https://github.com/xxlong0/Wonder3D
- Materialize: https://www.boundingboxsoftware.com/materialize/
- DeepMotion: https://www.deepmotion.com/
