import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import type { Callout as CalloutProps } from "../schema";

const ANCHORS: Record<string, React.CSSProperties> = {
  "top-left": { top: 40, left: 40, transformOrigin: "top left" },
  "top-right": { top: 40, right: 40, transformOrigin: "top right" },
  "bottom-left": { bottom: 90, left: 40, transformOrigin: "bottom left" },
  "bottom-right": { bottom: 90, right: 40, transformOrigin: "bottom right" },
};

export const Callout: React.FC<CalloutProps & { accent: string }> = ({
  text,
  subText,
  color,
  atSec,
  durationSec,
  anchor = "top-right",
  accent,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const startFrame = Math.round(atSec * fps);
  const rel = frame - startFrame;
  if (rel < 0) return null;

  const enter = spring({ frame: rel, fps, config: { damping: 14, stiffness: 180 } });
  const accentColor = color ?? accent;

  let opacity = 1;
  if (durationSec != null) {
    const endFrame = Math.round(durationSec * fps);
    opacity = interpolate(rel, [endFrame - 8, endFrame], [1, 0], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    });
    if (rel > endFrame) return null;
  }

  return (
    <div
      style={{
        position: "absolute",
        ...ANCHORS[anchor],
        transform: `scale(${enter})`,
        opacity,
        background: "rgba(0,0,0,0.78)",
        border: `3px solid ${accentColor}`,
        borderRadius: 10,
        padding: "14px 20px",
        minWidth: 240,
        maxWidth: 420,
        boxShadow: "0 8px 28px rgba(0,0,0,0.45)",
      }}
    >
      <div style={{ color: accentColor, fontSize: 28, fontWeight: 700, fontFamily: "Arial, sans-serif" }}>
        {text}
      </div>
      {subText && (
        <div style={{ color: "white", fontSize: 16, fontFamily: "Arial, sans-serif", marginTop: 6, opacity: 0.9 }}>
          {subText}
        </div>
      )}
    </div>
  );
};
