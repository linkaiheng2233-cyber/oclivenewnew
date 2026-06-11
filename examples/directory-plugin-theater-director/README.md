# Theater Director Directory Plugin（最小骨架）

Theater 专用 **directory 插件**，实现 `theater.director.*` RPC（不占六槽）。契约见 [`handoff/RFC_THEATER_DIRECTOR_PLUGIN.md`](../../handoff/RFC_THEATER_DIRECTOR_PLUGIN.md)。

## 要求

- Node.js 18+
- OCLive 桌面端 directory 插件扫描已启用

## 安装

复制本目录到 `{app_data}/plugins/com.oclive.theater.director/`（或开发者 `extra_plugin_roots` 下同名文件夹）。

**默认不 bundled** 于 Theater 安装包；Mode 1 无需本插件。

## RPC 方法

| 方法 | 说明 |
|------|------|
| `theater.director.ping` | 健康检查 |
| `theater.director.validate_rules` | 校验 beats / scene_id |
| `theater.director.inject_beat` | 返回可播放 beat 对象 |
| `theater.director.switch_scene` | 返回 skeleton 路径 hint |

## 烟测

```powershell
node rpc_server.mjs
# 应打印 OCLIVE_READY http://127.0.0.1:<port>/rpc
```

在宿主启用插件并授予 `process:spawn` 后，经 `directory_plugin_invoke` 调用上述方法；Theater 六槽 smoke（`npm run test:distro:smoke`）**不应**因安装本插件而改变。

## 文件

| 文件 | 作用 |
|------|------|
| `manifest.json` | 插件 id、`process:spawn` 权限 |
| `rpc_server.mjs` | JSON-RPC 侧车 |
