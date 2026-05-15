# 安全审查范围与局限性（SECURITY_AUDIT_SCOPE）

本文说明 **当前仓库已做的安全相关工作** 与 **刻意未覆盖的范围**，避免对外过度宣称。实现细节以源码与 CI 为准。

**相关文档**：[KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) · [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md) · 根目录 [AGENTS.md](../../AGENTS.md)

---

## 本轮已完成（工程内）

- **`unsafe` 块**：已做 **全量清单与注释**（含并发与不变式说明）；见各 `src-tauri/src/**/*.rs` 中 `# Safety` / 模块头注释。
- **取消与并发**：`process_message` 路径、`PluginHost` 解析、**可取消 LLM**（如 `llm_cancelable` 相关模块）的 **锁顺序、`.await` 边界与取消语义** 已文档化于源码注释与关键模块头。
- **`cargo audit`**：已定期执行；**漏洞级** 命中已建档至 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md)。
- **并发审查**：对 **`Arc` / `Mutex` / `JoinHandle`** 与 **异步取消** 在主编排路径上做过 **针对性** 代码审阅（非形式化验证）。

---

## 本轮未覆盖（已知局限）

- **第三方供应链**：crate **作者信誉、发布历史、构建可重复性** 等未做系统审计。
- **Miri**：未对全部 `unsafe` 做 **Miri 全量**；仅在关键路径评估可行性。
- **模糊测试（fuzzing）**：**未**建立 `cargo-fuzz` / `proptest` 等基础设施。
- **侧信道**：**未**分析时序、功耗等侧信道风险。
- **威胁建模（STRIDE 等）**：**未**对全产品做完整建模；仅对 **对话主编排链路** 做并发与取消向审查。

---

## 第三方风险（模型、插件与用户数据）

工程审查（上节）**不**覆盖用户自行安装的模型权重、第三方插件代码、以及用户配置的 Remote 出站路径之合规性。**产品向法律与责任边界**见 [DISCLAIMER.md](../legal/DISCLAIMER.md)（[English](../../creator-docs-en/legal/DISCLAIMER.md)）。

---

## 后续计划（滚动）

1. **每个功能周期**：运行 `cargo audit` 并更新 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md)。
2. **Miri**：引入 **允许失败** 的 Miri CI job，从 **最小 `unsafe` 闭包** 起扩大覆盖。
3. **模糊测试**：评估对 **协议解析**、**Prompt 拼接边界**、**不可信 JSON** 等输入引入 `proptest` 或 `cargo-fuzz`。
4. **Tauri / gtk-rs 警告链**：跟踪 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) 中的 *unmaintained* 集群，随 **Tauri 大版本** 升级收敛。

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-05-15 | 增加「第三方风险」小节，链至 `legal/DISCLAIMER.md`。 |
| 2026-05-13 | 初版：定义已完成范围与已知局限。 |

---

[English](../../creator-docs-en/security/SECURITY_AUDIT_SCOPE.md)
