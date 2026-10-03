# 角色包与蓝图 · 职责边界（SSOT）

**读者**：创作者、宿主集成方、Cursor / Agent。  
**状态**：2026-10-04。最小角色内容边界已确认；共享逻辑 DTO / 无 I/O 校验（§0.2）、可选本地资产有界读取（§0.3）、可选静态 PNG 校验（§0.4）及调用方指定 JSON 文件的加载准备（§0.5）已实现，CLI 现可显式调用该准备入口；builtin Prompt 的私有角色适配见 §0.6，最小逻辑定义到独立 Prompt Base 的增量适配见 §0.7，两种来源的独立最小 Host 案例见 §0.8。跨发行版保留逻辑契约，不要求统一磁盘封装或生成器；旧公开接口仍耦合完整 `Role`，**参考 Host** 生命周期接入尚未实现。Stable v4 扩展外壳是**参考宿主蓝图版本**，不是 kernel canonical role-pack schema；v2 保持兼容，**v3 双核**见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)（Opt-in Beta，默认关）。

| 文档 | 用途 |
|------|------|
| 角色包（入门） | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 蓝图 / 系统配置 | [SETTINGS_REFERENCE.md](../creator-docs/cli/SETTINGS_REFERENCE.md) |
| **蓝图目录 `blueprint/`（拉取式、本体保持瘦）** | **[BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)** |
| **蓝图扩展外壳 / 资源协调** | **[RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)** |
| 双核对齐 | [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) |

---

## 0. 三层 contract，不得混称

**2026-09-11 归属澄清**：本文沿用早期名称 `Kernel Minimal Role Contract`，指 OCLive 跨发行版的最小角色数据契约，不表示小 Kernel 核心必须解析角色包、读取资产或拥有角色生命周期。小 Kernel / 六槽 / Host 权责统一见 [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)。角色准备与激活由外围加载适配/Host 负责，再映射为能力输入；本次不重命名现有代码、不改变下述数据要求，也未完成生命周期接线。

| 层 | 只负责 | 不负责 |
|----|--------|--------|
| **Kernel Minimal Role Contract** | 定义可识别的最小角色数据：非空 persona prompt + 至少 1 个视觉资产引用；生命周期接入另由 Host 适配 | 不定义关系、好感度、视觉渲染或发行版产品语义，不选择 LLM/插件/backend |
| **参考宿主装配蓝图** | OCLive 完整参考运行时的 `slot_registry`、backend/provider/model 路由、运行策略与扩展声明 | 不是内核最小角色格式，也不要求第三方发行版原样采用 |
| **发行版 / 产品角色包** | ChatPro、VS Code、游戏版或第三方产品自己的 UI、资产、语音、市场元数据、版本与扩展 | 不得反向扩大 kernel minimal contract；只需通过适配器映射到最小角色数据与宿主能力装配 |

**当前事实（不是目标边界）**：参考宿主的 `RoleStorage` 仍把角色内容和装配配置从同一目录聚合；v2/v3/v4 以 **`pipeline.ocblueprint`** 为入口，legacy 以 `manifest.json` + `settings.json` 为入口。Stable v4 只是在这条**参考宿主组合格式**中的 Stable 蓝图版本。第三方发行版可以维护自己的磁盘格式，再分别映射为内核角色定义与宿主能力绑定，不必复制 v4 产品外壳。

### 0.1 已确认的最小逻辑 contract

**OCLive 的最小角色数据契约只要求非空 persona prompt + 至少 1 个视觉资产引用；发行版 richer role-pack format 自行维护。7 图情绪集是推荐的跨发行版能力，不是最小角色的进入条件。** 这是数据边界，不要求小 Kernel 拥有 Role Runtime，也不代表当前 Host 加载器已能直接激活仅有这两项的目录。逻辑合格、实体可读、媒体有效和 Host 可用分别判断。

| 作者侧必需内容 | 边界 |
|--------|------|
| 非空 persona / role prompt | 角色的人设正文；当前参考宿主对应 `core_personality.txt`，该文件名不成为跨发行版要求 |
| 至少 1 个视觉资产 | 单图即可；不强制情绪标签、七图目录、`portrait_catalog`、桌面立绘格式或渲染器 |

**可选内容**：标准化视觉槽位（如现有七图情绪集）和其他扩展元数据。视觉资产属于可携带角色内容；显示方式、情绪选图和硬件输出由发行版或其能力实现负责。无显示设备的宿主可以不渲染，但这不取消最小角色定义中的资产要求。

**关系决策已关闭**：`1b8f1c37` 中“关系是否必填”的待决项不再存在。`relations`、`default_relation`、favorability / affection 整体不属于 kernel minimal role contract；内核不要求关系，也不因为缺少关系而注入默认关系。角色包是否具有关系概念，对最小 contract 完全透明。当前参考宿主校验要求关系，而运行时与测试允许空关系表，其已有中性 fallback 只是当前实现事实，不升级为最小契约。

