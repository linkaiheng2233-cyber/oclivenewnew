# 安全审查范围与局限性（SECURITY_AUDIT_SCOPE）

本文说明 **当前仓库已做的安全相关工作** 与 **刻意未覆盖的范围**，避免对外过度宣称。实现细节以源码与 CI 为准。

**相关文档**：[KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) · [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md) · 根目录 [AGENTS.md](../../AGENTS.md)

---

## 本轮已完成（工程内）

- **`unsafe` 块**：已做 **全量清单与注释**（含并发与不变式说明）；见各 `distros/desktop-tauri/src/**/*.rs` 中 `# Safety` / 模块头注释。
- **取消与并发**：`process_message` 路径、`PluginHost` 解析、**可取消 LLM**（如 `llm_cancelable` 相关模块）的 **锁顺序、`.await` 边界与取消语义** 已文档化于源码注释与关键模块头。
- **`cargo audit`**：已定期执行；**漏洞级** 命中已建档至 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md)。
- **`cargo deny`**：根 `deny.toml`；**dimension5 检查项之一（licenses+bans）** + `oclive lint --deny`；策略见 [SUPPLY_CHAIN.md](./SUPPLY_CHAIN.md)。
- **并发审查**：对 **`Arc` / `Mutex` / `JoinHandle`** 与 **异步取消** 在主编排路径上做过 **针对性** 代码审阅（非形式化验证）。
- **本机 HTTP API**：除 `/health` 外默认要求 `OCLIVE_API_TOKEN`；无令牌时服务拒绝启动，只有显式 `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` 可降级。
- **不可信路径**：角色/场景/目录插件 ID、角色资源路径与角色包 ZIP 解包已加入单段校验、containment 与 Windows 路径回归测试。
- **插件 UI 最小隔离**：发行构建不再把目录插件 Vue 编译进主 WebView；不安全 inline Vue 仅允许 Vite DEV + `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` 双重显式 opt-in。
- **凭据扫描**：高置信扫描发现一枚自初始提交存在的 API 密钥；工作树已清除，维护者确认 N1N 已在提供商侧彻底销毁旧密钥。Git 历史按决定保留，K-SECRET-01 已关闭。

---

## 本轮未覆盖（已知局限）

- **第三方供应链**：crate **作者信誉、发布历史、构建可重复性** 等未做系统审计。
- **npm 开发工具链**：K-SUPPLY-12 修复树的完整与生产扫描均为 0，peer 树合法，主 CI 已同时持有两类高危门禁；冻结提交远端通过前仍只算本地验证，且自动扫描不等于依赖作者信誉或可重复构建审计。
- **Miri**：未对全部 `unsafe` 做 **Miri 全量**；仅在关键路径评估可行性。
- **模糊测试（fuzzing）**：已建立 **`kernel/fuzz/`**（libFuzzer）与 **`oclive_validation` proptest** harness（见 [FUZZING.md](../testing/FUZZING.md)）；独立 Nightly `fuzz` job 失败会真实变红并保留 artifact，但不阻塞 main。
- **Loom 并发模型**：`distros/desktop-tauri/tests/loom_concurrency.rs`（JSON-RPC 请求 ID、`narrative_hint` 缓存 RwLock 模型）；`oclive test --loom` / 独立 Nightly `loom` job。主仓 **无 `unsafe` 块**；Loom 覆盖逻辑并发而非 FFI，Nightly 失败不替代人工根因判断。
- **侧信道**：**未**分析时序、功耗等侧信道风险。
- **威胁建模（STRIDE 等）**：**未**对全产品做完整建模；仅对 **对话主编排链路** 做并发与取消向审查。
- **插件强隔离**：HTML fallback 仍共享 `https://ocliveplugin.localhost` origin，尚未完成每插件独立 origin / iframe 原生 E2E；签名严格模式仍是 opt-in。发行版禁 inline Vue 是最小止血，不等于插件沙箱已经完成。
- **Git 历史凭据**：仓库工作树不再含明文密钥；旧提交仍可读取已撤销的值，因此历史可见性作为已接受残余风险保留，不等同于仍有效的凭据。

---

## 第三方风险（模型、插件与用户数据）

工程审查（上节）**不**覆盖用户自行安装的模型权重、第三方插件代码、以及用户配置的 Remote 出站路径之合规性。第三方插件在签名与独立 origin 完成前不属于可信代码；发行构建仅保证它不会以 inline Vue 继承主页面权限。**产品向法律与责任边界**见 [DISCLAIMER.md](../legal/DISCLAIMER.md)（[English](../../creator-docs-en/legal/DISCLAIMER.md)）。

---

## 后续计划（滚动）

1. **每个功能周期**：运行 `cargo audit` 并更新 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md)。
2. **Miri**：引入 **允许失败** 的 Miri CI job，从 **最小 `unsafe` 闭包** 起扩大覆盖。
3. **模糊测试**：持续扩展 `kernel/fuzz/` 目标与 proptest 属性；对发现 crash 建立最小复现入库。
4. **Tauri / gtk-rs 警告链**：跟踪 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) 中的 *unmaintained* 集群，随 **Tauri 大版本** 升级收敛。
5. **npm 开发工具链**：先做可达性与受支持版本矩阵，再升级 ESLint/WebDriver/目录插件编译链；禁止用强制安装掩盖 peer 冲突。

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-08-01 | K-SUPPLY-12 本地修复后完整 npm 图为 0，并把 full dev audit 升为硬门禁；远端结论待冻结提交。 |
| 2026-08-01 | 区分生产 npm 硬门禁与完整 dev graph 风险，登记 K-SUPPLY-12。 |
| 2026-07-17 | 增补 HTTP 鉴权、路径 containment、插件 inline Vue fail-closed、历史凭据与共享 origin 局限。 |
| 2026-05-15 | 增加「第三方风险」小节，链至 `legal/DISCLAIMER.md`。 |
| 2026-05-13 | 初版：定义已完成范围与已知局限。 |

---

[English](../../creator-docs-en/security/SECURITY_AUDIT_SCOPE.md)
