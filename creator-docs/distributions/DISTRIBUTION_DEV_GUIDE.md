# 发行版开发指南（Distribution Dev Guide）

> 目标：任何人只依赖 OOCP 文档即可实现一个新的 oclive 发行版（VSCode / CLI / 移动端 / 桌宠）

---

## 1) 你需要实现什么

发行版（distribution）本质上是一个 **OOCP 客户端**：

- 负责：UI/交互、平台集成（VSCode API、终端、移动端通知等）
- 不负责：对话调度、插件路由、角色包解析、记忆/情绪/事件/Prompt/LLM/Agent 的内核实现

内核与发行版唯一通信语言：**OOCP**。

---

## 2) 启动本地内核（OOCP 服务端）

在本仓库根目录：

```bash
# 启动「纯内核」服务端（推荐，默认端口 48888）
npm run oocp:kernel:serve
```

说明：

- `oocp:kernel:serve` 启动 `oclive_kernel_server`（不依赖 Tauri）。
- `oocp:serve` 仍保留：启动桌面端运行时的 `--api` 模式（兼容旧工作流），但平台化方向建议以纯内核为主。

可选环境变量：

- `OOCP_API_PORT`：覆盖监听端口（兼容旧 `OCLIVE_API_PORT`）
- `OOCP_API_TOKEN`：设置后启用鉴权（方案 A）

---

## 3) 最快的“发行版验收”命令（一条命令）

```bash
npm run oocp:smoke:spawn
```

该命令会：

1. 自动挑一个可用端口启动 core
2. 等待 `/health` 就绪
3. 跑 OOCP smoke（连接 WS、读取 capabilities、调用 `role.list`）
4. 结束后自动关闭 core

---

## 4) OOCP 连接信息

- **WS URL**：`ws://127.0.0.1:<port>/oocp`
- **capabilities**：连接后服务端首帧发送
- **鉴权**：见 [`creator-docs/oocp/OOCP_TRANSPORTS.md`](../oocp/OOCP_TRANSPORTS.md)

---

## 5) Node/TS 客户端 SDK（推荐）

本仓库内置最小 OOCP client SDK：

- 位置：`tools/oocp-client/`
- API：`connectOocp({ url, token, timeoutMs })` → `client.connect()` → `client.call(method, params)`

发行版（例如 VSCode）可以直接复用它，避免重复实现协议细节与消息类型。

---

## 6) 最小闭环 Checklist（你实现发行版时要跑通的）

- [ ] 能连接 OOCP WS
- [ ] 能读取 capabilities 并显示 version
- [ ] 能创建 session：`session.create`
- [ ] 能发消息并显示 **`reply`**：`chat.send_message`
- [ ] 能正确处理 error（至少显示 `error.code` 与 `error.message`）

---

## 7) 参考实现

- VSCode 发行版：`distributions/vscode/`