**发行版能力与宿主装配**：ChatPro 自行定义关系、好感度和 ChatPro runtime semantics；直播发行版自行定义 stream state / audience interaction model；其他发行版维护自己的扩展模型。七维人格、场景、知识、`memory_seed`、作者/展示名/产品版本、UI、语音、市场信息可由产品包承载，但没有内核统一解释或注入默认值的义务。`slot_registry`、`runtime_config`、backend/provider/model、URL、资源预算与权限授权属于独立宿主装配输入。

生命周期仍需要技术标识与命名空间；适配/加载边界可提供内部角色句柄和传输版本信息。这些是技术封装，不增加作者侧必填内容，也不要求采用发行版的 `meta.id/name/version`。磁盘文件名、传输 schema 与视觉资产描述/解析规则由发行版适配器确定；CLI 的显式文件参数不冻结跨发行版格式。若将来确有跨发行版交换包需求，再独立提出版本化格式，不反向扩张当前最小数据契约。

```text
发行版角色包 ──发行版适配器──> 最小角色定义（persona + 视觉资产）──> 内核生命周期
宿主配置 ─────宿主装配─────> 能力绑定 ──> 六槽 ports / 外围设施
```

这两个输入可以由同一个发行版适配器准备，但不能再用一个“完整 v4 角色包”名称把它们视为同一层。共享逻辑投影、本地文件读取、可选媒体能力与加载准备见 §0.2–0.5；CLI 显式文件校验见 §0.5。可选本地格式示例和参考 Host 生命周期适配仍由 [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) 的 `D-CLI-BLUEPRINT-05` 分阶段跟踪，不再以统一磁盘入口/生成器作为 Kernel 合规前置。下文 §1 起记录当前参考宿主的组合格式，不将其关系字段或蓝图要求反向纳入最小 contract。

### 0.2 第一代码切片：共享逻辑投影（无 I/O）

`oclive_kernel_types::MinimalRoleDefinition` 是内核消费者的类型入口；定义及校验实现在 [`oclive_validation::minimal_role`](../kernel/crates/oclive_validation/src/minimal_role.rs)，沿用已有共享类型重导出方向，不新增 crate 或循环依赖。

| 字段 | 当前逻辑校验 |
|------|--------------|
| `persona_prompt: String` | 必填，去除空白后非空；保留原正文，不注入模板 |
| `visual_assets: Vec<String>` | 必填，至少一个非空资产引用；保留顺序，引用由适配器解释，不规定路径、URI scheme 或情绪槽位 |

纯函数 `validate_minimal_role_definition` 校验已构造的 DTO；`parse_minimal_role_definition` 从 JSON 投影到同一 DTO 并执行同一校验。JSON 缺少必需字段、类型错误或内容为空时返回错误；未知字段（包括关系、默认关系、好感度、蓝图和扩展元数据）被忽略，不解析其产品含义，也不保留到序列化结果。因此该 DTO **不能作为 richer product pack 的无损编辑/回写模型**。发行版自己的版本、扩展和七图标签由适配层另行维护。

**验收范围**：通过只证明“非空人设 + 非空资产引用列表”的逻辑结构成立，不证明资产存在、访问安全、媒体可解码或生命周期可用。这两个逻辑函数不读取文件、获取 URL、注入关系默认值或构造旧 `Role`；本地文件快照由 §0.3 的可选适配入口另行读取，媒体另按 §0.4 或发行版自己的能力验证。现有 `RoleStorage`、`pack validate` profile 和 `init` 均未切换到这条新入口。JSON 投影也没有成为统一磁盘格式或版本封装。

测试包含共享校验边界和 [`minimal_role_contract.rs`](../kernel/crates/oclive_kernel_types/tests/minimal_role_contract.rs) 的两种合成产品映射；它们不代表 ChatPro/直播发行版的实际加载、视觉渲染或跨宿主回合验收。

### 0.3 第二代码切片：本地资产字节快照（适配层）

可选的 native 入口 [`oclive_validation::load_minimal_role_local_assets`](../kernel/crates/oclive_validation/src/minimal_role_local_assets.rs) 接受调用方指定的资产根目录、`MinimalRoleDefinition`、单资产字节上限与总字节上限，返回与 `visual_assets` 顺序一致的 `Vec<Vec<u8>>`。**根目录和预算是宿主输入，不是新增角色字段**；此入口不接入回合编排，也不替换现有目录加载器。

- 先复用 §0.2 的逻辑校验，再在任何文件系统访问前检查全部引用。此适配器只接受 `/` 分段的相对路径；复用已有 portable path segment 校验，允许中文和段内空格，拒绝空段、点/隐藏段、绝对路径、反斜杠、URL、Windows 设备名/非法字符、首尾空白、尾点和超过 128 字节的段。其他发行版仍可为逻辑 DTO 提供不同的资产来源适配器。
- 根目录与文件路径 canonicalize 后按路径组件检查包含关系；允许解析到根目录内的链接，拒绝解析到包外的链接。只读取可访问、非空的普通文件，打开前和打开后均检查文件类型与长度。
- 两个预算均须为正，单资产预算须小于 `usize::MAX`。有界读取至多多读 1 字节检测超限，不静默截断；累计返回的载荷不超过总预算，重复引用独立读取并重复计费。读取失败或任一资产不合格时不返回部分成功结果。预算限制载荷，不承诺整个进程的内存上限。
- 返回拥有所有权的字节快照，消费者应使用快照，不重新打开未经复查的路径。错误只含字段/索引与原因，不回显人设正文、资产引用、绝对路径或底层 OS 错误内容。

