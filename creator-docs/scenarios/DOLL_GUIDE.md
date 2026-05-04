# 场景化建设指南：AI 情感陪伴玩偶（接入 oclive 内核）

本文面向 **校企合作项目 / 硬件方工程师**：在树莓派或同类嵌入式 Linux 上，把 **oclive 内核** 作为玩偶的「对话与记忆中枢」，跑通 **可对话的原型**。命令与路径均可直接复制后按需替换绝对路径。

**一句话价值**：oclive 为玩偶提供带 **长期记忆、情绪与复杂情感管线、可定制人格与场景** 的 **「AI 灵魂」**；硬件负责 **拾音、播音与动作**，内核负责 **理解与编排**（含可选 **Agent + MCP 技能侧车** 扩展物理能力）。

**文档锚点**（契约与深度说明）：

- 角色包：`roles/README_MANIFEST.md`、`roles/manifest.template.json`、`roles/settings.template.json`
- 无头服务：`crates/oclive_kernel_server/README.md`、`examples/kernel_remote_simple/README.md`
- 插件与侧车：`creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md`、`REMOTE_PLUGIN_PROTOCOL.md`
- 第九模块（专家模型设施）：`creator-docs/kernel/MODULE_9_EXPERT_MODELS_FACILITY.md`
- LLM 采样环境变量：见下文「进阶调参」与源码 `crates/oclive_kernel_runtime/src/infrastructure/llm_params.rs`

---

## 1. 场景定位与硬件要求

### 1.1 推荐硬件清单（最简可演示）

| 组件 | 说明 |
|------|------|
| 主控 | **树莓派 4/5** 或同等 **64 位 ARM / x86_64 Linux**（内存建议 **≥ 4GB**，本地跑 7B 级模型更从容） |
| 麦克风 | USB 麦克风或 I2S 麦克风板 |
| 扬声器 | 3.5mm / USB / I2S 功放 + 喇叭 |
| 存储 | SD / NVMe 足够存放系统、模型与 SQLite 数据库 |
| 网络 | 可选；本地 **Ollama** 可离线，云端 API 则需出站 HTTPS |

### 1.2 软件依赖与一次性初始化

以下示例以 **Debian / Ubuntu 系** 为准（树莓派 OS 亦适用）。

**1）系统工具与编译依赖（用于从源码构建 `oclive_kernel_server`）**

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config git curl \
  libssl-dev clang cmake libwebkit2gtk-4.0-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

说明：与主仓库 CI 在 Linux 上构建 Tauri/内核链路的依赖 **对齐**，可避免链接期缺库；若你仅使用 **预编译的 `oclive_kernel_server` 二进制**，可酌情删减 GUI 相关包，但需自行保证 `libssl` 等与二进制匹配。

