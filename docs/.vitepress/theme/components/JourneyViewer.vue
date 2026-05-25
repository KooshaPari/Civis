<template>
  <div class="journey-viewer">
    <div class="journey-header">
      <h3>{{ title || 'Journey' }}</h3>
      <div class="header-right">
        <div class="progress">Step {{ currentStep + 1 }} / {{ journey.keyframe_count }}</div>
        <div class="status" :class="currentStepStatus">
          {{ currentStepStatus === 'passed' ? '✓ Passed' : '✗ Failed' }}
        </div>
      </div>
    </div>

    <div class="journey-main">
      <div class="frame-container">
        <div v-if="currentFrame" class="frame-wrapper">
          <img
            :src="currentFrame.screenshot_path"
            :alt="`Step ${currentStep}`"
            class="frame-image"
          />
          <!-- Annotation overlays -->
          <svg
            v-if="currentAnnotations.length > 0"
            class="annotations"
            :viewBox="viewBox"
            preserveAspectRatio="none"
          >
            <g v-for="(annotation, idx) in currentAnnotations" :key="idx">
              <!-- Bounding box -->
              <rect
                :x="annotation.bbox.x"
                :y="annotation.bbox.y"
                :width="annotation.bbox.width"
                :height="annotation.bbox.height"
                :class="`annotation-box annotation-${annotation.type}`"
              />
              <!-- Label background -->
              <rect
                :x="annotation.bbox.x"
                :y="Math.max(0, annotation.bbox.y - 24)"
                :width="annotation.bbox.width"
                :height="24"
                :class="`annotation-label-bg annotation-${annotation.type}`"
              />
              <!-- Label text -->
              <text
                :x="annotation.bbox.x + 5"
                :y="Math.max(18, annotation.bbox.y - 6)"
                class="annotation-label"
              >
                {{ annotation.label }}
              </text>
            </g>
          </svg>
        </div>
      </div>

      <div class="step-info">
        <h4 v-if="currentFrame">{{ currentFrame.intent }}</h4>
        <div v-if="currentFrame && currentFrame.assertions" class="assertions">
          <div
            v-if="currentFrame.assertions.must_contain && currentFrame.assertions.must_contain.length"
            class="must-contain"
          >
            <strong>Must contain:</strong>
            <ul>
              <li v-for="(item, idx) in currentFrame.assertions.must_contain" :key="`mc-${idx}`">
                {{ item }}
              </li>
            </ul>
          </div>
          <div
            v-if="currentFrame.assertions.must_not_contain && currentFrame.assertions.must_not_contain.length"
            class="must-not-contain"
          >
            <strong>Must not contain:</strong>
            <ul>
              <li v-for="(item, idx) in currentFrame.assertions.must_not_contain" :key="`mnc-${idx}`">
                {{ item }}
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>

    <div class="journey-controls">
      <button
        @click="previousStep"
        :disabled="currentStep === 0"
        class="control-btn"
        title="Previous step (or press Left arrow)"
      >
        ← Previous
      </button>
      <button
        @click="togglePlay"
        class="control-btn play-btn"
        :title="isPlaying ? 'Pause playback' : 'Play through all steps'"
      >
        {{ isPlaying ? '⏸ Pause' : '▶ Play' }}
      </button>
      <button
        @click="nextStep"
        :disabled="currentStep === journey.steps.length - 1"
        class="control-btn"
        title="Next step (or press Right arrow)"
      >
        Next →
      </button>
      <select v-model="playSpeed" class="speed-select" title="Playback speed">
        <option value="slow">Slow (2s/frame)</option>
        <option value="normal">Normal (1s/frame)</option>
        <option value="fast">Fast (500ms/frame)</option>
      </select>
    </div>

    <div class="journey-gallery">
      <div
        v-for="(step, idx) in journey.steps"
        :key="`thumb-${idx}`"
        class="thumbnail"
        :class="{ active: idx === currentStep }"
        @click="currentStep = idx"
        :title="`Jump to frame ${idx}: ${step.intent}`"
      >
        <img :src="step.screenshot_path" :alt="`Frame ${idx}`" />
        <span class="frame-number">{{ idx }}</span>
        <div class="frame-status" :class="getFrameStatus(step)"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

export interface JourneyStep {
  index: number
  slug: string
  intent: string
  screenshot_path: string
  assertions?: {
    must_contain?: string[]
    must_not_contain?: string[]
  }
}

export interface Journey {
  id: string
  intent: string
  keyframe_count: number
  passed: boolean
  steps: JourneyStep[]
}

export interface Annotation {
  bbox: { x: number; y: number; width: number; height: number }
  label: string
  type: 'passed' | 'failed' | 'info'
}