**文件系统前提**：调用方必须在读取期间防止根目录、路径组件和文件被并发更改。canonicalize 检查不是抗竞争替换的文件系统沙箱，也不识别硬链接的来源；这条便捷入口不能直接承担敌对可变目录的隔离边界。

**尚未保证**：非空字节不等于有效视觉资产。此入口不检查图片格式、解码、像素/帧预算或可渲染性，不下载 URL、不指定七图集、不注入产品语义；媒体校验由独立媒体适配能力负责（首个可选能力见 §0.4）。它也不定义独立磁盘包 schema、生命周期标识或 CLI 行为，不能据此宣称“一图 + prompt 已能被当前宿主直接加载”。

验收包含 [本地文件集成测试](../kernel/crates/oclive_validation/tests/minimal_role_local_assets.rs)（8 项）与读取失败/读取中超限单元测试（2 项），全部使用临时夹具。Windows 目录 junction 的包内允许/包外拒绝已在本机执行通过；Unix 对应测试分支尚未在本机执行。这不是媒体有效性或真实跨宿主运行验收。

### 0.4 第三代码切片：可选静态 PNG 媒体能力

**已确认范围**：静态 PNG + 进程内有界解码，不承诺硬隔离。入口为 [`oclive_validation::static_png::validate_static_png`](../kernel/crates/oclive_validation/src/static_png.rs)，仅在显式启用 `media-png` feature 后编译。默认逻辑校验依赖图不引入 PNG 解码器；这不是全发行版格式清单，也不增加作者必填字段。

输入是已读取的字节快照与调用方 `StaticPngLimits`：输入字节数、宽、高、总像素、输出帧缓冲字节数、解码器内部预算均须为正。两个内存预算分别控制本适配器输出缓冲与解码器的 best-effort 分配，输入、分配器开销和部分内部开销不在其中；**不能加总后声称进程硬内存上限**。尺寸在 IHDR 后、其余元数据和像素解码前检查。完整解码静态帧并读至 IEND，启用 CRC（含附加块）与 Adler-32 检查，拒绝缺失尾部和尾随字节。

| 结果 | 精确语义 |
|------|----------|
| `Ok(StaticPngInfo)` | 此快照在本次预算下完成静态 PNG 像素解码；只返回宽高与 identity 输出大小，不保留像素/元数据，不保证任何宿主实际可渲染 |
| `Unsupported(NotPng)` | 无 PNG 签名，包括 JPEG/WebP/未知字节；不判断其有效性，不得因此将 minimal role 判为非法。非 PNG 在 PNG 输入预算之前分类；非空的截断 PNG 签名前缀另判 malformed |
| `Unsupported(Animation)` | 有 APNG 块标记，不静默只验第一帧；不证明该动画自身有效，也不实现动画 |
| `Invalid(...)` | 已识别 PNG 在本适配策略下损坏或超出输入/尺寸/像素/输出/解码器预算；超预算不是“在所有宿主都损坏” |
| `InvalidLimits` / `AllocationFailed` / `DecoderFailure` | 调用方配置、分配失败或意外解码 API 失败；不伪装成角色内容错误 |

