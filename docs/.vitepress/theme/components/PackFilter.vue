<template>
  <div class="pack-filter">
    <div class="filter-container">
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search packs by name, author, type..."
          class="search-input"
        />
        <span class="search-icon">🔍</span>
      </div>

      <div class="filter-row">
        <div class="filter-group">
          <label class="filter-label">Type:</label>
          <div class="filter-options">
            <button
              v-for="type in typeOptions"
              :key="type"
              :class="['filter-button', { active: selectedTypes.includes(type) }]"
              @click="toggleType(type)"
            >
              {{ formatType(type) }}
            </button>
          </div>
        </div>

        <div class="filter-group">
          <label class="filter-label">Sort:</label>
          <select v-model="sortBy" class="sort-select">
            <option value="name">Name (A-Z)</option>
            <option value="version">Latest Version</option>
            <option value="author">Author</option>
            <option value="type">Pack Type</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="hasActiveFilters" class="filter-summary">
      <span>{{ filteredCount }} pack<span v-if="filteredCount !== 1">s</span> found</span>
      <button @click="clearFilters" class="clear-button">Clear filters</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

type PackType = 'content' | 'balance' | 'ruleset' | 'scenario' | 'total_conversion' | 'utility'

const props = defineProps<{
  packCount: number
}>()

const emit = defineEmits<{
  filter: [{ search: string; types: PackType[]; sort: string }]
}>()

const searchQuery = ref('')
const selectedTypes = ref<PackType[]>([])
const sortBy = ref('name')

const typeOptions: PackType[] = ['content', 'total_conversion', 'balance', 'scenario', 'utility', 'ruleset']

const hasActiveFilters = computed(() => {
  return searchQuery.value.length > 0 || selectedTypes.value.length > 0 || sortBy.value !== 'name'
})

const filteredCount = computed(() => {
  // This will be updated by parent component
  return props.packCount
})

const formatType = (type: string): string => {
  return type
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

const toggleType = (type: PackType) => {
  const index = selectedTypes.value.indexOf(type)
  if (index > -1) {
    selectedTypes.value.splice(index, 1)
  } else {
    selectedTypes.value.push(type)
  }
  emitFilter()
}

const emitFilter = () => {
  emit('filter', {
    search: searchQuery.value.toLowerCase(),
    types: selectedTypes.value.length > 0 ? selectedTypes.value : [],
    sort: sortBy.value
  })
}

const clearFilters = () => {
  searchQuery.value = ''
  selectedTypes.value = []
  sortBy.value = 'name'
  emitFilter()
}

// Watch for changes
import { watch } from 'vue'
watch(searchQuery, emitFilter)
watch(sortBy, emitFilter)
</script>

<style scoped>
.pack-filter {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 32px;
}

.filter-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.search-box {
  position: relative;
  width: 100%;
}

.search-input {
  width: 100%;
  padding: 12px 16px 12px 40px;
  background: var(--vp-c-bg);
  border: 2px solid var(--vp-c-divider);
  border-radius: 8px;
  font-size: 14px;
  color: var(--vp-c-text-1);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--vp-c-brand);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--vp-c-text-3);
}

.filter-row {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
  align-items: flex-start;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
  min-width: 200px;
}

.filter-label {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--vp-c-text-2);
  letter-spacing: 0.5px;
}

.filter-options {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-button {
  padding: 6px 12px;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--vp-c-text-2);
}

.filter-button:hover {
  border-color: var(--vp-c-brand);
  background: var(--vp-c-brand);
  color: white;
}

.filter-button.active {
  background: var(--vp-c-brand);
  border-color: var(--vp-c-brand);
  color: white;
  font-weight: 500;
}

.sort-select {
  padding: 6px 12px;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  font-size: 13px;
  color: var(--vp-c-text-1);
  cursor: pointer;
  transition: all 0.2s ease;
}

.sort-select:focus {
  outline: none;
  border-color: var(--vp-c-brand);
}

.sort-select:hover {
  border-color: var(--vp-c-brand);
}

.filter-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 16px;
  border-top: 1px solid var(--vp-c-divider);
  font-size: 13px;
  color: var(--vp-c-text-2);
}

.clear-button {
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--vp-c-text-2);
}

.clear-button:hover {
  border-color: var(--vp-c-brand);
  color: var(--vp-c-brand);
}

@media (max-width: 768px) {
  .pack-filter {
    padding: 16px;
  }

  .filter-row {
    flex-direction: column;
    gap: 16px;
  }

  .filter-group {
    min-width: unset;
  }
}
</style>
