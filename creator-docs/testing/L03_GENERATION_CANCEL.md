# L03：取消当前轮生成（chat_generation_cancel）

## 结论（截至当前 `main`）

在 `src-tauri/src/domain/chat_engine/` 下检索 **cancel / abort / interrupt / stop / halt** 及公开 Tauri 命令名 **`chat_generation_cancel`**：**未发现**「取消正在进行的 LLM 生成」的已实现 API 或引擎内钩子。

`process_message` 路径为单次 `await` 驱动的主对话编排，无协作式取消令牌暴露给前端。

## 建议清单状态

将 **L03** 记为 **计划中**（或从「已完成」清单移除），直至以下任一落地方案明确并实现：

- Tauri 命令（如 `cancel_chat_generation`）+ 引擎内可取消的 LLM 调用边界；或
- 明确废弃该能力并在产品文档中说明。

## 相关阅读

- 安全审计范围中对「可取消 LLM」的期望表述见 `creator-docs/security/SECURITY_AUDIT_SCOPE.md`（与实现状态可能不一致时，以本文件为准）。
