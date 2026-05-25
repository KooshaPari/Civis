import React from "react";
import { useCurrentFrame, useVideoConfig, spring } from "remotion";
import type { CalloutProps } from "../types";

export const CalloutBox: React.FC<CalloutProps> = ({
  text,
  subText,
  color,
  startFrame,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const relativeFrame = Math.max(0, frame - startFrame);
  const scale = spring({
    frame: relativeFrame,
    fps,
    config: { damping: 12, stiffness: 180 },
  });

  return (
    <div
      style={{
        position: "absolute",
        top: 120,
        right: 40,
        transform: `scale(${scale})`,
        transformOrigin: "top right",
        background: "rgba(0,0,0,0.75)",
        border: `3px solid ${color}`,
        borderRadius: 8,
        padding: "14px 20px",
        minWidth: 280,
        maxWidth: 380,
      }}
    >
      <div
        style={{
          color,
          fontSize: 28,
          fontWeight: 700,
          fontFamily: "Arial, sans-serif",
          letterSpacing: 0.5,
        }}
      >
        {text}
      </div>
      {subText && (
        <div
          style={{
            color: "white",
            fontSize: 16,
            fontFamily: "Arial, sans-serif",
            marginTop: 6,
            opacity: 0.9,
          }}
        >
          {subText}
        </div>
      )}
    </div>
  );
};
