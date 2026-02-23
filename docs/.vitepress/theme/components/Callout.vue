<template>
  <div :class="['callout', type]">
    <p class="title">{{ title }}</p>
    <div class="body"><slot /></div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ type?: 'tip' | 'warning' | 'danger' | 'note'; title?: string }>()
</script>
