# OCLive 工程纪律审查清单（阶段 ⑤ · 项目定制）

与通用 `~/.cursor/skills/dev-pipeline/discipline-review.md` 合并。每项 **PASS / FAIL / N/A**；G FAIL 阻塞进 ⑥。只审 diff applicable 项，但 N/A 必须能说明原因。

**证据状态**：Implemented（未验完）· Locally verified（本地 applicable 全绿）· Done-eligible（所需远程证据齐全）。

## G 全局硬约束（按 diff Applicable）

| ID | 检查 | 方法 | FAIL 条件 |
|----|------|------|-----------|
| G1 | 角色包任务未改蓝图/六槽/发行版 runtime | diff `slot_registry` · `plugin_backends` · `runtime_config` · distro profile | 角色任务出现上述变更 |
| G2 | 未删 RFC Draft 仍接线的 wiring | diff 删改设施/侧通道 | 无授权删 wiring |
| G3 | 未把 archive / `04_4.6` / WEEKLY_DEV_GUIDE 当 truth | rg 新增文档链接 | 链归档当 SSOT |
| G4 | Cargo.lock → audit + 台账 | `cargo audit` · 中英 `KNOWN_VULNERABILITIES.md` | lock 变但未更新或不一致 |
| G5 | monorepo 路径 SSOT | rg stale `src-tauri/roles` · `join("roles")` | 新引入 stale 路径 |
| G6 | 编排位置 | diff 不在 `process_message`/`turn_pipeline` | API/`lib.rs` 新增业务编排 |
| G7 | DTO/契约 | 字段 `reply` 非 `response` | 自造别名/字段 |
| G7b | 错误码 SSOT | `check-error-codes-drift.mjs`；无 message 字符串匹配分支 | 未跑 drift；新增 `includes('plugin_backends:')` 等 |
| G8 | doctest | 公开 API 后 `cargo test --workspace --doc` | 应跑未跑或 FAIL |
| G9 | 反冗余 | 重复 struct/死 import/平行 helper | 本任务引入明显重复 |
| G10 | 模块定义 | MODULE_MAP 外无模块关系长文 | AGENTS/README 复制模块表 |
| G11 | 文档新建 | 无 RFC 不新建顶层 handoff/creator-docs md | 静默新建 |
| G12–G16 | 文档 SSOT | AI_CHANGE_BOUNDARIES §文档纪律 | 跨 SSOT 复制/未登记 |

## 代码编写纪律

| # | 检查 | 方法 |
|---|------|------|
| CD1 | 先 grep 复用 | rg 关键符号 |
| CD2 | 生产无 unwrap/expect | rg 变更 `.rs`，排除 test |
| CD3 | enum 穷尽 match | 读 diff |
| CD4 | 新字段 `..Default::default()` | 读构造点 |
| CD5 | 无 speculative 泛型/trait | 读 public API |
| CD6 | 复制≥2 已抽 helper 或 N/A | 读 diff |
| CD7 | `build_prompt` 未用 `?` | 触及 prompt_builder |
| CD8 | Tauri 命令仅 `api/*.rs` | diff 路径 |
| CD9 | SQL 表名以 `migrations/001_init.sql` 为准 | 触及 SQL |
| CD10 | NAMING_CONVENTIONS §4.2 | 读 use |
| CD11 | AppError SSOT 链 | error → generate → kernelErrorCodes → apiErrors → ERROR_CODES |

## 架构边界

| # | 检查 | FAIL 条件 |
|---|------|-----------|
| AR1 | `steps[]` 不参与首轮调度 | 新增 steps 调度 |
| AR2 | 记忆三套不混用 | chat 表当 memory 真源 |
| AR3 | 复杂情感 ≠ 六槽 emotion | `complex_emotion` 冒充六槽键 |
| AR4 | 独立通道不进六槽 | voice.asr 等进 `plugin_backends` |
| AR5 | 角色包 vs 蓝图 | 创作者字段写入 `slot_registry` |
| AR6 | 无第二套六槽 parser | CLI/诊断平行解析 |
| AR7 | CLI→host 依赖有文档 | 新增 host 依赖未更新 COMPATIBILITY/CLI guide |

