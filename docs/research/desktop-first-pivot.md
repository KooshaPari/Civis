# Desktop-First Pivot

This project is now desktop-first.

## Client priority

1. **Primary: Bevy standalone** - `clients/bevy-ref`
   - Target: desktop-first native client on Bevy 0.18
   - Must lead with DX12 Ultimate, DXR ray tracing, DLSS upscaling, mesh shaders, and DirectStorage where the platform supports them
   - This is the first target for future feature work

2. **Secondary: Godot client** - `clients/godot-ref`
   - Role: UX and editor companion
   - Keep it useful for iteration, inspection, and companion workflows

3. **Tertiary: Unreal showcase** - `clients/unreal-show`
   - Role: visual demo and showcase surface
   - Useful for presentation quality and high-end rendering references

4. **Deprecated: web/dashboard**
   - Keep buildable for CI smoke and basic regression coverage
   - Stop investing in new capability work
   - Bug fixes only

5. **Backup: Fyrox**
   - Keep as the Rust-native fallback option
   - Use only if Bevy cannot cover a required desktop path

## Policy

- All future feature work goes to Bevy first.
- The web/dashboard path is maintenance-only.
- New desktop capability should be designed for native desktop delivery first, with the other clients treated as support surfaces.

## Implications

- Update implementation plans to treat Bevy as the first-class client.
- Preserve web/dashboard CI smoke so the repo still has a cheap regression check.
- Align docs, PRDs, and roadmaps with the desktop-first ordering above.
