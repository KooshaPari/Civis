import React from "react";
import { AbsoluteFill, Sequence, staticFile } from "remotion";
import { TitleCard } from "../components/TitleCard";
import { ModsButtonFeature } from "./ModsButtonFeature";
import { F9OverlayFeature } from "./F9OverlayFeature";
import { F10MenuFeature } from "./F10MenuFeature";

// 38s total × 30fps = 1140 frames
// Frame layout:
//   0  – 89 :  intro title card  (3s)
//   90 – 389:  mods feature      (10s)
//  390 – 689:  F9 feature        (10s)
//  690 – 989:  F10 feature       (10s)
//  990 –1139:  outro title card  (5s)

const INTRO_TTS = staticFile("/intro.mp3");
const OUTRO_TTS = staticFile("/outro.mp3");

export const DINOForgeReel: React.FC = () => (
  <AbsoluteFill style={{ background: "#0a0a0f" }}>
    <Sequence from={0} durationInFrames={90}>
      <TitleCard
        title="DINOForge Mod Platform"
        subtitle="Feature Demonstration"
        audioSrc={INTRO_TTS}
      />
    </Sequence>

    <Sequence from={90} durationInFrames={300}>
      <ModsButtonFeature />
    </Sequence>

    <Sequence from={390} durationInFrames={300}>
      <F9OverlayFeature />
    </Sequence>

    <Sequence from={690} durationInFrames={300}>
      <F10MenuFeature />
    </Sequence>

    <Sequence from={990} durationInFrames={150}>
      <TitleCard
        title="All Features Confirmed"
        subtitle="DINOForge is ready for testing"
        audioSrc={OUTRO_TTS}
      />
    </Sequence>
  </AbsoluteFill>
);
