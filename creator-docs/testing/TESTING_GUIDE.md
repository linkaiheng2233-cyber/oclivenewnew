# 性能与长稳测试指南（测试前）

面向在本地跑 **v2 蓝图 / Monolith** 工程基准的集成方。命令均以仓库根执行；`-o` 指向**已 `init` 且可 `cargo build --release`** 的内核工程目录。

**相关**：[PERFORMANCE.md](../getting-started/PERFORMANCE.md)（表格与陷阱）· [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)（`bench` 全参数）· [TEST_OUTPUT_SCHEMA.md](TEST_OUTPUT_SCHEMA.md)（`oclive test --json`）

---

## 三种测试一览

| 测试类型 | 目的 | 命令（复制后替换 `<工程>` / `<monolith工程>`） | 预计耗时 | 验收标准 |
|----------|------|-----------------------------------------------|----------|----------|
| **高耦合矩阵** | Monolith **4×3** 档位×preset 组合性能 | `cargo run -p oclive-cli -- --experimental bench --matrix --release -o <monolith工程> --json > matrix.json` | **2–4 小时** | `matrix.json` 含 12 组数据；将 p50 填入 [PERFORMANCE.md](../getting-started/PERFORMANCE.md) 矩阵表 |
| **冷启动** | 进程冷启到首条 `/chat` 可回复 | `cargo run -p oclive-cli -- --experimental bench --cold-start --cold-start-runs 5 -o <工程>` | **约 30 分钟** | 5 轮冷启/热启延迟稳定；无超时（工程须 `--api` + `OCLIVE_HTTP_API_MOCK_LLM=1`） |
| **长稳运行** | 长时间 RSS / 泄漏与子进程回收 | `cargo run -p oclive-cli -- --experimental bench --soak --soak-real-time --soak-duration 72 --soak-sample-interval 60 -o <工程> --output <soak.json>` | **真实 72h**（本地加速见 PERFORMANCE §5.7） | RSS 增长不超过 20%；零请求/采样失败；无提前退出；`process_reaped=true` |

---

## 前置条件

1. **Monolith 矩阵**：`oclive init --monolith`（或已有 `monolith.toml` 的工程）；可选 `--kernel-source` 指向本仓以链接真实 runtime。  
2. **冷启动 / 长稳**：工程能 `cargo run --release -- --api`；建议 `OCLIVE_HTTP_API_MOCK_LLM=1`。  
3. **角色包**：v2 蓝图示例见 `distros/chat-pro/roles/mumu/pipeline.ocblueprint`（仅作负载参考，非 v1 对比）。

---

## 推荐顺序

1. 先 **冷启动**（快速发现 API / 链接问题）  
2. 再 **矩阵**（调 Monolith 焊接组合）  
3. 最后 **长稳**（专用机器过夜；本地先用加速模式或 `--soak-real-time --soak-duration 0.01` 校准）

---

## 结果解读（填入 PERFORMANCE.md）

跑完 `oclive bench` 后，将 JSON / 终端输出归档到本地（**勿提交** `bench_history.json` / `matrix.json`）。本节说明如何把数字写进 [PERFORMANCE.md](../getting-started/PERFORMANCE.md) 并判断是否达标。

### 矩阵（`bench --matrix`）

| 字段 | 含义 | 填表方式 |
|------|------|----------|
| `standard_ms` / `monolith_ms` | 该档位×preset 下子进程热循环耗时 | 取 JSON 中该组合的 **p50**（或终端摘要中位数），填入 PERFORMANCE §5.3 表格对应单元格 |
| 12 组 | 4 档位 × 3 preset | 缺一组则矩阵不完整，需重跑或检查 `monolith.toml` |

**填表说明（PERFORMANCE §5.3）**：表头为 preset（minimal / mixed / full），行名为档位（none / latency / memory / embedded）；单元格写 **毫秒数 + 可选备注**（如 `142ms @ 2026-05-22 win11`）。

### 冷启动（`bench --cold-start`）

| 指标 | 正常范围（参考） | 异常信号 |
|------|------------------|----------|
| 首条 `/chat` 延迟（冷） | 与机器相关；**5 轮中位数应稳定**（相对波动 &lt; 约 30%） | 单轮超时、端口未就绪、中位数逐轮翻倍 |
| 热启动平均 | 通常 **明显低于** 冷启动 | 热启 ≥ 冷启 → 检查进程是否未退出、MOCK LLM 未生效 |
| API 就绪时间 | 数秒内（本机 Release + MOCK） | &gt; 60s → 工程未 `--api` 或链接错误 |

将 5 轮 **冷启动中位数** 记入本地 `bench_history` 或 PERFORMANCE 旁注，不作为硬门禁。

### 长稳（`bench --soak`）

| 指标 | 告警阈值 | 解读 |
|------|----------|------|
| RSS 终值 vs 首样本 | **终值 ≤ 首样本 × 1.2** | 超出则可能存在泄漏或缓存未释放 |
| `warmup_chats` | **1** | 热身在计时与 RSS 基线之前完成，避免把首聊懒加载误判为泄漏 |
| `successful_chats` / `failed_chats` | 成功数持续增长且失败为 **0** | 失败不会再被伪计为成功；先查 HTTP、内核错误和 DB 连接 |
| `sampling_failures` | **0** | 非零表示无法取得实际内核 PID 的资源样本，报告不能作为泄漏证据 |
| `process_early_exit` / `process_reaped` | `null` / **`true`** | 分别验证长稳期间未提前退出、结束后子进程已被 wait 回收 |

默认 `--soak-duration` 是名义时长的加速采样（墙钟约 2s×小时数，最短 8s、上限 120s）；**72h 真长稳必须显式加 `--soak-real-time`** 并使用 `--output <文件>` 原子保存证据；`--json` 仍专用于 stdout 管道。两种模式均直接采样构建后的内核 PID，而不是 `cargo run` 包装进程；正式时长与 RSS 基线从一次成功热身后开始。

### 与 `oclive test` 报告

`oclive test --json` 的 `checks[]` 与性能无关；性能数据只来自 **`bench`** 子命令。CI 默认不跑 matrix/soak（耗时），与 [OVERVIEW.md](OVERVIEW.md) 一致。

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
