# 性能与长稳测试指南（测试前）

面向在本地跑 **v2 蓝图 / Monolith** 工程基准的集成方。命令均以仓库根执行；`-o` 指向**已 `init` 且可 `cargo build --release`** 的内核工程目录。

**相关**：[PERFORMANCE.md](../getting-started/PERFORMANCE.md)（表格与陷阱）· [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)（`bench` 全参数）· [TEST_OUTPUT_SCHEMA.md](TEST_OUTPUT_SCHEMA.md)（`oclive test --json`）

---

## 三种测试一览

| 测试类型 | 目的 | 命令（复制后替换 `<工程>` / `<monolith工程>`） | 预计耗时 | 验收标准 |
|----------|------|-----------------------------------------------|----------|----------|
| **高耦合矩阵** | Monolith **4×3** 档位×preset 组合性能 | `cargo run -p oclive-cli -- bench --matrix --release -o <monolith工程> --json > matrix.json` | **2–4 小时** | `matrix.json` 含 12 组数据；将 p50 填入 [PERFORMANCE.md](../getting-started/PERFORMANCE.md) 矩阵表 |
| **冷启动** | 进程冷启到首条 `/chat` 可回复 | `cargo run -p oclive-cli -- bench --cold-start --cold-start-runs 5 -o <工程>` | **约 30 分钟** | 5 轮冷启/热启延迟稳定；无超时（工程须 `--api` + `OCLIVE_HTTP_API_MOCK_LLM=1`） |
| **长稳运行** | 长时间 RSS / 泄漏趋势 | `cargo run -p oclive-cli -- bench --soak --soak-duration 72 -o <工程> --json` | **名义 72h**（本地加速见 PERFORMANCE §5.7） | **最终 RSS ≤ 首样本 × 1.2**；`--json` 可归档 |

---

## 前置条件

1. **Monolith 矩阵**：`oclive init --monolith`（或已有 `monolith.toml` 的工程）；可选 `--kernel-source` 指向本仓以链接真实 runtime。  
2. **冷启动 / 长稳**：工程能 `cargo run --release -- --api`；建议 `OCLIVE_HTTP_API_MOCK_LLM=1`。  
3. **角色包**：v2 蓝图示例见 `roles/mumu/pipeline.ocblueprint`（仅作负载参考，非 v1 对比）。

---

## 推荐顺序

1. 先 **冷启动**（快速发现 API / 链接问题）  
2. 再 **矩阵**（调 Monolith 焊接组合）  
3. 最后 **长稳**（专用机器过夜；本地可用较短 `--soak-duration` 冒烟）

---

## 与 CI / 其它测试的关系

| 命令 | 说明 |
|------|------|
| `oclive test -o <工程>` | 工程自检：`cargo check`、clippy、角色包 `pack validate` |
| `oclive test --json` | 机器可读报告，见 [TEST_OUTPUT_SCHEMA.md](TEST_OUTPUT_SCHEMA.md) |
| `oclive test --ci-parity` | 对齐 `.github/workflows/ci.yml`（含 `cargo-audit` 若已 `ci init`） |
| OOCP / `npm run test:unit` | 协议与前端烟测，见 [OVERVIEW.md](OVERVIEW.md) |

---

[English](../../creator-docs-en/testing/TESTING_GUIDE.md)
