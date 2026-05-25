import { registerRoot, Composition } from "remotion";
import React from "react";
import { ModsButtonFeature } from "./compositions/ModsButtonFeature";
import { F9OverlayFeature }  from "./compositions/F9OverlayFeature";
import { F10MenuFeature }    from "./compositions/F10MenuFeature";
import { DINOForgeReel }     from "./compositions/DINOForgeReel";

const RemotionRoot: React.FC = () => (
  <>
    <Composition
      id="ModsButtonFeature"
      component={ModsButtonFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="F9OverlayFeature"
      component={F9OverlayFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="F10MenuFeature"
      component={F10MenuFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="DINOForgeReel"
      component={DINOForgeReel}
      durationInFrames={1140}
      fps={30}
      width={1280}
      height={800}
    />
  </>
);

registerRoot(RemotionRoot);
