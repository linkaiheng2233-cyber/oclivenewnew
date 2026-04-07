# 策略插件化发布前检查清单（1页）

## 适用范围

- 发布前策略相关改动（`policy.toml`、策略实现、`chat_engine` 编排、`policy_e2e_matrix`）必跑。
- 适用于 4.5 / 4.6 协作交接与 CI 发布门禁。

**补充（2026-04）**：虚拟时间、场景、导出与前端工具条说明见 `handoff/17_TIME_SCENE_EXPORT_HANDOFF.md`；发布整体验证时建议与主界面「虚拟时间 / 导出」联调一并执行。

## 发布前必做（Blocking）

- [ ] 配置检查
  - [ ] `src-tauri/config/policy.toml` 可解析（无未知字段、无拼写错误）。
  - [ ] `default_profile`、`profiles.*`、`scene_bindings` 关系正确（无悬空 profile）。
- [ ] 代码门禁
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy --all-targets -- -D warnings`
  - [ ] `cargo test --tests`
- [ ] 策略回归
  - [ ] `cargo test --test policy_e2e_matrix`
  - [ ] 快照差异为 0；若有预期行为变更，先评审再更新快照。
- [ ] 事务与一致性
 - [ ] 动态加载检查
   - [ ] 调用 `reload_policy_plugins` 成功返回，并输出 scene 绑定数量。
   - [ ] 重载失败时返回错误但不影响当前服务可用性。
  - [ ] `send_message` 路径仍走 `apply_chat_turn_atomic`。
  - [ ] 没有把策略逻辑回写到 `DbManager`。

## 快照更新流程（仅在“确认变更合理”后）

1. 本地运行：
   - PowerShell: `$env:UPDATE_POLICY_SNAPSHOTS='1'; cargo test --test policy_e2e_matrix`
2. 审阅变更：
   - 检查 `src-tauri/tests/snapshots/policy_e2e_matrix.json` 指标变化是否符合预期。
3. 清理环境变量并复跑：
   - `Remove-Item Env:UPDATE_POLICY_SNAPSHOTS -ErrorAction SilentlyContinue`
   - `cargo test --test policy_e2e_matrix`
4. 在 MR/PR 描述中记录：
   - 变更指标、原因、风险评估、回滚策略。

## 失败处置速查

- `policy config parse failed`
  - 检查 TOML 顶层结构与字段名；优先对照 `15_POLICY_PLUGIN_GUIDE_AND_ROADMAP.md`。
- `policy snapshot mismatch`
  - 先看 diff 输出字段（expected/actual）；确认是否预期行为变化。
- `exploratory should persist more memories...`
  - 检查 `scene_bindings` 是否生效，`home/school` 是否绑定到预期 profile。

## 交接最小包（给接手团队）

- `handoff/15_POLICY_PLUGIN_GUIDE_AND_ROADMAP.md`（原理/流程/风险）
- `handoff/16_POLICY_RELEASE_CHECKLIST.md`（执行清单）
- `src-tauri/config/policy.toml`（当前策略配置）
- `src-tauri/tests/policy_e2e_matrix.rs` + `tests/snapshots/policy_e2e_matrix.json`（可验证基线）

## 发布通过标准

- 门禁命令全绿；
- 快照回归无非预期漂移；
- 文档同步（15/16 与配置一致）；
- 交接团队可按清单在 30 分钟内完成一次完整验证。