## 静态门禁脚本

| 脚本 | 何时必跑 |
|------|----------|
| `node scripts/check-domain-layering.mjs` | 触及 kernel 分层 |
| `node scripts/check-stale-paths.mjs` | 路径/文档链接/布局 |
| `node scripts/check-error-codes-drift.mjs` | error / apiErrors / ERROR_CODES |
| `node scripts/check-doc-mirror.mjs` | creator-docs 中英 |
| `node scripts/dimension5-acceptance.mjs --ci` | 改 dimension5/CI/门禁组合，或 L 全量验收；**N 以脚本输出为准** |
| `node scripts/check-doc-registry.mjs` | 新建/变更 handoff 或 creator-docs 顶层 |
| `node scripts/check-markdown-links.mjs` | 触及 `human-docs/modules`（**默认不扩全仓历史文档**） |
| `npm run check:rust` | Rust workspace / 分层变更；纯前端 TS 走对应 build/unit |
| `npm run check:ci-local` | 债收口 / 集成行为 / 跨宿主；纯静态门禁改动不自动强制 |
| `npm run check:release` | 发版级 |
| `cargo test --workspace --doc` | G8 applicable |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 发版或用户要求 |
| `cargo test --locked -p oclive-cli -- --test-threads=1` | CLI crate 全测；E2E 嵌套 Cargo，串行避免 package-cache 锁竞争 |

## CI 推送节奏

| 检查 | PASS | FAIL |
|------|------|------|
| 开发期门禁 | 按变更面窄测，本地提交形成可审查切片 | 每个小改都推送并依赖远端全矩阵找错 |
| PR 状态 | 未冻结时保持 draft；ready 后需继续实质开发则先转回 draft | 在 ready PR 上连续推送并反复触发正式全量门禁 |
| 里程碑冻结 | applicable 本地门禁通过后冻结 HEAD，一次推送 | 本地未收口就反复推 ready PR |
| 失败处理 | 读取失败 job、根因窄测、修复后再推 | 无变化重复 rerun 确定性失败 |
| 绿灯证据 | PR 评论/交付报告先记，随下次实质提交入账 | 只为回写 run ID 追加提交并再触发全矩阵 |
| Done 绑定 | 远端 success 对应报告中的完整 SHA | 用旧 SHA 的绿灯证明新 HEAD |
| 门禁语义 | `ci-draft-gate` 仅开发反馈；ready/main 以 `ci-gate` 为准 | 用草稿选择性结果声称可合并或 Done |

## Done 证据

遵循 [`AI_VERIFICATION_PROTOCOL.md`](../../../handoff/AI_VERIFICATION_PROTOCOL.md)：

- 禁止仅凭本地 PASS / Plan `completed` / 单点测 将 TECHNICAL_DEBT Partial→Done
- 声称 main 恢复或债收口：须远程 `ci.yml` 对目标提交 **conclusion=success**（记 run URL/databaseId）
- 没有 push/外部写权限：结论写 **Locally verified，等待维护者远程 CI**；台账保持 Partial/OPEN，不得把权限缺失包装成 Done
- 台账回写：**HEAD SHA** · 日期 · 命令/CI 证据
- 远端已绿后不得为台账回写单独制造新 HEAD；证据先记 PR 评论/交付报告，随下一次实质提交入账
- dimension5：以脚本结尾为准；`--ci` 跳过 sample lib tests 但仍计入
- invoke 条数：以 [`INVOKE_HOTPATH_MATRIX.md`](../../../handoff/INVOKE_HOTPATH_MATRIX.md) 为准
- 禁止 L0 观察冒充 P0/P1 或 OPEN

## 输出示例

```markdown
## OCLive 工程纪律审查

| ID | 结果 | 证据 |
|----|------|------|
| G6 | PASS | 编排仅 turn_pipeline |
| G7b | PASS | check-error-codes-drift exit 0 |
| evidence-state | PASS | Locally verified / Done-eligible |
| remote-ci | PASS/N/A | ci.yml run … @ HEAD … / 无 push 权限，未入账 Done |

**结论**：PASS
```
