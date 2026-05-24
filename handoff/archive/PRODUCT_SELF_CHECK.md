# oclive 功能自检清单

**用途**：发版前或里程碑收口时，按表逐项核对**本仓库（oclivenewnew）**与姊妹仓能力边界。  
**最近自检**：2026-05-23（本机：`cargo test` 关键集成测、`npm run test:unit`、根目录 `cargo audit` / `cargo deny`）。  
**相关**：[PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)（差距/待办）· [PRODUCT_RELEASE_CHECKLIST.md](./PRODUCT_RELEASE_CHECKLIST.md)（发版闸门）

---

## 图例

| 标记 | 含义 |
|------|------|
| ✅ | 本仓代码/测试/文档已对齐，或 CI 已覆盖 |
| 🔶 | 能力存在，但需本地长时跑、专用工程或另仓验收 |
| 📦 | 主要在 **oclive-studio** 等姊妹仓，不在本仓验证 |
| ➖ | 本清单项不适用示例包 / 计数口径见注 |

---

## 一、内核与编排（oclivenewnew）

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 从蓝图 `slot_registry` 动态构建编排流程 | `cargo run -p oclive-cli -- pack validate roles/mumu` | ✅ |
| 单类型单实例正常执行 | OOCP **S0–S12**（CI `oocp-test-suite`）；本地需起 `--api` 后 `node examples/oocp-test-suite/run.mjs` | ✅ |
| 单类型多实例串行（如两个 memory） | `cargo test -p oclivenewnew-tauri --test slot_runner_p4` | ✅ |
| 双 LLM 串行 last-wins | `slot_runner_p4::dual_llm_slots_call_both_serially` | ✅ |
| memory 多实例合并去重 | `slot_runner_p4::dual_memory_slots_merge_without_panic` | ✅ |
| event 多实例 | `SlotRunner::event_last_wins`（last-wins，非 memory 式去重合并） | ✅ |
| emotion / prompt / llm 多实例 last-wins | `slot_runner.rs` 各 `*_last_wins` | ✅ |
| agent 多插件并行、工具集合并 | `PluginHost` / `SlotResolver::wrap_agent_if_merged` | ✅ |
| 抽象情感 `narrative_hint` 注入 Prompt | `cargo test -p oclivenewnew-tauri --test narrative_hint_contract_audit` | ✅ |
| 会话级后端覆盖 | `cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix` | ✅ |
| 启动健康检查（七槽、文件、DB、可选 LLM） | `domain/startup_health.rs` + `process_message` 首轮 `ensure_once` | ✅ |
| 错误处理统一 | `PluginHostError` / `CoPresentError` / `ProcessMessageError` | ✅ |
| 日志统一（tracing） | `src-tauri/src` 无 `log::` 引用 | ✅ |
| 生产路径零裸 unwrap（lib） | `cd src-tauri && cargo clippy --lib -- -D warnings -W clippy::unwrap_used` | ✅ |

**OOCP 说明**：默认 **13** 项（`S0`–`S12`）；**S13** 双核降级为可选（`--include-s13` 或 `OCLIVE_OOCP_INCLUDE_S13=1`）。口径见 [OOCP_TEST_SUITE.md](../creator-docs/testing/OOCP_TEST_SUITE.md)。

```powershell
# 本地 OOCP（仓库根）
$env:OCLIVE_HTTP_API_MOCK_LLM = '1'
# 终端 1: cargo run -p oclivenewnew-tauri -- --api
# 终端 2:
node examples/oocp-test-suite/run.mjs
# node examples/oocp-test-suite/run.mjs --include-s13
```

---

## 二、蓝图与角色包

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| `pipeline.ocblueprint` 为配置中枢 | 默认 `pack validate`（v2/v3） | ✅ |
| `meta` 人格、关系等 | `cargo test -p oclivenewnew-tauri --test blueprint_v2_mumu_load` | ✅ |
| `meta.personality` 七维向量 | 迁移后行为见 mumu 包与集成测 | ✅ |
| `slot_registry` 动态槽位 | 架构图节点由 `slot_registry` 派生 | ✅ |
| `groups` 逻辑分组 | 架构图 `ArchGroupNode` | ✅ |
| `module_relations` 自动派生 | 禁止落盘（`FORBIDDEN_ROOT_KEYS`）；前端 `buildBlueprintEdges` | ✅ |
| `runtime_config`（v3） | `oclive_validation::runtime_config` + Schema | ✅ |
| `reply_quality_anchor` 在蓝图侧 | `runtime_config.reply_quality_anchor` | ✅ |
| v1 → v2 迁移 | `oclive pack migrate-to-blueprint` | ✅ |
| v2 → v3 迁移工具 | 手写 v3 示例 + `init --dual-core`；批量迁移延后（Q18） | 🔶 |
| 角色包 vs 蓝图职责分离 | [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) | ✅ |
| `pack validate --profile creator` | 用**纯创作者包**验证（含 `prompts/`、`meta` 仅 §2 创作者字段、不校验 `slot_registry`）；示例：`cargo run -p oclive-cli -- pack create …` 后 `pack validate --profile creator <包根>`。**勿**用 `roles/mumu` 代表 creator 目标 — mumu 为**完整示例包**（含 evolution、完整 `slot_registry` 等），对该 profile **预期失败** | ✅ |

