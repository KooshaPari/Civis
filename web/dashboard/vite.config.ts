import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/events": "http://localhost:9090",
      "/snapshot": "http://localhost:9090",
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
