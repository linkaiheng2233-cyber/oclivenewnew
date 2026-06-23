# 测试输出与契约（TEST_OUTPUT_SCHEMA）

## `oclive test --json`

**Schema 文件**：[`kernel/crates/oclive-cli/schemas/oclive_test_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_test_report.schema.json)

**示例**：

```json
{
  "schema_version": 1,
  "summary": { "passed": 3, "failed": 0, "skipped": 1 },
  "suites": [
    { "name": "cargo check", "status": "passed", "duration_ms": 1200 },
    { "name": "oocp", "status": "skipped", "duration_ms": 0, "detail": "skipped (--skip-oocp)" }
  ],
  "failures": []
}
```

| 字段 | 说明 |
|------|------|
| `summary.passed` / `failed` / `skipped` | 套件计数 |
| `suites[]` | 每项 `name`、`status`（`passed` \| `failed` \| `skipped`）、可选 `duration_ms`、`detail` |
| `failures[]` | 失败项：`suite`、可选 `file` / `line`（Rust 套件暂无行号时为 `null`）、`error` |

**CI 消费**：`oclive test -o . --json \| jq '.summary.failed'` 应为 `0`；非零时进程 **exit 1**。

**注意**：`oclive test --ci-parity --json` 仍输出 job 列表（历史格式）；默认 `test --json` 使用上表结构。

---

## Tauri / Rust 集成测

- **主对话响应**：前后端契约以 **`oclive_kernel_types`** DTO 为准；用户可见回复字段名为 **`reply`**。
- **`distros/desktop-tauri/tests/*.rs`**：使用 **`serde_json`** 断言；fixture 建议放在 `distros/desktop-tauri/tests/fixtures/`。

## 前端

- **CI**：`npm run test:unit`（Vitest，15 tests）+ `npm run build`。
- **Ubuntu CI**：`npm run test:e2e:preview`（Playwright + `vite preview`）。

## HTTP 本地 API（`--api`）

- **`POST /chat`**：成功体含 **`reply: string`**；与 `SendMessageResponse` 对齐。详见根目录 [README.md](../../README.md)。

## 其它 JSON 报告

| 命令 | Schema |
|------|--------|
| `oclive bench --json` | [`oclive_bench_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json) |
| `oclive doctor --json` | [`oclive_doctor_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_doctor_report.schema.json) |

---

[English](../../creator-docs-en/testing/TEST_OUTPUT_SCHEMA.md)
