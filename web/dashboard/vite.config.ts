import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const WATCH = process.env.CIVIS_WATCH_HTTP ?? "http://127.0.0.1:9090";
const SERVER = process.env.CIVIS_SERVER_HTTP ?? "http://127.0.0.1:3000";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/events": WATCH,
      "/snapshot": WATCH,
      "/terrain": WATCH,
      "/control": WATCH,
      "/healthz": SERVER,
      "/replay": SERVER,
      "/ws": {
        target: SERVER.replace(/^http/, "ws"),
        ws: true,
      },
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index: path.resolve(__dirname, "index.html"),
        status: path.resolve(__dirname, "status.html"),
      },
    },
  },
});
