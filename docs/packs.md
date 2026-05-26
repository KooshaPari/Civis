---
title: Pack Registry
description: Browse official and community DINOForge mod packs
---

# Pack Registry

Browse available mod packs. Install any pack via the [DINOForge Installer](/guide/getting-started).

<script setup>
import { data as registry } from './.vitepress/registry.data.ts'
</script>

<div v-if="registry && registry.packs">

## Featured Packs

<div v-for="pack in registry.packs.filter(p => p.featured)" :key="pack.id" style="border:1px solid var(--vp-c-divider); border-radius:8px; padding:1rem 1.25rem; margin:1rem 0;">

### {{ pack.name }}

**Author:** {{ pack.author }} &nbsp;·&nbsp; **Version:** `{{ pack.version }}` &nbsp;·&nbsp; **Type:** `{{ pack.type }}`

{{ pack.description }}

<div v-if="pack.requires_spawner" style="background:var(--vp-c-warning-soft); border-left:4px solid var(--vp-c-warning); padding:0.75rem; margin:0.5rem 0; border-radius:4px;">
<strong style="color:var(--vp-c-warning);">⚠️ Requires DINOForge M9+</strong><br/>
This pack requires the M9 unit spawner system for full functionality. Stat overrides apply immediately; custom units require the spawner system.
</div>

<div style="display:flex; gap:0.4rem; flex-wrap:wrap; margin:0.5rem 0;">
<span v-for="tag in pack.tags" :key="tag" style="background:var(--vp-c-brand-soft); color:var(--vp-c-brand-1); border-radius:4px; padding:0.1rem 0.5rem; font-size:0.8em;">{{ tag }}</span>
</div>

**Framework:** `{{ pack.framework_version }}`
<span v-if="pack.conflicts_with && pack.conflicts_with.length"> &nbsp;·&nbsp; **Conflicts with:** {{ pack.conflicts_with.join(', ') }}</span>

<a :href="pack.download_url" style="display:inline-block; margin-top:0.5rem; padding:0.4rem 1rem; background:var(--vp-c-brand-1); color:#fff; border-radius:6px; text-decoration:none; font-size:0.9em;">Download</a>
&nbsp;
<a :href="pack.repo" style="display:inline-block; margin-top:0.5rem; padding:0.4rem 1rem; border:1px solid var(--vp-c-divider); border-radius:6px; text-decoration:none; font-size:0.9em;">Source</a>

</div>

## All Packs

<div v-for="pack in registry.packs.filter(p => !p.featured)" :key="pack.id" style="border:1px solid var(--vp-c-divider); border-radius:8px; padding:1rem 1.25rem; margin:1rem 0;">

### {{ pack.name }}

**Author:** {{ pack.author }} &nbsp;·&nbsp; **Version:** `{{ pack.version }}` &nbsp;·&nbsp; **Type:** `{{ pack.type }}`

{{ pack.description }}

<div v-if="pack.requires_spawner" style="background:var(--vp-c-warning-soft); border-left:4px solid var(--vp-c-warning); padding:0.75rem; margin:0.5rem 0; border-radius:4px;">
<strong style="color:var(--vp-c-warning);">⚠️ Requires DINOForge M9+</strong><br/>
This pack requires the M9 unit spawner system for full functionality. Stat overrides apply immediately; custom units require the spawner system.
</div>

<div style="display:flex; gap:0.4rem; flex-wrap:wrap; margin:0.5rem 0;">
<span v-for="tag in pack.tags" :key="tag" style="background:var(--vp-c-brand-soft); color:var(--vp-c-brand-1); border-radius:4px; padding:0.1rem 0.5rem; font-size:0.8em;">{{ tag }}</span>
</div>

**Framework:** `{{ pack.framework_version }}`

<a :href="pack.download_url" style="display:inline-block; margin-top:0.5rem; padding:0.4rem 1rem; background:var(--vp-c-brand-1); color:#fff; border-radius:6px; text-decoration:none; font-size:0.9em;">Download</a>
&nbsp;
<a :href="pack.repo" style="display:inline-block; margin-top:0.5rem; padding:0.4rem 1rem; border:1px solid var(--vp-c-divider); border-radius:6px; text-decoration:none; font-size:0.9em;">Source</a>

</div>

</div>
<div v-else>

Registry data unavailable — check the [raw registry.json](/registry.json).

</div>

---

## Submit Your Pack

To submit a community pack to the registry, open a pull request editing
[`docs/public/registry.json`](https://github.com/KooshaPari/Dino/edit/main/docs/public/registry.json)
and add your entry following the [registry schema](/registry-schema.json).
