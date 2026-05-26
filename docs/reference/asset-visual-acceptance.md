# Asset Visual Acceptance Criteria

Use this checklist when reviewing pack assets for production screenshots, showcase captures, or journey proof material.

## Pass Criteria

- The asset resolves to a real 3D mesh, not a primitive fallback, billboard, impostor, sprite card, or placeholder stub.
- The asset has non-zero depth in the source mesh and renders as a volumetric object from at least two distinct camera angles.
- The asset is not a temporary Unity primitive, a copied fallback prefab, or a "manual_placeholder" / "placeholder_pending_download" entry.
- The asset manifest, intake notes, and validation report agree that the asset is ready for prototype or release use.

## Fail Criteria

- Any bundle path or asset manifest explicitly marks the model as placeholder, fallback, or pending download.
- The generated prefab or preview is built from a primitive shape because no mesh was available.
- The content is visually flat enough that it reads as a card, billboard, or silhouette in normal gameplay camera distance.
- The asset requires a "replace later" note in the intake record to achieve screenshot quality.
- The screenshot evidence depends on a static noon-only sun, ineffective shader/lighting toggles, flat unlit colors, or missing terrain/water motion to look complete.

## Required Evidence

- Source model file present in the raw or imported asset directory.
- Validation report showing mesh coverage and technical status are complete.
- Screenshot or render proof from the actual asset, not the placeholder fallback.

## Review Rule

If a map object is intended only as a temporary placeholder, it may remain in the working area, but it must not be used in production screenshots or showcase journeys until it passes this checklist.

## Related References

- [ADR-010: Deterministic Star-Wars-Style Asset Intake Pipeline](../adr/ADR-010-asset-intake-pipeline.md)
- [Pack Cookbook](../guide/pack-cookbook.md)
- `packs/warfare-starwars/assets/policies/intake_rules.yaml`
- [Phenotype Journey Visual Acceptance Gate](../qa/phenotype-journey-visual-acceptance.md)
