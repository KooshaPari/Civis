/**
 * Journey Generator Utility
 *
 * Helpers for generating Journey manifests from automated tests
 * Includes validation, transformation, and export functions
 */

import type {
  Journey,
  JourneyStep,
  Annotation,
  AnnotationMap,
  ValidationResult,
  ValidationError
} from '../.vitepress/theme/types'

/**
 * Generate a journey manifest from screenshot and test data
 *
 * @example
 * ```ts
 * const journey = createJourney({
 *   id: 'us-f1-1-game-launch',
 *   intent: 'User launches game with mod loaded',
 *   steps: [
 *     {
 *       intent: 'Launch Steam',
 *       screenshot: 'screenshots/01.png',
 *       assertions: { must_contain: ['Play button'] }
 *     }
 *   ]
 * })
 * ```
 */
export function createJourney(input: {
  id: string
  intent: string
  steps: Array<{
    slug?: string
    intent: string
    screenshot: string
    assertions?: {
      must_contain?: string[]
      must_not_contain?: string[]
    }
  }>
  passed?: boolean
}): Journey {
  const steps: JourneyStep[] = input.steps.map((step, index) => ({
    index,
    slug: step.slug || `step-${index}`,
    intent: step.intent,
    screenshot_path: step.screenshot,
    assertions: step.assertions
  }))

  return {
    id: input.id,
    intent: input.intent,
    keyframe_count: steps.length,
    passed: input.passed ?? true,
    steps
  }
}

/**
 * Validate a journey manifest
 */
export function validateJourney(journey: unknown): ValidationResult {
  const errors: ValidationError[] = []
  const warnings: ValidationWarning[] = []

  if (!journey || typeof journey !== 'object') {
    return {
      valid: false,
      errors: [{ field: 'root', message: 'Journey must be an object' }],
      warnings: []
    }
  }

  const j = journey as Record<string, unknown>

  // Required fields
  if (!j.id || typeof j.id !== 'string') {
    errors.push({ field: 'id', message: 'id must be a non-empty string' })
  }
  if (!j.intent || typeof j.intent !== 'string') {
    errors.push({ field: 'intent', message: 'intent must be a non-empty string' })
  }
  if (typeof j.keyframe_count !== 'number' || j.keyframe_count < 1) {
    errors.push({ field: 'keyframe_count', message: 'keyframe_count must be a positive number' })
  }
  if (typeof j.passed !== 'boolean') {
    errors.push({ field: 'passed', message: 'passed must be a boolean' })
  }
  if (!Array.isArray(j.steps)) {
    errors.push({ field: 'steps', message: 'steps must be an array' })
  }

  // Validate steps
  if (Array.isArray(j.steps)) {
    if (j.steps.length !== j.keyframe_count) {
      warnings.push({
        field: 'steps',
        message: `steps length (${j.steps.length}) doesn't match keyframe_count (${j.keyframe_count})`
      })
    }

    j.steps.forEach((step, idx) => {
      if (typeof step !== 'object' || !step) {
        errors.push({ field: 'steps', message: `step ${idx} is not an object`, step: idx })
        return
      }

      const s = step as Record<string, unknown>

      if (typeof s.index !== 'number' || s.index !== idx) {
        errors.push({
          field: 'steps.index',
          message: `step ${idx} index must be ${idx}`,
          step: idx
        })
      }
      if (!s.slug || typeof s.slug !== 'string') {
        errors.push({
          field: 'steps.slug',
          message: `step ${idx} must have a non-empty string slug`,
          step: idx
        })
      }
      if (!s.intent || typeof s.intent !== 'string') {
        errors.push({
          field: 'steps.intent',
          message: `step ${idx} must have a non-empty string intent`,
          step: idx
        })
      }
      if (!s.screenshot_path || typeof s.screenshot_path !== 'string') {
        errors.push({
          field: 'steps.screenshot_path',
          message: `step ${idx} must have a non-empty string screenshot_path`,
          step: idx
        })
      }
    })
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings
  }
}

/**
 * Transform test automation output to journey manifest
 *
 * @example
 * ```ts
 * const testResults = await runE2ETests({
 *   scenario: 'game-launch',
 *   captureScreenshots: true
 * })
 *
 * const journey = transformTestToJourney(testResults, {
 *   baseDir: '/screenshots'
 * })
 * ```
 */
