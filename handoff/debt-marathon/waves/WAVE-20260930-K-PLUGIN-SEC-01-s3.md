# K-PLUGIN-SEC-01 · Stage 3 Windows 原生隔离取证

**SSOT 范围**：本页只记录 Stage 3 在 Windows WebView2 的原生边界观察、测试判据修正与续跑坐标；债务当前状态见[主台账](../../TECHNICAL_DEBT_INVENTORY.md)，契约见[插件计划](../long-plans/K-PLUGIN-SEC-01.md)。
**最后更新**：2026-09-30。

| 字段 | 本轮事实 |
|---|---|
| 债 ID / Stage | K-PLUGIN-SEC-01 / 3 |
| base HEAD | `44d952b3711c646b903333d65a116ed2a7db6366`；开场工作树干净 |
| claim | Windows 原生隔离证据补齐；未改插件权限、运行时协议或生产安全策略 |
| 结果 | 隔离 WebView2 CDP **限定通过**；仓库原生回归判据已修正，尚待更新后在远端原生任务执行 |
| 状态 | K-PLUGIN-SEC-01 保持 **Partial**，机器计划仍 `currentStage: 3`，队列仍 `implemented` |

## 原生结果与边界

在仓库忽略目录 `.cursor/plans/plugin-native-s3-20260930-r0/` 隔离构建 `test-harness` 桌面变体，以 `TAURI_CONFIG` 仅为测试窗口加入 `--remote-debugging-port=0`；源码保持本轮 base，生产默认配置未改。桌面程序为 53,158,400 B / SHA256 `04E710D768B743636139F75FD45BEA3BED1E620E49AAF66FB9149E3AAC57A6CD`。`OCLIVE_DESKTOP_ISOLATED_TEST=1` 只在该编译特性内跳过系统级深链注册和首轮全局热键；应用数据、角色、插件、WebView2 用户目录及 kernel 子进程均指向本轮隔离树，未发模型/语音请求。

| 观察 | C6 实测 |
|---|---|
| 原生入口 | WebView2 发布 CDP 端口；真实桌面主窗口 `https://tauri.localhost/`，插件页 `https://ocliveplugin.localhost/com.oclive.example.minimal/ui/index.html` 返回 200 |
| 全壳隔离 | iframe `sandbox=allow-scripts`；主壳 `.left-pane` 不存在；插件读取 `window.parent.document.body` 被浏览器拒绝 |
| 桥正路径 | `OclivePluginBridge` 存在；`#boot` 得到 `get_directory_plugin_bootstrap` 的权威响应并含自身 ID |
| 直接 IPC 负路径 | Windows 插件页的 `__TAURI_INTERNALS__` 是对象，直接调用已注册的只读 `get_directory_plugin_bootstrap` **失败**，返回 `Origin header is not a valid URL`；不能再用“对象必须不存在”代替授权结果 |
| 另一插件 DOM | 可将 Voice 的公开 HTML 装入二级 sandbox iframe，资源 200；外层插件的 `child.contentDocument` 为 `null`，子页的 `button#record` 实际存在。此证据只证明跨框 DOM 不可读，不声称静态资源不可请求 |
| 停机 | C6 native exit `0`、事实 `ok=true`；受管进程快照 9 项，事后存活 0；API/CDP 监听器存活 0 |

原始 C6 事实[逐字节副本](./WAVE-20260930-K-PLUGIN-SEC-01-s3.cdp-C6-facts.json)为 6,419 B / SHA256 `2C86EC9D568EE80D2B233220795485C8625077C150758726826DCF2FCACBD455`；C6 运行的[仓库内探针](../../../distros/chat-pro/e2e/tauri-plugin-cdp.mjs)为 SHA256 `A9CA727DBF8817FEC2D46AE431B2D51F0B8CE97EAAC425E7D4D8726C52CE78B4`。本机原始启动日志、退出件、进程快照、截图与 A1–C5 尝试保留在上述忽略树；提交的 JSON 不含完整原生日志，因此不能单凭它证明清理或退出码。

## 失败尝试与测试修正

- A1/A2 的 Windows `tauri-driver` 均在 WebDriver session 创建时失败（`DevToolsActivePort file doesn't exist`），断言一次也没有执行；A2 已换成匹配本机 WebView2 Runtime 的 Edge Driver，故不能把失败简单归因于版本号。此失败是测试入口限制，不作为产品隔离失败。
- C1/C2 的 CDP 已连上真实窗口，但首次未重载页面，观察到 `about:blank` 中间帧；C3 监听并重载后执行了断言，发现 Windows `__TAURI_INTERNALS__` 对象，按旧测试判据失败。C4 直接 IPC 取证确认拒绝原因；C5 改用“直接授权必须拒绝、broker 必须成功”的安全判据后通过，C6 由仓库内探针再次通过。各 attempt 独立保存，未把失败改写成通过。
- 本轮修改 `distros/chat-pro/e2e/tauri-native.spec.ts` 的 Stage 3 原生测试，并登记独立 Windows CDP 探针：跨平台接受“无 direct IPC API”或“注册命令被拒”，拒绝成功或超时；增加另一插件 iframe 加载与 DOM 不可读断言。没有改变产品源码或 release 配置。新 WebDriver 断言在本地列举为 2 条测试，但 Windows 原命令的 session 仍不可建立；Windows C6 是 CDP 等价原生取证，不冒充该命令通过。

## 验收与续跑

本轮代码已通过定向 ESLint、仓库 `vue-tsc --noEmit -p tsconfig.app.json`、`check:module-compat`、原生测试枚举及 `git diff --check`。文档默认/改文链接、旧路径、登记、债计划结构和编码检查均通过；Dimension 5 `--ci` 最终 **PASS (29 checks)**。其首轮失败来自本机缺少默认 `py` 启动器，按脚本支持的 `OCLIVE_VOICE_PYTHON` 指向便携 Python 后，语音 ratchet 与整套门禁均原生 exit 0；未改全局环境。直接 `tsc -p distros/chat-pro/tsconfig.json` 的 Vue SFC 模块解析失败是错误入口，不能作为项目 typecheck 判据。目标提交的 CI 仍独立核对。

下一步是在当前源码的 Ubuntu Nightly 原生 WebDriver 任务中实际执行**更新后的** `plugin isolation` 断言，并核对精确 SHA、退出码和 job 结果。Windows CDP 作为另一平台的补充取证，不能代替 Linux CI；Stage 4 的可信安装身份仍依赖 K-SUPPLY-09，Full 不结案。若 CI 失败，先读原生步骤日志修复测试或实现，禁止放宽 DOM/IPC 拒绝断言。

`retry_safe`：只读复核、未使用的独立本机 attempt 可重试；已保存的 A1–C6 目录、原始日志及事实件不可覆盖。下一条精确命令：`node scripts/check-debt-marathon.mjs`，随后核对目标提交对应的 Nightly `e2e-tauri` job。
