<template>
  <div class="pack-card">
    <div class="card-image">
      <img :src="pack.iconUrl || '/packs/default-icon.png'" :alt="pack.name" class="card-icon" />
      <div class="card-badge">
        <span class="type-badge" :class="`type-${pack.type}`">{{ formatType(pack.type) }}</span>
      </div>
    </div>
    <div class="card-content">
      <h3 class="card-title">{{ pack.name }}</h3>
      <p class="card-meta">
        <span class="version">v{{ pack.version }}</span>
        <span class="author">by {{ pack.author }}</span>
      </p>
      <p class="card-description">{{ truncateDescription(pack.description) }}</p>
      <div class="card-footer">
        <div class="content-stats">
          <span v-if="pack.factionCount > 0" class="stat">{{ pack.factionCount }} faction<span v-if="pack.factionCount !== 1">s</span></span>
          <span v-if="pack.unitCount > 0" class="stat">{{ pack.unitCount }} unit<span v-if="pack.unitCount !== 1">s</span></span>
          <span v-if="pack.buildingCount > 0" class="stat">{{ pack.buildingCount }} building<span v-if="pack.buildingCount !== 1">s</span></span>
        </div>
        <a :href="pack.url" class="view-button">View Details →</a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Pack {
  id: string
  name: string
  version: string
  author: string
  type: 'content' | 'balance' | 'ruleset' | 'scenario' | 'total_conversion' | 'utility'
  description: string
  url: string
  iconUrl?: string
  factionCount: number
  unitCount: number
  buildingCount: number
  weaponCount?: number
  doctrineCount?: number
}

defineProps<{
  pack: Pack
}>()

const formatType = (type: string): string => {
  return type
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

const truncateDescription = (description: string, maxLength: number = 120): string => {
  if (!description) return ''
  return description.length > maxLength ? description.substring(0, maxLength) + '...' : description
}
</script>

<style scoped>
.pack-card {
  display: flex;
  flex-direction: column;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s ease;
  height: 100%;
}

.pack-card:hover {
  border-color: var(--vp-c-brand);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
  transform: translateY(-2px);
}

.card-image {
  position: relative;
  width: 100%;
  aspect-ratio: 1 / 1;
  background: linear-gradient(135deg, var(--vp-c-bg), var(--vp-c-bg-soft));
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.card-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.pack-card:hover .card-icon {
  transform: scale(1.05);
}

.card-badge {
  position: absolute;
  top: 8px;
  right: 8px;
}

.type-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  backdrop-filter: blur(10px);
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.type-content {
  background: rgba(59, 130, 246, 0.8);
  border-color: rgba(59, 130, 246, 1);
}

.type-total_conversion {
  background: rgba(168, 85, 247, 0.8);
  border-color: rgba(168, 85, 247, 1);
}

.type-balance {
  background: rgba(34, 197, 94, 0.8);
  border-color: rgba(34, 197, 94, 1);
}

.type-scenario {
  background: rgba(249, 115, 22, 0.8);
  border-color: rgba(249, 115, 22, 1);
}

.type-utility {
  background: rgba(14, 165, 233, 0.8);
  border-color: rgba(14, 165, 233, 1);
}

.type-ruleset {
  background: rgba(236, 72, 153, 0.8);
  border-color: rgba(236, 72, 153, 1);
}

.card-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  padding: 16px;
}

.card-title {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.card-meta {
  margin: 0 0 12px 0;
  font-size: 12px;
  color: var(--vp-c-text-3);
  display: flex;
  gap: 12px;
  align-items: center;
}

.version {
  padding: 2px 8px;
  background: var(--vp-c-bg);
  border-radius: 4px;
}

.author {
  color: var(--vp-c-text-2);
}

.card-description {
  margin: 0 0 12px 0;
  font-size: 13px;
  color: var(--vp-c-text-2);
  line-height: 1.5;
  flex: 1;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.content-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
}

.stat {
  padding: 4px 8px;
  background: var(--vp-c-bg);
  border-radius: 4px;
  color: var(--vp-c-text-2);
}

.view-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  background: var(--vp-c-brand);
  color: white;
  border-radius: 6px;
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.view-button:hover {
  background: var(--vp-c-brand-dark);
  transform: translateX(2px);
}
</style>
