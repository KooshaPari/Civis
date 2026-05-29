# Headless / Scriptable Adobe Automation

Scope: batch art generation and export workflows for Photoshop, Illustrator, After Effects, and Premiere on Windows. Sources are Adobe docs plus the public Adobe scripting guides where Adobe’s own docs are incomplete.

## Executive summary

- Photoshop and Illustrator are scriptable, but their strongest automation model is still in-process scripting launched from the app UI or via external COM/automation wrappers.
- After Effects is the clearest true headless option because `aerender.exe` is documented for command-line rendering.
- Premiere Pro supports ExtendScript and some command-line pass-through, but Adobe’s own guidance says panels are the recommended execution path and command-line behavior varies by platform.
- If the goal is deterministic, scalable batch rendering/generation, Adobe is usually best for fidelity and compatibility with existing PSD/AI/AE/Premiere assets. If the goal is throughput, containerization, parallelism, and cloud-native scale, non-Adobe pipelines are usually better.

## Photoshop

### Invocation paths

- UI script launch: `File > Scripts` runs `.js` / `.jsx` files from the Photoshop scripts folder, or arbitrary scripts via `File > Scripts > Browse`.
- External automation on Windows: Adobe documents COM-capable scripting languages such as VBScript for external control.
- UXP: Adobe now points developers to UXP scripting documentation for newer scripting APIs.
- `Photoshop.exe -r`: I could not verify an official Adobe doc for a supported `-r` CLI switch. Treat this as undocumented unless you have a locally validated build or internal Adobe guidance.

### Input / output

- Inputs: PSD/PSB, JPEG, PNG, TIFF, PDF, and other Photoshop-openable raster assets.
- Outputs: raster exports and layered documents, usually via script-driven `saveAs`, `export`, or action-like automation.
- Typical batch patterns: open file, manipulate layers/effects/text, export per preset, close without saving, repeat.

### Limitations

- Most Photoshop automation is not truly headless in the server sense; the app still runs and many workflows assume an interactive desktop session.
- COM automation is Windows-only.
- Scripted UI/dialog behavior can block unattended runs unless dialogs are suppressed and the script is written defensively.
- UXP is the strategic direction, but many legacy ExtendScript workflows still exist in the ecosystem.

### Practical read

Photoshop is good for batch raster compositing, mockups, text replacement, layer automation, and export preset generation. It is weaker as a headless render farm component because it remains GUI-first and license/session management still matters.

Source pointers:

- Adobe Photoshop scripting overview: https://helpx.adobe.com/photoshop/using/scripting.html

## Illustrator

### Invocation paths

- UI script launch: `File > Scripts` and `File > Scripts > Other Script` run `.jsx` files.
- Drag-and-drop `.jsx` is supported, but Adobe warns it is not the recommended/safest method.
- External automation: Illustrator supports Microsoft Visual Basic, AppleScript, JavaScript, and ExtendScript.

### Input / output

- Inputs: AI, SVG, PDF, EPS, placed raster/vector assets, and any source Illustrator can open.
- Outputs:
  - SVG via `Document.exportFile()` with `ExportOptionsSVG`.
  - PNG via `ExportOptionsPNG24`.
  - Other export/save targets through the Illustrator DOM.
- For batch art generation, Illustrator is especially useful when the source of truth is vector and the deliverable is SVG, PDF, or raster exports derived from vector layouts.

### Limitations

- Like Photoshop, Illustrator scripting is usually app-hosted rather than true headless server execution.
- The scripting surface is strong, but reliable automation still depends on dialogs, fonts, linked assets, and color-profile availability on the host.
- Drag-and-drop execution can trigger warnings and is not the preferred production path.

### Practical read

Illustrator is the best Adobe option for scripted logo/icon/layout generation and SVG-first pipelines. It is less attractive if you need fully isolated Linux/container execution or large-scale parallel workers.

Source pointers:

- Adobe Illustrator scripting overview: https://helpx.adobe.com/illustrator/using/automation-scripts.html
- SVG export API: https://ai-scripting.docsforadobe.dev/jsobjref/ExportOptionsSVG/
- PNG export API: https://ai-scripting.docsforadobe.dev/jsobjref/ExportOptionsPNG24/

## After Effects

### Invocation paths

- Headless render CLI: `aerender.exe` is Adobe’s documented command-line renderer.
- Adobe’s docs say to locate `aerender` under the After Effects install tree and run `aerender -help` for usage.
- Scripted scene setup: ExtendScript can author compositions, layers, keyframes, render queues, and render settings before calling render workflows.

### Input / output

