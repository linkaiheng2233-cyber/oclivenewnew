import antfu from '@antfu/eslint-config'

export default antfu({
  vue: true,
  typescript: true,
  ignores: [
    'dist',
    'node_modules',
    'distros/desktop-tauri',
    'playwright-report',
    'test-results',
    'distributions',
    'docs',
    'handoff',
    'creator-docs',
    'creator-docs-en',
    'examples',
    'scripts',
    'e2e',
    'crates',
    'distros/chat-pro/roles',
    '**/*.md',
  ],
  rules: {
    // Legacy codebase: introduce lint without blocking on pre-existing patterns
    'import/no-mutable-exports': 'off',
    'no-alert': 'off',
    'no-cond-assign': 'off',
    'no-control-regex': 'off',
    'no-useless-return': 'off',
    'node/prefer-global/process': 'off',
    'unused-imports/no-unused-vars': 'warn',
    'vue/custom-event-name-casing': 'off',
    'vue/valid-template-root': 'off',
  },
})
