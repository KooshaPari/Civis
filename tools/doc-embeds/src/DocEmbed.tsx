import React from "react";
import { AbsoluteFill, Audio, Sequence, Series, staticFile, useVideoConfig } from "remotion";
import type { EmbedSpec, Scene } from "./schema";
import { DEFAULTS } from "./schema";
import { SceneView } from "./components/SceneView";

/** Frames a single scene occupies. */
export function sceneFrames(scene: Scene, fps: number): number {
  if (scene.holdSec != null) return Math.round(scene.holdSec * fps);
  if (scene.clipSec != null) return Math.round(scene.clipSec * fps);
  return Math.round(4 * fps); // sensible default for an unspecified clip
}

export function totalFrames(spec: EmbedSpec): number {
  const fps = spec.fps ?? DEFAULTS.fps;
  return spec.scenes.reduce((sum, s) => sum + sceneFrames(s, fps), 0);
}

const CaptionBar: React.FC<{ title: string; subtitle?: string; accent: string }> = ({
  title, subtitle, accent,
}) => (
  <div
    style={{
      position: "absolute",
      bottom: 0,
      left: 0,
      right: 0,
      height: 56,
      background: "linear-gradient(0deg, rgba(0,0,0,0.85), rgba(0,0,0,0.0))",
      display: "flex",
      alignItems: "center",
      padding: "0 28px",
      gap: 14,
      fontFamily: "Arial, sans-serif",
    }}
  >
    <div style={{ width: 10, height: 10, borderRadius: "50%", background: accent }} />
    <span style={{ color: "white", fontSize: 22, fontWeight: 700 }}>{title}</span>
    {subtitle && <span style={{ color: "rgba(255,255,255,0.65)", fontSize: 16 }}>{subtitle}</span>}
  </div>
);

export const DocEmbed: React.FC<EmbedSpec> = (spec) => {
  const { fps } = useVideoConfig();
  const accent = spec.accent ?? DEFAULTS.accent;
  const sourceWidth = spec.width ?? DEFAULTS.width;
  const sourceHeight = spec.height ?? DEFAULTS.height;

  return (
    <AbsoluteFill style={{ background: "#0b0f14" }}>
      <Series>
        {spec.scenes.map((scene, i) => (
          <Series.Sequence key={i} durationInFrames={sceneFrames(scene, fps)}>
            <SceneView
              scene={scene}
              width={sourceWidth}
              height={sourceHeight}
              accent={accent}
              sourceWidth={sourceWidth}
              sourceHeight={sourceHeight}
            />
          </Series.Sequence>
        ))}
      </Series>

      {spec.audioSrc && (
        <Sequence from={0}>
          <Audio src={staticFile(spec.audioSrc)} />
        </Sequence>
      )}

      <CaptionBar title={spec.title} subtitle={spec.subtitle} accent={accent} />
    </AbsoluteFill>
  );
};
