# 2026-10-27 · DeepSeek 全仓问题清单复核

**受检源码**：`c9360444d0f5f4d5f87e2a5632c62731a7e2cd7e`，复核开始时工作树干净。第三方报告引用的也是这一源码基线。本文件是带日期的审查记录，不替代架构、发布状态或债务台账。

复核开始于 2026-10-27，完成于 2026-10-28；文件名沿用被审报告日期。

**结论**：外部清单确实发现了上一轮前端专项未覆盖的数据保护、授权一致性和插件地址校验缺口；不能把其中 56 项全部当作已证实缺陷，也不能直接接受其“2 个 P0”的优先级。当前证据没有证实这两项在默认发行路径上的 P0 定性。最应先处理的是角色包覆盖导入、角色删除失败时的半完成状态、授权持久化失败仍生效，以及 RPC 对外部地址的 loopback 误判。

本轮只复核并报告，**没有修产品源码、修改债状态、改变功能冻结、重打安装包、删除用户文件或调整远端仓库设置**。controller 自查，`independent=false`；没有独立审查者或子 Agent。原报告中的勾选、修复命令和产品取舍均按待核实材料处理。

## 1. 方法与证据身份

依据 [多轮巡检手册](../../RECURRING_OPTIMIZATION_PLAYBOOK.md)、[AI 改动边界](../../AI_CHANGE_BOUNDARIES.md)、[核实协议 §2.9](../../AI_VERIFICATION_PROTOCOL.md)及现行 OCLive 流水线。56 项按七批、每批八项完成 D1 定点核对；涉及真实安全、持久化或具体矛盾的条目有限延伸 D2。仅对四个具名假设实施隔离反例，不扩展成无止境全仓探测。

原件与本轮材料在本机 `E:\OCLive\oclive-dev-artifacts\deepseek-recheck-20261027\`。第三方原件 SHA256：`33A337B2D59E712FF3688C2E3B0AE6E0299AF04656E83B02DD31B246CDDF1AF0`。该目录保留计划、harness 源码、编译与运行日志、链接检查和远端只读查询证据；不随本报告自动分发。

| 本轮证据 | 实际结果 | 能证明什么／不能证明什么 |
|---|---|---|
| 实际 Host 公共 `import_role_pack`，合成角色目录覆盖自身 | `self_import_ok=true manifest_survives=false remaining_files=0` | 确认成功返回却清空目录的反例；未操作用户角色，也未进行普通磁盘故障注入 |
| 实际 `HighRiskGrantStore`，将隔离 app_data 设为普通文件迫使持久化失败 | `grant_returned_error=true memory_granted=true` | 确认 Err 与内存授权已生效并存；初始授权为 false，未操作应用 HTTP 或真实授权文件 |
| 实际 `utils::block_on`，调试版 current_thread Tokio + catch_unwind | `current_thread_panicked=true` | 确认该 runtime 组合下 panic；没有声称默认多线程桌面已崩溃 |
| 原样提取实际 `rpc_url_is_loopback`，只做字符串分类 | 对 `http://127.0.0.1:80@evil.example/rpc` 返回 true；标准 URL 解析的 host 是 `evil.example` | 确认分类绕过，调用链继续核到 RPC 客户端；未向外部发送请求或验证真实外泄 |
| `check-markdown-links --tracked`，受检 HEAD | PASS，627 个 Markdown 文件 | 活跃默认范围链接通过；不代表链接文字、版本和契约正确 |
| 显式选择五个 archive 历史文档检查 | exit 1，存在失效相对链接 | 确认历史导航欠账；不等于活跃文档门禁失效 |
| GitHub `main` branch protection 只读查询 | required check 为 `ci-gate`；enforce_admins=false；无 required reviews | 确认当前设置；不是擅自更改保护策略的授权 |

外部 harness 首次编译曾因本轮夹具把 `HighRiskGrantStore::load` 的参数写错而失败；修正夹具后编译 exit 0，原失败日志保留。该失败不是产品缺陷。四个探针均在仓库外、新建且验证边界的隔离目录运行，不使用用户数据、真实 Key 或收费模型。

