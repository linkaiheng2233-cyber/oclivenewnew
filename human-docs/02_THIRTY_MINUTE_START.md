# 02 · 三十分钟跑通主仓

> **读者**：已 clone 本仓、要第一次启动桌面客户端的工程师。  
> **读完能做什么**：安装依赖、启动 `tauri:dev`、跑通 `npm run check` 日常门禁。  
> **耗时**：约 30 分钟（含首次 Rust 编译）。  
> **下一篇**：[03 术语表](03_GLOSSARY.md) + [04 工程约束](04_ENGINEERING_RULES.md)。

---

## 前置

| 项 | 要求 |
|----|------|
| **Node.js** | 18+、`npm` |
| **Rust** | stable toolchain |
| **Windows** | **Visual Studio Build Tools**（MSVC 链接器） |
| **Ollama** | 可选；未安装时对话可能失败，但 **编译与 `npm run check` 不依赖** |

**Cargo 产物目录**：根目录 [`.cargo/config.toml`](../.cargo/config.toml) 将 `target-dir` 指到仓库外 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`（与源码分离，便于清理）。

---

## 三命令（核心路径）

在仓库根目录 `oclivenewnew/`：

```bash
npm install
npm run tauri:dev
npm run check
```

| 命令 | 作用 |
|------|------|
| `npm install` | 前端依赖；首次 `tauri:dev` 会驱动 `src-tauri` 构建 |
| `npm run tauri:dev` | 桌面客户端 + 热重载 |
| `npm run check` | 日常门禁：`vite build` + `cargo fmt` / `clippy` / `cargo test --lib` |

---

## 验证分级

| 级别 | 命令 | 何时跑 |
|------|------|--------|
| **日常** | `npm run check` | 每次 PR 前 |
| **发版 / 改引擎** | `npm run check:release` | 触及编排、持久化、HTTP 契约 |
| **仅 Rust** | `cargo test --workspace` | 只改 `kernel/crates/*` |
| **仅前端** | `npm run test:unit` | 只改 `distros/` 前端（`check:release` 已含；Playwright 仍仅 CI Ubuntu） |
| **可选 Ollama** | 启动 Ollama 后在应用内对话一轮 | 验证端到端 LLM（非编译必需） |

完整表：[CONTRIBUTING.md §测试要求](../CONTRIBUTING.md#测试要求合并前建议全绿)

---

## 本地 HTTP API（可选）

与 GUI 同一二进制，加 `--api` 启动无头 HTTP（默认 `:8420`）。见根 [README.md](../README.md)「本地 HTTP API」节。CI 烟测常用 `OCLIVE_HTTP_API_MOCK_LLM=1`。

---

## 常见问题

| 症状 | 处理 |
|------|------|
| Windows 链接错误 | 安装 VS Build Tools，选「使用 C++ 的桌面开发」 |
| 首次编译很慢 | 正常；产物在仓库外 `target-dir` |
| `tauri:dev` 找不到 `dist/` | 先 `npm run build` 或让 dev 脚本完成首轮构建 |
| 对话失败、编译成功 | 安装 Ollama 或见 [ERROR_CODES](../creator-docs/getting-started/ERROR_CODES.md) |

---

## 验收

- [ ] `npm run tauri:dev` 能打开桌面窗口（或构建无报错）
- [ ] `npm run check` 全绿
- [ ] 知道发版前还要跑 `npm run check:release`

---

## 深度链接

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [USER_MANUAL](../creator-docs/getting-started/USER_MANUAL.md)（用户向安装）
- [CONFIGURATION_FILES](../creator-docs/guides/CONFIGURATION_FILES.md)（`app.db` 路径）
