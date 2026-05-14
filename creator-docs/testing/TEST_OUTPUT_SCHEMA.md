# 测试输出与契约（TEST_OUTPUT_SCHEMA）

## Tauri / Rust

- **主对话响应**：前后端契约以 **`src-tauri/src/models/dto.rs`** 为准；用户可见回复字段名为 **`reply`**（不是 `response`）。
- **集成 / API 测试**：`src-tauri/tests/*.rs` 使用 **`serde_json`** 断言结构；**无** 统一 machine-readable schema 文件；若引入 JSON fixture，建议放在 `src-tauri/tests/fixtures/` 并在本文件索引。

## 前端

- **CI 当前守门**：`npm run build`（生产 bundle 可编译）。
- **单元测试（Vitest 等）**：`package.json` **未** 配置 `test:unit`；若后续添加，应在本节记录 **`npm run test:unit`** 与覆盖率/快照策略。

## HTTP 本地 API（`--api`）

- **`POST /chat`**：成功体含 **`reply: string`**；与 `SendMessageResponse` 对齐。详见根目录 [README.md](../../README.md)「本地 HTTP API」。

---

[English](../../creator-docs-en/testing/TEST_OUTPUT_SCHEMA.md)
