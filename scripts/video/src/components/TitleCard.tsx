import React from "react";
import { AbsoluteFill, Audio } from "remotion";

interface TitleCardProps {
  title: string;
  subtitle?: string;
  audioSrc?: string;
}

export const TitleCard: React.FC<TitleCardProps> = ({ title, subtitle, audioSrc }) => (
  <AbsoluteFill
    style={{
      background: "#0a0a0f",
      justifyContent: "center",
      alignItems: "center",
      flexDirection: "column",
      gap: 16,
    }}
  >
    {audioSrc && <Audio src={audioSrc} />}
    <div
      style={{
        color: "white",
        fontSize: 56,
        fontWeight: 800,
        fontFamily: "Arial, sans-serif",
        textAlign: "center",
        letterSpacing: 2,
      }}
    >
      {title}
    </div>
    {subtitle && (
      <div
        style={{
          color: "#94a3b8",
          fontSize: 28,
          fontFamily: "Arial, sans-serif",
          textAlign: "center",
        }}
      >
        {subtitle}
      </div>
    )}
  </AbsoluteFill>
);
