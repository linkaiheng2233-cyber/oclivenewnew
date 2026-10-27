# 2026-10-27 · Chat Pro 近期工程专项巡检

**受检 HEAD**：`8aee17cc03a0979efdc4d0b4ebb04bda55963665`。本报告记录该源码快照；报告本身晚于受检提交。

下文 §1–§5 保留巡检当时的事实和行号；后续源码修复及与旧安装包的区别见 [§6 修复续接](#review-fixes)。

**结论**：快档基线与现有前端回归通过；有限反例验证确认 **1 条 P1、3 条 P2**。其中干净工作区的 npm 安装解析会失败，不能以当前已安装 node_modules 上的构建成功代替可重建性。其余为云端探测的表单/生命周期边界和诊断认证信息脱敏缺口。未确认默认聊天编排或六槽职责出现新的越界。以上仅限本轮八项，不是全仓合规、全安全审计或正式发布验收。

## 1. 范围、纪律与证据身份

- 依据：[多轮巡检手册](../../RECURRING_OPTIMIZATION_PLAYBOOK.md)、[AI 改动边界](../../AI_CHANGE_BOUNDARIES.md)、[核实协议](../../AI_VERIFICATION_PROTOCOL.md)及仓库 OCLive 工程流水线。任务 M；快档基线＋八项近期专项；未评分。
- 审查对象：近期 Chat Pro 新前端、交互、打包及直接关联 Host 消费路径。选取八项，完成八项有限核对；五项在已查层级 Supported，R2/R3/R7 有确认缺口。未声称枚举全仓所有能力，也没有把未入选能力计为已查。
- controller 自查，`independent=false`。没有子 Agent 或独立审查者。反例夹具只用本机 mock/虚构认证数据；不发真实模型请求、不调用收费 API、不解冻双核 Beta/expert/v3。
- 初始工作树干净。产品源码、manifest、lock、权限、依赖、CI 及技术债状态均未修改。本轮只新增本报告；临时探针位于 Git 忽略的 `target/round-review-20261027/`，原件另存本机 artifact。
- 当前相对已验发行源码 `b4341eb22e987ffef4a19ccad3784e2402cd2bd1`，实现与依赖图没有差量；变化为文档和导航元数据。因此下文明确区分本轮重跑和原发行证据复用，不把父提交测试写成当前 SHA 新跑。
- 本机原始材料：`E:\OCLive\oclive-dev-artifacts\chatpro-round-review-20261027\`。包含计划、命令脚本、退出码、日志、隔离 manifest/lock、反例源码及文件哈希。材料不随仓库自动分发；接手者可按下述步骤重跑。旧发行原件在相邻 `chatpro-internal-rc2-20261027/`。

## 2. 发现清单（按处理优先级）

### F1 · P1 · 内部 workspace 版本范围失配，干净 npm 安装失败

**位置**：[shared/package.json:10](../../../distros/shared/package.json#L10)、[chat-pro/package.json:3](../../../distros/chat-pro/package.json#L3)、[theater/package.json:3](../../../distros/theater/package.json#L3)；lock 的 `packages["distros/shared"].dependencies` 保留同样范围。CI 的 [npm ci 入口](../../../.github/workflows/ci.yml#L284)直接受影响。

**机制**：两个实际 workspace 包版本均为 `0.6.0-rc.2`，shared 却声明 `@oclive/chat-pro` 和 `@oclive/theater` 为 `^0.5.2`。该范围不能接受新的预发布版本；npm 尝试从 registry 补齐旧版本，已安装的 workspace 链接不能证明依赖树有效。

**实证**（当前 SHA；L2；R7 的构建入口边界）：

1. `npm ls @oclive/chat-pro @oclive/theater --all --json` → exit **1 / ELSPROBLEMS**，两包都为 invalid。
2. 把根 package.json、package-lock.json 和三个 workspace package.json 原样复制到没有 node_modules 的隔离目录，执行 `npm ci --ignore-scripts --dry-run --offline --no-audit --no-fund` → **ENOTCACHED**。这一步单独不能证明在线安装失败，原失败日志保留。
3. 同一原样夹具执行 `npm ci --ignore-scripts --dry-run --no-audit --no-fund --fetch-retries=0 --fetch-timeout=10000 --registry=https://registry.npmjs.org` → exit **1 / E404**，请求 `@oclive/chat-pro@^0.5.2`。
4. 对照夹具仅把 shared manifest 与 lock 中这两个范围改为 `*`，同一离线 dry-run → exit **0**。该对照没有改产品文件，也没有实际安装依赖。

**影响/排除**：阻碍新检出源码、干净构建环境及采用 npm ci 的 CI；与当前机器已有依赖时 lint/build/单测通过并不矛盾。没有执行新的远端 CI，不能声称某个远端 run 已失败；没有由此判定已产出的安装程序无法启动。

**最小建议**：将内部 workspace 范围与当前版本策略对齐，并同步 lock；可以使用符合项目策略的兼容范围，不必改包架构。随后在空依赖目录完成真正的 npm ci、依赖树核对和受影响前端检查。建议补上内部范围一致性检查，使版本发布不能再次留下失配。

### F2 · P2 · “测试云端连接”校验当前表单，却探测已保存配置

**位置**：[useModelManagerBody.ts:641](../../../distros/shared/src/components/model/useModelManagerBody.ts#L641)、[llmSettings.ts:178](../../../distros/shared/src/api/llmSettings.ts#L178)、[测试按钮:679](../../../distros/shared/src/components/model/ModelManagerBody.vue#L679)。

**调用链**：当前 URL/模型/Key 非空校验 → `probeCloudLlm(currentRoleId)` → invoke 仅传角色和 session → 后端使用已保存配置。测试按钮在表单变更时仍可点击；现有 `modelManager.probeNeedSave` 文案未接到该模板。

**实证**（当前 SHA；L2；R2）：真实 composable 在已保存端点 A 的状态下，把表单改为尚未保存的端点/模型 B，再触发探测。API mock 实际只收到 `role-a`，没有 B 的任何配置；仍弹出通用“云端 API 连通性正常”。探针保存在 `repro/model-probe.test.ts`，第一项。

**影响/排除**：A 正常、B 错误时，用户可能把 A 的成功误认成 B 已验证；首次配置时也可能因未保存而遭遇与当前输入不符的失败。不是请求真的发送到了 B，也没有证明配置保存动作丢失。

**最小建议**：沿用现有保存后测试的设计，比较当前表单与保存快照；有未保存变化时明确要求先保存并禁用独立测试。若选择测试未保存配置，应另行明确临时探测请求语义，不要仅修改成功文案掩盖差异。补“保存 A → 编辑 B → 点击测试”的回归。

### F3 · P2 · 云端探测完成通知未受面板生命周期约束

**位置**：[onProbeCloud:657](../../../distros/shared/src/components/model/useModelManagerBody.ts#L657)、[runCloudProbeAfterSave:630](../../../distros/shared/src/components/model/useModelManagerBody.ts#L630)、[卸载钩子:795](../../../distros/shared/src/components/model/useModelManagerBody.ts#L795)。

**机制**：加载、保存和模型列表已有 generation/角色检查，独立探测及保存后的探测却没有相同归属检查。卸载只递增其他操作的 generation；探测 promise 成功/失败后仍调用全局 toast。全局 toast 的 [showToast](../../../distros/shared/src/composables/useAppToast.ts#L37)会替换当前通知。

**实证**（当前 SHA；L2；R2）：让独立探测返回 deferred promise；真实 composable 发起后卸载组件，再完成 promise，仍观察到 `showToast('success', 'modelManager.probeOk')`。探针第二项通过的是“缺陷可复现”断言，不是修复验收。

**影响/排除**：已关闭或已切换操作上下文后，旧结果仍可能覆盖当前提示，且文案没有发起角色/端点信息。没有证明角色数据库被串写；也不要求远端服务因为 UI 关闭就已经物理停止。

**最小建议**：为两条探测入口捕获发起角色、配置快照和操作 generation；卸载/归属变化后不再提交 UI 结果。保存后 await 探测返回时也应复核生命周期，再决定后续回调或刷新。补 deferred 成功/失败与卸载、角色切换的回归。

### F4 · P2 · Basic 认证只遮住 scheme，凭据仍留在诊断显示与复制中

**位置**：[diagnosticText.ts:6](../../../distros/shared/src/utils/diagnosticText.ts#L6)；消费者 [ErrorNotice:19](../../../distros/shared/src/components/ErrorNotice.vue#L19)、[useSupportDiagnostics:29](../../../distros/shared/src/composables/useSupportDiagnostics.ts#L29)及 [reportText:77](../../../distros/shared/src/composables/useSupportDiagnostics.ts#L77)。

**机制**：通用 `authorization` 正则在首个空白处停止。`Authorization: Basic <base64凭据>` 会变成 `Authorization: [redacted] <base64凭据>`，认证值仍可恢复。对结果再次执行同一脱敏也不能消除后半段。

**实证**（当前 SHA；L2；R3）：用虚构 `review-user:synthetic-password` 构造 Basic 串，通过真实 ErrorNotice 及 SupportDiagnostics 组件。其 Base64 凭据同时残留于界面和 clipboard 报告；两项反例均复现。测试没有读取真实密钥、真实用户日志或发送认证请求。

**影响/排除**：当错误/应用自有日志确实包含此类认证头时，用户复制诊断材料可能把认证凭据一并带出。没有证据表明当前日志已经含真实 Basic 密钥，不能称为已发生泄露。现有 Bearer、api_key 与 URL userinfo 的测试仍通过；缺口是认证格式覆盖不完整。

**最小建议**：按认证头边界遮盖完整认证值，兼顾字符串和结构化错误中的表示；保留必要错误上下文。对 Bearer、Basic、带引号及 CRLF 等形式补表驱动回归，并验证重复脱敏以及显示/复制/导出同一路径。

## 3. 八项声明的有限核对记录

调查深度 D1/D2/D3 与证据等级 L1/L2/L3 不互相替代。下表 Supported 只约束列明的实现/夹具层级。单项均在计划的 D1 15 分钟及有触发条件的追加预算内停止；没有因文件行数或接口数量追加重构。

| ID / 当前声明 | 责任层、入口与直接消费者 | 本轮核对与证据 | 分类、限制与下一动作 |
|---|---|---|---|
| R1 角色记录隔离，多个记录线程不拆角色记忆/关系 | `useRoleWorkspace.ts` → `chatStoreSend.ts` 的 seq/role/thread/scene stale 检查；Tauri `chat_api_body` 透传 chat_thread_id；Host `chat_thread.rs` 将 transcript ID 与角色状态命名空间分开 | D1＋持久化 D2；本轮浏览器 role-workspace-paths 包含跨角色切换、新记录和继续记录；原 b434 结果含 thread UUID 校验、DB 重开幂等、线程独立日志/共享角色状态测试 | **Supported**（所列层级）。未穷举所有快速切换与取消落库窗口，不把客户端取消等同远端回滚；既有 D-HOST-RECOVERY-01 保持原状态 |
| R2 模型保存/探测的异步归属与反馈 | 模型面板 → useModelManagerBody → llmSettings invoke → 已保存配置服务 | D1＋有具体矛盾的生命周期 D2；现有保存/旧请求测试通过；另两项本地反例证明 F2/F3 | **Implementation gap**。现有保存成功路径受保护，不扩为所有模型设置串写；按 F2/F3 定点补齐 |
| R3 诊断显示/复制脱敏，旧复制结果不污染新通知 | ErrorNotice、useSupportDiagnostics、diagnosticText；`read_support_logs` → support_logs 自有目录限流读取 | D1＋安全 D2/D3；现有 Bearer/旧复制/关闭后日志响应测试通过；Basic 经真实两组件的反例复现 F4。原 Rust 结果含路径限制、200 行/96 KiB 边界 | **Boundary leak**（条件性认证信息）。不能扩大为全格式脱敏保证；没有真实凭据泄露证据 |
| R4 多消费者共享明暗/缩放，无重复监听遗留 | useOcliveAppearance 的模块 refs、mountCount、detached scope、最终卸载清理；默认壳与设置消费者 | D1；本轮 shared appearance 单测及 sidebar-theme 浏览器回归通过，含系统偏好、重新挂载、存储不可写和最后消费者清理 | **Supported**（同一应用窗口/测试层级）。未验证多原生窗口之间的实时 localStorage 同步 |
| R5 模态焦点打开、关闭、卸载及键盘回环 | useModalFocusRestore；默认壳帮助、模型/组件面板等明确 opt-in trap 的消费者 | D1；本轮六项直接生命周期单测通过，含初始已打开、Tab/Shift+Tab、关闭、v-if 卸载、过期 opening 及非模态；浏览器交互回归通过 | **Supported**（所列消费者/夹具）。未声称所有旧壳、全部弹窗或嵌套原生窗口均已覆盖 |
| R6 Win98 装饰串行，关闭可恢复原生装饰请求 | useEasterEggSkin 的 queue/generation；Win98TitleBar 与 main capability 权限 | D1；本轮直接测试含 in-flight setDecorations 串行恢复、共享监听/最终清理/权限失败；四项 Win98 浏览器回归通过 | **Supported**（调用/渲染层级）。本轮未操作原生标题栏；mock 成功不证明 OS 实际装饰已经切换 |
| R7 资源排除本地状态，用户库与安装分离，核对构建前提 | tracked allowlist staging → Tauri resources → desktop_library 的 .seed 原子目录与 app_data；workspace manifest/lock 为直接构建入口 | D1＋安装边界 D2。读取保留用户包/开发目录测试与旧候选原件；另隔离 npm dry-run 与范围修正对照确认 F1 | **Implementation gap**（构建前提）；安装目录/用户库隔离在旧候选有支持。未在本轮重装或启动新原生包 |
| R8 默认聊天走既有 Host，冻结实验默认关 | 默认壳 → Tauri send_message → ChatBackend::Http → KernelHttpClient → process_message/dispatch；Cargo 默认 features；MODULE_MAP 的 chatpro-deferred-surfaces | D1；源码链明确，无本轮新增业务编排；Host 默认 feature=[]，桌面默认未开 dual_core；实验 dispatch/post 受 cfg 门控；原发行链复用 | **Supported**（所列默认接线与编译开关）。隐藏剧情/语音不等同内核停用，普通 LoRA 不算被冻结；不重审所有六槽实现或实验启用矩阵 |

## 4. 实际执行与复用

| 类别 | 命令/证据 | 结果与身份 |
|---|---|---|
| 本轮基线 | `Check-Baseline.ps1`：Dimension5 `--ci`、分层、docs+code 旧路径、模块兼容 | 4 个命令均 exit 0；Dimension5 31 项 PASS。该档 cargo-audit 使用 `--no-fetch --stale`，不能称作刚获取漏洞库 |
| 本轮前端 | `Review-Frontend.ps1`：lint、typecheck、unit、Theater、build:e2e、Playwright | 6 项均 exit 0；shared **372**、Chat Pro **133**、Theater **53**、浏览器 **40** 通过。浏览器为 Edge msedge、4187 隔离 preview 与 invoke fixtures；不是真实模型服务 |
| 本轮反例 | `node node_modules/vitest/vitest.mjs run --config target/round-review-20261027/vitest.config.mts` | 2 文件、**4 项**均复现所断言缺陷；不得计入“已修复”测试数 |
| 本轮构建边界 | npm ls、原样隔离 offline/online dry-run、两范围修正后的隔离对照 | npm ls 1；原样 offline 1/ENOTCACHED；原样 online 1/E404；对照 offline 0。dry-run 不等同真正完整安装 |
| 原发行证据复用 | b434 的 `check-release.log` / `build-result.json`，源码与依赖图差量为空 | Host lib **652**；相关 desktop_library、support_logs、chat_thread 与 transcript tests 在原日志为 ok；release/ci-local/audit/deny/module/package 均 0。这些命令本轮没有全量重跑 |
| 原安装证据复用 | b434 的 NSIS/MSI、安装前后清单、独立启动原件 | 只支持那个候选的安装资源、474 文件前后不变和独立启动结果；不能代替本轮原生交互、干净 Windows 或真实模型验收 |

旧发行原件的首次 source guard 曾因 staging 删除两份跟踪 `.gitkeep` 而失败，随后按冻结字节恢复并保留首次记录；这是已知构建副作用。本轮未执行 staging、未重打包，也未把该记录包装成从未失败。

## 5. 处置与未测边界

**本轮修复：无。** 报告确认问题并给最小处理范围，没有暗中修改产品逻辑、自动扩大冻结范围或改债务状态。F1–F4 是本报告的本地编号，已对照主债务/变更台账关键词，未确认同一机制已有现行编号；不以此保证全历史语义去重，也没有直接新增 OPEN/Done。正式入账应补 L3 的 owner/愿景影响与对应任务切片。

建议下一步先修 F1，核实干净 npm ci；随后修 F4 的条件性凭据泄漏，再处理 F2/F3 的探测归属，分别补有意义的反例回归。依赖元数据修复后只扩大受影响的构建/前端验证，不能因此重跑无差量的所有 Rust 任务，也不需要重做 UI 架构。

本轮未做真实模型质量/吞吐测试、付费云请求、语音/硬件、QQ/OneBot、Production Stream、全部六槽/插件契约矩阵、完整性能测量、原生窗口操作、陌生机器安装、更新签名、公网发布或新远端 CI。现有的插件签名/隔离父债、恢复崩溃窗口、小 Kernel 物理边界等继续由各自 SSOT 管理，不由本报告升级状态。

按巡检手册 §8，本次可作为下一条实际巡检记录；滚动表行可随后续有实质内容的文档提交入账，本轮不为补日志单独推送或触发 CI。

<a id="review-fixes"></a>

## 6. 修复续接 · 2026-10-27 · Locally verified

用户随后授权按 F1 → F4 → F2 → F3 逐项修复。尺寸 M：仅内部依赖元数据、显示/导出脱敏与模型面板状态修补，不改公共 wire、权限、Host 编排、持久化、版本号或冻结能力。controller 单写并做最终只读自查，`independent=false`；不是技术债 Done 或主分支 CI 恢复声明。

**源码检查点**：`f03be6f868bfd7073bb694d981bfe06fefbc695b`；此前分类提交为 `b71c3ec5`（原巡检报告）、`f560003e`（workspace 引用）、`cefc9f4b`（脱敏）、`7bdbedcd`（保存后测试）、`f03be6f8`（探测生命周期）。前端整体验证发生在 `7bdbedcd` 加最终 F3 工作区上，执行前已冻结产品文件并记录 SHA-256；提交后逐文件核对仍一致。后续仅同步文档，不把旧安装证据升级成新包验收。

| 发现 | 实际修复与受影响链 | 本地验收 |
|---|---|---|
| F1 | shared 的两个私有 workspace 引用与 lock 改为既有的 `*` 引用方式；没有升级第三方依赖或另造 semver parser。manifest → lock → npm 链接/安装消费者 | 新空目录真实 `npm ci` 通过，包含正常安装脚本；隔离与当前工作区的三个内部包 `npm ls --all` 均 exit 0。首次离线安装缺 `vitest-4.1.12.tgz` 缓存而 ENOTCACHED，原件保留；另一个空目录按原 lock 在线安装通过，不把离线缓存不足归给产品 |
| F4 | 共同 redactor 处理完整 Basic/Bearer、quoted/escaped 和代理认证值；保持重复脱敏稳定及相邻行上下文。原始错误/落盘日志不改；ErrorNotice、Help 复制/导出共用出口 | 修前 11 项失败/6 项通过；修后定向五文件 26 项通过，最终完整 shared 回归含表驱动与真实组件显示、复制、导出。只用虚构认证数据 |
| F2 | 云端表单比较当前角色的已保存快照；未保存时按钮禁用、原因可读，handler 也阻止旧配置被当成新输入测试；加载失败不沿用另一角色的配置 | URL/模型/Key 编辑、还原、保存重载、角色加载失败的共享组件回归；新浏览器用例实际验证编辑 → 禁用 → 保存 → 可测 → 再编辑/还原 |
| F3 | 两个探测入口共用 generation/role/生命周期检查；角色或配置变化使旧请求失去 UI 更新资格，旧 finally 不清新请求状态。保存响应按原请求记录模型历史；编辑中的草稿不被旧探测刷新覆盖；已保存但连通性失败仍可完成保存，并单独发 warning | 模型面板共 25 项通过，包含独立/保存后探测的成功与失败、卸载、新旧角色交错、切走再切回、保存中/探测中编辑，以及当前失败仍结束保存 |

**最终适用检查**：完整 lint、typecheck、shared **397** 项、Chat Pro **133** 项、Theater **53** 项、生产构建、module-compat、共享 i18n、e2e 构建和 Edge 浏览器 **41** 项均通过。浏览器和单测的云服务为 invoke fixture，没有真实外部请求或额度消费。桥接构建产物经核对没有语义差量；模块兼容仍为 10 slots / 9 manifests / 7 UI contributions，插件索引为 6，不能据此推导所有原生插件都已实机验证。

**G17 与纪律**：已改共享显示出口、模型组件/状态、依赖引用和对应测试。Tauri/Host API、权限、角色包、官方插件资源与姊妹仓没有字段/格式变化，已核对无需同步实现；Theater 共享消费者以其原 smoke/单测回归。没有 Rust/Cargo.lock、门禁或 CI 配置变化，不重跑无差量的 Rust 全量链；旧 Dimension5 31 项仍只绑定受检 `8aee17cc`，本轮新增前端实现由上述定向与整体检查证明。没有擅自把后台请求已完成写成远端取消。

**文档分责**：中英 CHANGELOG 只记用户可见变化；中英 USER_MANUAL 说明先保存后测试；中英 PROJECT_CURRENT_STATUS 明确旧包不包含后续修补；本报告仅维护该次执行事实，巡检滚动表只链接本报告。没有修改模块定义或技术债状态。

**失败/副作用原件**：`E:\OCLive\oclive-dev-artifacts\chatpro-review-fixes-20261027\`。保留首次红测、离线安装失败、lint 修正及 F2 夹具纠正日志（初次 URL 选择器选到了隐藏的本地输入；修正为云端输入后重跑，不记成额外产品缺陷）。含执行计划、干净安装夹具、每步退出码和产品字节身份；不随仓库自动携带。

**交付限制**：源码修复完成并本地验证，现有桌面 rc.2 安装包仍是 `b4341eb2`。未重新打包/安装、未验证真实云模型、未执行新的远端 CI、未推送或发布，不授予新候选包或全仓 Done 状态。下一步如安排安装内测，应从修后源码冻结新的候选身份，再走原打包与原生验收流程。
