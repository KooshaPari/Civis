import React from "react";
import { useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import type { Highlight as HighlightProps, CursorHighlight as CursorProps } from "../schema";

/**
 * Highlight + cursor coords are authored in SOURCE pixel space. Scenes scale
 * the source to fill the canvas, so we receive an explicit scale factor to map
 * source coords -> canvas coords.
 */
interface ScaleCtx {
  scaleX: number;
  scaleY: number;
  accent: string;
}

export const HighlightRect: React.FC<HighlightProps & ScaleCtx> = ({
  x, y, width, height, color, label, atSec, durationSec, style = "pulse",
  scaleX, scaleY, accent,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const rel = frame - Math.round(atSec * fps);
  if (rel < 0) return null;
  if (durationSec != null && rel > Math.round(durationSec * fps)) return null;

  const stroke = color ?? accent;
  const pulse = style === "pulse"
    ? interpolate(Math.sin((rel / fps) * Math.PI * 2), [-1, 1], [0.35, 1])
    : 1;

  return (
    <div
      style={{
        position: "absolute",
        left: x * scaleX,
        top: y * scaleY,
        width: width * scaleX,
        height: height * scaleY,
        border: `4px solid ${stroke}`,
        borderRadius: 6,
        boxShadow: `0 0 ${12 * pulse}px ${stroke}, inset 0 0 ${8 * pulse}px ${stroke}`,
        opacity: pulse,
      }}
    >
      {label && (
        <div
          style={{
            position: "absolute",
            top: -34,
            left: 0,
            background: stroke,
            color: "#0b0f14",
            fontSize: 16,
            fontWeight: 700,
            fontFamily: "Arial, sans-serif",
            padding: "3px 10px",
            borderRadius: 5,
            whiteSpace: "nowrap",
          }}
        >
          {label}
        </div>
      )}
    </div>
  );
};

export const Cursor: React.FC<CursorProps & ScaleCtx> = ({
  x, y, atSec, kind = "ripple", color, scaleX, scaleY, accent,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const rel = frame - Math.round(atSec * fps);
  if (rel < 0) return null;

  const stroke = color ?? accent;
  const cx = x * scaleX;
  const cy = y * scaleY;

  if (kind === "ring") {
    return (
      <div style={ringStyle(cx, cy, 28, stroke, 1)} />
    );
  }

  // ripple: one-second expanding fading ring
  const t = (rel % fps) / fps;
  const size = interpolate(t, [0, 1], [16, 90]);
  const opacity = interpolate(t, [0, 1], [0.9, 0]);
  return (
    <>
      <div style={ringStyle(cx, cy, size, stroke, opacity)} />
      <div style={ringStyle(cx, cy, 14, stroke, 0.95)} />
    </>
  );
};

function ringStyle(cx: number, cy: number, size: number, color: string, opacity: number): React.CSSProperties {
  return {
    position: "absolute",
    left: cx - size / 2,
    top: cy - size / 2,
    width: size,
    height: size,
    borderRadius: "50%",
    border: `3px solid ${color}`,
    opacity,
    pointerEvents: "none",
  };
}