- Inputs: `.aep` project files, imported footage/assets, comp templates, image sequences, audio.
- Outputs: image sequences, movie files, and render outputs configured through render settings and output modules.
- Common batch patterns:
  - Generate a comp from script.
  - Queue it.
  - Render with `aerender.exe`.
  - Post-process output with another tool.

### Limitations

- `aerender` renders queued AE projects, but it is not a general media pipeline by itself.
- Project dependencies still need to resolve on the host machine: fonts, plugins, codecs, linked footage, expressions, and effects availability can all break unattended renders.
- Real-time interactive debugging is separate from headless render behavior.

### Practical read

After Effects is the strongest Adobe fit for automated motion graphics, animated lower thirds, title cards, and loading-screen style video assets when you need AE fidelity. It is the only one of the four with a clearly documented headless render path.

Source pointers:

- Adobe After Effects automation overview: https://helpx.adobe.com/after-effects/using/automation.html
- Adobe After Effects automated rendering and network rendering: https://helpx.adobe.com/after-effects/using/automated-rendering-network-rendering.html

## Premiere Pro

### Invocation paths

- ExtendScript: Premiere Pro exposes a broad ExtendScript API for project, sequence, metadata, export, and rendering control.
- Command-line execution: Adobe’s public scripting guide says scripts can be passed on a command line with additional configuration, but this is not recommended and behavior varies by platform.
- Modern extensibility: Premiere has moved to UXP-based extensibility; ExtendScript remains supported for now, but Adobe’s guide says no further ExtendScript API improvements are planned.

### Input / output

- Inputs: Premiere projects, media assets, sequences, metadata, render/export presets.
- Outputs: encoded video exports via Premiere/Media Encoder workflows.
- Scriptable automation can create or modify projects, sequences, tracks, markers, and export jobs.

### Limitations

- Premiere is not a clean headless batch engine.
- Adobe recommends CEP/UXP panel-driven execution rather than command-line scripting.
- Exporting usually depends on Adobe Media Encoder or Premiere’s own rendering pipeline, so the same host/plugin/codec caveats apply.

### Practical read

Premiere is usable for scripted assembly and export of editorial timelines, but it is not the first choice for fully unattended batch rendering. It is better for controlled editorial automation than for render-farm style asset generation.

Source pointers:

- Premiere Pro scripting guide: https://ppro-scripting.docsforadobe.dev/
- How to execute scripts: https://ppro-scripting.docsforadobe.dev/introduction/how-to-execute-scripts/

## SOTA comparison: Adobe vs non-Adobe

### Where Adobe wins

- Highest fidelity when the source assets already live in PSD/AI/AE/Premiere formats.
- Best compatibility with industry-standard layer styles, text layouts, fonts, effect stacks, and motion templates.
- Mature scripting surfaces for existing creative pipelines.
- `aerender` gives a real headless render option for motion graphics, which is valuable for production batch jobs.

### Where Adobe loses

- Harder to scale like a service: GUI apps, licensing, profile/font dependencies, and workstation management reduce operational simplicity.
- Automation surfaces are fragmented across ExtendScript, UXP, COM, CEP, and app-specific CLIs.
- Cross-platform portability is poor for COM-based Windows automation.
- Concurrency is usually one app session per machine, not a cheap container-per-job model.

### What non-Adobe stacks do better

- Deterministic headless execution in containers or VMs.
- Easier horizontal scaling and job scheduling.
- Lower licensing and workstation maintenance burden.
- More predictable text/vector/raster pipelines when built from code-first primitives.

### Where non-Adobe stacks lose

- They rarely match Adobe’s exact rendering and typography behavior on native PSD/AI/AE/Premiere source material.
- Recreating advanced layer effects, blend modes, type layout, and motion-graphics conventions can be expensive.
- Migrating existing creative workflows can be a large rewrite.

### Decision rule

- Choose Adobe if the deliverable must match designer-authored source files closely and the team can tolerate desktop-app automation.
- Choose non-Adobe if the main goal is scale, reliability, and infrastructure efficiency, and the creative pipeline can be expressed in code-first primitives.

## Recommendation for DINOForge

- Use Adobe automation only when the pipeline must consume PSD/AI/AE/Premiere-native assets or when fidelity is non-negotiable.
- Prefer `aerender` for any motion-graphics or animated loading-screen job that already lives in After Effects.
- Prefer Illustrator scripting for SVG/icon/layout generation where source art is vector.
- Prefer Photoshop scripting/COM/UXP for raster batch composites, but avoid treating it like a scalable render service.
- For anything that needs reproducible, high-throughput generation, keep a non-Adobe fallback path in the platform design.

