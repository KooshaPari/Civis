# Skybox placeholders (Task #985)

## Current pack content check

- No SW skybox bundles or skybox material entries were found in `packs/warfare-starwars` during the scan for
  `skybox`, `environment`, or skybox-like asset references.
- Placeholder skyboxes are therefore supplied by runtime code as procedural solid-color materials for now.

## Placeholder mapping

- `tatooine` → warm tan placeholder
- `naboo` → blue placeholder
- `umbara` → dark placeholder
- `coruscant` → city-orange placeholder

## Follow-up

- Replace the procedural placeholders with real skybox bundles/materials once SW map-specific skybox sources are available.