上一轮当前源码上的前端证据可以准确复用：lint/typecheck、shared 397／Chat Pro 133／Theater 53 单测、构建及本地 Edge 浏览器 41 项通过，详见 [前轮修复续接](WAVE-20261027-CHATPRO-ROUND-REVIEW.md#review-fixes)。**这些通过不覆盖本次新增的文件失败、授权持久化、URL 欺骗或原生窗口故障路径。** 本轮没有重新执行完整 Rust/前端套件，也没有冒称新的远端 CI、真实模型或原生安装验收。

下文状态使用：`Implementation gap` 为实际机制缺口；`Boundary leak` 为边界校验缺口；`Semantic drift` 为叙述偏离现行契约；`Evidence gap` 为缺少足以支撑原结论的证据；`Supported` 为已查范围符合现行约定；`Unknown` 为本轮不能裁定。静态确认机制与实跑反例分别标明，不能互相替代。

## 2. 最需要补充分处理的地方

### 2.1 角色包覆盖导入：真实数据保护缺口，先于一般 UI 优化

[role_pack.rs:273](../../../kernel/crates/oclive_kernel_host/src/infrastructure/role_pack.rs#L273)先校验源包，再在覆盖模式下 `remove_dir_all(dest)`，然后建目录、复制。没有源／目标同址拒绝，也没有 staging、备份和恢复。隔离调用实际 API 证明：把合成的已安装角色目录作为源、覆盖自身，会返回成功，但目录被清空。外部报告描述的复制失败丢旧包也有同一先删后写机制支撑，不过本轮没有注入那一种 IO 故障。

此外，复制遍历的 `filter_map(|e| e.ok())` 会忽略遍历错误。修复不能只加一个同址判断：还应验证源目标重叠关系、在独立 staging 完整复制和校验后替换，并保留失败可恢复性。

### 2.2 删除角色：错误返回与数据已删除可能同时发生

[delete.rs:19](../../../kernel/crates/oclive_kernel_host/src/service/role/delete.rs#L19)先删除 DB 命名空间，再尝试清镜像、删角色目录和插件状态。目录或后续状态清理失败时返回错误，前面的删除已经发生；镜像失败还只记录警告。该顺序静态成立，但本轮未用 Windows ACL 故障模拟用户数据删除。

应把删除设计成可恢复的多阶段操作，明确哪些失败可以重试、哪些已经提交；仅把某个目录改名并不足以解决 DB、镜像、插件状态和中途退出之间的一致性。

### 2.3 授权：持久化失败不能留下“报错但已授权”

[high_risk_grants.rs:240](../../../kernel/crates/oclive_kernel_host/src/infrastructure/high_risk_grants.rs#L240)先改内存再 persist；实际探针确认授权 API 返回 Err 后，`is_network_granted` 已为 true。撤销也有相反方向的一致性风险：内存已撤销但落盘失败，重启可能恢复旧授权。

这不是普通文案问题。建议先形成新快照，可靠持久化成功后再提交内存状态，并覆盖 grant/revoke 两条路径及重启恢复；不是只把错误提示写得更明显。

### 2.4 RPC：现有字符串检查把外部 URL 当成本机

[transport.rs:278](../../../kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/runtime/transport.rs#L278)用拆分字符串识别 host，把 URL userinfo 当成 host。ready URL 与 [RPC 调用:221](../../../kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/mod.rs#L221)都依赖它；后一条路径创建的客户端不再提供可兜底的授权检查。因此不能仅以“后面也会检查权限”排除此项。

应复用标准 URL／IP 解析，核对真实 host，并明确 userinfo、IPv6、重定向的策略。当前只确认绕过与可达调用链，尚未做实际出网攻击演示。

### 2.5 状态轮询与生产诊断：前端通过不代表这两条边界已覆盖

[useKernelStatus.ts](../../../distros/shared/src/composables/useKernelStatus.ts)在发出轮询前检查 loading，但等待后没有核对聊天轮次是否已变；旧快照可能覆盖新一轮 favorability/emotion/relation。更直接的关联缺口是 [fetchRoleSnapshot:54](../../../distros/shared/src/api/kernel.ts#L54)直接 fetch `/role_snapshot`，没有 API token；该路由属于受保护路由，而桌面返回的是实际内核地址。现有流式聊天已走认证 Rust transport，快照轮询还未同样接齐。

本轮静态链路已确认，未重新启动原生应用实测轮询 401。应在受认证 Host 通道上补轮询，再用 turn epoch／版本约束防旧响应覆盖，避免为了修轮询把高权限 token 暴露给渲染层。外部 UI-09 的“每八秒未处理拒绝”则不成立：被调函数已捕获异常并返回 null。

另外，全局 Vue 错误处理只写 console，生产构建删除 console；Sentry 又依赖配置。默认发行仍缺可靠的本地诊断／局部恢复路径。不能据此宣称任何组件异常都必然白屏，但这个兜底没有充分完成。

### 2.6 安全与文档：需补闭环，但不要混成同一类 P0

深链安装前缺少明确同意，**在旧 shell／高级功能可达时**仍是缺口；当前角色工作区传入的开关关闭该路径，所以不是默认安装即任意网页静默安装。云端密钥被持久化为明文并加入进程环境，授权启动的插件子进程会继承环境；这也是实际未收口点，但原生进程授权本来就不是 OS 沙箱。

文档整理确实漏了 README 首例的人设路径、COMPATIBILITY 的旧版本和几处失效代码锚点。编写器／Studio 更麻烦：RFC 宣称迁移落地，实际内测包仍带旧编写器，公开仓库也未标 archived。应先厘清实际交付归属，再同步文档，不能只把全部入口替换为 Studio。

## 3. 56 项逐条判定

锚点按受检 HEAD；下面的行号和路径是定位证据，不把静态阅读写成动态故障测试。原 ID 保留，原 P0/P1/P2/P3 不作为本轮修复优先级。

### B1 · 数据与信任入口（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| P0-02 | **Implementation gap；原 P0 影响面夸大。** 高级路径确实先安装后展示权限提示；默认角色工作区禁用该深链处理路径。没有网页零点击安装实测。 | `chat-pro/useAppBootstrap.ts:50–88`、`useMainShell.ts:131,292`、Host `deep_link.rs:41–71`。以后先入队、预览、明确确认；仓库白名单属于待定产品策略。 |
| KER-02 | **Implementation gap，实际反例确认。** 覆盖自身可成功返回并擦空目录；一般复制失败也缺旧包恢复。 | Host `infrastructure/role_pack.rs:273–339`；见 §2.1。加 staging／校验／可恢复替换及源目标关系拒绝。 |
| KER-03 | **Implementation gap，静态确认失败顺序。** DB 先删，后续目录／插件状态失败不能回滚；未做真实 ACL 故障注入。 | Host `service/role/delete.rs:19–55`。设计删除阶段及恢复记录，再测试失败与重启。 |
| SEC-04 | **Evidence gap + 有限硬化缺口。** API token 明文落盘属实；bridge 二级 token 缺省放行属实；但授权路由仍受外层 API token 保护，不能说任何无 token 本机进程均可调用。 | `kernel_lifecycle/spawn.rs:116–125`；Host `http_api/bridge.rs:23–34`、`mod.rs:204–212,268–309`。先定义本机同用户／不可信子进程威胁模型及授权 scope；DPAPI 不能自动隔离同一用户的恶意进程。不得直接删掉现行 Host 授权 API。 |
| SEC-05 | **Boundary leak，源代码链成立。** Key 文件为明文，写入进程环境，原生插件 Command 不清理继承环境；启动插件仍需 process:spawn 授权。 | `user_llm_secrets.rs:19–47`、`directory_plugins/runtime/spawn.rs:126,181–205`。收紧子进程环境、密钥存储；保留必要 PATH／代理等，不盲目 env_clear。未读取真实 Key。 |
| SEC-06 | **Boundary leak，实际分类反例确认。** userinfo 外部 URL 误判 loopback，RPC 也使用同一判断，后续无可靠兜底。 | `directory_plugins/runtime/transport.rs:255–291`、`remote_plugin/mod.rs:221–263`。标准 URL/IP 解析并补授权、重定向回归；未实测外泄。 |
| SEC-08 | **Boundary leak，条件成立。** 资产解析只做词法前缀检查，is_file 跟随链接；子路径若含 symlink/junction 可逃出根。未搭建 Windows junction 攻击。 | `directory_plugins/runtime/transport.rs:42–80`、桌面 `lib.rs:122–134`。检查最终 canonical 路径和链接策略，兼顾缺失文件与平台差异。 |
| SEC-09 | **Implementation gap / 原“立即覆盖”需收窄。** `/chat` 接受 role_path 是外部 Host 试聊契约；同 id 不同路径会发生缓存身份冲突，但已有 role_cache 优先，不是每次都立刻替换当前 mumu。 | Host `http_api/chat.rs:123–179`、`state/mod.rs:405–417,458–474`。核对外部路径的身份命名空间与隔离；不能直接仅准 roles_dir，破坏 Studio/OOCP 接入。 |

### B2 · 正确性与数据边界（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| P0-01 | **Semantic drift，真实上手文档错误；不足以定 P0。** README 教写 system.md 与现行 core_personality.txt 人设路径不符。不同 profile/legacy 回退不同，“照做必失败”过于绝对。 | `README.md:66`、`README.en.md:55`、角色包校验／人设加载。修正中英首例，并执行建包→校验→载入黄金路径。 |
| KER-01 | **Implementation gap，特定 runtime 下实跑 panic。** current_thread + block_in_place 不兼容；默认桌面多线程与隔离包装不能据此判定已崩溃。 | Host `utils/block_on.rs:12–14`、`remote_plugin/jsonrpc.rs:86–106`。界定同步 API 的 runtime 契约与有效调用者；spawn_blocking().await 不是同步泛型 API 的直接替换。 |
| KER-10 | **Implementation gap，实际反例确认。** grant persist 报错后内存仍授权；revoke 有重启恢复旧值的对应风险。 | `high_risk_grants.rs:240–263`。比原 P2 更值得优先处理；验收同时覆盖磁盘失败、内存值和重启值。 |
| KER-11 | **Implementation gap，恢复诊断不足。** migration 原错误会返回，但 restore backup 与 marker 的 Result 被忽略，可能掩盖二次失败。不是整个迁移静默成功。 | `state/app_state_builder.rs:158–172`。保留主错误并附恢复结果；故障注入验证，不自行新增未经约定的错误文件格式。 |
| KER-12 | **Implementation gap，静态并发窗口。** mirror 固定 .json.tmp，store 中无对应路径锁，append/rebuild 可互扰；尚未复现并发。SQLite 主存储未据此证实损坏。 | `chat_storage/mirror.rs:276–281`、`HybridConversationStore::rebuild_mirror`。唯一 tmp 名不解决旧快照覆盖，需一起设计同 session 的写序。 |
| KER-13 | **Implementation gap，静态确认缺少解压预算。** 角色 ZIP 有路径穿越防护，但无条目数／展开总量上限。 | `infrastructure/role_pack.rs:203–238`。设置并验证实际读取字节预算、单项和总项上限；未制作解压炸弹。 |
| UI-07 | **Implementation gap，竞态机制成立；补充认证断点。** 请求发出后缺 turn freshness；快照 HTTP 又未带保护路由所需认证。 | `useKernelStatus.ts:26–54`、`api/kernel.ts:54–79`、Host `/role_snapshot` 及桌面 `kernel_lifecycle/status.rs:54–56`。先认证 Host 代理，再 epoch/版本约束。未原生实测。 |
| UI-08 | **Implementation gap / 部分有意行为。** 新发送 supersede 旧流是明确设计；abort 清临时气泡且不立即补拉，可能短期丢呈现，未证明持久化永久丢失。 | `ChatInput.vue:61–73`、`chatStoreSend.ts:112,217–220,274–277`；既有 `D-HOST-RECOVERY-01` 尚有 Deferred 窗口。明确取消占位／补拉策略；不能用禁止再次发送代替决策。 |

### B3 · 安全能力与发布语义（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| SEC-01 | **Implementation gap，但属已登记未完成能力。** 当前摘要不证明作者身份，严格模式默认关闭，ZIP/更新路径未全面覆盖。主台账已经把插件安全 Stage 4 标 Partial，可信发布者依赖 K-SUPPLY-09 Deferred。 | `plugin_installer.rs:333–382`、`scripts/pack_plugin.mjs:110+`、主债表 `K-PLUGIN-SEC-01`。不能把此前切片完成说成全签名已完成，也不能自动解冻签名／密钥／撤销体系。 |
| SEC-02 | **Evidence gap，运行时校验缺少不等于原债假收口。** K-SUPPLY-02 Done 的范围是生成 Release SHA256SUMS，不是启动前认证二进制。 | `kernel_lifecycle/spawn.rs:183`、构建 checksum 脚本、债表对应 scope。若要阻止同用户换 exe，需可信校验根；同目录可替换 hash 文件不能建立信任。 |
| SEC-03 | **Boundary leak，危险原语存在；外部 RCE 链未证实。** native 写文件接受任意非空路径；名为 file_manager 的 open 实际调 opener.open_path，能由系统关联打开文件。但受信任主窗口 IPC 可达不等于插件 iframe 可达。 | 桌面 `api/desktop_fs.rs:13–24`、`api/llm_settings/commands.rs:116–126`；插件 bridge 白名单不含 write_user_text_file，已有隔离负向证据。以后约束保存对话框授权/用途和打开路径；未写启动脚本或执行恶意文件。 |
| SEC-07 | **Boundary leak / 硬化未收口。** 插件响应缺 child CSP，宿主 default-src 包含自定义 scheme；仍有 opaque iframe sandbox，不能推导成已能读宿主 DOM。 | 桌面 `tauri.conf.json:26`、`lib.rs:52–63`，Host `plugin_protocol.rs:46–56`。需连同 frame/script/connect 与 bridge 策略测试；直接移除 scheme 可能破坏插件。 |
| UI-01 | **Implementation gap，生产兜底不足。** Vue 全局 errorHandler 只有 console，生产 drop console；Sentry 按配置加载；未见统一局部 ErrorBoundary。不能断言所有异常必然全白屏。 | `chat-pro/src/main.ts:32–34,109+`、Theater main、Vite drop 配置。补受控本地诊断和可恢复局部错误界面，错误脱敏沿用现有公共层。 |
| UI-09 | **Supported；原具体故障不成立。** fetchRoleSnapshot 内部已 catch 网络／解析异常并返回 null，不能由 interval 外层没有 catch 推导周期性 unhandled rejection。 | `api/kernel.ts:54–79`。轮询认证、freshness 和可观测性分别见 UI-07/UI-01，不重复计成这一故障。 |
| UI-10 | **Implementation gap，仅显式不安全开发模式。** 扫描异常后继续加载属实，但要求 DEV 且 VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1，发行路径默认不可达。 | `vueComponentSecurity.ts:16–19`、`compilePluginVueSfc.ts:195`、`AsyncPluginVue.vue:91–113`。开发硬化可 fail-closed，不能描述为默认生产宿主执行插件。 |
| REL-01 | **Supported / 原发布结论不成立。** 已有 rc.1、rc.2 和 review-fixes 基线 tag；未发 stable 0.6.0 是明确候选状态，Unreleased 正确；checksum workflow 还支持 workflow_dispatch。 | 实际本地 tag、`PROJECT_CURRENT_STATUS`、`release-kernel-checksums.yml`。公开发布／签名／updater 未验须保持边界，不自动发 stable tag。 |

### B4 · 工程门禁（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| CI-01 | **Semantic drift，汇总表达可改善；不是已证假绿。** --ci 跳过 sample lib tests 并计入 checks 是脚本及 AGENTS 明示约定，N checks 不能说 N 测试已执行。 | `dimension5-acceptance.mjs:171–172,255`、AGENTS 测试段。建议显示 executed/skipped，保留整体检查职责；不能借此否定其它独立测试日志。 |
| CI-02 | **Supported；“CI 默认不查 drift”不成立。** 单独脚本参数是可选，但 Dimension5 已显式传 --warn-drift-high-traffic，并跑对应自测。 | `dimension5-acceptance.mjs:126–127`。mtime 漂移只是辅助信号，不是镜像语义相等证明。 |
| CI-03 | **Supported + 历史导航欠账。** archive 默认排除是有意范围；本轮显式历史文档测试确实失败，活跃 tracked 检查通过。 | `check-markdown-links.mjs` SKIPPED_DIRECTORY_NAMES 与本轮两份日志。按需修历史入口，不强制全部历史快照进入当前 merge gate。 |
| CI-04 | **Evidence gap / 选择性自动化缺口。** 找到 ignored 测试且 CI 无统一 --ignored，但有真实模型、owned child、run_id 与授权隔离要求；不是可无条件开启的一组普通单测。 | `process_message` A-turn harness 的 ignore 标记与说明、CI/scripts 搜索。本轮未重新精确计数 26。只将具备 mock 和独占夹具的选定用例纳入自动门禁。 |
| CI-05 | **Implementation gap，原生门禁覆盖不足属实；非运行故障实证。** Linux 浏览器门禁不等于 Windows WebView；nightly native 非阻塞为现行策略。本机 Edge 41 项也不能替代原生窗口。 | CI/nightly 工作流及前轮证据。确定原生最小阻塞 smoke 与成本；未建立新的远端 Windows/干净机器证明。 |
| CI-06 | **Implementation gap，可重建性改善。** cargo-deny/cargo-fuzz/tauri-driver 安装未固定 --version；--locked 只锁该次包的依赖，不固定工具版本。未发现由此导致的当前失败。 | `ci.yml:294`、`nightly-advisory.yml:72,148`。选兼容版本后固定并更新缓存 key；不是立即全链供应链失守的证据。 |
| CI-07 | **Evidence gap / 可加一致性检查。** 尚缺统一耦合版本检查，但本轮当前桌面耦合值与 MSI 映射一致；CLI、runtime 等独立 SemVer 不应强制同号。 | manifest、Wix、`RELEASE_VERSIONING`。仅校验该耦合的字段；已有 rc tag，不能说所有 tag 缺失。 |
| CI-08 | **Supported（设置事实）/ 策略待定。** 查询确认 admin 可绕过、仅 ci-gate 必需、无强制评审；ci-gate 实际聚合并验证 selected CI responsibilities，并非空壳。 | 本轮 branch-protection.log；`ci.yml:621+`、`ci-execution-policy.mjs`。单维护者政策与权限需明确，不能擅自更改远端保护。 |

### B5 · 文档（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| DOC-01 | **Semantic drift，确实漏改。** COMPATIBILITY 中英仍缓存 host 0.5.2、要求与 root 同号；部分发布段还与独立产品 SemVer 冲突。 | `creator-docs/COMPATIBILITY.md:9,13–14,154,168` 及英文镜像。回链版本 SSOT，保留准确兼容范围；这是前轮文档整理未覆盖充分的部分。 |
| DOC-02 | **Evidence gap / Unknown。** 1,227 次状态词不是 1,227 个冲突；外部未给“5 组冲突”的具体相互矛盾句子和 scope。Stable 格式与候选产品可以同时成立。 | `handoff/README.md` 文档状态判读已区分职责／候选／实现／暂停／历史。需要逐组句子反例，不强行全仓 front-matter 或按关键词卡 CI。 |
| DOC-03 | **Semantic drift，部分成立。** 旧 migrations 路径、domain/repository.rs、desktop src/error.rs 是失效代码锚点；两条 `./distros/TAURI_V2...` 从 handoff 相对解析却真实存在。不是五处都不存在。 | `CHAT_STORAGE_ARCHITECTURE.md:235`、`CLAUDE.md:24–25`、`BUS_FACTOR_NOTES.md:97` 需修；`AI_CHANGE_BOUNDARIES.md:208`、`AI_READING_INDEX.md:229` 不要误改。 |
| DOC-04 | **Evidence gap。** 行数比、13 对小于 40% 不能证明契约遗漏；英文入口已有 Full/Summary/Pending 口径，不能为了消警把全部英文承诺降成摘要。 | 中英 README 与镜像治理。需要具体关键条款缺译反例；本轮未完成逐段翻译审校。 |
| DOC-05 | **Semantic drift，实际交付归属冲突。** RFC_STUDIO_MERGE 标已落地且旧工具 Deprecated，但 rc2 实际带旧编写器；三个 GitHub 仓均未 archived，旧工具有更晚 push。不能仅按 RFC 换全部入口。 | RFC、当前状态／rc2 清单；本轮只读仓库 metadata。先核实实际可用、维护与交付入口，再由 owner 同步各页面。 |
| DOC-06 | **Semantic drift，工作目录说明不足；“脚本不存在”不成立。** root 没有 test:json，examples/oocp-test-suite/package.json 却定义了该脚本。 | `OOCP_TEST_SUITE.md:52` 及 example package/README。明确 cd 或 --prefix，避免用户在 root 误跑；无需另造测试命令。 |
| DOC-07 | **Supported；原建议扩大既定范围。** registry 脚本明示只查 handoff 根的新文档登记，并对指定 canonical blocks 做递归检查；studio 有自己的 README 导航。 | `check-doc-registry.mjs:4,50`、`handoff/studio/README.md`。新子目录导航可优化，但未证明违反现行 G16 必须递归所有文档。 |
| DOC-08 | **Semantic drift，小型定位数字漂移。** L0–L8 与实际 L0–L7 不符；invoke 缓存 13 与表中 15 行/16 名不符；CLAUDE 缓存索引行数也不宜当稳定接口。 | `AGENTS.md:54,64`、`AI_READING_INDEX.md:93`、`INVOKE_HOTPATH_MATRIX.md`。去掉不必要常量、回链 SSOT，不由名称数量重写测试成绩。 |

### B6 · 错误与维护（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| KER-04 | **Supported；原架构违规判定不成立。** Theater director 是现行模块地图明示的独立圈外通道，不要求经过普通 process_message。 | `MODULE_MAP_AND_HANDOFF.md:712` 与 AI 边界、`scene_director.rs` 说明。不能为“唯一编排”把它合并回聊天主链。 |
| KER-05 | **Evidence gap。** Unknown、Result<String>、anyhow 计数不能单独证明公开命令丢失错误码；command_error 的历史未迁完说明可以作为检查线索。 | `command_error.rs:196–197`、现有 CommandError/UI presentation。先给出具体调用链“生产者→序列化→动作”反例，再分边界迁移，不做全仓字符串替换。 |
| KER-06 | **Evidence gap。** 大文件和 churn 是维护信号，不是本轮确认缺陷，不能按行数触发重构。 | 所列 scene_director/event_impact_ai/slot_runner/agent；现行 G9 和大文件拆分规则。需先记录实际维护障碍、职责耦合与回归预算。 |
| KER-07 | **Implementation gap（依赖整洁），非运行故障。** optional tauri 确无直接 tauri:: 使用，但 tauri-commands feature 控制 CommandError 序列化，删除 feature/依赖需校验消费矩阵。 | Host Cargo.toml:20,26、`command_error.rs:103`。说明用途或去掉多余重依赖，不能顺手改变命令契约。 |
| KER-08 | **Supported；原可移植性推断不成立。** 指出的 distro 常量是 LEGACY_MIGRATIONS_REL；还有 canonical Host 路径、环境覆盖和 bundled 搜索，并非仅此硬编码。 | `infrastructure/sql_migrate.rs:14–16` 及迁移路径解析。兼容回退不应被当作当前唯一入口删除。 |
| KER-09 | **Evidence gap / Unknown。** 外部自己说明多数 unwrap 有前置守卫；没有逐个给出无守卫可达反例。本轮只实证 KER-01，不据“33”宣称另有 32 个 panic。 | 按实际边界挑反例；不能把有意 invariant、测试、feature 条件代码一律改成 ?，改变返回契约。 |
| UI-02 | **Evidence gap，原计数忽略公共层。** error toast 最终进入 Toast.vue→ErrorNotice→toErrorPresentation，多数调用点无需重复转换。旧链路仍可能不够具体，但须逐条举证。 | shared `Toast.vue:29` 和 ErrorNotice 公共层。不能由 65 处调用／9 文件使用推导 56 处漏接，也不能重复建设错误展示框架。 |
| UI-03 | **Implementation gap，部分旧模态未接齐。** TimeDial、SettingsAdultTab、TheaterCastPanel 的指定模态缺 focus trap／恢复；尚未精确确认外部“10 处”总数，部分属高级／隐藏功能。 | `TimeDial.vue:195`、`SettingsAdultTab.vue:483`、`TheaterCastPanel.vue:528`。可复用现有 focus helper；前轮只修主路径模态，不是全模态收口。 |

### B7 · 其它结构、资源与体验（8 项）

| ID | 复核状态与实际边界 | 证据／下一步 |
|---|---|---|
| UI-04 | **Evidence gap。** 大组件、store 互调和 workspace 环依赖是结构信号，尚无具体初始化故障或维护阻塞证明。 | 所列 ModelManager/AdultTab/FluentShell/store。前轮已修内部 workspace 版本失配，不等于拆完架构；后续按真实障碍分域，避免全仓大拆。 |
| UI-05 | **Implementation gap，真实 i18n 尾部。** voiceSpeakErrors、useVoiceAutoTts 有硬编码中文，主要位于尚未默认开放的语音路径；仓库也有 shared i18n 检查，不能说 shared 完全无守卫。 | `voiceSpeakErrors.ts:3–28`、`useVoiceAutoTts.ts:665,669`。在既有公共层补中英文并细化守卫，不自动解冻语音。 |
| UI-06 | **Supported / 原版本冲突结论夸大。** lock 实际只有 vue-i18n 11.4.8、Sentry 10.70.0，较低 ^ 范围可兼容；Vitest 有 4.1.11/4.1.12 工具版本差异但未证冲突。 | manifest 与 lock；前轮空目录 npm ci 已通过。可统一工具版本，不能把范围文本不同直接叫运行依赖不兼容。 |
| UI-11 | **Evidence gap。** chunk 警告阈值偏大不证明实际卡顿；Sentry 在 DSN 配置且未 opt-out 时才懒加载，存在产物不等于首屏下载。 | Vite chunkSizeWarningLimit 与 chat main 的 shouldLoadSentry。先测入口包/下载/交互；按 DSN 裁剪是可选体积优化，不是已证发布阻断。 |
| HYG-01 | **Supported（数量观察），清理范围未证。** 本轮确认 34 worktrees、78 本地分支；未核全部“62 已合并可删”。可能有其它聊天的修改和忽略文件。 | git 只读清单。不能批量删除；先逐项核所有权、快照和可恢复性。数量不是产品 P2。 |
| HYG-02 | **Unknown（体积），不是实现缺陷。** target/plans 被忽略属实；本轮未重测 31.9GB 等磁盘数字，也未证明可以安全回收。 | 只保留环境维护建议。需区分共享构建缓存、原始验收证据和个人计划，不能自动清空。 |
| HYG-03 | **Supported（资产成本），策略待定。** 跟踪角色图片实数 120、179153494 bytes（约 170.85 MiB）；未复测外部 pack 的 287MB。 | Git 跟踪资产清单。离线示例素材有用途，LFS/发布制品托管涉及 clone 与离线交付取舍，不按大小直接迁移。 |
| HYG-04 | **Supported（BOM 事实），功能缺陷未证。** 4 个跟踪文件带 BOM：一份 archive、历史 SQL 039、两份 Rust。当前 Rust 构建接受，历史 SQL checksum 改变只警告继续，并不因此拒绝启动。 | 当前编码脚本已有活跃文档/角色 JSON 范围。避免随手改已发布 migration 的原始字节和历史快照；按需扩守卫，不能虚构 BOM 必然导致运行失败。 |

## 4. 建议后续处理顺序与完成条件

以下是复核后的建议，**尚未实施**；不把所有外部建议自动纳入待办，也不改变已冻结能力的状态。

1. **数据保护与授权一致性**：KER-02、KER-03、KER-10。先写明失败／重启后的可恢复状态，再实现；必须覆盖同址导入、复制中途失败、目标删除失败、grant/revoke persist 失败。验收不能只有正常路径成功。
2. **插件与外部输入边界**：先 SEC-06，再 KER-13、SEC-08、条件深链确认。逐个做允许与拒绝用例；同时审查重定向、最终路径和安装前后的授权时序。SEC-03/04/05/07 连同现有插件隔离计划处理，明确原生进程授权与 UI iframe 的不同边界，不宣称 OS 沙箱。
3. **默认发行的可诊断性**：UI-07 的认证 Host 轮询与新旧响应排序、UI-01 的生产错误落地／局部恢复，随后限定 UI-08 的取消与补拉体验。用真实可控 HTTP/native transport 证明，mock Vue 成功不足以替代认证接线。
4. **有明确反例的文档小修**：README 人设路径、中英 COMPATIBILITY、三个代码锚点、OOCP 工作目录及缓存数字。Studio／旧编写器入口先确认实际交付方，再统一状态，不以较新 RFC 自动覆盖当前发行事实。
5. **剩余针对性加固**：KER-01/11/12、旧模态 focus、语音 i18n、CI 工具 pin 和最小 Windows native gate，按受影响路径分批。大文件、行数比、分支、缓存、图片体积只有在明确收益、所有权和验收后再做。

可信发布者签名、公开 stable 版本、Windows 签名/updater、Studio 交付归属和分支保护策略仍需按各自 SSOT／产品决策推进。本次复核不把这些“未完成／未发布／待决”改写成“前轮修复失败”，也不把它们省略成“已经都安全”。

## 5. 本次复核的实际收口

上一轮四项修复仍有原有范围内的验证支持；它们没有覆盖本轮确认的导入／删除／授权／RPC 缺口。前轮文档整理也确有漏点，尤其 COMPATIBILITY 和 README 首例，不能称为全仓文字已经对齐。

56 项均已做有限定位与判读，**四个具名反例实际执行**。其余静态机制、已登记未完成项、误判和 unknown 在表中分别保留。本报告不等于全仓无缺陷证明、渗透测试、公开发布批准或新的安装包验收；后续修复应绑定新源码 SHA、具体差量和对应失败路径的证据。
