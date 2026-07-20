import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import type { OutputChunk } from "rollup";

const INITIAL_ENTRY_BUDGET_BYTES = 450 * 1024;

function initialEntryBudget() {
  return {
    name: "civis-initial-entry-budget",
    generateBundle(_options: unknown, bundle: Record<string, unknown>) {
      const entry = Object.values(bundle).find(
        (asset): asset is OutputChunk =>
          typeof asset === "object" &&
          asset !== null &&
          (asset as { type?: string }).type === "chunk" &&
          Boolean((asset as { isEntry?: boolean }).isEntry) &&
          String((asset as { facadeModuleId?: string | null }).facadeModuleId ?? "").endsWith(
            "/src/main.tsx",
          ),
      );
      if (!entry) return;

      const bytes = Buffer.byteLength(entry.code, "utf8");
      if (bytes > INITIAL_ENTRY_BUDGET_BYTES) {
        this.error(
          `Civis initial dashboard entry is ${bytes} bytes; budget is ${INITIAL_ENTRY_BUDGET_BYTES} bytes`,
        );
      }
    },
  };
}

const WATCH_PORT = process.env.CIV_WATCH_PORT ?? "9090";
const SERVER_PORT = process.env.CIV_SERVER_PORT ?? "3000";
const WATCH = process.env.VITE_CIVIS_WATCH_HTTP ?? `http://127.0.0.1:${WATCH_PORT}`;
const SERVER = process.env.VITE_CIVIS_SERVER_HTTP ?? `http://127.0.0.1:${SERVER_PORT}`;

export default defineConfig({
  plugins: [react(), initialEntryBudget()],
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
