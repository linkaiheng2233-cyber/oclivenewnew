# 插件权限（Permissions）v1

> 目标：把插件“能做什么”从隐式行为升级为可声明、可授权、可撤销的权限边界。  
> 说明：当前实现以 **权限 token（string）** 为主，后续可升级为结构化 schema，但 token 命名必须稳定。

---

## 1. 设计原则

- **来源为主，安全为辅**：权限展示配合“官方/第三方/侧载”来源与签名状态。
- **安装时一次性授权**：安装前展示声明权限，用户确认后安装；安装后可随时撤销。
- **最小化**：插件只声明必需权限；高风险权限组合必须二次确认。

---

## 2. 权限 token 命名约定（v1）

### 2.1 宿主桥接（OclivePluginBridge）

这些权限来自 `manifest.shell.bridge.invoke` / `ui_slots[*].bridge.invoke` 的能力白名单。

- `read:conversation`
- `read:conversations`
- `read:roles`
- `read:current_role`
- `write:memory`
- `write:emotion`
- `write:event`
- `write:prompt`
- `write:settings`
- `export:conversation`
- `import:role`
- `delete:role`

> 兼容：`bridge.invoke` 里既允许写命令名，也允许直接写权限 token。宿主会把命令名映射为 token。

### 2.2 目录插件 RPC（高风险）

- `rpc:invoke`：允许通过 `directory_plugin_invoke` 透传 JSON-RPC 调用。

**建议**：默认不开启；若插件需要 RPC 能力，应在市场安装时触发二次确认，并在已安装插件页明确标注。

### 2.3 网络/文件/进程（预留给后续细粒度 enforcement）

当前 UI/索引可先使用如下 token 表达意图（后续会在宿主关键路径进行强制校验）：

- `network:*` 或 `network:domain:<example.com>`
- `filesystem:read:<path>` / `filesystem:write:<path>`
- `shell:commands:<name>`（或更细的 allowlist）
- `process:spawn`：**已在 v1 强制校验**——当插件被配置为任意固定模块的 `directory` 后端时，宿主在启动其 RPC 子进程前会检查该权限；未授权则拒绝并回退到 builtin（同时写入审计）。

---

## 3. 高风险组合（必须二次确认）

以下任一条件满足时，安装流程应触发二次确认：

- 同时声明 `network:*`（或任意 `network:`）+ 任意 `filesystem:` + 任意 `shell:` / `process:spawn`
- 声明 `rpc:invoke`

---

## 4. 与索引（plugins.json）关系

市场索引条目使用 `permissions: string[]`，由插件作者声明。宿主会：

- 安装前展示权限列表并要求确认
- 安装后提供权限开关（可撤销）
- 运行时对关键能力点执行权限校验并记录审计元数据

