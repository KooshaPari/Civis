import React from "react";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = process.env["RAW_F10_PATH"] ?? "";
const TTS_FILE = process.env["TTS_F10_PATH"] ?? "";

export const F10MenuFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="[ F10 ] Mod Menu"
    calloutSubText="Browse and toggle mod packs"
    calloutColor="#60a5fa"
    audioSrc={TTS_FILE}
    rawDurationFrames={240}
  />
);
