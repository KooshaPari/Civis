/**
 * Canonical annotation schema for the generic doc-embed pipeline.
 *
 * An EmbedSpec is the single input that fully describes one annotated embed.
 * It is intentionally project-agnostic: paths are resolved relative to the
 * annotations.json file's directory, so the whole spec + its assets can be
 * dropped into any repo.
 */

/** A timed callout box (title + optional subtitle) that springs in. */
export interface Callout {
  /** Bold headline text. */
  text: string;
  /** Optional secondary line. */
  subText?: string;
  /** Hex accent color, e.g. "#34d399". Defaults to theme accent. */
  color?: string;
  /** Seconds (relative to embed start) at which it animates in. */
  atSec: number;
  /** Seconds it stays on screen. Defaults to until next callout / end. */
  durationSec?: number;
  /** Corner anchor. Defaults to "top-right". */
  anchor?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}

/** A highlight rectangle drawn over a region (pixel coords in source space). */
export interface Highlight {
  x: number;
  y: number;
  width: number;
  height: number;
  /** Hex stroke color. Defaults to theme accent. */
  color?: string;
  /** Optional label rendered above the rect. */
  label?: string;
  atSec: number;
  durationSec?: number;
  /** "pulse" animates the border; "static" holds. Defaults to "pulse". */
  style?: "pulse" | "static";
}

/** A cursor highlight ring drawn at a point (e.g. click location). */
export interface CursorHighlight {
  x: number;
  y: number;
  atSec: number;
  /** Ripple = expanding ring (click); ring = persistent. Default "ripple". */
  kind?: "ripple" | "ring";
  color?: string;
}

/**
 * One scene in the embed. Either a still image held for `holdSec`, or a
 * video clip. Annotations are layered on top, timed relative to scene start.
 */
export interface Scene {
  /** Relative path (from annotations.json) to a PNG/JPG still OR an mp4. */
  src: string;
  /** Treat src as a still image and hold it for this many seconds. */
  holdSec?: number;
  /** For video src: clip duration in seconds (defaults to whole clip). */
  clipSec?: number;
  /** Optional Ken-Burns style zoom on a still: [fromScale, toScale]. */
  zoom?: [number, number];
  callouts?: Callout[];
  highlights?: Highlight[];
  cursors?: CursorHighlight[];
}

/** The full embed specification — the pipeline's only input contract. */
export interface EmbedSpec {
  /** Stable id; used for default output filenames. */
  id: string;
  /** Human title shown in the persistent caption bar. */
  title: string;
  /** Optional subtitle / project tag in the caption bar. */
  subtitle?: string;
  /** Output dimensions. Defaults to 1280x800. */
  width?: number;
  height?: number;
  /** Frames per second. Defaults to 30. */
  fps?: number;
  /** Theme accent hex. Defaults to "#34d399". */
  accent?: string;
  /** Optional voiceover / music track (relative path). */
  audioSrc?: string;
  scenes: Scene[];
}

export const DEFAULTS = {
  width: 1280,
  height: 800,
  fps: 30,
  accent: "#34d399",
} as const;
