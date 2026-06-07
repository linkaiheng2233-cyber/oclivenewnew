/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly MODE: string
  readonly VITE_SENTRY_DSN?: string
  readonly VITE_OCLIVE_SHELL?: 'tool' | 'fluent'
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