---

## 三、插件系统

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 目录插件 install / uninstall | `oclive plugin install` / `uninstall` | ✅ |
| 依赖拓扑安装 | `plugin install` 解析依赖 | ✅ |
| 市场 browse / search / install | `oclive market` | ✅ |
| `complex_emotion` 支持 directory | `provides` / 权限校验 | ✅ |
| 高风险权限（process / network / MCP） | 授权 + `permission_three_way_consistency` | ✅ |
| 插件测试框架 | `oclive plugin test` | ✅ |
| **`official-vue-test-runner`（T14）** | `plugins/official-vue-test-runner/` + 编写器「前端测试」面板 JSON-RPC | ✅ |
| 官方极简插件管理 UI | `SimplePluginManager` + `uiStore` 开关 | ✅ |
| 权限三层分离 | [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) §7 | ✅ |

---

## 四、双核机制

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 双核默认关闭 | 未 `dual_core.enabled` + 空 `experimental` → 单核 `co_present` | ✅ |
| `oclive init --dual-core` | 生成 `schema_version: 3` 模板 | ✅ |
| `DualPipelineRunner` + 七槽 method | `dual_pipeline` 单测 + [METHOD_REGISTRY.md](../creator-docs/dual-core/METHOD_REGISTRY.md) | ✅ |
| 实验核优先 / 失败静默降级 | `run_with_fallback` + `target=oclive_dual_core` 日志 | ✅ |
| `TurnRollbackSnapshot` 三字段 | `narrative_hint`、`emotion_state`、`user_presence_scene` | ✅ |
| OOCP S13 | `examples/oocp-test-suite/fixtures/dual-core-fallback/` + `--include-s13` | ✅ |
| Monolith + 双核 | `init --monolith --dual-core`；保留调度器 | ✅ |
| 双核可视化 | 架构图横幅 + `zone` 标签 | ✅ |
| method 注册表 / explain | `METHOD_REGISTRY.md` + `oclive explain DUAL_CORE` | ✅ |
| 开发者指南 | [DEVELOPER_GUIDE.md](../creator-docs/dual-core/DEVELOPER_GUIDE.md) | ✅ |
| 与角色包边界 | [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) §5 | ✅ |

---

## 五、Monolith（高耦合编译）

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| `oclive init --monolith` | 生成 `monolith.toml` + 双 binary | ✅ |
| `weld_modules` / `exclude` | `monolith.toml` | ✅ |
| 标准 vs 焊接等价 | `oclive bench --equivalence` | ✅ |
| 性能矩阵 12 组合 | `oclive bench --matrix --release` | 🔶 待本地 2–4h |
| 冷启动 | `oclive bench --cold-start --cold-start-runs 5` | 🔶 待本地 ~30min |
| 长稳 72h | `oclive bench --soak --soak-duration 72 --json` | 🔶 待专用机 |

---

## 六、脚手架（`oclive-cli`）

**计数**：`oclive --help` 列出 **24** 个顶层子命令（含 `blueprint`、`debug`、`help` 等）。若只计产品化子命令约 **21–22**。

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| `init`（交互 / 非交互 / 模板 / `--dual-core` / `--monolith` / `--smart` / `--quick` / `--dry-run` / `--from-existing` 等） | `oclive init --help` · [OCLIVE_CLI_GUIDE.md](../creator-docs/cli/OCLIVE_CLI_GUIDE.md) | ✅ |
| `build` / `bench` / `dev` / `pack` / `plugin` / `doctor` / `test` / `lint` / `ci` | 各子命令 `--help` | ✅ |
| `explain`（错误码 + `DUAL_CORE`） | `oclive explain DUAL_CORE` | ✅ |
| `config` / `registry` / `compose` / `market` / `template` / `kernel` | CLI 已实现 | ✅ |
| `dashboard` / `learn` / `collab` / `completions` / `profile` | CLI 已实现 | ✅ |

