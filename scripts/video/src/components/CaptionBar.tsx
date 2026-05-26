import React from "react";

export const CaptionBar: React.FC = () => (
  <div
    style={{
      position: "absolute",
      bottom: 0,
      left: 0,
      right: 0,
      background: "rgba(0,0,0,0.75)",
      padding: "8px 0",
      textAlign: "center",
      fontFamily: "Arial, sans-serif",
      fontSize: 16,
      color: "white",
    }}
  >
    F9 = Debug Overlay &nbsp;|&nbsp; F10 = Mod Menu &nbsp;|&nbsp; Mods Button = Native Menu Injection
  </div>
);
