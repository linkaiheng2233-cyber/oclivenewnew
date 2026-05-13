# kernel_directory_plugin_simple — 目录插件 SDK 极简示例

演示符合 **`type: ocliveplugin`** 的 **manifest + Node JSON-RPC 侧车**，与 [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) 一致。

## 实现的 JSON-RPC 方法

| 方法 | 用途 |
|------|------|
| **`echo.ping`** | 返回 `pong` 与入参 `text`，便于 **单独启动侧车** 后用 curl 验证 |
| **`memory.rank`** | 与 [memory 目录示例](../oclive-memory-builtin-directory/) 同形；将角色 **memory 槽** 设为 `directory` 时由内核调用 |

响应头须带 **`x-oclive-remote-protocol: oclive-remote-jsonrpc-v1`**（本示例已设置）。

## 前置条件

- **Node.js 18+**（`node` 在 PATH）
- 已能运行 **Oclive 桌面** 或 **`oclive_kernel_server`**（目录插件由宿主扫描并 `spawn` 子进程）

## 1. 安装插件到扫描目录

任选其一（与 [directory-plugin-minimal](../directory-plugin-minimal/README.md) 相同思路）：

**A. 与 `roles` 同级 `plugins/`**

```text
<你的 roles 父目录>/
  roles/
  plugins/
    com.oclive.sdk.directory_simple/   ← 将本目录内容复制于此
      manifest.json
      rpc_server.mjs
      ui/
```

**B. 开发者 `extra_plugin_roots`**

在应用数据目录的 **`oclive_host_plugins.json`** 中开启 `developer_mode`，把**本文件夹的父目录**或**本文件夹本身**加入 `extra_plugin_roots`（规则见 DIRECTORY_PLUGINS.md）。

重启宿主后，在 **插件管理** 中应能看到插件 id **`com.oclive.sdk.directory_simple`**（必要时点 **重新扫描**）。

## 2. 单独验证 `echo.ping`（不经过完整对话）

1. 在本目录执行：

   ```bash
   node rpc_server.mjs
   ```

2. 终端会打印一行：`OCLIVE_READY http://127.0.0.1:<port>/rpc`  
3. 用下面命令替换 `<RPC_URL>`（保留 `/rpc` 路径）：

```bash
curl -sS -X POST "<RPC_URL>" \
  -H "Content-Type: application/json; charset=utf-8" \
  -H "x-oclive-remote-protocol: oclive-remote-jsonrpc-v1" \
  -d '{"jsonrpc":"2.0","id":1,"method":"echo.ping","params":{"text":"hello"}}'
```

预期 `result` 中含 `"pong":true` 与 `"text":"hello"`。

## 3. 让内核在对话中调用 `memory.rank`

1. 在目标角色的 **`settings.json`**（或编写器等价配置）中设置：
   - **`plugin_backends.memory`**：`"directory"`
   - **`directory_plugins.memory`**：`"com.oclive.sdk.directory_simple"`
2. 保存后 **重新加载角色** 或重启应用。
3. 发送一条会走记忆检索的对话；若侧车被调用，可在 Node 进程标准输出或宿主日志中看到 RPC 活动（具体日志 target 以当前版本为准）。

> 若只想确认「插件已加载、进程已拉起」，完成 **§1** 后在插件列表中看到该插件即可；**§2** 可独立验证 RPC 协议无误。

## 4. 权限与 manifest

本示例 **未** 声明整壳 `bridge.invoke` 大列表，仅最小 **`shell.entry`** + **`process`**，适合作为 **纯侧车** SDK 模板。若需整壳 / 插槽权限，请在此基础上扩展，并参考 [PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)。

## 5. 相关文档

- [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — JSON-RPC 形状
- [KERNEL_SDK.md](../../creator-docs/kernel/KERNEL_SDK.md) — 嵌入 / 服务 / 示例索引
