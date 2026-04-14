# Local Plugin Bridge 规范（Phase 2 草案）

本文定义 oclive 本地插件统一规范（spec-first），用于后续接入 `WASM` 与 `Native Process` provider。

## 1. 目标

- 统一本地 provider 的发现、注册与能力声明。
- 在运行时加载前完成版本门禁，避免旧 runtime 误读新规范。
- 与现有 `settings.json -> plugin_backends` 兼容；未启用本地 provider 时行为不变。

## 2. Provider 描述结构

本地 provider 必须提供描述对象（逻辑等价 JSON）：

```json
{
  "provider_id": "local.demo.memory",
  "schema_version": 1,
  "min_runtime_version": "0.2.0",
  "capabilities": ["memory", "prompt"]
}
```

字段约束：

- `provider_id`：非空字符串，建议全局唯一（如 `local.vendor.feature`）。
- `schema_version`：本地桥接规范版本，当前仅支持 `1`。
- `min_runtime_version`：可选，**语义化版本**（与角色包 `manifest.min_runtime_version` 相同规则，如 `0.2.0`）；非法字符串或宿主版本低于要求时拒绝注册。
- `capabilities`：可选数组，取值：`memory` / `emotion` / `event` / `prompt` / `llm`（JSON 中为 snake_case，与宿主 `serde` 一致）。

## 3. 版本门禁规则

- `schema_version == 0`：非法，拒绝。
- `schema_version > 1`：当前 runtime 拒绝加载（提示需升级 oclive）。
- `min_runtime_version` 不满足：拒绝加载（与 manifest 同语义）。
- 任一门禁失败时：该 provider 不进入注册表，不影响主对话链路。

## 3.1 宿主版本与错误文案

- 门禁比较使用宿主程序版本（与 `CARGO_PKG_VERSION` 一致）。
- 失败时错误信息区分 **角色包 manifest** 与 **本地插件描述**（实现上本地路径使用 `validate_min_runtime_version_for_local_plugin`，避免与角色包加载报错混淆）。

## 4. Runtime 行为（当前阶段）

- Runtime 提供 `LocalPluginBridge` 抽象接口与 `LocalPluginRegistry` 注册表。
- 注册表仅做：
  - 版本校验（`schema_version` / `min_runtime_version`）
  - capability 索引（按模块查询可用 provider）
- 当前阶段不直接把本地 provider 绑定到 `send_message` 主链路；后续在 provider 实现落地后接入。

## 4.1 文件清单发现（已实现：`file_manifest`）

- 目录：`<roles 根目录>/_local_plugins/`（与 `RoleStorage` 使用的 `roles_dir` 一致；开发时多为仓库内 `roles/_local_plugins/`）。
- 扫描该目录下扩展名为 `.json`（大小写不敏感）的文件；每个文件反序列化为一个 `LocalPluginProviderDescriptor`。
- 解析失败或读文件失败：跳过该文件并打 `oclive_plugin` 警告日志，不阻塞启动。
- 启动时由宿主将发现结果依次 `register_provider`；**同一 `provider_id` 若多次出现，后注册的覆盖先注册的**（目录遍历顺序依赖平台，**请勿依赖覆盖顺序**，每个 id 建议只配置一次）。

## 5. 与现有后端体系关系

- 现有 `BackendRegistry + PluginResolver` 保持稳定。
- 本地 provider 先以“注册骨架”接入 registry（不改变默认解析）。
- 未来会在 resolver 层引入 provider 选择策略，并保持 remote/builtin 回退语义。

## 5.1 与 `plugin_backends.memory`

- 当角色包或会话覆盖将 `plugin_backends.memory` 设为 **`local`** 时，宿主从 `LocalPluginRegistry` 中选取具备 `memory` 能力的 provider。
- 可选字段 **`plugin_backends.local_memory_provider_id`**（与 `memory` 同级）：非空时精确匹配已注册 `provider_id`；未命中则回退字典序并打 `warn`。
- 未指定 `local_memory_provider_id` 且存在多个 memory provider 时：按 **`provider_id` 字典序取第一个**，并打歧义 `warn`（建议在角色包中写明 id）。
- **当前实现**：记忆排序仍委托 **`builtin_v2`**，便于在无 WASM/进程侧车前保持对话一致；宿主侧 `MemoryRetrieval::diagnostic_local_provider_id` 可观测选中 id。
- **会话级**：与 `SendMessageRequest.session_id` 对齐时，可用 Tauri **`set_session_plugin_backend`**（`module = memory` 且可选 **`local_memory_provider_id`**）写入覆盖；用 **`get_role_info`** 的同名 **`session_id`** 读回合并后的 `plugin_backends_effective`（详见 [PLUGIN_V1.md](PLUGIN_V1.md)「会话级覆盖」）。

