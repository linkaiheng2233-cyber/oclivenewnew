# 内置记忆排序 — 目录插件形态（Kernel V2 阶段 5-1）

将 **`MemoryRetrieval::rank_memories`** 中与 `BuiltinMemoryRetrievalV2` 等价的逻辑，以 **Node JSON-RPC 侧车** 提供，供 `plugin_backends.memory = directory` 使用。

## 与进程内 `default-memory-providers` 的关系

- 官方 **`full`** profile 仍通过 **`default-memory-providers`** 链接设施 crate **`oclive_memory_builtin`**（进程内 Builtin），无需安装本插件即可使用 `memory = builtin` / `builtin_v2`。
- 若构建时 **关闭** `default-memory-providers`，宿主 **无** 进程内 Builtin；此时可将角色包 `memory` 设为 **`directory`**，并把本插件 id 填入 `directory_plugins.memory`，以 **子进程 RPC** 恢复排序能力（需授予 **`process:spawn`**）。

## 安装

1. 将本目录复制到与 `roles` 同级的 `plugins/com.oclive.builtin.memory/`，或把父目录加入 `oclive_host_plugins.json` 的 `extra_plugin_roots`（见 `DIRECTORY_PLUGINS.md`）。
2. 在角色 `settings.json` 的 `plugin_backends` 中设置 `memory: "directory"`，并配置 `directory_plugins.memory: "com.oclive.builtin.memory"`。
3. 本机需 **Node.js 18+**（`node` 在 PATH）。

## 协议

- 实现 **`memory.rank`**，参数与宿主 `RemoteMemoryRetrievalHttp` 一致（`memories`、`user_query`、`scene_id`、`limit`），返回 `ordered_ids: string[]`。
- 算法与 Rust `oclive_memory_builtin` 的 **V2** 一致；`user_query` 为空时行为与 **V1**（纯 importance×weight）一致。

## 市场 / 打包

- `manifest.json` 符合目录插件扫描约定；若上架 `.oclive-plugin`，请沿用本目录结构并补全市场元数据（见 `PLUGIN_V1.md`）。