interface Props {
  journey: Journey
  title?: string
  annotations?: Record<number, Annotation[]>
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  annotations: undefined
})

const currentStep = ref(0)
const isPlaying = ref(false)
const playSpeed = ref('normal')
let playbackInterval: ReturnType<typeof setInterval> | null = null

const currentFrame = computed(() => props.journey.steps[currentStep.value])
const currentAnnotations = computed(() => props.annotations?.[currentStep.value] ?? [])
const currentStepStatus = computed(() => {
  // Determine pass/fail from assertions or manifest
  return props.journey.passed ? 'passed' : 'failed'
})

const viewBox = computed(() => {
  // Standard widescreen viewport for game screenshots
  return '0 0 1920 1080'
})

const nextStep = () => {
  if (currentStep.value < props.journey.steps.length - 1) {
    currentStep.value++
  } else if (isPlaying.value) {
    isPlaying.value = false
  }
}

const previousStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

const togglePlay = () => {
  isPlaying.value = !isPlaying.value
}

const getFrameStatus = (step: JourneyStep): string => {
  // Return 'passed' or 'failed' based on assertions
  if (!step.assertions) return 'unknown'

  const hasRequirements = step.assertions.must_contain && step.assertions.must_contain.length > 0
  const hasExclusions = step.assertions.must_not_contain && step.assertions.must_not_contain.length > 0

  // In a real scenario, you'd verify these against actual screenshot content
  // For now, inherit from journey pass/fail status
  return props.journey.passed ? 'passed' : 'failed'
}

// Keyboard navigation
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'ArrowRight') {
    event.preventDefault()
    nextStep()
  } else if (event.key === 'ArrowLeft') {
    event.preventDefault()
    previousStep()
  } else if (event.key === ' ') {
    event.preventDefault()
    togglePlay()
  }
}

// Auto-play logic
watch(isPlaying, (playing) => {
  if (playbackInterval) {
    clearInterval(playbackInterval)
    playbackInterval = null
  }

  if (playing) {
    const speedMs =
      playSpeed.value === 'slow' ? 2000 :
      playSpeed.value === 'fast' ? 500 :
      1000

    playbackInterval = setInterval(() => {
      if (currentStep.value < props.journey.steps.length - 1) {
        nextStep()
      } else {
        isPlaying.value = false
      }
    }, speedMs)
  }
})

// Watch playSpeed changes while playing
watch(playSpeed, () => {
  if (isPlaying.value) {
    isPlaying.value = false
    isPlaying.value = true
  }
})

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (playbackInterval) {
    clearInterval(playbackInterval)
  }
})
</script>

<style scoped>
.journey-viewer {
  max-width: 1200px;
  margin: 2rem auto;
  background: var(--vp-c-bg-soft);
  border-radius: 8px;
  padding: 2rem;
  border: 1px solid var(--vp-c-divider);
  font-family: var(--vp-font-family-base);
}

.journey-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  border-bottom: 2px solid var(--vp-c-divider);
  padding-bottom: 1rem;
  gap: 2rem;
}

.journey-header h3 {
  margin: 0;
  font-size: 1.5rem;
  flex-shrink: 0;
  color: var(--vp-c-text-1);
}

.header-right {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.progress {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  white-space: nowrap;
}

.status {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  font-weight: bold;
  font-size: 0.9rem;
  white-space: nowrap;
  min-width: 100px;
  text-align: center;
}

.status.passed {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid #22c55e;
}

.status.failed {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid #ef4444;
}

.journey-main {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 2rem;
  margin-bottom: 2rem;
}

.frame-container {
  background: #1a1a1a;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid var(--vp-c-divider);
  aspect-ratio: 16 / 9;
}

.frame-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
}

.frame-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
}

