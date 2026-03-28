import React from "react";
import { FeatureScene } from "../components/FeatureScene";

// Paths injected via environment variables by the prove-features orchestrator.
// Defaults allow the Remotion Studio to load without errors.
const RAW_CLIP = process.env["RAW_MODS_PATH"] ?? "";
const TTS_FILE = process.env["TTS_MODS_PATH"] ?? "";

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
