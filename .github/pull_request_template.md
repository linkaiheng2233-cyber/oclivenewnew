## 摘要

<!-- 一句话说明本 PR 做什么 -->

## 检查清单

- [ ] 本地已跑过 `npm run check`（或发版相关改动已跑 `npm run check:release`）
- [ ] 契约 / manifest / 对外文档有变时已同步更新
- [ ] 若改动 `domain/**`、`Cargo.lock`、`CHANGELOG*`、`.github/workflows/ci.yml` 或 host→runtime 再导出路径，已跑 `node scripts/dimension5-acceptance.mjs --ci`（见 [CONTRIBUTING.md](CONTRIBUTING.md)「Dimension 5 基线」）

### 按变更类型（见 [human-docs/08_PR_GATE_MATRIX.md](human-docs/08_PR_GATE_MATRIX.md)）

- [ ] **docs-only** → 仅 `check-changelog-parity`（若适用）
- [ ] **frontend** → `npm run test:unit` + `npm run build`
- [ ] **kernel** → `npm run check` 或 `check:release`
- [ ] **Cargo.lock** → `npm run check:ci-local`
