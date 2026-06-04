import { registerRoot, Composition } from "remotion";
import React from "react";
import { DocEmbed, totalFrames } from "./DocEmbed";
import type { EmbedSpec } from "./schema";
import { DEFAULTS } from "./schema";

/**
 * Single generic composition. The actual embed is fully described by the
 * EmbedSpec passed via input props (`--props ./annotations.json`).
 * `calculateMetadata` derives duration + dimensions from that spec, so one
 * composition renders ANY embed across ANY project.
 */
const FALLBACK: EmbedSpec = {
  id: "placeholder",
  title: "doc-embed (no props)",
  scenes: [{ src: "placeholder.png", holdSec: 3 }],
};

const RemotionRoot: React.FC = () => (
  <Composition
    id="DocEmbed"
    component={DocEmbed}
    defaultProps={FALLBACK}
    calculateMetadata={({ props }) => {
      const spec = props as EmbedSpec;
      return {
        durationInFrames: Math.max(1, totalFrames(spec)),
        fps: spec.fps ?? DEFAULTS.fps,
        width: spec.width ?? DEFAULTS.width,
        height: spec.height ?? DEFAULTS.height,
      };
    }}
  />
);

registerRoot(RemotionRoot);
