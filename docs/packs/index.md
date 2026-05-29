---
title: Pack Registry
layout: doc
---

<script setup>
import { ref, computed, onMounted } from 'vue'
import PackCard from '../.vitepress/theme/components/PackCard.vue'
import PackFilter from '../.vitepress/theme/components/PackFilter.vue'

const packs = ref([])
const filteredPacks = ref([])
const filterState = ref({
  search: '',
  types: [],
  sort: 'name'
})

onMounted(async () => {
  try {
    const response = await fetch('/packs/registry.json')
    const data = await response.json()
    packs.value = data.packs || []
    applyFilters()
  } catch (error) {
    console.error('Failed to load pack registry:', error)
  }
})

const applyFilters = () => {
  let results = [...packs.value]

  // Apply search filter
  if (filterState.value.search) {
    const query = filterState.value.search
    results = results.filter(pack => {
      const name = pack.name.toLowerCase()
      const author = (pack.author || '').toLowerCase()
      const description = (pack.description || '').toLowerCase()
      return name.includes(query) || author.includes(query) || description.includes(query)
    })
  }

  // Apply type filter
  if (filterState.value.types.length > 0) {
    results = results.filter(pack => filterState.value.types.includes(pack.type))
  }

  // Apply sorting
  switch (filterState.value.sort) {
    case 'name':
      results.sort((a, b) => a.name.localeCompare(b.name))
      break
    case 'version':
      results.sort((a, b) => {
        const versionA = a.version.split('.').map(Number)
        const versionB = b.version.split('.').map(Number)
        for (let i = 0; i < 3; i++) {
          if ((versionA[i] || 0) !== (versionB[i] || 0)) {
            return (versionB[i] || 0) - (versionA[i] || 0)
          }
        }
        return 0
      })
      break
    case 'author':
      results.sort((a, b) => (a.author || '').localeCompare(b.author || ''))
      break
    case 'type':
      results.sort((a, b) => a.type.localeCompare(b.type))
      break
  }

  filteredPacks.value = results
}

const handleFilter = (newFilter) => {
  filterState.value = newFilter
  applyFilters()
}

const filteredCount = computed(() => filteredPacks.value.length)
</script>

# DINOForge Pack Registry

Explore all available content packs for DINOForge. Packs extend gameplay with new units, buildings, factions, economies, scenarios, and more. From total conversions to balance tweaks, find the perfect mod for your playstyle.

<PackFilter :pack-count="filteredCount" @filter="handleFilter" />

<div class="packs-grid">
  <PackCard
    v-for="pack in filteredPacks"
    :key="pack.id"
    :pack="pack"
  />
</div>

<div v-if="filteredPacks.length === 0" class="no-results">
  <p>No packs found matching your filters.</p>
  <p class="hint">Try adjusting your search or clearing filters to see all available packs.</p>
</div>

<div class="registry-info">
  <h2>Machine-Readable Registry</h2>
  <p>A JSON registry is available at <a href="/packs/registry.json">/packs/registry.json</a> for programmatic access (e.g., package managers, CLI tools, launchers).</p>
</div>

<div class="getting-started">
  <h2>Getting Started</h2>
  <div class="getting-started-cards">
    <div class="info-card">
      <h3>📖 Install Your First Pack</h3>
      <p>Learn how to install and manage packs in the <a href="/guides/your-first-mod">Your First Mod</a> guide.</p>
    </div>
    <div class="info-card">
      <h3>🎨 Create Your Own Pack</h3>
      <p>Ready to mod? Start with the <a href="/guides/your-first-mod">Pack Author Guide</a> to build your own content.</p>
    </div>
    <div class="info-card">
      <h3>🤝 Share Your Pack</h3>
      <p>Built something awesome? Open a <a href="https://github.com/KooshaPari/Dino">GitHub issue or PR</a> to submit your pack to the registry.</p>
    </div>
  </div>
</div>

<style scoped>
.packs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  margin: 32px 0;
}

.no-results {
  text-align: center;
  padding: 64px 24px;
  color: var(--vp-c-text-2);
}

.no-results p {
  margin: 8px 0;
  font-size: 15px;
}

.no-results .hint {
  color: var(--vp-c-text-3);
  font-size: 13px;
}

.registry-info {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 24px;
  margin: 48px 0 32px 0;
}

.registry-info h2 {
  margin: 0 0 16px 0;
  font-size: 18px;
}

.registry-info p {
  margin: 0;
  color: var(--vp-c-text-2);
}

.registry-info a {
  color: var(--vp-c-brand);
  text-decoration: none;
  font-weight: 500;
}

.registry-info a:hover {
  text-decoration: underline;
}

.getting-started {
  margin: 48px 0 0 0;
}

.getting-started h2 {
  margin: 0 0 24px 0;
  font-size: 20px;
}

.getting-started-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
}

.info-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s ease;
}

.info-card:hover {
  border-color: var(--vp-c-brand);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.info-card h3 {
  margin: 0 0 12px 0;
  font-size: 15px;
  color: var(--vp-c-text-1);
}

.info-card p {
  margin: 0;
  font-size: 13px;
  color: var(--vp-c-text-2);
  line-height: 1.6;
}

.info-card a {
  color: var(--vp-c-brand);
  text-decoration: none;
  font-weight: 500;
}

.info-card a:hover {
  text-decoration: underline;
}

@media (max-width: 768px) {
  .packs-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
  }

  .registry-info {
    padding: 16px;
  }

  .getting-started-cards {
    grid-template-columns: 1fr;
  }
}
</style>
