# Water / Fluid / Wave Blocker Note

**Date:** 2026-05-23  
**Scope:** Visual acceptance for water, fluid motion, wave response, and shoreline interaction  
**Status:** **BLOCKED** — no accepted visual evidence yet

## Result

Current evidence does **not** satisfy water acceptance. The reported state is consistent with a flat/static surface and no surfaced water mesh, blobbification, fluid physics, wave motion, shoreline response, or shader/material hook path.

Treat any screenshot or gameplay clip as **not accepted** for water until a real water surface/material/animation path exists and is wired into the scene.

## What was checked

- Repository search did not surface a dedicated runtime water system or shoreline config path.
- The existing wave-related content in this repo is gameplay-domain wave logic, not fluid rendering or physics.
- No water-specific validation seam was found that would make flat/static imagery pass as implementation evidence.

## Evidence from repo state

- `schemas/wave.schema.json` and `schemas/wave-collection.schema.json` describe gameplay wave definitions only.
- The current water symptom report has no corresponding runtime water surface hook, shader assignment, or shoreline configuration in the scoped docs/schema surface.
- Existing acceptance/receipt patterns in `docs/qa/` require concrete visual or runtime proof before promotion.

## Acceptance criteria

- A visible water surface is present in-game.
- The surface has a real material/shader path, not a placeholder or baked static plane.
- Motion is observable: animation, wave activity, or other fluid response is visible in capture.
- Shoreline or contact response is visible where water meets terrain or objects.
- The acceptance note can point to a concrete implementation path, not just a screenshot of a flat surface.

## Actionable next step

Add the real water surface/material/animation path first, then capture a fresh visual proof set against this note. Until then, keep this item marked blocked for QA and product acceptance.