export function transformTestToJourney(
  testOutput: any,
  options: {
    baseDir?: string
    journeyId?: string
    includeMetadata?: boolean
  } = {}
): Journey {
  const baseDir = options.baseDir || ''
  const steps: JourneyStep[] = (testOutput.steps || []).map((step: any, index: number) => ({
    index,
    slug: step.name || `step-${index}`,
    intent: step.description || step.name || '',
    screenshot_path: `${baseDir}/${step.screenshot || `${index}.png`}`.replace(/^\//, ''),
    assertions: {
      must_contain: step.expectedText || step.assertions?.mustContain || [],
      must_not_contain: step.forbiddenText || step.assertions?.mustNotContain || []
    }
  }))

  return {
    id: options.journeyId || testOutput.testId || testOutput.name || 'generated-journey',
    intent: testOutput.description || testOutput.intent || 'Generated from test automation',
    keyframe_count: steps.length,
    passed: testOutput.passed !== false && !testOutput.failed,
    steps
  }
}

/**
 * Create annotations from text positions (e.g., from OCR)
 *
 * @example
 * ```ts
 * const annotations = createAnnotationsFromOCR([
 *   { text: 'Play Button', x: 1650, y: 50, width: 200, height: 100, type: 'passed' }
 * ])
 * ```
 */
export function createAnnotationsFromOCR(
  detections: Array<{
    text: string
    x: number
    y: number
    width: number
    height: number
    type?: 'passed' | 'failed' | 'info'
    confidence?: number
  }>
): AnnotationMap {
  const annotations: AnnotationMap = {
    0: detections
      .filter(d => d.confidence === undefined || d.confidence > 0.7)
      .map(d => ({
        bbox: { x: d.x, y: d.y, width: d.width, height: d.height },
        label: d.text,
        type: d.type || 'info'
      }))
  }
  return annotations
}

/**
 * Export journey to JSON
 */
export function exportJourneyJSON(journey: Journey): string {
  return JSON.stringify(journey, null, 2)
}

/**
 * Export journey to YAML
 */
export function exportJourneyYAML(journey: Journey): string {
  const lines: string[] = [
    `id: ${journey.id}`,
    `intent: ${journey.intent}`,
    `keyframe_count: ${journey.keyframe_count}`,
    `passed: ${journey.passed}`,
    `steps:`
  ]

  journey.steps.forEach(step => {
    lines.push(`  - index: ${step.index}`)
    lines.push(`    slug: ${step.slug}`)
    lines.push(`    intent: ${step.intent}`)
    lines.push(`    screenshot_path: ${step.screenshot_path}`)

    if (step.assertions) {
      lines.push(`    assertions:`)
      if (step.assertions.must_contain?.length) {
        lines.push(`      must_contain:`)
        step.assertions.must_contain.forEach(item => {
          lines.push(`        - ${item}`)
        })
      }
      if (step.assertions.must_not_contain?.length) {
        lines.push(`      must_not_contain:`)
        step.assertions.must_not_contain.forEach(item => {
          lines.push(`        - ${item}`)
        })
      }
    }
  })

  return lines.join('\n')
}

/**
 * Generate Markdown documentation from journey
 */
export function generateJourneyMarkdown(
  journey: Journey,
  options?: { includeImages?: boolean; imageBase?: string }
): string {
  const imageBase = options?.imageBase || '.'
  const lines: string[] = [
    `# ${journey.intent}`,
    '',
    `**Journey ID:** \`${journey.id}\`  `,
    `**Status:** ${journey.passed ? '✓ Passed' : '✗ Failed'}  `,
    `**Total Frames:** ${journey.keyframe_count}`,
    ''
  ]

  journey.steps.forEach((step, idx) => {
    lines.push(`## Step ${idx + 1}: ${step.intent}`)
    lines.push('')

    if (options?.includeImages) {
      lines.push(`![Step ${idx}](${imageBase}/${step.screenshot_path})`)
      lines.push('')
    }

    if (step.assertions) {
      if (step.assertions.must_contain?.length) {
        lines.push('**Must contain:**')
        step.assertions.must_contain.forEach(item => {
          lines.push(`- ${item}`)
        })
        lines.push('')
      }

      if (step.assertions.must_not_contain?.length) {
        lines.push('**Must not contain:**')
        step.assertions.must_not_contain.forEach(item => {
          lines.push(`- ${item}`)
        })
        lines.push('')
      }
    }
  })

  return lines.join('\n')
}

/**
 * Compare two journeys for differences
 */
export function compareJourneys(journeyA: Journey, journeyB: Journey) {
  const diffSteps: number[] = []
  const minSteps = Math.min(journeyA.steps.length, journeyB.steps.length)

  for (let i = 0; i < minSteps; i++) {
    const stepA = journeyA.steps[i]
    const stepB = journeyB.steps[i]

    if (
      stepA.intent !== stepB.intent ||
      stepA.screenshot_path !== stepB.screenshot_path ||
      JSON.stringify(stepA.assertions) !== JSON.stringify(stepB.assertions)
    ) {
      diffSteps.push(i)
    }
  }

  if (journeyA.steps.length !== journeyB.steps.length) {
    for (let i = minSteps; i < Math.max(journeyA.steps.length, journeyB.steps.length); i++) {
      diffSteps.push(i)
    }
  }

  const similarity = Math.round(((minSteps - diffSteps.length) / minSteps) * 100)

  return {
    journeyA,
    journeyB,
    diffSteps,
    similarity
  }
}

/**
 * Merge multiple journeys into a collection
 */
export function createJourneyCollection(
  journeys: Journey[],
  metadata: {
    id: string
    name: string
    description: string
    author?: string
    tags?: string[]
  }
) {
  return {
    id: metadata.id,
    name: metadata.name,
    description: metadata.description,
    journeys,
    metadata: {
      author: metadata.author || 'Unknown',
      created: new Date().toISOString(),
      tags: metadata.tags || [],
      version: '1.0.0'
    }
  }
}

interface ValidationWarning {
  field: string
  message: string
  step?: number
}

export type { ValidationWarning }
