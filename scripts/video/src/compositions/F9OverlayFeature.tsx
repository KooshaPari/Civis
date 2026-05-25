import React from "react";
import { staticFile } from "remotion";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = staticFile("/raw_f9.mp4");
const TTS_FILE = staticFile("/f9.mp3");

export const F9OverlayFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="[ F9 ] Debug Overlay"
    calloutSubText="Live entity counts and runtime stats"
    calloutColor="#fbbf24"
    audioSrc={TTS_FILE}
    rawDurationFrames={240}
  />
);
