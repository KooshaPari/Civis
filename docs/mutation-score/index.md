---
title: Mutation Testing
description: Stryker.NET mutation score for DINOForge SDK
---

# Mutation Testing

Stryker.NET runs weekly (Monday 6am UTC) against the DINOForge SDK to verify test quality — not just coverage but whether tests actually *detect bugs*.

<script setup>
import { ref, onMounted } from 'vue'
const data = ref(null)
onMounted(async () => {
  try {
    const r = await fetch('/Dino/mutation-score/latest.json')
    data.value = await r.json()
  } catch {}
})
</script>

<div v-if="data">

## Score: {{ data.mutation_score }}%

| Metric | Count |
|--------|-------|
| Killed (tests caught the bug) | {{ data.killed }} ✅ |
| Survived (tests missed the bug) | {{ data.survived }} ❌ |
| Timeout | {{ data.timeout }} |
| Total mutants | {{ data.total }} |

*Last run: {{ new Date(data.timestamp).toLocaleString() }}*

</div>
<div v-else>

## Score: Loading...

Mutation test results will appear here after the next scheduled run (Monday 6am UTC).

</div>

## What is Mutation Testing?

Mutation testing validates test quality by:

1. **Creating mutants** — Introducing intentional bugs (mutations) into the code
2. **Running tests** — Executing the full test suite against each mutant
3. **Measuring kills** — Counting how many mutations tests caught (killed)
4. **Computing score** — `score = killed / total_mutants × 100`

A high mutation score means tests are effective at detecting real bugs. A low score means tests are passing without actually validating behavior.

## Score Interpretation

- **> 85%**: Excellent test coverage and quality
- **70-85%**: Good, but some edge cases may not be covered
- **< 70%**: Tests may be brittle or incomplete

## SDK Focus

We run Stryker against `src/SDK/` because:

- SDK is the public API surface — mutations here affect all downstream code
- ContentLoader, Registries, and Validators are mission-critical
- Test quality directly impacts mod stability

## Automated Gates

Mutation score is visible here but is not yet a required CI gate. Future: integrate with pull requests to prevent score degradation.

## Running Locally

```bash
cd src/SDK
dotnet stryker
```

Results appear in `StrykerOutput/index.html`.
