# 开发者快速入门（从零到首次对话）

面向**第一次接触本仓库**的开发者：复制命令即可，无需事先了解 oclive 架构。预计 **15 分钟内**可读完并跑通桌面端首次对话。

更完整的文档索引见 **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)**；本页只覆盖最短路径。

---

## 1. 环境准备

### 必需工具

| 工具 | 用途 | 建议版本 |
|------|------|----------|
| **Git** | 克隆仓库 | 任意近期版本 |
| **Node.js** | 前端构建、Tauri CLI 脚本 | **18+** |
| **npm** | 安装 JS 依赖（本仓库以 `package-lock.json` 为准；亦可用 pnpm/yarn，命令等价替换） | 随 Node 自带 |
| **Rust（stable）** | `src-tauri` 与内核 crate | 当前 stable |
| **Ollama** | 本地对话默认走 Ollama API | 以 [ollama.com](https://ollama.com) 为准 |

**Windows**：还需安装 **Visual Studio Build Tools**（含 C++ 桌面开发工作负载），否则 Rust 链接阶段可能失败。

### 安装验证（复制执行）

```bash
git --version
node --version
npm --version
rustc --version
cargo --version
ollama --version
```

若某项报错，请先完成对应安装再往下。

---

## 2. 克隆与编译

### 克隆仓库

```bash
git clone https://github.com/oclive-app/oclivenewnew.git
cd oclivenewnew
```

（若使用 SSH，将 URL 换为你的 `git@github.com:...`。）

### 安装前端依赖

```bash
npm install
```

使用 **pnpm** 或 **yarn** 时，在本目录执行各自等价的 `install` 即可（例如 `pnpm install`）。

### 首次编译 Rust（可选预检）

在仓库**根目录**（与 `Cargo.toml` 同级）执行：

```bash
cargo check --workspace
```

仅检查桌面壳层时也可：

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

完整桌面安装包构建见下文「常用命令速查」中的 `npm run tauri:build`。

---

## 3. 首次运行（最简路径）

### 3.1 启动 Ollama 并准备模型

确保本机 **Ollama 服务已运行**（安装包通常含开机自启）。示例角色 **`roles/mumu/settings.json`** 中默认模型为 **`qwen2.5:7b`**，可先拉取：

```bash
ollama pull qwen2.5:7b
```

若你更想用自己已有的模型，见下文「切换模型」：把 `settings.json` 里的 `model` 改成你已 `ollama pull` 的名称即可。

### 3.2（推荐）指定角色包根目录

与根目录 [README.md](../../README.md) 一致：将 **`OCLIVE_ROLES_DIR`** 指向**角色根目录**（其下为多个 `角色id/` 文件夹，每个内有 `manifest.json`）。

**Linux / macOS（bash/zsh）：**

```bash
export OCLIVE_ROLES_DIR="/绝对路径/oclivenewnew/roles"
```

**Windows（PowerShell，按你的实际路径修改）：**

```powershell
$env:OCLIVE_ROLES_DIR = "D:\oclivenewnew\roles"
```

本仓库已自带示例包，例如 **`mumu`**（沐沐）、**`shimeng`**、**`枫侵月`**。未设置该变量时，开发构建也可能通过 Tauri 资源带上 `roles/`，但**显式设置可避免路径歧义**，推荐始终设置。

### 3.3 启动桌面开发版

在仓库根目录、且已执行过 `npm install`：

```bash
npm run tauri:dev
```

- 首次运行会编译 Rust，可能较慢。
- 前端开发服务器默认与 **`tauri.conf.json`** 中配置一致（通常为 **`http://localhost:1420`**）。

### 3.4 在应用内完成首次对话

1. 等待桌面窗口打开。  
2. 在界面中选择/加载一个角色（例如 **沐沐 `mumu`**）。  
3. 确认本机 **Ollama** 可用且已有所选模型。  
4. 在聊天框发送一句话，收到模型回复即表示端到端打通。

若报错与模型或网络相关，可先在同一台机器上执行 `ollama run qwen2.5:7b` 做独立验证。

---

## 4. 最小配置修改（高频）

### 切换角色包

- **开发期**：在 **`OCLIVE_ROLES_DIR`** 指向的根下，每个子目录即一个角色 id；应用内切换角色即可。  
- **从 zip 导入**：应用内支持导入角色包（见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)）。  
- **与编写器联调**：编写器导出包后解压到上述根目录，或配合独立仓库 **oclive-launcher** 安装 zip（见 [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) 快速入口）。

### 切换模型（Ollama）

编辑对应角色目录下的 **`settings.json`**，修改顶层 **`model`** 字段，例如：

```json
"model": "qwen2.5:7b"
```

改为本机已存在的 Ollama 模型名后，保存并**重新加载角色**（或重启应用）。勿使用未 `ollama pull` 的模型名。

### 全局默认模型（可选）

环境变量 **`OLLAMA_MODEL`** 可作为全局回退（与角色包配置的关系见仓库文档）；日常开发以角色 **`settings.json`** 为准最直观。

---

## 5. 常用命令速查

在仓库**根目录**执行：

| 命令 | 说明 |
|------|------|
| `npm run tauri:dev` | 启动 Tauri 开发模式（桌面 + 热更新前端） |
| `npm run dev` | 仅启动 Vite 前端（无桌面壳，调 UI 时用） |
| `npm run build` | 生产构建前端静态资源到 `dist/` |
| `npm run tauri:build` | 打桌面安装包（会先走前端构建与 Rust release） |
| `npm run check` | 日常检查：`vite build` + `cargo fmt` / `clippy` / `cargo test --lib`（见 `package.json`） |
| `npm run check:release` | 接近 CI 的完整检查（含全量 `cargo test`） |
| `cargo check --workspace` | 仅 Rust：工作区快速类型检查（根 `Cargo.toml` workspace） |
| `cargo test --workspace` | 仅 Rust：工作区全部测试 |

仅前端预览构建结果：

```bash
npm run build
npm run preview
```

---

## 6. 下一步

- **文档总索引与按主题深入**：[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)  
- **项目全貌、三件套分工**：[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)  
- **角色包目录与 `OCLIVE_ROLES_DIR`**：[CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md)  
- **根目录构建、CI、测试说明**：[README.md](../../README.md)  
- **内核与扩展点**：[../kernel/KERNEL_BOUNDARY.md](../kernel/KERNEL_BOUNDARY.md)、[../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)

欢迎在本指南基础上再读索引中的专题文档，按任务深挖即可。
