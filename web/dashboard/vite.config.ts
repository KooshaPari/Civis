import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const WATCH = "http://localhost:9090";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/events": WATCH,
      "/snapshot": WATCH,
      "/terrain": WATCH,
      "/control": WATCH,
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
