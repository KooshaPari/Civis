import React from "react";
import { staticFile } from "remotion";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = staticFile("/raw_f10.mp4");
const TTS_FILE = staticFile("/f10.mp3");

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
