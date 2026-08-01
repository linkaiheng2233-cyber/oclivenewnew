# 内核调试（`oclive debug`）

在工程根目录使用主仓 **`oclivenewnew`** 内核（推荐 `oclive init --kernel-source <仓库根>`）时，可逐步观察 **`process_message`** 各阶段输入/输出摘要。

## 用法

```bash
cargo run -p oclive-cli -- --experimental debug -o .
cargo run -p oclive-cli -- --experimental debug -o . --step user_emotion_analyze
cargo run -p oclive-cli -- --experimental debug -o . --json --message "测试"
```

- 设置环境变量 **`OCLIVE_DEBUG_TRACE=1`** 并启动 **`--api`** HTTP 服务（默认 Mock LLM）。
- 内核向 **stderr** 打印前缀为 **`OCLIVE_DEBUG_TRACE`** 的 JSON 行。
- CLI 发送一条 **`POST /chat`** 后汇总展示。

## 常见步骤名

| step | 说明 |
|------|------|
| `load_recent_context` | 加载近期对话上下文 |
| `user_emotion_analyze` | 用户情绪分析 |
| `event_estimate` | 事件估计 |
| `memory_rank` | 记忆检索排序 |
| `build_prompt` | 组装 Prompt |
| `llm_generate` | 主 LLM 生成 |
| `postprocess` | 回复后处理与 bot 情绪 |

详见主仓 `crates/oclive_kernel_host/src/domain/debug_trace.rs` 与 `chat_engine/turn_pipeline/`。
