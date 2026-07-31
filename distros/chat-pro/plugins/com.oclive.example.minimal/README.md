# 最小目录插件示例（Directory Plugin Minimal）

演示：**manifest**、**整壳**（可选 **`shell.vueEntry`** 走宿主 Vue，否则 **`shell.entry`** HTML）、**Node 子进程 JSON-RPC 侧车**（stdout 打印 `OCLIVE_READY`）。

## 要求

- 本机已安装 **Node.js 18+**（`node` 在 PATH 中），用于启动 `rpc_server.mjs`。
- Oclive 桌面端已按仓库实现启用目录插件与 `ocliveplugin` 协议。

## 安装到宿主

任选其一：

1. **与 `roles` 同级**：将本目录整体复制为  
   `<你的 roles 父目录>/plugins/com.oclive.example.minimal/`  
   （例如仓库根若存在 `roles/`，则放到 `plugins/com.oclive.example.minimal/`）。
2. **开发者额外根（C1）**：在应用数据目录的 `oclive_host_plugins.json` 中设置  
   `"developer_mode": true`，并把本仓库的 **`examples/`** 或 **`examples/directory-plugin-minimal` 的父目录** 加入 `extra_plugin_roots`（插件根必须是该路径下的**一级子目录**；若直接把本文件夹加入 `extra_plugin_roots`，则路径应指向 **`…/examples/directory-plugin-minimal` 本身**）。

## 宿主配置示例

在 **`%APPDATA%/<厂商>/…/oclive_host_plugins.json`**（与 `app.db` 同级目录）新建或合并：

```json
{
  "shell_plugin_id": "com.oclive.example.minimal"
}
```

重启应用后，发行构建会把 **`shell.entry`** HTML 挂载到不透明源全屏 iframe；HTML 页经 parent broker 提供的 **`OclivePluginBridge`** 调用 `get_directory_plugin_bootstrap` 并打印 JSON。只有 Vite DEV + `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` + `force_iframe_mode=false` 同时成立时才会在宿主页内执行 `shell.vueEntry`。`manifest.json` 含 **`"type": "ocliveplugin"`** 与 **`shell.bridge.invoke`**（含 `send_message`、`read:conversation`、`get_current_role` 等权限示例），用于接管主对话能力。

## 与 `plugin_backends` 联调

在角色包 `settings.json` 的 `plugin_backends` 中为某一模块设置 `directory`，并在 `directory_plugins` 对应槽位填入 **`com.oclive.example.minimal`**。本示例 RPC 实现了最小的 **`memory.rank`**（按输入 memories 原顺序返回 `ordered_ids`），可用于烟测；其它方法返回 JSON-RPC `method_not_found` 类错误属正常。

## 文件说明

| 文件 | 作用 |
|------|------|
| `manifest.json` | 插件 id、shell（`entry` / `vueEntry`）、process |
| `Shell.vue` | 整壳 Vue 入口（与 `shell.vueEntry` 对应） |
| `ui/index.html` | 整壳 HTML 入口（`vueEntry` 不可用或强制 iframe 时） |
| `rpc_server.mjs` | 监听随机端口，打印 `OCLIVE_READY http://127.0.0.1:<port>/rpc` |
