# A1（测试与质量闸门）收口摘要

**口径**：**A1 = 仓库内可稳定回归、且已写入 CI 或 handoff 的「可 CI 子集」**；**不将安装包签名 / Tauri 原生窗 / WebDriver 全屋** 虚构为已完成（见发版表 **A1.1c**）。

## 已交付

| 编号 | 内容 | 入口 |
|------|------|------|
| **A1.1a** | HTTP `--api` 进程重启后再对话 | [`scripts/e2e-core-api-restart.mjs`](../scripts/e2e-core-api-restart.mjs)，CI **`oocp-test-suite`** |
| **A1.1b** | `vite build` + `vite preview` + Playwright 首屏 | [`e2e/preview-shell.spec.ts`](../e2e/preview-shell.spec.ts)，`npm run test:e2e:preview`；**CI：Ubuntu `frontend` only**（`PW_TEST_USE_EXTERNAL` + 后台 `vite preview`，见 `.github/workflows/ci.yml`） |
| **A1.2** | 九条 `invoke` 宿主热路径 `*_impl` 链 | [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md)，[`src-tauri/tests/invoke_hotpath_matrix.rs`](../src-tauri/tests/invoke_hotpath_matrix.rs) |
| **A1.3** | 闸门与 CI 对齐说明 | [`CONTRIBUTING.md`](../CONTRIBUTING.md)、[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) |
| **A1.4** | 回归清单聚合（链到既有 guides） | [`PRODUCT_RELEASE_CHECKLIST.md`](./PRODUCT_RELEASE_CHECKLIST.md)「回归与手工」 |

## 刻意不记入 A1（另立项）

- **A1.1c**：安装包 / **Tauri 原生窗口** / 真 `invoke` 全屋 GUI E2E（WebDriver、发行流水线或专用 driver）。
- **`invoke` golden JSON**、全 handler 表：仍属增强项，见 [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md)「仍属后续增强」。

## 维护者自检命令（发版前建议）

`npm run test:unit` → `npm run build && npm run test:e2e:preview` → `npm run check:release`；改 HTTP/编排时关注 **`oocp-test-suite`** 与 **`test:e2e:core-api-restart`**。
