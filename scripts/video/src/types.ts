export interface CalloutProps {
  text: string;
  subText?: string;
  color: string;        // hex, e.g. "#34d399"
  startFrame: number;   // frame at which spring animation begins
}

export interface FeatureSceneProps {
  videoSrc: string;           // absolute path to raw clip
  calloutText: string;
  calloutSubText?: string;
  calloutColor: string;
  audioSrc: string;           // absolute path to MP3
  rawDurationFrames: number;  // frames in source clip (6s=180, 8s=240)
}