**2）Rust（stable）**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc --version
```

**3）Ollama（本地推理，与示例角色默认一致）**

按 [Ollama 官方安装说明](https://ollama.com/) 安装后：

```bash
sudo systemctl enable --now ollama
ollama pull qwen2.5:7b
```

示例包 `roles/mumu` 的 `settings.json` 中默认 `model` 为 **`qwen2.5:7b`**，与上述命令一致。

**4）可选：Node.js 18+ 或 Python 3.10+（跑示例客户端 / 自写语音桥接脚本）**

```bash
# Node（示例：使用 NodeSource 仓库，请按官方文档更新版本号）
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
node --version
```

```bash
python3 --version
```

**5）克隆 oclive 主仓库并构建无头内核**

```bash
cd /opt
sudo mkdir -p oclive && sudo chown "$USER:$USER" oclive
cd /opt/oclive
git clone https://github.com/oclive-app/oclivenewnew.git src
cd src
cargo build --release -p oclive_kernel_server
```

构建产物路径（默认 cargo target 若未改写）：

```bash
ls -la target/release/oclive_kernel_server
```

若你使用仓库外 `target-dir`（见根目录 `.cargo/config.toml`），请到该目录下查找同名二进制。

---

## 2. 角色包定制与配置

内核通过环境变量 **`OCLIVE_ROLES_DIR`** 指向 **角色根目录**（其下为 `角色id/manifest.json`）。下面以 **复制官方示例「沐沐（mumu）」** 为起点，改造成 **儿童向陪伴玩偶**。

### 2.1 复制示例包并注册新角色 id

```bash
export OCLIVE_ROLES_DIR="/opt/oclive/roles"
mkdir -p "$OCLIVE_ROLES_DIR"
cp -a /opt/oclive/src/roles/mumu "$OCLIVE_ROLES_DIR/doll_friend"
```

编辑 **`$OCLIVE_ROLES_DIR/doll_friend/manifest.json`**：

- 将 **`id`** 改为 **`doll_friend`**（与目录名一致）。
- 将 **`name` / `description`** 改为产品对外名称与简介。
- 在 **`user_relations`** 中为「**家人 / 监护人 / 孩子**」增补或改写 **`prompt_hint`**，明确要求：**用语简短、温柔、积极，避免恐吓与说教，不输出不适宜儿童的内容**。
- 将 **`default_relation`** 改为 **`family`**（若你希望默认按「家人与孩子」关系开场）。

关系与 manifest 字段语义见 **`roles/README_MANIFEST.md`**。

### 2.2 调整场景文案（示例：「家」场景）

编辑 **`$OCLIVE_ROLES_DIR/doll_friend/scenes/home/description.txt`**，写入与玩偶人设一致的 **场景叙述**（例如：温馨儿童房、安抚型陪伴等）。其它场景同理修改 `scenes/<scene_id>/description.txt` 与 `scene.json`。

### 2.3 使用 `settings.json` 注入「质量锚点」与引擎参数

在 **`$OCLIVE_ROLES_DIR/doll_friend/settings.json`** 中确保存在 **`schema_version`** 与 **`model`**；并增加（或合并） **`reply_quality_anchor`** 字段（模板说明见 **`roles/settings.template.json` 顶部的 `_oclive_creator_guide`**）：

```json
{
  "schema_version": 1,
  "model": "qwen2.5:7b",
  "reply_quality_anchor": "你是孩子的陪伴玩偶，回答要短、温柔、积极；用孩子能听懂的词；不要恐吓、不要说教；不确定时鼓励孩子找家长。",
  "identity_binding": "global",
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "ollama"
  }
}
```

保存后，**重新加载角色**（重启 `oclive_kernel_server` 或重新 `POST /chat` 指定新 `role_path`）即可生效。

### 2.4 第九模块（专家模型设施）：「人格配方」与 LoRA

**第九模块**管理 **ExpertGraph**（基座 GGUF、多 **LoRA**、可选 **Prompt 风格节点**），与 `plugin_backends` 六槽 **不同构**；边界见 **`creator-docs/kernel/MODULE_9_EXPERT_MODELS_FACILITY.md`**。

图结构在源码中的 JSON 形状见 **`crates/oclive_kernel_runtime/src/models/expert_models.rs`**（节点类型 `base_model` / `lora_adapter` / `prompt_style`）。下面是一份 **示意配置**（**路径必须改为你设备上真实存在的绝对路径**）：

```json
{
  "version": 1,
  "nodes": [
    {
      "type": "base_model",
      "id": "base",
      "gguf_path": "/opt/oclive/models/base.Q4_K_M.gguf"
    },
    {
      "type": "lora_adapter",
      "id": "shy_doll_lora",
      "gguf_path": "/opt/oclive/models/shy_doll_lora.gguf",
      "strength": 0.85,
      "enabled": true,
      "order": 1
    },
    {
      "type": "prompt_style",
      "id": "child_friendly",
      "style": {
        "replyQualityAnchor": "语气害羞、温柔，句子短，适合安抚小朋友。",
        "corePersonality": "胆小但善良的布偶，会认真听孩子说话。"
      }
    }
  ],
  "edges": [
    { "from": "base", "to": "shy_doll_lora" }
  ]
}
```

**落地说明（产线常见两条路径）**：

1. **桌面端 oclive**：在应用内 **专家模型 / Module 9** 相关界面写入角色默认或会话覆盖后，配置进入 **`role_runtime`** 持久化（见迁移 `018_expert_models.sql`）。玩偶量产后可 **复用同一份 SQLite 与模型文件布局**。  
2. **仅无头 `kernel_server`**：需你方在集成层调用与 Module 9 对应的 **内核 API / 管理流程**（以 `creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md` 与运行时 DTO 为准），或先在桌面端生成配置再 **拷贝数据库**。

若短期只想 **快速验证** 而不接本地 GGUF 管线：可继续用 **Ollama** 的单一 `model` 名；LoRA 可在 Ollama 侧用 **Modelfile** 预先合成自定义模型标签，再写入 `settings.json` 的 **`model`** 字段（与第九模块 **二选一或分阶段** 上线均可）。

---

## 3. 技能侧车接入（语音与 Agent）

「技能侧车」在 oclive 体系里通常指：**HTTP JSON-RPC Remote 插件**、**目录插件子进程**，或 **Agent 可调用的 MCP 工具服务**。玩偶场景下，建议拆成 **语音桥接** + **能力工具** 两层。

### 3.1 「基础语音对话侧车」思路（工程上可复制的最小闭环）

内核 **HTTP 试聊** 入参是 **文本**（`POST /chat`）。硬件侧车职责建议固定为：

1. **麦克风 → STT（语音转写）** → 得到字符串 `user_text`。  
2. 调用内核：`POST /chat`，body 含 `role_path`、`message: user_text` 等（与 **`examples/kernel_remote_simple/client.py`** 一致）。  
3. 取返回 JSON 的 **`reply`** 字段 → **TTS（文字转语音）** → 扬声器播放。

**示例：用 shell 伪代码串起流程（请将 STT/TTS 换为你选用的命令或 HTTP 服务）**

```bash
export KERNEL_URL="http://127.0.0.1:48888"
export ROLE_PATH="/opt/oclive/roles/doll_friend"