---

## 七、工作室（oclive-studio）

**📦 本仓不验收**：启动/创作双模式、创作模式 E2E 5/5、`OnboardingWizard`、深链接 `oclive-studio://`、创作模式懒加载 chunk 等，见独立仓 **[oclive-studio](https://github.com/oclive-app/oclive-studio)** 与 [RFC_STUDIO_MERGE.md](../creator-docs/rfc/RFC_STUDIO_MERGE.md)。

**本仓（运行时）相关**：

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 环境诊断 | `run_environment_diagnostics` · 设置页 | ✅ |
| 试聊 / HTTP API | `oclivenewnew-tauri --api` + OOCP / 工作室拉起 | ✅ |
| 首屏 Web 烟测 | `npm run test:e2e:preview`（CI Ubuntu `frontend`） | ✅ |

---

## 八、CI 与质量

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| `cargo clippy -D warnings` | CI `rust` job（`src-tauri/`） | ✅ |
| `cargo test` | CI `rust` job（含 `src-tauri/tests/`） | ✅ |
| `npm run test:unit` | CI `frontend`（Ubuntu） | ✅ |
| OOCP S0–S12（+ 可选 S13） | CI `oocp-test-suite` | ✅ |
| 模糊测试 | CI `fuzz` job | ✅ |
| loom | CI `loom` job（`continue-on-error`） | ✅ |
| ARM64 交叉编译 | CI `rust-arm64-cross` | ✅ |
| `cargo audit` 漏洞级 | 根目录 `Cargo.lock`；[KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md) | ✅ |
| `cargo deny licenses` | 根目录 `deny.toml` · `licenses ok` | ✅ |

**本地快速复现**：

```powershell
Set-Location D:\oclivenewnew
$env:CARGO_BUILD_JOBS = '1'
cargo clippy --workspace --all-targets --all-features -- -D warnings
Set-Location src-tauri; cargo test
Set-Location ..
npm run test:unit
npm run build
cargo audit
cargo deny check licenses
```

---

## 九、文档体系

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 中文核心文档 ≥ 45 | `creator-docs/**/*.md`（当前约 **80**） | ✅ |
| 英文核心文档 ≥ 25 | `creator-docs-en/**/*.md`（当前约 **54**） | ✅ |
| 活跃文档无 v1 主路径 | [V1_CLEANUP_AUDIT.md](./V1_CLEANUP_AUDIT.md) | ✅ |
| v1 仅迁移/RFC/CHANGELOG | 见 V1 清理审计 | ✅ |
| 新贡献者入口 | [CONTRIBUTING.md](../CONTRIBUTING.md) · [BUS_FACTOR_NOTES.md](./BUS_FACTOR_NOTES.md) | ✅ |
| 测试指南 | [TESTING_GUIDE.md](../creator-docs/testing/TESTING_GUIDE.md) | ✅ |
| 双核 RFC + 指南 + 注册表 | `creator-docs/rfc/` + `dual-core/` | ✅ |
| 功能文档矩阵 | [FEATURE_DOC_COVERAGE.md](./FEATURE_DOC_COVERAGE.md) | ✅ |

---

## 十、安全与合规

| 功能 | 验证方式 | 状态 |
|------|---------|------|
| 供应链漏洞（漏洞级） | 根目录 `cargo audit` 退出码 **0** | ✅ |
| 许可证 | `cargo deny check licenses` | ✅ |
| 免责声明 | [DISCLAIMER.md](../creator-docs/legal/DISCLAIMER.md) | ✅ |
| 审查边界 | [SECURITY_AUDIT_SCOPE.md](../creator-docs/security/SECURITY_AUDIT_SCOPE.md) | ✅ |
| 三层权限 | 角色包 / 蓝图 / 用户授权 | ✅ |

> **不宣称零 warning**：`cargo audit` 仍有 *unmaintained* 等警告级条目，见 KNOWN_VULNERABILITIES。

---

## 总结

| 类别 | 结论 |
|------|------|
| 本仓自动化可验证项 | **已通过**（见上表 ✅） |
| 待本地长时 | 矩阵 / 冷启 / 长稳 bench（🔶） |
| 另仓验收 | 工作室完整 UX（📦 oclive-studio） |
| OOCP 本地 | 需自建 `--api` 后跑 `run.mjs` |

维护时请同步更新 **最近自检**日期与「➖ / 📦」备注，避免把 **mumu** 或 **studio** 能力误标为本仓必过项。
