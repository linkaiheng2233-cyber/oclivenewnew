# RFC：内核与发行版目录解耦（Monorepo 重组）

> **状态**：已执行（Phase 0–5）  
> **品牌**：对外仍称 **OCLive / oclivenewnew**；物理目录分离为 `kernel/` + `distros/`。  
> **不在本 RFC**：从 `oclive_kernel_host` 剥离 `domain/theater/*`（HTTP `/theater/scene` 仍在内核）；仓库改名。

---

## 1. 目标

1. **叙事清晰**：新人 1 分钟内能答——内核在哪、Chat Pro 在哪、剧场在哪。
2. **编译边界**：删除 `distros/chat-pro/` 仍可 `cargo run -p oclive_kernel_server -- --api`；删除 `kernel/` 则整个平台不存在。
3. **姊妹仓预留**：对齐 `oclive-vscode` 模式，未来可拆 `oclive-chat-pro` 独立仓（见 [附录 A](#附录-a-姊妹仓-oclive-chat-pro-拆分决策门)）。

---

## 2. 终态目录树

```text
oclivenewnew/
├── README.md
├── Cargo.toml              # workspace：kernel crates + desktop-tauri
├── package.json            # npm workspaces 根
├── creator-docs/           # 平台契约（暂留根）
├── handoff/
│   └── distros/            # 发行版 handoff 索引
├── kernel/
│   ├── kernel/crates/             # 12 个 Rust crate
│   ├── kernel/fuzz/
│   ├── data/plugins.json
│   └── examples/             # 内核向示例（oocp、distro-profiles 等）
└── distros/
    ├── shared/               # @oclive/desktop-shared
    ├── chat-pro/             # Chat Pro 前端 + roles + plugins
    ├── theater/              # AI Theater 前端
    └── desktop-tauri/        # 共享 Tauri 宿主（原 src-tauri）
```

---

## 3. 资产归属表

| 资产 | 位置 |
|------|------|
| `kernel/crates/*`, `fuzz`, `oclive-cli` | `kernel/` |
| `src/api`, `stores`, 通用组件/composables | `distros/shared/` |
| ToolShell / FluentShell | `distros/chat-pro/src/` |
| TheaterShell / `composables/theater` | `distros/theater/src/` |
| `src-tauri`, bundled kernel | `distros/desktop-tauri/` |
| `distros/chat-pro/roles/`, `distros/chat-pro/plugins/`, `distros/chat-pro/e2e/` | `distros/chat-pro/` |
| `creator-docs/`, `handoff/` | **根**（索引标明 kernel vs distro） |

---

## 4. 关键设计

### 4.1 单一 Tauri 宿主 + 双 Vite 入口

`distros/desktop-tauri` 仍是一个 Tauri 工程；`beforeDevCommand` / `beforeBuildCommand` 根据 `OCLIVE_TAURI_SHELL=chat-pro|theater` 在对应 Vite 根启 dev/build。

- Chat Pro：`VITE_OCLIVE_SHELL` 空或 `tool`（默认 ToolShell）。
- Theater：`VITE_OCLIVE_SHELL=theater`（独立 `App.vue`，仅 TheaterShell）。

### 4.2 npm workspaces

```json
"workspaces": ["distros/shared", "distros/chat-pro", "distros/theater"]
```

根脚本保留：`npm run tauri:dev` → chat-pro；`npm run tauri:dev:theater` → theater。

### 4.3 Cargo workspace

```toml
members = [
  "kernel/crates/oclive_kernel_host",
  # ...
  "distros/desktop-tauri",
  "kernel/fuzz",
]
```

`distros/desktop-tauri/Cargo.toml`：

```toml
oclive_kernel_host = { path = "../../kernel/crates/oclive_kernel_host", features = ["tauri-commands"] }
```

`.cargo/config.toml` **保留在仓库根**（单一 `target-dir`）。

### 4.4 「删文件夹」语义

| 操作 | 效果 |
|------|------|
| 删除 `distros/chat-pro/` | 无法构建 Chat Pro；内核 HTTP 仍可用 |
| 删除 `distros/theater/` | 无法构建剧场包；Chat Pro 不受影响 |
| 删除 `distros/desktop-tauri/` | 无桌面壳；内核 HTTP 仍可用 |
| 删除 `kernel/` | 整个平台不存在 |

**不是**运行时热插拔；是 **monorepo 编译与打包边界**。

---

## 5. 分阶段执行记录

| Phase | 内容 | 验收 |
|-------|------|------|
| 0 | 本 RFC + stale-paths 清单 + 巡检手册前置区 | 零行为变更 |
| 1 | `git mv` kernel/crates/fuzz → `kernel/`；Cargo/脚本 path | `cargo test -p oclive_kernel_host --lib`；`dimension5 --ci` |
| 2 | `distros/shared` + chat-pro/theater 前端拆分 | 双 `vite build`；`test:theater:smoke` |
| 3 | `src-tauri` → `distros/desktop-tauri`；双入口打包 | `tauri:dev` / `tauri:dev:theater` |
| 4 | README / AGENTS / CI / CHANGELOG | `check:ci-local` |
| 5 | 附录 A 决策门（文档 only） | RFC 合入 |

---

## 6. 回滚策略

- 每 Phase 独立 PR；失败则 `git revert` 该 PR，不跨 Phase 回滚。
- Phase 1–3 期间剧场功能改动在 Phase 3 前冻结；`test:theater:smoke` 每 PR 必跑。
- 路径 ratchet：`scripts/check-stale-paths.mjs` 扩展禁止根 `kernel/crates/`、`distros/desktop-tauri/` 回潮。

---

## 附录 A：姊妹仓 `oclive-chat-pro` 拆分决策门

> **Phase 5 · 仅文档 · 本轮不执行（Deferred）**  
> **状态（2026-06-24）**：三道门均未过 → **`oclive-chat-pro` 拆仓本轮不执行**。  
> - **P0-STRANGER**（剧场 5 人陌生人测试）：**OPEN** — 见 [`handoff/theater/PLAYTEST_MATRIX.md`](../theater/PLAYTEST_MATRIX.md)  
> - **Chat Pro 可传播 demo**：未交付成片  
> - **内核 `kernel-v0.x` tag**：未稳定发版节奏  
> 维持 monorepo `kernel/` + `distros/` 物理分层；文档路径收尾见 [`STALE_PATHS_MIGRATION_CHECKLIST.md`](STALE_PATHS_MIGRATION_CHECKLIST.md) §5–§6。

### 何时拆独立仓

满足**全部**条件后再决策：

1. **Theater 陌生人测试通过**（60 秒「卧槽」体验；见 `handoff/theater/PLAYTEST_MATRIX.md`）。
2. **Chat Pro 至少一条可传播 demo**（stream UI 或首启 onboarding 成片）。
3. **内核发布节奏稳定**：`oclive_kernel_server` 可打 `kernel-v0.x` tag 供下游 pin。

### 推荐拆法

- **迁出**：`distros/{shared,chat-pro,theater,desktop-tauri}` 整体 → 新仓 `oclive-chat-pro`；或先迁 `chat-pro + desktop-tauri + shared`，theater 随后。
- **依赖**：
  - 开发期 monorepo：`path = "../../oclivenewnew/kernel/crates/..."`
  - 发版：`oclive_kernel_* = { git = "https://github.com/.../oclivenewnew", tag = "kernel-v0.x", package = "..." }`
- **保留**：`oclivenewnew` 叙事收敛为 **kernel-only**（类比 Linux 内核仓）；`creator-docs/` 契约仍可在内核仓或双仓同步。

### 不拆的信号

- Theater 仍在陌生人测试窗口；
- 内核 API 每周 breaking；
- CI 无法在姊妹仓独立绿（缺 golden / OOCP 镜像）。

### 决策记录模板

```markdown
## Decision: oclive-chat-pro split
- Date:
- Gate 1 Theater playtest: pass / fail
- Gate 2 Chat Pro demo: link
- Gate 3 kernel tag: kernel-vX.Y.Z
- Outcome: split / defer / partial (shared+chat-pro only)
```