# 1) 录音（示例：arecord，按设备改 -D）
arecord -f cd -d 5 /tmp/doll_in.wav

# 2) STT：此处替换为你安装的离线/在线识别命令，输出一行 user_text
# user_text=$(your_stt /tmp/doll_in.wav)

# 3) 调内核（与仓库示例一致，需本机已安装 Python3）
user_text="妈妈不在家，我有点怕。"
python3 /opt/oclive/src/examples/kernel_remote_simple/client.py \
  --base-url "$KERNEL_URL" \
  --role-path "$ROLE_PATH" \
  --message "$user_text"

# 4) TTS：将上一步打印的 reply 转音频并播放（替换为你的 tts 命令）
# your_tts "$reply_wav" && aplay "$reply_wav"
```

完整前置条件与参数表见 **`examples/kernel_remote_simple/README.md`**。

### 3.2 通过 Agent + MCP 调用「物理能力」侧车（示例：播放摇篮曲）

**Agent（第七模块）** 可通过 **MCP** 调用外部工具（需 **`kernel-agent`** 能力与用户授权；`stdio` / `network` 出站需按产品安全策略授予）。MCP 清单目录：

```text
$OCLIVE_APP_DATA_DIR/mcp-servers/*.json
```

无头部署时务必设置 **`OCLIVE_APP_DATA_DIR`**（见 `crates/oclive_kernel_server/README.md` 与 `http_api` 中的约定），例如：

```bash
export OCLIVE_APP_DATA_DIR="/var/lib/oclive"
sudo mkdir -p "$OCLIVE_APP_DATA_DIR/mcp-servers"
sudo chown -R oclive:oclive /var/lib/oclive
```

**1）编写 MCP 服务清单 `doll_skills.json`**

HTTP 侧车需 **`transport": "http"`** 且提供 **`url`**（实现需响应内核发出的 **`list_tools` / `call_tool` 形态**，见 `crates/oclive_kernel_runtime/src/infrastructure/mcp_client.rs` 的 POST JSON 约定）：

```bash
cat <<'EOF' | sudo tee /var/lib/oclive/mcp-servers/doll_skills.json
{
  "id": "doll_hardware",
  "name": "Doll hardware bridge",
  "transport": "http",
  "url": "http://127.0.0.1:8766/mcp",
  "timeout_ms": 15000,
  "tools": [
    { "name": "play_lullaby", "description": "播放摇篮曲或安抚音频" },
    { "name": "led_calm", "description": "呼吸灯舒缓模式" }
  ]
}
EOF
```

**2）侧车 HTTP 服务（示意）**

你方可用任意语言实现 **`POST http://127.0.0.1:8766/mcp`**：收到 `{"tool":"play_lullaby","params":{...}}` 时在本机调用 `aplay` / `mpv` 播放指定文件，并返回 JSON `result`。生产环境请补 **鉴权、超时、音量上限与儿童安全策略**。

**3）角色侧启用 Agent 槽位**

在 **`settings.json`** 的 **`plugin_backends`** 中增加（若与模板字段一致）：

```json
"agent": "builtin"
```

目录插件版 Agent 侧车见 **`examples/oclive-agent-builtin-directory/README.md`**（`agent.process` JSON-RPC）。  
当模型在对话中触发 **工具调用**（如「孩子哭了，放一首摇篮曲」）时，由 **Agent 编排** 调用 MCP **`play_lullaby`**；具体是否触发取决于 **系统提示 / 工具 schema / 模型行为**，需在联调中迭代 **提示词与工具描述**。

**安全提示**：`stdio` 型 MCP 等价 **`process:spawn`**；HTTP 型需 **`network:*`** 授权。校企部署务必走 **显式授权 + 审计日志**，见仓库 `AGENTS.md` 内核约束说明。

---

## 4. 部署与高可用运行（systemd）

### 4.1 安装二进制与目录布局（示例）

```bash
sudo install -m 0755 /opt/oclive/src/target/release/oclive_kernel_server /usr/local/bin/oclive_kernel_server
sudo mkdir -p /etc/oclive
sudo tee /etc/oclive/env <<'EOF'
OCLIVE_ROLES_DIR=/opt/oclive/roles
OCLIVE_APP_DATA_DIR=/var/lib/oclive
OOCP_API_PORT=48888
# OCLIVE_DB_PATH=/var/lib/oclive/oclive.db
EOF
```

### 4.2 systemd 单元：`Restart=always`

```bash
sudo tee /etc/systemd/system/oclive-kernel.service <<'EOF'
[Unit]
Description=Oclive kernel server (HTTP/OOCP)
After=network-online.target ollama.service
Wants=network-online.target

[Service]
User=oclive
Group=oclive
EnvironmentFile=/etc/oclive/env
WorkingDirectory=/var/lib/oclive
ExecStart=/usr/local/bin/oclive_kernel_server
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

sudo useradd --system --home /var/lib/oclive --shell /usr/sbin/nologin oclive || true
sudo mkdir -p /var/lib/oclive
sudo chown -R oclive:oclive /var/lib/oclive

sudo systemctl daemon-reload
sudo systemctl enable --now oclive-kernel.service
```

### 4.3 开机自启与健康检查

```bash
sudo systemctl is-enabled oclive-kernel.service
sudo systemctl status oclive-kernel.service --no-pager
curl -sS http://127.0.0.1:48888/health
```

### 4.4 日志与 7×24 运维常用命令

```bash
sudo journalctl -u oclive-kernel.service -f
sudo journalctl -u oclive-kernel.service --since "1 hour ago" --no-pager
sudo journalctl -u oclive-kernel.service -p err -b --no-pager
```

容器化备选见 **`docker-compose.kernel-server.yml`** 与 **`Dockerfile.kernel-server`**（默认端口 **48888**）。

---

## 5. 进阶调参与社区联动

### 5.1 合作方可自行调整的关键参数（中文说明）

1. **主对话「创造性」——采样温度 / top-p**  
   通过环境变量覆盖（默认见 `llm_params.rs`）：  
   - **`OCLIVE_LLM_TEMPERATURE`**：主对话温度（默认 **0.8**），调高更接近发散、拟人「俏皮」，调低更稳、少跑题。  
   - **`OCLIVE_LLM_TOP_P`**：默认 **0.9**，与温度共同影响多样性。  
   标签类子任务另有 **`OCLIVE_LLM_TAG_TEMPERATURE`** / **`OCLIVE_LLM_TAG_TOP_P`**。

   在 systemd 的 `Environment=` 或 `/etc/oclive/env` 中追加一行即可，例如：

   ```bash
   echo 'OCLIVE_LLM_TEMPERATURE=0.65' | sudo tee -a /etc/oclive/env
   sudo systemctl restart oclive-kernel.service
   ```

2. **性格演化速度（避免「一轮对话人设剧变」）**  
   在角色包 **`settings.json`** 的 **`evolution`** 中调节 **`max_change_per_event`**、**`max_total_change`** 等（模板见 **`roles/settings.template.json`**）。

3. **记忆检索场景偏好**  
   在 **`memory_config.topic_weights`** 中提高「家庭 / 安抚 / 日常」等主题在特定 `scene` 下的权重，使玩偶 **更常记起与当前场景相关的安抚经历**（字段说明见 **`roles/README_MANIFEST.md`**）。

### 5.2 社区与反馈

欢迎在校企项目结束后，将 **角色包设计、侧车接口与硬件拍档方案** 分享到 **GitHub Discussions**（本仓库启用时）或你们院系共建的 **论坛 / 社区站**。产品层对「论坛 + 角色包 + 插件」三板块的设想见 **`creator-docs/roadmap/COMMUNITY_WEB_VISION.md`**，可与教研课题、开源贡献结合。

---

## 6. 验收检查清单（原型）

- [ ] `ollama pull` 的模型名与 **`settings.json` → `model`** 一致。  
- [ ] `curl http://127.0.0.1:48888/health` 返回 **`ok`**。  
- [ ] `python3 examples/kernel_remote_simple/client.py --role-path ...` 能打印 **`reply`**。  
- [ ] 角色 **`id` 与目录名一致**，且位于 **`OCLIVE_ROLES_DIR`** 下。  
- [ ] `systemctl status oclive-kernel` 为 **active**，异常时 **`journalctl`** 可见重启记录。  
- [ ] （可选） MCP 清单在 **`$OCLIVE_APP_DATA_DIR/mcp-servers/`** 且侧车 progress 授权与联调通过。

---

**祝合作顺利。** 若文档与最新契约不一致，请以仓库内 **`creator-docs/getting-started/DOCUMENTATION_INDEX.md`** 索引到的 **契约文档** 为准。
