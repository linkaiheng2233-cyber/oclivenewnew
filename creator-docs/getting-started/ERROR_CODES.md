# 错误码与排障速查

面向用户与开发者的统一错误语义。目标：**先自助定位，再高质量提 issue**。

## 1) 运行时 HTTP API（`/chat`）错误码

返回体（示例）：

```json
{
  "error": {
    "code": "invalid_role_path",
    "message": "role_path 不是目录：D:\\roles\\demo",
    "hint": "请传入包含 manifest.json 的角色目录绝对路径"
  }
}
```

| code | 含义 | 常见原因 | 建议 |
|------|------|----------|------|
| `empty_message` | 消息为空 | 输入只有空格/换行 | 输入至少 1 个可见字符 |
| `invalid_role_path` | 角色路径不是目录 | 路径拼错、指到了文件 | 传入 `{roles_root}/{role_id}` 目录绝对路径 |
| `load_role_failed` | 角色目录加载失败 | `manifest/settings` 缺失或结构错误 | 用编写器“运行全部检查”，核对目录树 |
| `chat_engine_failed` | 对话引擎内部失败 | 侧车超时、模型不可用、运行时状态异常 | 查看运行时日志 `oclive_chat` / `oclive_plugin` |

---

## 2) Remote JSON-RPC 错误码（侧车建议）

宿主会记录 `code/message/data`，并在失败时回退内置实现。推荐约定：

| code | name | 语义 |
|------|------|------|
| `-32700` | `parse_error` | 请求体不是合法 JSON |
| `-32600` | `invalid_request` | JSON-RPC 包结构错误 |
| `-32601` | `method_not_found` | 方法不存在 |
| `-32602` | `invalid_params` | 参数缺失或类型不匹配 |
| `-32603` | `internal_error` | 侧车内部错误 |
| `-32010` | `plugin_timeout` | 侧车上游调用超时 |
| `-32011` | `auth_failed` | token 无效或权限不足 |
| `-32012` | `rate_limited` | 命中限流 |
| `-32013` | `upstream_unavailable` | 上游服务不可用 |

---

## 3) 提 issue 最少信息（建议）

1. `error.code`、`error.message`、`error.hint`（若有）  
2. 触发动作（检测 API / 发送消息 / 自动启动）  
3. 环境变量是否设置（仅变量名，不贴密钥值）：  
   - `OCLIVE_REMOTE_PLUGIN_URL`
   - `OCLIVE_REMOTE_LLM_URL`
   - `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`
   - `OCLIVE_REMOTE_LLM_TIMEOUT_MS`
4. 关键日志片段（`oclive_chat` / `oclive_plugin`）

