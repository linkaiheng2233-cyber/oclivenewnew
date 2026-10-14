# 独立 Base 库的原工程兼容迁移准备

**SSOT 范围 / 日期**：本片源码归属迁移的范围、临时分发接线、实际验证与后续交接；2026-10-14。职责仍由[模块表](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)拥有，不复制六槽规则。
**身份**：controller 实施与语义自查，`independent=false`。状态：**Locally verified（本机临时接线下的迁移准备）**；原主树与正式分发尚未切换。

## 基线与维护者决策

原工程从 SDK 冻结 `80ab85824849a466674169a3be34d94a92117886` 开片，分支 `codex/kernel-base-compat-20261014-r0`。独立库位于 `E:/OCLive/oclive-kernel`，固定本地提交 `605914cfb48c977db1e24675e039b8b198857b55` / `checkpoint/kernel-base-extraction-20261014-r0`，本片不改它的源码或标签。

维护者确认：只提取既有公共库与最简 Host，不新增统一运行执行层。独立库持有相关定义，原三个 crate 保留兼容入口。远端仓库由维护者创建，收到地址前只做本地整理与验证，不代建远端、不推送两仓。

本片 root manifest 的 `oclive_kernel_base` 使用本机临时 path。它只服务当前开发验证，**不能作为最终交付前提**；准备分支不合入 main、不推送。收到地址后必须改为不可变 Git revision 或明确 package 版本，并重新验证实际来源变化。

## 实际改动

| 位置 | 改动及原因 |
|---|---|
| workspace manifest、三个兼容 crate manifest | 显式依赖独立 Base 包；contracts 为身份测试新增 validation 的 dev 依赖 |
| contracts/types 的 `slot_base.rs` | 以 re-export 保持原 root/module import 与同一 trait/请求/失败类型身份 |
| validation 的 `minimal_role.rs` | re-export 逻辑定义及 parse/validate；文件/资产与 rich 格式校验留在原 crate |
| contracts root 旧注释 | 说明兼容归属与已有显式 Host 绑定，移除过期“尚未接 Host”措辞 |
| `kernel_base_identity.rs` | 六 trait 双向对象 coercion、六类请求跨路径赋值、失败完整保留、实际 Memory 经旧入口调用、三路径最小角色同一类型 |
| 模块表、安全中英文 | 同步唯一源码 owner、缓存 audit 与临时来源限制；不改变职责或债务 Done |

公共签名、字段、失败理由、borrow/local-future 边界与 rich 接口保持。共享消费者、Host 业务正文、角色包、前端/Tauri 传输、工具权限、取消/重试及调度策略没有改动。

## 红绿与当前已通过的验证

先加依赖和身份回归、保留旧定义，`C01` 编译 native101：32 个 E0308/E0277 命中新旧 trait/请求/失败/最小角色双定义不可互用。该红说明迁移前身份不同，不是旧 Base 缺能力。

替换为 re-export 后，同一回归 `C02` 4/4 通过；最终测试格式后的 `C11` 仍 4/4。没有新增转换器或复制类型来绕过编译器。

| 检查 | 已取得结果 | 范围 |
|---|---|---|
| `C03` 三个旧 crate 常规测试 | 247 passed / 0 failed，exit0 | 包括默认启用的 doctest；不是整个 workspace |
| `C04` runtime `--tests` | 331 passed / 0 failed，exit0 | 包含共享最小角色、六槽消费与参考实现；不包含 runtime 的默认关闭 doctest |
| `C05` Host 四个具名集成目标 | 31 passed / 0 failed，exit0 | Base LLM、最小角色 Agent、HTTP、公共 API；合成隔离夹具，没有 ignored 实况场景 |
| `C12` SDK 配方 | 1 个 E2E passed，exit0 | 实际 CLI 生成树外工程，metadata/check/test/run 及坏 Memory 副本的具名失败；不是任意作者模块自动验收 |
| `C07/C08` 分层/module-compat | exit0 | 原门禁与公共插件/模块登记；无策略改变 |
| `C10` fmt | exit0 | 新测试两处换行修正；`C09` 首次格式失败日志保留 |
| `C13` 显式 workspace doctest | 46 passed / 0 failed，exit0 | 实际选中Host/runtime的显式doctest；不是仅列测试目标 |
| `C14` 五个受影响 crate Clippy | all-targets / no-deps / D warnings，exit0 | 三兼容crate、runtime及Host；没有用它替代独立库自己的检查 |
| `C15–C20` 文档检查 | 全exit0 | 默认与变更链接、登记、镜像、编码、stale-paths |

独立库自己的37项普通/文档测试是上一片固定605914的证据，本片未伪称重新执行。迁移后，原types/validation中的11项内嵌单测及5项公共面doctest归独立库所有；本仓仍保留旧入口fixture和新身份回归。**外部依赖的测试不会自动成为本仓workspace测试**，因此正式收口必须分别取得独立库与原工程的适用证据，不能用任意一侧的绿灯代替另一侧。

本片复核按最终源码差量核定：三旧入口只有re-export，没有新增类型/trait/转换器；既有rich面、运行业务与共享消费源码保持。四项身份回归实际通过，锁差量只包含预定包/边。失败留存为C01预期编译红与C09执行者自己写的新测试格式问题；没有掩盖或重跑业务attempt。

## 锁文件与供应链

锁条目 698 → 699；仅新增 `oclive_kernel_base 0.1.0`，三个原兼容 crate 的依赖边变化。684 个 registry 完整记录（含版本、checksum、依赖）逐一与原锁相等，未升级既有第三方包。

`C06 cargo audit --no-fetch --stale --json` native0：缓存1308条公告，DB Git HEAD `d5f46becf4d442d460e1fce5e12f2c0f00924868`，漏洞0、既有 warning7（unmaintained6 / unsound1）。没有刷新公告库，也没有改变 ignore、TLS 或 signing 决策。

## 交接、停止线与未完成项

原始证据与 lock 对账：`E:/OCLive/_worktrees/wiring-view-20261013-r0/.cursor/plans/kernel-base-compat-20261014-r0/`；这是本机获取位置，不是可携带依赖。关联源码/测试与本报告可随后续正式 Git 交付，接手者须按实际分发来源复现。

- main仍为 `08348f31b8f3459537c13227302bf4b8e273baa8`；视频准备、旧 SDK 提交/tag与独立库605914不改写。
- 远端地址、独立库远端推送/CI、正式分发来源切换、原工程准备片的合入/正式CI尚未完成。
- `K-CORE-BOUNDARY-01` 不因本地准备标 Done；签名、TLS、双核、Full韧性、多Agent及高级情绪记忆等冻结不解开。
- 本片不宣称全部 invocation 约束已有统一执行引擎，也不为真实模型、TTS、平台实机、崩溃窗口或浏览器发行形态出具新结论。

类型身份、真实消费与适用回归足以验收此准备片即停，不继续扩大调查或复制其余案例。下一动作是接收维护者创建的远端地址，固定发布来源，再对这次来源切换收口。
