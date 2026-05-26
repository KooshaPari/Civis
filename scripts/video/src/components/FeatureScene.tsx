import React from "react";
import { AbsoluteFill, Audio, Video } from "remotion";
import { CalloutBox } from "./CalloutBox";
import { CaptionBar } from "./CaptionBar";
import type { FeatureSceneProps } from "../types";

export const FeatureScene: React.FC<FeatureSceneProps> = ({
  videoSrc,
  calloutText,
  calloutSubText,
  calloutColor,
  audioSrc,
}) => (
  <AbsoluteFill>
    {/* Raw clip — freeze-frames on last frame once source ends */}
    <Video
      src={videoSrc}
      style={{ width: "100%", height: "100%", objectFit: "cover" }}
    />

    {/* Neural TTS voiceover */}
    <Audio src={audioSrc} />

    {/* Spring-physics callout — animates in at frame 15 (0.5s delay) */}
    <CalloutBox
      text={calloutText}
      subText={calloutSubText}
      color={calloutColor}
      startFrame={15}
    />

    {/* Permanent caption bar */}
    <CaptionBar />
  </AbsoluteFill>
);