**非目标**：颜色管理、EXIF 解释、重编码、缩略图和可选元数据语义验证。text / ICC 内容跳过，像素按 identity 解码并丢弃；此检查不是文件净化器，不能据此绕过其他渲染器自己的防护。没有子进程、硬执行超时或恶意输入的进程级隔离。PNG 限制的上游说明见 [Decoder::set_limits](https://docs.rs/png/0.18.1/png/struct.Decoder.html#method.set_limits)。

[媒体定向测试](../kernel/crates/oclive_validation/tests/static_png.rs) 使用合成字节与临时目录，覆盖损坏/截断、各预算、校验和、尾部、APNG、其他格式和“逻辑 → 文件快照 → 媒体能力”组合。默认 feature 与启用 feature 分别回归；本切片未启用现有宿主/CLI 的自动媒体校验，不构成生命周期或跨宿主运行验收。

### 0.5 第四代码切片：调用方指定文件的本地加载准备

[`oclive_validation::minimal_role_local_file::load_minimal_role_local_file`](../kernel/crates/oclive_validation/src/minimal_role_local_file.rs) 将现有逻辑解析器与本地字节读取组合为**只读准备入口**，不是第二套产品包解析器，也不是角色激活 API。调用方指定资产根目录、定义文件相对路径、定义文件字节上限、单资产与资产合计字节上限；例子中的 `content.json` 只是调用方的选择，不注册统一文件名、格式版本或目录扫描规则。

- 定义文件与资产共用 §0.3 的相对路径、canonical 包含关系、非空普通文件和有界读取策略；先校验调用预算及定义路径，再读取定义文件，严格检查 UTF-8，复用 §0.2 的 JSON/逻辑校验，最后读取资产。资产引用始终相对于**资产根目录**，不是定义文件所在子目录。
- 返回 `LocalMinimalRoleSnapshot`：定义与对应字节保存在私有字段中，只提供只读定义及按原顺序配对的 `(引用, 字节)` 迭代器；重复引用保留且重复计费。失败不返回半成品，后续源文件变化不改变已返回内容。默认 `Debug` 只展示资产数量，诊断不回显路径、人设或载荷。
- 定义文件预算与资产预算独立，均由调用方提供；定义原始缓冲在解析后、资产读取前释放。它们不是整个进程的内存/时间硬上限，解析后的字符串与容器另有开销。调用方仍须在整个调用期间防止根目录、路径和文件被并发改动；这不是可变目录的原子快照或文件系统隔离机制。
- 未知产品字段仍被忽略并丢弃，不从中加载额外路径或注入默认关系/能力；不适合 richer pack 无损回写。返回快照只证明逻辑与本地实体检查通过；PNG/其他媒体验证及宿主能否消费各自独立，启用 `media-png` 也不会让此加载函数自动调用解码器。

**参考 Host 生命周期接入仍缺失**：当前 [`OcliveKernel::load_role`](../kernel/crates/oclive_kernel_host/src/role_kernel.rs) 仍委托原角色服务；[`load_role_impl`](../kernel/crates/oclive_kernel_host/src/service/role/mod.rs) 从存储取得完整 `Role` 并建立运行态，缓存也存储 `Arc<Role>`。旧 [`PromptInput`](../kernel/crates/oclive_kernel_types/src/prompt.rs) / [`PromptAssembler`](../kernel/crates/oclive_kernel_contracts/src/prompt_assembler.rs) 同样引用该完整模型。因此不能把本入口返回值直接送入现有生命周期，不能通过补齐旧 `Role` 的产品默认值宣称接入成功。维护者已选择保留旧接口、增量接入；§0.7 的 Prompt Base 能力不改变这条生命周期链。真实发行版适配与跨宿主运行验收也未完成；统一磁盘入口已不作为它们的前置要求。

CLI 现提供 `pack validate-minimal-local <asset-root> <definition-reference>`，仅复用本节的只读准备入口；三个字节预算默认为 64 KiB / 4 MiB / 16 MiB 且可由调用方覆盖。命令不扫描目录、不规定文件名、不生成包或旧 `Role`，也不自动启用 §0.4 的 PNG 校验。原 `pack validate/create` 的参考宿主语义保持不变。CLI 成功只证明定义和本地非空资产快照在所给预算下可读取，不是生命周期激活。

[加载准备测试](../kernel/crates/oclive_validation/tests/minimal_role_local_file.rs) 包含 9 项默认测试与 1 项 `media-png` 组合测试；定义文件的包内/包外链接检查复用 [本地文件集成测试](../kernel/crates/oclive_validation/tests/minimal_role_local_assets.rs) 的临时链接夹具。测试不使用官方角色包、真实消息、宿主回合或持久化状态。

### 0.6 过渡切片：builtin Prompt 私有角色适配与共用文本段

2026-09-12，`PromptBuilder` 的普通与分段入口共用私有 [`RolePromptContext`](../kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/role_context.rs)，集中选择旧 `Role` 的名字、有效人设、人设来源、描述与当前关系显示名。该投影只借用所需值，不持有或克隆完整 `Role`；它是**现有参考 builtin 实现的兼容细节，不是 Kernel 公共数据契约，也不是 Minimal Role 必需字段清单**。

| 本片改变 | 明确保留的边界 |
|---|---|
| 核心人设与性格补充段落不再直接接收 `&Role`；用户身份段落使用已解析的关系显示名 | 原覆盖/空白回退、显示名查找、档案净化和原始文案保持不变，不添加默认关系 |
| 普通 Prompt 与 stable/dynamic 分段共享角色字段选择 | 两种输出各自保持原字节与分段边界；不要求普通输出与分段拼接彼此相等 |
| 普通与分段入口共用语气、内容和页脚的私有文本 helper；身份和状态段接收所需的窄值 | 字段选择仍来自旧输入，记忆证据净化与产品文案不变；这不是新的六槽输入或固定 Kernel 流水线 |
| 旧角色字段选择可在私有适配点局部核对 | 公开 `PromptInput` / `PromptAssembler` 以及 quality-anchor / topic-hint 入口仍引用旧模型；Host、remote wire、角色生命周期均未迁移 |

**验证范围**：在生产代码仍为 `6e5da56c` 时，先加入两项特征测试（四组合成输入，分别固定完整输出、stable prefix 和 dynamic suffix 的 SHA-256，并检查关键文本行为），原版 Prompt 定向测试 41 项通过；重构后沿用同一基线，[测试源码](../kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/tests.rs) 不自动更新期望值。runtime 库测试 200 项、定向 Prompt 测试及 Clippy/格式检查本地通过。此证据不表示公开 Prompt 解耦、真实模型效果、角色激活或跨发行版运行已验收。

**共用文本段回归**：另在未改生产代码的 `0ea96034` 上捕获一组合成动态输入，固定普通输出和两个分段的 3 个摘要；它覆盖临时状态、旧聊天记忆净化、身份模板、扩展段、上一轮约束与空用户输入。沿用前述 12 个摘要，重构后 Prompt 定向测试 42 项、runtime 库测试 201 项及 Clippy/格式/分层、module-compat 本地通过。Host 的远端输入快照与缓存路径选择两项纯契约测试通过；未调用真实插件服务或模型。两种布局没有被合并为同一输出。

**当前实施止点**：本节只确认现有 builtin 的 legacy Role 私有适配和共同文本段复用，不要求新增文本准备对象、独立布局层或新的公共输入。小 Kernel / Host / Adapter 的已确认分工仍以 [MODULE_MAP §0.1](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) 为准；其 §0.2 候选与 §0.3 未完成接口对照，不是自动实施未来架构的授权。公开 Prompt 输入迁移、Minimal Role 生命周期、CLI 生成器和跨宿主接入继续是尚未完成的工作，范围须另行明确；CLI 的显式文件校验只属于 §0.5 的准备能力。不得将私有投影升级成公共 schema、补齐旧 `Role` 的产品默认值冒充接入，或由此恢复 Event Stream/R7、扩展 Memory contract。

本轮实施止点和第二片撤回结论只约束当前切片的必要性，不是永久禁止准备层、布局层或其他设计；未来有已确认需求时，可在既定权责边界内重新评估，涉及公共契约或职责变化仍须先行确认。当前不因此添加预留机制，也不恢复已撤回方案。

**第二片必要性审查（2026-09-12，稳定基线 `3cde11f746537d2809ec198021488fa8454d28f7`）**：未提交方案通过局部等价测试，不等于新增层有必要。按 [AI_CHANGE_BOUNDARIES 的 G9/G12–G13](AI_CHANGE_BOUNDARIES.md) 与上述已确认边界收缩如下；此记录不新增 Kernel 职责。

| 第二片对象 | 最小处置与理由 |
|---|---|
| `legacy_preparation.rs`、`layout.rs`、`PreparedPromptText` | 撤回。它们引入五段文本中间表示及准备/布局分层，主要服务未来输入迁移；§0.6 的既有私有角色适配已经足以标明当前 legacy 耦合，公共边界并未要求这些新机制。 |
| `mod.rs` 的 preparation → layout 接线 | 恢复为稳定基线。保留现有普通/分段入口和共同文本 helper，不改六槽合同、Host 编排或 Adapter 行为。 |
| 新增纯布局四组合测试、准备对象跨借用期测试 | 随新抽象撤回；不为了测试尚未要求的中间对象而保留实现。稳定基线原有 42 项 Prompt 测试与 15 个固定输出摘要原样保留。 |

本次纠偏后的运行源码与上述稳定基线一致；只有本节的当前实现止点与审查记录变更。它不是公开 Prompt 解耦、生命周期接入或物理拆分的完成声明。

### 0.7 增量切片：最小逻辑定义的 Prompt Base 能力

维护者选择保留参考运行时旧公开接口、增量接入最小角色。新增 [`MinimalRolePrompt`](../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_prompt.rs)：它借用 §0.2 的 `MinimalRoleDefinition`，构造时复用共享逻辑校验，作为可经 `&dyn PromptBase` 调用的一个独立实现。定义可以来自 §0.5 的本地快照，也可以由其他发行版适配器提供；文件读取和资产字节仍由 Host/Adapter 持有。它不构造完整 `Role`，不补角色名、关系、人格向量或 `slot_registry`。输入材料与人设放在不同文本段，当前调用若带额外非空 `requirements` 则明确返回 `Unsupported`，不清空真实要求以换取表面成功。文本段标记不保证下游模型抵抗注入。

[外部 crate 定向测试](../kernel/crates/oclive_kernel_runtime/tests/minimal_role_prompt.rs) 分别用一图加 prompt 的本地快照和纯内存逻辑定义驱动同一真实 Base trait 调用，并检查额外要求的拒绝。这只证明**最小内容可进入一个 Prompt 能力实现**；逻辑构造本身不证明资产存在。没有在 `AppState` 注册它，没有让参考 Host 的 `load_role` / `process_message` 接受最小目录，也不证明 LLM、视觉显示或跨宿主回合。旧 `PromptInput` / `PromptAssembler` 与完整角色缓存保持原样；不得把本能力调用误写成参考 Host 角色激活。

### 0.8 独立最小 Host 装配案例

维护者选择从内核能力向外延伸，先做独立 Host 案例，保留参考 Host 的旧接口。[`minimal_role_host` 示例](../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs)让同一 Host 从两种来源装配：调用方指定的本地文件快照，或另一发行版提供的纯内存定义与资产字节。两者都由 Host 给出技术 ID、校验定义与资产配对、按 ID 选择角色，再把**逻辑定义**交给 §0.7 的 Prompt Base，与内存 Echo LLM 完成一轮。错误身份在能力调用前拒绝，额外要求的 `Unsupported` 在 LLM 前停止；缺失、空白或不配对的资产不构造 Host。该案例不写入作者角色定义的技术 ID、关系或 backend 默认值。

案例内的内容来源接口只属于示例 Host，不是新的 Kernel 公共 port；纯内存来源使用合成资产，不代表已有第二个发行版已实接。即时 future 驱动器只适合本例的内存能力；真实 Host 必须自行管理异步调度、资源权限、生命周期和领域效果。示例的 5 项隔离测试与原生运行验证**两种来源可走同一独立装配路径**，不是参考 `AppState`、ChatPro、真实模型、持久库、视觉渲染、跨平台发行版或安全隔离验收。跨发行版只固定 §0.1 的逻辑契约；磁盘封装和版本由发行版适配，不将示例 `content.json` 升为统一文件名。参考 Host 兼容接入仍由技术债分片管理，不能因本例将 `D-CLI-BLUEPRINT-05` 标成 Done。

## 1. 当前参考宿主内部划分

| 组件 | 职责 | 面向 |
|------|------|------|
| **角色内容** | 角色身份、人格、关系、提示词与场景内容 | 创作者 |
| **蓝图** | 槽位实例、后端路由、模型名、交互/记忆/远程策略、双核开关等系统配置 | 高级开发者 / 参考宿主管理员；仅 `inference_profile` 可由编写器以受限表单向创作者开放 |

**物理落盘（今日参考宿主）**：v2/v3/v4 均以 **`distros/chat-pro/roles/{id}/pipeline.ocblueprint`** 为加载入口；Stable v4 是当前 Stable **蓝图**格式。**逻辑上**仍需分责；外置片段、扩展载荷、专家修订与说明放入 **`distros/chat-pro/roles/{id}/blueprint/`**，经 `includes` 或 v4 `extensions.*.config_ref` 引用（见 [BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)），**禁止**把长文与向导结果搅进蓝图 JSON。

**legacy**：`manifest.json` + `settings.json` 已废弃，**不得**与 `pipeline.ocblueprint` 并存；引擎字段应视为**蓝图侧**，非「角色门面」。

---

## 2. 当前参考宿主角色包可编辑内容（创作者）

### 2.1 `meta` 创作者子集（v2/v4）

| 字段 | 说明 |
|------|------|
| `id` | 角色 id（与目录名一致） |
| `name` | 展示名 |
| `version` | 包版本 |
| `author` | 作者 |
| `description` | 简介 |
| `personality` | 七维人格（对象或 7 元数组） |
| `relations` | 用户关系定义 |
| `default_relation` | 默认关系 id |
| `scenes` | 场景 id 列表（与 `scenes/` 目录一致） |

可选创作者向 **`meta`**（剧情/人设，非引擎路由）：

| 字段 | 说明 |
|------|------|
| `life_trajectory` / `life_schedule` | 异地/人生轨迹文案（见 README_MANIFEST） |
| `evolution.personality_source` | **仅 v2 兼容落点**；Stable v4 由高级运行时视图写入 `runtime_config.evolution.personality_source` |

### 2.2 目录与文件（非 JSON 槽位）

| 路径 | 说明 |
|------|------|
| **`core_personality.txt`** | **Tier0 人设唯一真源**（`PromptBuilder` 只读此文件 + 蓝图 `meta` 元数据；**不**接入 `prompts/system.md`） |
| `memory_seed.json` | 可选、创作者维护的只读前置记忆；与用户运行时 LTM、STM、聊天记录分离，详见 [`ROLE_PACK_SPEC`](../creator-docs/role-pack/ROLE_PACK_SPEC.md#persona--memory-独立迁移契约) |
| `prompts/` | **可选创作辅助**：`reply_quality_anchor.md` 人类可读镜像（Stable v4 运行时 SSOT 为 `runtime_config.reply_quality_anchor`；v2 兼容 `meta`；否则用内核默认）、creator profile 校验目录；**非** Tier0 人设来源 |
| `scenes/{id}/` | 场景 `scene.json`、`description.txt` 等 |
| `knowledge/` | 世界观 Markdown（内容向） |
| `assets/` | 立绘、头像等；**v0.4+ 草案**：`config.json` → `portrait_catalog` 指向 `assets/images/` 等路径（见 [RFC_PORTRAIT_FACILITY.md](../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md)） |
| `config.json` | 可选引擎参数：`memory` / `relation` / `turn_thinking`（Wave F 路由 + ephemeral，见 [ROLE_PACK_SPEC §9.11](../creator-docs/role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f-co-present-路由)） |
| `ui.json` | **前端布局**（非后端；见 CONFIGURATION_FILES） |
| `author.json` | 作者元数据、推荐插件（须用户确认才生效） |

### 2.3 创作者不应直接改（属蓝图；理想推理表单除外）

v2 兼容包可能把系统配置写在 **`meta`**；Stable v4 必须只写 **`runtime_config`**，宿主对 `meta.*` 的读取仅用于旧包回退：

- 已迁至 **`runtime_config.*`**（见 §3.3）：`interaction_mode`、`memory_config`、`reply_quality_anchor`、`remote_fallback_to_builtin`（包级建议）、`dual_core` 等
- 过渡期仍可能出现在 **`meta.*`**（宿主只读兼容）

**唯一受限例外**：Stable v4 `runtime_config.inference_profile` 可由角色包编写器以非技术表单编辑，用于表达采样、输出/上下文预算、推理强度与性能优先级等**可移植理想意图**。表单不得暴露或写入模型名、GGUF、本地路径、GPU 层数、线程数或实际后端；这些仍由 Chat Pro 设置页与宿主决定。字段真源见 [`SETTINGS_REFERENCE`](../creator-docs/cli/SETTINGS_REFERENCE.md#runtime_configinference_profilestable-v4)。

**禁止**创作者包内单独开启双核（见 §5.1）。

---

## 3. 蓝图专属（系统配置）

### 3.1 `pipeline.ocblueprint` 蓝图段

| 键 / 段 | 说明 |
|---------|------|
| `slot_registry` | 多实例槽：`type`、`backend`、`plugin`、`model`、`url`、`position`… |
| `groups` | 架构图分组（可选） |
| `pipeline` | 双核 Beta：`stable` / `experimental` + `depends_on`（**schema v3 · 已实现但冻结、默认关闭**） |
| `slot_registry.*.zone` | 冻结 v3 双核归属（**已实现校验与运行时筛选**；v4 不接受） |

**禁止落盘**：`module_relations`、`steps`、`entry`（校验报错；运行时派生）。

**`blueprint/` 卫星目录**（可选）：`includes/`、`overlays/`、`revisions/`、`docs/` — **不**替代 `pipeline.ocblueprint` 路径；专家文档放此处**不影响**默认蓝图校验（详见 [BLUEPRINT_FOLDER_LAYOUT.md](./BLUEPRINT_FOLDER_LAYOUT.md)）。

### 3.2 通用蓝图扩展外壳（Stable v4）

通用扩展沿用“底座归 OCLive、载荷归扩展作者”的原则，但不把第三方字段不断追加到蓝图根：

| OCLive 维护 | 扩展作者维护 |
|-------------|--------------|
| `extensions` 容器、实例 ID、`capability`、可选 `provider`、`required`、安全 `config_ref`、缺失/降级语义 | `config_ref` 指向的载荷 schema、实现、UI、迁移、许可证、文档与支持 |

- **角色内容扩展**（例如 Chat Pro `adult_extension.json`）与**蓝图能力扩展**是两种契约；可以使用同一分责原则，但不得互相冒充。
- 蓝图只声明能力意图；宿主把蓝图、`HostProfile`、用户设置和能力注册表编译为进程内 `ExecutionPlan`。
- 使用共享 GPU/内存/进程的能力另接 Resource Adapter；纯文本或纯配置扩展不需要资源适配器。
- 未知可选扩展须保留并可见降级；未知必需扩展允许查看角色以修复，但不得激活该蓝图。
- v4 已实现外壳、路径安全、required/optional 与编写器 round-trip；v2/v3 仍严格拒绝该字段。
- Capability Registry 与只读 Plan Compiler 已落地：只有宿主登记真实消费者且 Provider/依赖/权限可用时才进入 ready；可选缺失结构化降级，必需缺失阻止激活。计划不启动 Provider、不写回角色包。首个宿主 Resource Coordinator 切片已接 LLM/Voice，但预算与租约仍不属于角色包字段。

完整边界与接入闭环只维护于 [蓝图扩展与资源协调 RFC](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)，本文不复制其字段和资源协议。

### 3.3 `runtime_config`（Stable v4 SSOT；v3 双核 Beta 兼容）

| 子字段 | 说明 |
|--------|------|
| `interaction_mode` | `immersive` \| `pure_chat` |
| `memory_config` | 记忆权重与场景策略 |
| `reply_quality_anchor` | 回复质量锚点全文 |
| `remote_fallback_to_builtin` | 包级 Remote 降级建议（宿主全局仍以 `app_settings` 为准） |
| `dual_core.enabled` | 双核开关，默认 **`false`** |
| `identity_binding` / `evolution` / `ollama_model` / `remote_presence` / `autonomous_scene` | 引擎策略（可选） |
| `inference_profile` | Stable v4 可移植理想推理意图；编写器受限表单可编辑，实际模型与机器参数不属于本段 |

v2 文件若含 `runtime_config`：`pack validate` **警告并忽略**；稳定蓝图请升 **`schema_version: 4`**，只有双核 Beta 使用 v3。

### 3.4 自 `settings.json` 剥离的引擎字段（legacy → 蓝图）

| legacy `settings.json` | canonical 蓝图落点 |
|------------------------|-------------|
| `plugin_backends` | `slot_registry` |
| `interaction_mode` | **`runtime_config.interaction_mode`**；v2 仅兼容 `meta.interaction_mode` |
| `memory_config` | **`runtime_config.memory_config`** |
| `evolution`（引擎参数） | **`runtime_config.evolution`** |
| `remote_presence` / `autonomous_scene` | **`runtime_config.*`** |
| `ollama_model` | `slot_registry` 中 `type: llm` 的 `model` 或 `runtime_config.ollama_model` |

### 3.5 包外配置与宿主权威

| 配置 | 落点 |
|------|------|
| `remote_fallback_to_builtin` | 宿主 **`app_settings`** / `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` 为运行权威；包内 `runtime_config` 只能提供建议 |
| Monolith `weld_modules` | 工程根 **`monolith.toml`**（不随角色包分发） |
| 目录插件 **`permissions`** | 插件 **`manifest.json`** + 用户 **`high_risk_grants.json`** |
| MCP server | `{app_data}/mcp-servers/*.json` + 用户授权 |
| GPU/内存预算、租约与抢占 | 宿主 Resource Coordinator + `HostProfile` / 用户本机策略；蓝图只声明能力和降级意图 |

---

## 4. 迁移与校验（路线图）

| 项 | 今日 | 目标 |
|----|------|------|
| 文件 | 单文件 `pipeline.ocblueprint` | 可选拆 `role.meta.json` + `pipeline.ocblueprint`（未排期） |
| 引擎字段 | v2 兼容读取 `meta.*` | v4 顶层 **`runtime_config`**，禁止与 `meta` 双写 |
| CLI | `pack validate` 全量 v2/v3/v4；`creator` 与 `portable-core` 是专用 profile，均不等于 kernel minimal；`validate-minimal-local` 只读校验显式文件和资产 | 保留显式本地适配入口；若提供生成样例，需标明其为可选本地格式，不把它定成所有发行版的统一包；不以迁移完整 v4 为目标 |
| 编写器 | 新建 v4；导入 v2 后无损保持 v2 | 默认「角色」视图 / 高级「蓝图」视图 |

**`--profile creator` 与完整示例包**：`distros/chat-pro/roles/mumu` 等**完整示例包**含 evolution、`slot_registry` 与引擎向字段，应用**默认** `pack validate`（全量 v2/v3/v4）。对 **`--profile creator`** 会失败 — **不是 bug**，说明该包超出「纯创作者子集」。验证 creator profile 请用 `pack create` 生成的最小包或仅含 §2 字段的包。

**v2 / v3 / v4 并存**：宿主不自动改写旧包；编写器导入 v2 后仍以 v2 导出，新建包默认 v4。

---

## 5. 双核与角色包

### 5.1 双核启用条件

| 决议 | 说明 |
|------|------|
| **归属** | **蓝图** `runtime_config.dual_core.enabled`，**非**角色包字段 |
| **默认** | **`false`**；与 Remote 降级一样对终端用户**静默** |
| **创作者** | **不得**在面向初级创作者的分发包中单独置 `enabled: true` |
| **开启方** | 宿主管理员、`oclive init --dual-core` 工程模板、集成方蓝图 |
| **legacy** | **`settings.json` 不含** `dual_core` |

### 5.2 Experimental 核与角色包

| 项 | 说明 |
|----|------|
| **角色包** | 只提供 Stable 灵魂（`meta` 子集、`prompts/`、`scenes/` 内容） |
| **Experimental** | `pipeline.experimental` + 开放 `type` 由**开发者蓝图**配置，非入门创作者职责 |
| **P4 运行时** | 仅 **`PluginHost` 当前七类 type（六槽 + `complex_emotion` 设施）**可执行；其余 type 校验可过、运行时报未实现（Q20） |
| **省略 `pipeline.stable`** | Stable 仍走 **`co_present` 硬编码**（Q19） |

详见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md)。

---

## 6. 加载链（供 Bus factor）

```text
distros/chat-pro/roles/{id}/pipeline.ocblueprint
  ├─ meta（创作者子集 + 过渡期引擎字段）
  ├─ runtime_config（v4 Stable 系统配置 SSOT；v3 双核 Beta 兼容）
  ├─ slot_registry（蓝图）
  ├─ groups / includes（蓝图）
  ├─ pipeline（仅 v3 双核 Beta）
  └─ extensions（仅 v4；required/optional 由能力计划解析）
        ↓
Capability Registry / Plan Compiler（已实现只读计划）→ SlotResolver / PluginHost → process_message
```

会话 **`set_session_slot_override`** 按 `slot_registry` 实例键覆盖 `backend` / `plugin` / `plugins` / `model` / `local_memory_provider_id`，**不写回**角色包；旧 **`set_session_plugin_backend`** 仅是六个默认实例键的兼容薄包装。高危能力仍走 **插件 manifest + grants**。

---

[English summary](../creator-docs-en/role-pack/ROLE_PACK_SPEC.md#0-role-pack-vs-blueprint-boundary)
