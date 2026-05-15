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

### 1.5) 首装常见：Ollama 与角色目录（A2.1 子集）

| 现象 | 常见原因 | 建议下一步 |
|------|----------|------------|
| 对话失败、日志或 UI 提示无法连接 **Ollama** | 本机未安装、服务未启动、端口非默认、模型未 `pull` | 安装并启动 [Ollama](https://ollama.com)；终端执行 `ollama list` / `ollama pull <模型>`；核对角色包或环境变量中的模型名 |
| **`invalid_role_path` / `load_role_failed`** | **`OCLIVE_ROLES_DIR`** 未指向含子目录的 roles 根，或子目录缺 `manifest.json` | 将变量设为 **各角色文件夹的父目录**；用 [启动器](https://github.com/linkaiheng2233-cyber/oclive-launcher) 一键配置或对照 [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |
| **目录不可写 / 权限**（系统弹窗或 Rust I/O 错误） | 杀毒拦截、用户目录权限、指向了只读介质 | 换可写路径或排除误拦；勿在只读共享盘上放 `app.db`（路径见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)） |

GUI 侧若仍展示英文底层错误句，属于 **A3.2 / A6** 持续扫尾；发版前可先依赖上述文档自助排障。

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
   - `OCLIVE_LLM_BACKEND`（`ollama` / `remote`；由启动器注入时可覆盖角色包 `plugin_backends.llm`）
   - `OCLIVE_REMOTE_PLUGIN_URL`
   - `OCLIVE_REMOTE_LLM_URL`
   - `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`
   - `OCLIVE_REMOTE_LLM_TIMEOUT_MS`
4. 关键日志片段（`oclive_chat` / `oclive_plugin`）

---

[English](../../creator-docs-en/getting-started/ERROR_CODES.md)