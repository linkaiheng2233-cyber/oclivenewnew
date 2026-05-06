# Linux 内核：多模态 / 硬件感知注入示例（契约外挂在进程外）

本目录说明如何在 **不修改内核 crate** 的前提下，将 ASR / TTS / CV 等模块的输出接入 `POST /chat`。

## 原则

1. 感知模块运行在 **独立进程**（或设备侧服务）。
2. 将结构化或自然语言摘要 **拼接进用户消息**（或未来扩展的专用 DTO 字段）。
3. 内核仍走统一的 `process_message` → Memory / Emotion / Complex Emotion 等管线。

## 示例：视觉上下文前缀

假设外挂视觉模块输出：`用户表情疲惫，环境光线较暗`。

可拼入 `message`：

```text
[视觉上下文] 用户表情疲惫，环境光线较暗。\n用户口述：我今天好累。
```

机器人侧由集成层先调用 CV，再组包调用 `/chat`。

## 脚本

见同目录 `chat_with_context.sh`（需 **curl**、**python3**；本机已启动 `oclive_kernel_server`，且 `ROLE_PATH` 指向含 `manifest.json` 的角色目录）。

若启用了 `OOCP_API_TOKEN`：

```bash
export OOCP_API_TOKEN=your-token
./chat_with_context.sh
```

## 与 OOCP

若交互走 OOCP WebSocket（`send_message` 等），同样在载荷里携带合成后的用户文本即可；契约仍以 `creator-docs/oocp/OOCP_SPEC_v0_1.md` 为准。
