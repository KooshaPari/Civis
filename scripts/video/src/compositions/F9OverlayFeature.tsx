import React from "react";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = process.env["RAW_F9_PATH"] ?? "";
const TTS_FILE = process.env["TTS_F9_PATH"] ?? "";

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
