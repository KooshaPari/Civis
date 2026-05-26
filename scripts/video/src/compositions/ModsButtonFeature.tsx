import React from "react";
import { staticFile } from "remotion";
import { FeatureScene } from "../components/FeatureScene";

// Use staticFile() for assets in public/assets/ during render.
// In the Studio, these are pre-copied from capture/tts directories.
const RAW_CLIP = staticFile("/raw_mods.mp4");
const TTS_FILE = staticFile("/mods.mp3");

export const ModsButtonFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="Mods Button Injected"
    calloutSubText="Native menu — under 10 seconds"
    calloutColor="#34d399"
    audioSrc={TTS_FILE}
    rawDurationFrames={180}
  />
);
