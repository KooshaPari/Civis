import React from "react";
import { AbsoluteFill, OffthreadVideo, Img, staticFile, useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import type { Scene } from "../schema";
import { Callout } from "./Callout";
import { HighlightRect, Cursor } from "./Highlight";

interface SceneViewProps {
  scene: Scene;
  width: number;
  height: number;
  accent: string;
  /** Source pixel dimensions for highlight/cursor coordinate mapping. */
  sourceWidth: number;
  sourceHeight: number;
}

const isVideo = (src: string) => /\.(mp4|mov|webm|mkv)$/i.test(src);

export const SceneView: React.FC<SceneViewProps> = ({
  scene, width, height, accent, sourceWidth, sourceHeight,
}) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();

  // Annotations are authored in source space; map to canvas (cover fit).
  const scaleX = width / sourceWidth;
  const scaleY = height / sourceHeight;

  // Optional Ken-Burns zoom for stills.
  let scale = 1;
  if (scene.zoom) {
    scale = interpolate(frame, [0, durationInFrames], scene.zoom, {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    });
  }

  const media = isVideo(scene.src) ? (
    <OffthreadVideo src={staticFile(scene.src)} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
  ) : (
    <Img
      src={staticFile(scene.src)}
      style={{ width: "100%", height: "100%", objectFit: "cover", transform: `scale(${scale})` }}
    />
  );

  return (
    <AbsoluteFill>
      {media}
      {(scene.highlights ?? []).map((h, i) => (
        <HighlightRect key={`h${i}`} {...h} scaleX={scaleX} scaleY={scaleY} accent={accent} />
      ))}
      {(scene.cursors ?? []).map((c, i) => (
        <Cursor key={`c${i}`} {...c} scaleX={scaleX} scaleY={scaleY} accent={accent} />
      ))}
      {(scene.callouts ?? []).map((c, i) => (
        <Callout key={`co${i}`} {...c} accent={accent} />
      ))}
    </AbsoluteFill>
  );
};