.annotations {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.annotation-box {
  stroke-width: 2;
  fill: none;
  opacity: 0.9;
}

.annotation-box.annotation-passed {
  stroke: #22c55e;
}

.annotation-box.annotation-failed {
  stroke: #ef4444;
}

.annotation-box.annotation-info {
  stroke: #3b82f6;
}

.annotation-label-bg {
  opacity: 0.85;
}

.annotation-label-bg.annotation-passed {
  fill: #22c55e;
}

.annotation-label-bg.annotation-failed {
  fill: #ef4444;
}

.annotation-label-bg.annotation-info {
  fill: #3b82f6;
}

.annotation-label {
  font-size: 12px;
  fill: white;
  font-weight: bold;
  font-family: monospace;
  pointer-events: none;
}

.step-info {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 1rem;
  background: var(--vp-c-bg);
  border-radius: 4px;
  border: 1px solid var(--vp-c-divider);
  overflow-y: auto;
  max-height: 450px;
}

.step-info h4 {
  margin: 0;
  font-size: 1.1rem;
  color: var(--vp-c-text-1);
}

.assertions {
  font-size: 0.85rem;
  line-height: 1.6;
  color: var(--vp-c-text-2);
}

.must-contain {
  padding: 0.75rem;
  background: rgba(34, 197, 94, 0.05);
  border-left: 3px solid #22c55e;
  border-radius: 2px;
  margin-bottom: 0.75rem;
}

.must-contain strong {
  color: #22c55e;
}

.must-contain ul {
  margin: 0.5rem 0 0 1.5rem;
  padding: 0;
  list-style: circle;
}

.must-not-contain {
  padding: 0.75rem;
  background: rgba(239, 68, 68, 0.05);
  border-left: 3px solid #ef4444;
  border-radius: 2px;
}

.must-not-contain strong {
  color: #ef4444;
}

.must-not-contain ul {
  margin: 0.5rem 0 0 1.5rem;
  padding: 0;
  list-style: circle;
}

.journey-controls {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 2rem;
  padding: 1rem;
  background: var(--vp-c-bg);
  border-radius: 4px;
  border: 1px solid var(--vp-c-divider);
  flex-wrap: wrap;
}

.control-btn,
.speed-select {
  padding: 0.5rem 1rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 4px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  cursor: pointer;
  font-size: 0.9rem;
  font-family: inherit;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.control-btn:hover:not(:disabled) {
  background: var(--vp-c-brand);
  color: white;
  border-color: var(--vp-c-brand);
  transform: translateY(-1px);
}

.control-btn:active:not(:disabled) {
  transform: translateY(0);
}

.control-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.play-btn {
  font-weight: bold;
  min-width: 120px;
}

.speed-select {
  cursor: pointer;
}

.speed-select:hover {
  background: var(--vp-c-brand);
  color: white;
  border-color: var(--vp-c-brand);
}

.journey-gallery {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding: 1rem;
  border-top: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  border-radius: 4px;
  scroll-behavior: smooth;
}

.journey-gallery::-webkit-scrollbar {
  height: 8px;
}

.journey-gallery::-webkit-scrollbar-track {
  background: var(--vp-c-bg-soft);
  border-radius: 4px;
}

.journey-gallery::-webkit-scrollbar-thumb {
  background: var(--vp-c-divider);
  border-radius: 4px;
}

.journey-gallery::-webkit-scrollbar-thumb:hover {
  background: var(--vp-c-text-3);
}

.thumbnail {
  flex-shrink: 0;
  width: 100px;
  height: 75px;
  border: 2px solid transparent;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  background: #000;
}

.thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.thumbnail.active {
  border-color: var(--vp-c-brand);
  box-shadow: 0 0 8px rgba(var(--vp-c-brand-rgb), 0.3);
}

.thumbnail:hover {
  border-color: var(--vp-c-brand-light);
  transform: scale(1.05);
}

.frame-number {
  position: absolute;
  bottom: 2px;
  right: 4px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 0.75rem;
  font-weight: bold;
  padding: 2px 4px;
  border-radius: 2px;
}

.frame-status {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #888;
}

.frame-status.passed {
  background: #22c55e;
}

.frame-status.failed {
  background: #ef4444;
}

.frame-status.unknown {
  background: #888;
}

/* Responsive design */
@media (max-width: 1024px) {
  .journey-main {
    grid-template-columns: 1fr;
  }

  .step-info {
    max-height: 300px;
  }
}

@media (max-width: 768px) {
  .journey-viewer {
    padding: 1rem;
  }

  .journey-header {
    flex-direction: column;
    gap: 1rem;
    align-items: flex-start;
  }

  .header-right {
    width: 100%;
    justify-content: space-between;
  }

  .journey-controls {
    justify-content: space-between;
  }

  .control-btn {
    flex: 1 1 auto;
    min-width: 70px;
  }

  .speed-select {
    flex: 1 1 auto;
    min-width: 120px;
  }

  .thumbnail {
    width: 80px;
    height: 60px;
  }

  .frame-number {
    font-size: 0.65rem;
  }
}

@media (max-width: 480px) {
  .journey-viewer {
    padding: 0.75rem;
    margin: 1rem auto;
  }

  .journey-header h3 {
    font-size: 1.2rem;
  }

  .progress,
  .status {
    font-size: 0.8rem;
  }

  .journey-controls {
    flex-direction: column;
  }

  .control-btn,
  .speed-select {
    width: 100%;
  }

  .thumbnail {
    width: 70px;
    height: 52px;
  }
}
</style>
