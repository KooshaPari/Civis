/**
 * Journey Viewer Type Definitions
 *
 * Complete TypeScript interfaces for the Journey Viewer component
 * and related data structures.
 */

/**
 * Bounding box for annotation overlays
 */
export interface BBox {
  /** Left edge coordinate in pixels (0-1920) */
  x: number
  /** Top edge coordinate in pixels (0-1080) */
  y: number
  /** Width in pixels */
  width: number
  /** Height in pixels */
  height: number
}

/**
 * Annotation overlay on a frame
 */
export interface Annotation {
  /** Bounding box coordinates and dimensions */
  bbox: BBox
  /** Display label for the annotation */
  label: string
  /** Type determines color: green (passed), red (failed), blue (info) */
  type: 'passed' | 'failed' | 'info'
}

/**
 * Assertions that must be true/false for a frame
 */
export interface Assertions {
  /** Elements that must be visible in the frame */
  must_contain?: string[]
  /** Elements that must NOT be visible in the frame */
  must_not_contain?: string[]
}

/**
 * Single step/frame in a journey
 */
export interface JourneyStep {
  /** 0-based frame index */
  index: number
  /** URL-friendly identifier for this step */
  slug: string
  /** What happens in this step (user-readable) */
  intent: string
  /** Path to screenshot (relative to docs root or absolute) */
  screenshot_path: string
  /** Assertions for this frame */
  assertions?: Assertions
}

/**
 * Complete journey manifest
 */
export interface Journey {
  /** Unique journey identifier */
  id: string
  /** High-level goal or description */
  intent: string
  /** Number of frames in this journey */
  keyframe_count: number
  /** Whether the entire journey passed */
  passed: boolean
  /** Array of frames/steps */
  steps: JourneyStep[]
}

/**
 * Annotations keyed by frame index
 */
export type AnnotationMap = Record<number, Annotation[]>

/**
 * Props for JourneyViewer component
 */
export interface JourneyViewerProps {
  /** The journey manifest (required) */
  journey: Journey
  /** Optional title displayed in header */
  title?: string
  /** Optional annotations for frames */
  annotations?: AnnotationMap
}

/**
 * Playback speed options
 */
export type PlaybackSpeed = 'slow' | 'normal' | 'fast'

/**
 * Frame status derived from journey status
 */
export type FrameStatus = 'passed' | 'failed' | 'unknown'

/**
 * Journey generator result from test automation
 */
export interface GeneratedJourney extends Journey {
  /** ISO timestamp when journey was generated */
  generated_at: string
  /** Which tool/framework generated this journey */
  generated_by: string
  /** Optional metadata about the test run */
  metadata?: Record<string, unknown>
}

/**
 * Journey collection for bulk operations
 */
export interface JourneyCollection {
  /** Collection identifier */
  id: string
  /** Display name */
  name: string
  /** Collection description */
  description: string
  /** Journeys in this collection */
  journeys: Journey[]
  /** Metadata about the collection */
  metadata?: {
    author?: string
    created?: string
    updated?: string
    tags?: string[]
    version?: string
  }
}

/**
 * Journey comparison result
 */
export interface JourneyComparison {
  /** First journey being compared */
  journeyA: Journey
  /** Second journey being compared */
  journeyB: Journey
  /** Frames that differ */
  diffFrames: number[]
  /** Overall similarity score (0-100) */
  similarity: number
}

/**
 * Export options for journey generation
 */
export interface ExportOptions {
  format: 'json' | 'yaml' | 'html' | 'markdown'
  includeAnnotations?: boolean
  includeAssets?: boolean
  minifyImages?: boolean
}

/**
 * Journey validation result
 */
export interface ValidationResult {
  valid: boolean
  errors: ValidationError[]
  warnings: ValidationWarning[]
}

/**
 * Validation error
 */
export interface ValidationError {
  field: string
  message: string
  step?: number
}

/**
 * Validation warning
 */
export interface ValidationWarning {
  field: string
  message: string
  step?: number
}

/**
 * Utility functions for journey operations
 */
export namespace JourneyUtils {
  /**
   * Validate a journey manifest
   */
  export function validateJourney(journey: unknown): ValidationResult

  /**
   * Create a journey from test results
   */
  export function createJourneyFromTests(
    testResults: any[],
    options?: Partial<Journey>
  ): Journey

  /**
   * Compare two journeys
   */
  export function compareJourneys(a: Journey, b: Journey): JourneyComparison

  /**
   * Export journey to different format
   */
  export function exportJourney(journey: Journey, options: ExportOptions): string

  /**
   * Generate annotations from OCR results
   */
  export function generateAnnotationsFromOCR(
    imageUrl: string,
    searchTerms: string[]
  ): Promise<AnnotationMap>
}
