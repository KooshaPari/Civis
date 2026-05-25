import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const WATCH_PORT = process.env.CIV_WATCH_PORT ?? "9090";
const SERVER_PORT = process.env.CIV_SERVER_PORT ?? "3000";
const WATCH = process.env.VITE_CIVIS_WATCH_HTTP ?? `http://127.0.0.1:${WATCH_PORT}`;
const SERVER = process.env.VITE_CIVIS_SERVER_HTTP ?? `http://127.0.0.1:${SERVER_PORT}`;

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
