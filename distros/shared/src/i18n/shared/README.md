# Shared i18n (cross-repo)

Canonical copy lives in **oclivenewnew** `src/i18n/shared/`. Sister repos **oclive-launcher** and **oclive-pack-editor** mirror the same files under their own `src/i18n/shared/`.

Run from oclivenewnew root:

```bash
npm run verify:shared-i18n
```

CI and `check:release` run this script; drift fails the build.

Sync mirrors after editing canonical files:

```bash
node scripts/sync-shared-i18n.mjs
```
