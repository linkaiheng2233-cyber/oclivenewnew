# 本地 Llama 目录插件 v1（com.oclive.llama.local）

本插件提供一个“官方附带但可替换”的本地 LLM 后端：

- 角色/会话后端：`plugin_backends.llm = directory`
- 槽位指定：`plugin_backends.directory_plugins.llm = "com.oclive.llama.local"`
- 宿主调用：HTTP JSON-RPC 2.0 的 `llm.generate` / `llm.generate_tag`

该插件的侧车进程会在需要时启动 `llama-server` 并转发推理请求；模型权重 **不随官方分发**（用户自行放置或提供下载 URL + sha256）。

---

## 1. 目录结构（随发行版附带）

```text
plugins/
  com.oclive.llama.local/
    manifest.json
    bin/
      oclive-llama-sidecar(.exe)
      llama-server(.exe)          # 可选：随发行版提供；缺失则降级为 stub
```

---

## 2. 一键启用（UI）

在 **插件与后端管理（Ctrl+Shift+F） → 后端** 页面：

- 填 `插件 ID = com.oclive.llama.local`
- 点击「一键启用」
- 首次会提示并授予权限：`process:spawn`（允许宿主启动该目录插件子进程）

启用后，本会话的 LLM 将切到 Directory，并写入：

- `llm = directory`
- `directory_plugins.llm = com.oclive.llama.local`

若需要回退：点击同页的「回滚上次覆盖」或在会话覆盖面板把 LLM 改回 `ollama`。

---

## 3. 模型目录与配置（v1）

### 3.1 模型目录

插件默认扫描：

- `{app_data}/models/gguf/` 下的 `*.gguf`

其中 `{app_data}` 与 `app.db` 同级（见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)）。

### 3.2 私有配置（plugin-data）

配置文件由宿主维护：

- `{app_data}/plugin-data/com.oclive.llama.local/config.json`

可在插件管理面板的插件详情中通过 **uiSchema 表单**编辑，核心字段：

- `modelPath`：GGUF 绝对路径（推荐）
- `llamaArgs`：传给 `llama-server` 的额外参数（空格分隔）

保存后宿主会向侧车发送 `config_updated({ config })`，侧车将停止当前 `llama-server`，下次推理时按新配置重启。

---

## 4. 可选：URL + sha256 下载（官方不内置链接）

侧车提供 `llama.download_model({ url, sha256, fileName? })`：

- 下载到 `{app_data}/models/gguf/`
- 写入临时 `.part` 文件
- 下载完成后校验 sha256（十六进制小写）
- 校验通过后原子替换为目标文件

该能力需要额外权限：`network:*`（计划在 UI 上显式提示并授权）。

---

## 5. JSON-RPC 方法（v1）

### 5.1 LLM（宿主会调用）

- `llm.generate({ model, prompt }) -> { text }`
- `llm.generate_tag({ model, prompt }) -> { text }`

### 5.2 侧车管理（调试/运维）

- `llama.status() -> { running, base_url }`
- `llama.stop() -> { ok }`
- `llama.list_models() -> { items: [{ name, path }] }`
- `llama.set_model({ modelPath }) -> { ok }`
- `llama.download_model({ url, sha256, fileName? }) -> { ok, path }`
- `config_updated({ config }) -> { ok }`

---

## 6. Windows 端到端验证（最小闭环）

1. 确认 `plugins/com.oclive.llama.local/` 随发行版存在，且 `bin/oclive-llama-sidecar.exe` 可被启动  
2. 在 UI 一键启用（会话覆盖）并授予 `process:spawn`  
3. 发送一条消息，观察对话走 Directory LLM：  
   - 未配置模型或未提供 `llama-server` 时：仍可返回 stub 文本（用于验证接线）  
   - 配置 `modelPath` 且提供 `llama-server` 后：应由 llama-server 实际生成

