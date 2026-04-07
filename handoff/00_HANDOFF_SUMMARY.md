# oclivenewnew 交接摘要（1 页）

## 一句话

**WEEK3 首轮 + WEEK3-003 已完成**：`domain/chat_engine::process_message` + Repository（Memory / Favorability）+ `LlmClient` + `send_message`；**回合持久化**经 **`apply_chat_turn_atomic`**（记忆 / 好感 / `events` 等在同一事务路径内，见 `infrastructure/db.rs`）；**集成测试**（`tests/chat_integration.rs`、`tests/week3_004_api.rs` 等 + `MockLlmClient`）；与 `migrations/001_init.sql`、v3.8 对齐；**质量门禁**（fmt / clippy `--all-targets` `-D warnings` / `cargo test`）已通过。

**下一步建议**：虚拟滚动、覆盖率与 e2e 见 **`19_RELEASE_CHECKLIST.md` §5、§8**；创作者文档与 `settings.json` 规范可对齐 `roles/README_MANIFEST.md`。

---

## API 与版本（0.2.0）

- **版本**：`package.json` / `tauri.conf.json` / `Cargo.toml` 为 **0.2.0**；变更说明见根目录 **`CHANGELOG.md`**。
- **API 增强**：`send_message` 响应新增 **`bot_emotion`**（角色本回合情绪标签）；`emotion` 仍为**用户**侧七维 DTO。详情见 **`20_SESSION_OPTIMIZATION_REPORT.md`**。

---

## 未完成项（规划）

- **虚拟滚动**、**测试覆盖率目标**、**CI/e2e**、**签名与自动更新**等：详见 **`19_RELEASE_CHECKLIST.md`** §5、§8。

---

## 现状快照

| 维度 | 状态 |
|------|------|
| 三件事原则（编排 / SQLx+Repository / `lib.rs` 注册） | ✅ |
| `chat_engine` | ✅ |
| Repository trait + SQLite 实现 | ✅ |
| `send_message` | ✅ |
| 单元测试 | ✅（数量以 `cargo test` 输出为准） |
| 集成测试 | ✅（`tests/chat_integration.rs`） |
| 事件写入 `events` 表 | ✅（经 `apply_chat_turn_atomic` 等路径，见 `db.rs`） |
| 其他 Tauri 命令 | ✅（多命令；真源 **`src-tauri/src/lib.rs`** `generate_handler`） |
| 前端 `tauri-api` 封装 | ✅（持续演进；与 invoke 名逐项对齐） |

---

## 与代码必须一致的事实

- **Tauri 版本**：`Cargo.toml` 为 **Tauri 1.5**（不是 2.x，除非你们已升级）。
- **`send_message` 响应字段**：`reply`（不是 `response`）；`emotion` 为 **`EmotionDto`（用户输入侧七维）**；**`bot_emotion` 为字符串（角色回复情绪标签）**；另有 `api_version`、`schema`、`events`、`favorability_current` 等。见 `src-tauri/src/models/dto.rs` 与 `CHANGELOG.md`。
- **调用方式**：通过前端 **`invoke('send_message', payload)`**，**没有**默认的 `http://localhost:5173/api/send_message` REST 接口（除非你们单独加 HTTP 服务）。
- **好感度**：编排层用 **`EventDetector::get_impact_factor`** 缩放增量，**无**独立 `FavorabilityEngine::calculate_delta` 模块。
- **性格演化**：**`PersonalityEngine::adjust_by_user_emotion` + `evolve_by_event`**，不是名为 `evolve` 的单一入口。

---

## 文档阅读顺序

1. 本文件（2 分钟）  
2. **`05_4.6_对接报告.md`（进度快照 + 代码锚点，可选速查）**  
3. **`04_4.6_PROJECT_TRUTH_CHECKLIST.md`（与 Cursor 对齐项目认知，接手必看）**  
4. `01_DEVELOPMENT_REPORT.md`（已完成工作与限制）  
5. `02_DEVELOPMENT_PLAN_v3.8.md`（后续任务与禁止项）  
6. `03_DEVELOPMENT_STANDARDS.md`（规范 + **对外部长文档的勘误**）

---

## 本地验证（后端）

```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --tests
cargo tauri dev
```

Ollama：`OLLAMA_BASE_URL`（默认 `http://localhost:11434`）、`OLLAMA_MODEL`（默认 `llama3.2`）。

角色包：`./roles/<role_id>/`（进程工作目录通常为仓库根目录）。
