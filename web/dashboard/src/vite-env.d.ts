/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CIVIS_WS_URL?: string;
  readonly VITE_CIVIS_WS_BINARY?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
