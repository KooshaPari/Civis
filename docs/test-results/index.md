---
title: Test Results
description: Live test suite status from latest CI run
---

# Test Results

<script setup>
import { ref, onMounted } from 'vue'

const results = ref(null)
const error = ref(null)

onMounted(async () => {
  try {
    const res = await fetch('/Dino/test-results/latest.json')
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    results.value = await res.json()
  } catch (e) {
    console.warn('Could not load test results', e)
    error.value = e.message
  }
})
</script>

<div v-if="results">

## Summary: {{ results.summary.passed }}/{{ results.summary.total }} passed ({{ results.summary.pass_rate }}%)

<div :style="{ background: results.summary.failed === 0 ? '#22c55e22' : '#ef444422', padding: '1rem', borderRadius: '8px', marginBottom: '1rem', border: results.summary.failed === 0 ? '1px solid #22c55e' : '1px solid #ef4444' }">

**{{ results.summary.failed === 0 ? '✅ All tests passing' : `❌ ${results.summary.failed} tests failing` }}**

Last updated: {{ new Date(results.timestamp).toLocaleString() }}

</div>

### Test Suites

| Suite | Total | Passed | Failed | Skipped |
|-------|-------|--------|--------|---------|
<span v-for="suite in results.suites" :key="suite.file">
| `{{ suite.file.split('/').pop() }}` | {{ suite.total }} | {{ suite.passed }} | {{ suite.failed }} | {{ suite.skipped }} |
</span>

</div>
<div v-else-if="error">

## Error Loading Test Results

Could not load test results: {{ error }}

Test results will appear here after the next CI run on the main branch.

</div>
<div v-else>

## Loading...

Fetching test results...

</div>
