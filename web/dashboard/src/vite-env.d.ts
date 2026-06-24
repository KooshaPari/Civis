/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CIVIS_WS_URL?: string;
  readonly VITE_CIVIS_WS_BINARY?: string;
  readonly VITE_CIVIS_WATCH_HTTP?: string;
  readonly VITE_CIVIS_SERVER_HTTP?: string;
  readonly VITE_CIV_SERVER_PORT?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
