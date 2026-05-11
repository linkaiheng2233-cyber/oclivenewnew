# 校企玩偶场景 · 轻量 Linux 内核交付

面向 **资源受限的嵌入式 Linux**（小芯片）：**本地 GGUF + llama-server** 为主路径，**可选**切到 **云端 JSON-RPC LLM 侧车**。与主仓契约对齐：`plugin_backends`、`OCLIVE_*` 环境变量、目录插件 `com.oclive.llama.local`。

## 1. 角色 `settings.json` 模板

复制并按产品改名合并：

- 文件：**`settings.doll-linux.template.json`**（本目录）
- 契约说明：**`creator-docs/plugin-and-architecture/PLUGIN_V1.md`**、**`LLAMA_DIRECTORY_PLUGIN_V1.md`**

要点：

- **`llm: "directory"`** 且 **`directory_plugins.llm": "com.oclive.llama.local"`** 才会走官方本地 Llama 目录插件（侧车进程内启动 `llama-server`）。
- **`event": "none"`**、**`agent": "none"`** 与下文「关 default-event / kernel-agent」的裁剪内核一致。

### 1.1 切换到云端 LLM（可选）

1. 将 **`plugin_backends.llm`** 改为 **`"remote"`**。
2. 删除或清空 **`directory_plugins.llm`**（避免与 remote 槽位冲突；专家图切云时宿主也会清理该槽，见运行时 `force_clear_directory_llm_slot` 语义）。
3. 启动前设置 **`OCLIVE_REMOTE_LLM_URL`**（JSON-RPC 端点，见 **`creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md`**）。
4. **注意**：`remote` 路径依赖内核内的 **HTTP LLM 客户端**；若使用本交付的「轻量」编译组合（**关闭 `default-llm-providers`**），则 **`llm = remote` 无法在进程内装配**，需改为 **仍开启 `default-llm-providers` 的构建**，或自建 **`llm = directory`** 的侧车实现 OpenAI/自建网关。产品决策见下表：

| 构建组合 | `llm = directory`（本地 llama） | `llm = remote`（`OCLIVE_REMOTE_LLM_URL`） |
|----------|-----------------------------------|--------------------------------------------|
| **本 README 推荐轻量 crate**（`doll-linux-embedded` / 下节 runtime features） | ✅ 推荐 | ❌ 不支持（未编译内置 Remote LLM 客户端） |
| **桌面默认 `full`** | ✅ | ✅ |

若玩偶 **必须** 在「最小二进制」上仍支持云端：请在 **`oclive_kernel_runtime`** 上 **额外开启 `default-llm-providers`**（体积会增大），或单独跑一个 **LLM directory 侧车** 转发到云 API。

## 2. 编译：`oclive_kernel_runtime` 校验

与 DeepSeek 方案一致（**关闭** `default-llm-providers`、`kernel-agent`、`role-pack-zip`、`market-sync`、`facility-classic-algorithms`、`default-event-providers`、`default-agent-providers`）：

```bash
cargo check -p oclive_kernel_runtime --no-default-features --features "kernel-http-api,lazy-init,default-memory-providers,default-emotion-providers,default-prompt-providers,default-complex-emotion-providers"
```

## 3. 编译：`oclive_kernel_server` 无头二进制

根目录 **`Cargo.toml`** 已为 **`oclive_kernel_server`** 增加聚合特性 **`doll-linux-embedded`**（转发到 `oclive_kernel_runtime` 的上述子特性）。**默认**仍与官方一致（`runtime-full` → `oclive_kernel_runtime/full`），不改变桌面与既有 Docker 构建。

玩偶镜像 / 交叉编译示例：

```bash
cargo build --release -p oclive_kernel_server --no-default-features --features doll-linux-embedded
```

产物：若未改写 `target-dir`，一般为 **`target/release/oclive_kernel_server`**（本仓可能将 `target` 指到仓库外，见根目录 **`.cargo/config.toml`**）。

## 4. 极简部署（Linux 小芯片）

### 4.1 准备 `llama-server`

- 从 **llama.cpp** 发行或自行交叉编译得到与设备 **ABI 一致** 的 **`llama-server`** 二进制。
- 将二进制放到目录插件包内：  
  **`plugins/com.oclive.llama.local/bin/llama-server`**（与 **`LLAMA_DIRECTORY_PLUGIN_V1.md`** 一致）。  
  若缺失：侧车仍可起，但推理会走 **stub** 占位，用于接线验证。

### 4.2 准备 GGUF

- 将 **`.gguf`** 放入 **`{OCLIVE_APP_DATA_DIR}/models/gguf/`**，或在  
  **`{OCLIVE_APP_DATA_DIR}/plugin-data/com.oclive.llama.local/config.json`**  
  配置 **`modelPath`**（绝对路径）及可选 **`llamaArgs`**。

### 4.3 插件包与角色目录

1. 部署官方 **`plugins/com.oclive.llama.local/`**（含 **`manifest.json`**、**`bin/oclive-llama-sidecar`**`，以及上节的 **`llama-server`**）。  
   - 侧车源码：**`src-tauri/sidecars/oclive-llama-sidecar/`**（可单独 `cargo build` 后拷贝到 `bin/`）。
2. 角色根目录：设置 **`OCLIVE_ROLES_DIR`**，其下 **`{role_id}/manifest.json`** + **`settings.json`**（使用本目录模板）。
3. SQLite 与数据目录：设置 **`OCLIVE_DB_PATH`**、**`OCLIVE_APP_DATA_DIR`**（生产建议 **`OCLIVE_REQUIRE_EXPLICIT_PATHS=1`**，见 **`docs/LINUX_KERNEL_DEPLOY.md`**）。

### 4.4 启动内核服务

```bash
export OCLIVE_ROLES_DIR="/opt/oclive/roles"
export OCLIVE_DB_PATH="/var/lib/oclive/app.db"
export OCLIVE_APP_DATA_DIR="/var/lib/oclive"
export OCLIVE_REQUIRE_EXPLICIT_PATHS=1
export OOCP_API_PORT=48888
./oclive_kernel_server
```

HTTP / WebSocket 入口与 **OOCP** 约定见 **`crates/oclive_kernel_server/README.md`** 与 **`creator-docs/oocp/OOCP_SPEC_v0_1.md`**。

## 5. 相关文档索引

- 玩偶场景总览：**`creator-docs/scenarios/DOLL_GUIDE.md`**
- 轻量特性矩阵：**`creator-docs/kernel/LIGHTWEIGHT_PROFILE.md`**
- Linux 运维：**`docs/LINUX_KERNEL_DEPLOY.md`**
