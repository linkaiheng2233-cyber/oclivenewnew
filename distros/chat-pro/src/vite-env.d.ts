/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly MODE: string
  readonly VITE_SENTRY_DSN?: string
  readonly VITE_OCLIVE_SHELL?: 'tool' | 'fluent' | 'theater'
  /** CP-INT R2: `'1'` compiles in the read-only test handle; unset in every normal build. */
  readonly VITE_OCLIVE_TEST_HARNESS?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
