# 角色包与蓝图 · 职责边界（SSOT）

**读者**：创作者、宿主集成方、Cursor / Agent。  
**状态**：2026-10-05。最小角色内容边界已确认；共享逻辑 DTO / 无 I/O 校验（§0.2）、可选本地资产有界读取（§0.3）、可选静态 PNG 校验（§0.4）及调用方指定 JSON 文件的加载准备（§0.5）已实现，CLI 现可显式调用该准备入口；builtin Prompt 的私有角色适配见 §0.6，最小逻辑定义到独立 Prompt Base 的增量适配见 §0.7，两种来源的独立最小 Host 案例见 §0.8，参考 Rust Host 的基础文本入口见 §0.9，共享准备与可选择 Prompt 的消费者见 §0.10，基础 HTTP / 桌面 IPC 适配见 §0.11，ChatPro 的临时状态 / 基础主界面接线见 §0.12–0.13，六槽可替换的共享消费入口见 §0.14，参考 Host 正文 Base 绑定与当前会话 Memory 见 §0.15–0.16。跨发行版保留逻辑契约，不要求统一磁盘封装或生成器；旧丰富接口仍耦合完整 `Role`，基础文本接线不等于发行版生产装配已消费全部六槽。Stable v4 扩展外壳是**参考宿主蓝图版本**，不是 kernel canonical role-pack schema；v2 保持兼容，**v3 双核**见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)（Opt-in Beta，默认关）。

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

**2026-10-04 维护者明确的兼容目标**：发行版开发者负责提供转换器，将最小内容接入自己的产品角色表示；也可将自有丰富格式投影到同一最小逻辑定义。符合这一接入目标的发行版应让最小角色完成基础交互闭环，关系、人格演化、语音等扩展缺失时明确报告不可用，而不把它们升级成基础内容的必填项。转换器不必生成统一磁盘包，也不得通过捏造扩展已实现、注入隐式产品默认值来假装兼容。

**六槽目标补充（同日维护者澄清）**：最小角色的通用运行涵盖 Memory / Emotion / Event / Prompt / LLM / Agent 六槽的基础消费，不止 Prompt / LLM 文本回复。同一份最小定义在发行版更换记忆或情绪实现后仍可使用，不要求角色作者为每种实现改包或补丰富字段。六槽是否参与某次操作及其真实依赖由 Host 决定，不强制每次调用全部六槽或按固定顺序运行；所选实现仍须支持本次用途并有合法资源。产品的关系、七维人格等扩展不可用，不能代替缺失的 Base 消费；反过来也不把 Base 报告解释为这些产品状态。可返回的完整角色上下文保留于 §0.13 的兼容切换，不改变这条六槽目标。

| 接入情况 | 基础闭环与扩展边界 |
|----------|--------------------|
| 最小内容合格，Host 的基础 Prompt / LLM 可用 | Host 准备内容与技术身份，调用能力并将结果交给调用方；无关系/人格扩展不应单独阻断基础文本交互 |
| 可选扩展缺失、未映射或当前不可用 | 发行版明确标注该扩展不可用；不填中性分数、默认关系或假输出冒充扩展执行，不暗示视觉资产已被渲染 |
| 调用方明确要求尚不支持的扩展，或基础能力本身不可用 | 如实返回对应不支持/不可用结果；不丢弃真实要求以获得成功，不承诺缺模型、权限或资源时仍能生成 |

这是各发行版应兑现的接入目标，**不是当前所有发行版已经通过验收**。§0.7–0.8 是能力与独立 Host 的有限实现；§0.9 已为参考 Rust Host 增加基础文本调用，旧丰富入口仍有完整 `Role` 与丰富回复 DTO 耦合，不能仅给旧返回字段填值就宣称最小闭环已接通。逐发行版的转换、基础路径和扩展不可用表示，随 D-CLI-BLUEPRINT-05 的实际切片分别落实。

**后续主流程目标（2026-10-04 确认）**：最小角色应进入发行版自己的主交互流程，通用适配由项目共同维护的共享运行库承担，放在小 Kernel 外；不另建只供演示的产品会话作为默认目标。小 Kernel 与六槽职责保持原边界，Host 仍绑定能力与资源、提供本次材料并应用结果，但不必为基础交互补齐 ChatPro 的丰富角色字段。共享适配只承诺其公开的基础操作；“任意六槽”限于契约合规、满足本次基础用途且实际可用的实现，不保证任意要求、缺资源或不支持的能力也能成功，不把六槽固定成六阶段。主流程不自动意味着历史、恢复、真实媒体或所有产品扩展已有实现。

生命周期仍需要技术标识与命名空间；适配/加载边界可提供内部角色句柄和传输版本信息。这些是技术封装，不增加作者侧必填内容，也不要求采用发行版的 `meta.id/name/version`。磁盘文件名、传输 schema 与视觉资产描述/解析规则由发行版适配器确定；CLI 的显式文件参数不冻结跨发行版格式。若将来确有跨发行版交换包需求，再独立提出版本化格式，不反向扩张当前最小数据契约。

```text
最小角色内容 / 发行版角色包 ──开发者转换器──> Host 角色表示与能力输入
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

### 0.9 参考 Rust Host 的增量基础文本入口

[`PreparedMinimalRole`](../kernel/crates/oclive_kernel_host/src/service/role/minimal.rs) 接收开发者已转换的定义、逐引用非空资产字节与 Host 技术 ID，或适配 §0.5 已读取的本地快照；构造时复用共享校验，字段私有，资产数与定义一致，持有自己的快照。它不规定引用 scheme、文件名或统一转换器协议，也不构造完整 `Role`、激活选角或写产品运行态。资产可供发行版消费者读取，但准备成功不表示媒体已解码或渲染；字节预算和转换真实性由提供者负责。

参考 [`OcliveKernel::process_minimal_message`](../kernel/crates/oclive_kernel_host/src/role_kernel.rs) 把该句柄与 `MinimalRoleMessageRequest { user_message, requirements }` 薄转发到现有 [`process_message.rs`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) 内的基础分支：共享 Prompt Base 整理人设与输入，沿用 Host 当前模型设置及已装配 `LlmClient` 的 `generate_with_opts`，一次非流式生成。不会新建第二套角色/六槽解析器，也不会把丰富 pipeline 的全部阶段当作基础文本前置。旧 `process_message`、RoleCache、`load_role`、收据和丰富 DTO 保持原调用面。

[`MinimalRoleMessageResponse`](../kernel/crates/oclive_kernel_types/src/models/dto/minimal_role.rs) 只提供技术 `role_id`、模型原样 `reply` 与 `product_extensions: unavailable`。这里表示**本基础路径未执行**关系、人格、情绪等产品扩展，不表示 Kernel Base 不可用或 Host 其他路径失效。不存在假中性分数、关系状态或聊天行 ID。非空额外要求经 Prompt 返回 typed `Unsupported`，在模型前停止；空消息返回原 `EmptyMessage`；模型或设置失败保留原 AppError，不返回安全话术、不由本编排新增重试。正常完成的空文本仍保留为空；质量与领域成功不能由返回字符串非空代替。

**当前证据与止点**：五项新增外部 crate 回归经生产 Host builder（临时 SQLite、内存模型）覆盖准备/一次调用、无伪造扩展与无角色/聊天/收据落库、拒绝要求/空输入、模型失败/已完成空输出及本地快照适配；既有丰富门面的用户、流式与事件授权回合回归通过。这里只是**嵌入式 Rust 单次文本入口**，没有最小角色的历史、多轮记忆、持久化恢复、流式传输、HTTP/Tauri 命令、ChatPro UI、视觉或真实模型/语音验收。Host 调度与资源策略仍由已装配客户端和调用方承担；Prompt Base future 不保证 `Send`。D-CLI-BLUEPRINT-05 保持 Partial，后续按实际发行版接入需求选择下一片，不为穷尽这些边缘路径扩证。

**生成库消费入口**：CLI 已链接的 `library` 直接重导出上述准备句柄、基础请求/结果、扩展状态与错误原类型，生成 README 和 rustdoc 分列基础与丰富路径；用法见 [CLI 指南](../creator-docs/cli/OCLIVE_CLI_GUIDE.md#生成物说明)。不新增包装回合、转换器协议、角色包格式或运行依赖；未链接的 serde stub 仍不可调用 Host。生成库调用仍限于本节的嵌入式基本文本，不表示 HTTP/Tauri/UI 接通。

### 0.10 共享消费者：复用准备并选择 Prompt

[`MinimalRolePromptConsumer`](../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs) 借用已校验的最小逻辑定义和调用方选择的 `&dyn PromptBase`。每次显式调用添加 §0.7 已有的人设/材料标题，人设字节原样作为独立片段，当前材料逐片保留字节、顺序、重复项和空项，`requirements` 原样交给所选 Prompt。Host 必须选择能满足当前用途的实现；消费者不清空要求，也不保证任意实现能处理要求。所选实现的正常空文本与完整错误直接返回，只有一次能力调用，无 fallback、重试或输出修补。

这是一份可复用的内容准备与消费实现，不是新的六槽 contract，也不是统一角色包磁盘格式。逻辑定义可来自开发者转换或已有本地准备；资产仍归 Host/Adapter 持有。它不读取文件、不构造完整 `Role`、不补扩展默认值，不驱动其它槽位或持有 Host 状态。六槽与 Host 的职责只维护于 [MODULE_MAP §3.1](MODULE_MAP_AND_HANDOFF.md#31-三层解耦)。这里的“无损”限于已有材料/要求和所选实现的结果/错误；新增标题是明确的准备规则，不表示原请求未增加任何片段，不承诺零分配/延迟，或丰富扩展被投影到 Base 后仍全部保留。

§0.7 的 `MinimalRolePrompt` 复用同一准备逻辑并选择原 `LiteralMaterialAssembler`；原输出字节、非空要求的提前 `Unsupported` 及完整错误说明保持不变，旧类型的 `Send/Sync` 属性也保留。新消费者沿用 Base 的本地 future，不增强线程或取消承诺。[外部 crate 测试](../kernel/crates/oclive_kernel_runtime/tests/minimal_role_consumer.rs) 验证 Host 自选 Prompt、额外要求与片段保持、全部 typed 失败/空结果原样传递、一次调用及真实 Pending 后的借用有效性；原 Prompt、独立 Host 示例和参考 Host 基础入口用于兼容回归。它不证明各发行版都已装配这层、所有六槽必须调用，或真实模型/媒体、HTTP/Tauri/UI 已接入；原债仍按实际发行版缺口保持 Partial。

**参考 Host 的显式选择入口**：[`OcliveKernel::process_minimal_message_with_prompt`](../kernel/crates/oclive_kernel_host/src/role_kernel.rs) 接收 §0.9 的准备句柄、原基础请求和调用者选定的 `&dyn PromptBase`。所选实现应消费准备后的材料，不应已配置同一角色的人设；共享消费者添加一次人设，并原样交付 `requirements`。旧 `process_minimal_message` 继续使用原 Literal 协议，不记住前一次选择。两种入口共用同一基础编排、当前模型设置、单次模型调用与基础结果；空用户输入先拒绝，Prompt 失败时不调用模型，完整错误、正常空输出与原模型失败均不改写。调用者仍负责能力是否满足用途及本地异步调度；没有全局注册、重试或产品扩展状态。

**独立 Host 的用法**：[`minimal_role_host` 示例](../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs) 的 `reply_with_prompt` 复用同一消费者，展示 Host 按次选择“逐片 JSON 引用”的能力。其有限要求属于示例，JSON 解码后的人设和材料内容、顺序逐片保持，不声称输出字节不变、防注入或 Kernel 已定义统一要求语言。原本地/纯内存来源、技术身份校验和默认 Literal 路径保持。参考 Host 的九项外部回归包含原五项兼容用例；独立示例六项及 native 运行通过。证据限于内存能力与隔离数据，不扩大 §0.9 的产品/传输范围；公开 Host 用法的 `no_run` rustdoc 仅证明编译。

**核心外共享基础文本操作**：同模块的 `MinimalRoleTextConsumer` 借用最小定义和调用者绑定的 `PromptBase` / `LlmBase`，复用原 Prompt 消费者，正常完成后将其输出逐字交给 LLM 一次，再原样返回正文。`MinimalRoleTextError::Prompt` / `Llm` 分辨失败阶段并保留完整 BaseCallError；Prompt 失败时模型零调用，不自动 retry/fallback，也不以正常空文本判定质量成功。非 Send 的本地异步实现可用，输入持有到 LLM 完成，调度/取消及外部效果仍由原契约与调用者负责。它不拥有角色身份、资产/状态/存储或资源授权，也不规定其它槽调用。独立示例的显式选择分支已使用这个操作；示例自己的窄错误表示只展示原因 kind，公共操作仍保留全部错误。八项外部回归包括旧四项，并验证两阶段完整失败、调用账、正常空值、逻辑拒绝及两个 Pending 边界；这不表示参考 Host 的旧 LlmClient 已迁移为 LlmBase，或 ChatPro 主流程接线已完成。

### 0.11 主流程接线的基础文本传输

参考 Host 提供受保护的 `POST /chat/minimal`；桌面注册 `send_minimal_message(req)`，经已有 `ChatBackend::Http` 与带默认鉴权头的 Rust 客户端调用。shared 的 [`sendMinimalMessage`](../distros/shared/src/api/chat.ts) 使用这个 IPC 命令，不从渲染层 fetch、获取令牌或换发旧 `/chat`。这是后续主聊天消费者的传输接口，**不是另一个基础会话产品**；本片没有接选角或 composer，也没有真实 IPC / TCP / webview 运行证据。

[`MinimalRoleLocalMessageRequest`](../kernel/crates/oclive_kernel_types/src/models/dto/minimal_role.rs) 分为 `source { role_id, asset_root, definition_reference }` 与 `message { user_message, requirements }`。这是参考 Host 的可选本地适配封装：开发者转换器显式提供技术身份、绝对资产根和相对定义引用；不要求统一文件名、产品关系 / 人格 / 元数据或装配蓝图。可选当前会话候选的增量字段另见 §0.16。来源与消息都拒绝未知字段，不能把 adult、scene 或收据身份混入以暗示这些能力已经执行。Host 复用 §0.5 的有界读取，采用 64 KiB 定义、4 MiB 单资产、16 MiB 总资产预算；预算属于这个 Host，不能扩大成通用最小内容合同或实际媒体有效性承诺。

canonical [`process_minimal_local_message`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) 负责调用准备与 §0.9 的既有基础编排，API 只转发结果。阻塞读文件与局部非 Send Base future 在 Host 的 blocking worker 中执行，借当前 Tokio Handle 获得 I/O；不改变六槽 future 的约束。没有当前 runtime 时返回 Host 错误，不在外部调用面制造 panic。原文只检查全空白，不裁剪人设、有效消息或正常模型输出；不注册丰富角色、不初始化产品状态、不创建聊天行 / 收据，也不自动重试或恢复。

这个默认传输绑定 Literal Prompt，只接受空 `requirements`；非空要求在 I/O / 模型调用前按本端点的输入约束返回既有 `INVALID_PARAMETER`，而不把它清空或将这个端点的限制推广到所有 Prompt。原进程内可选 Prompt 仍返回完整 typed Base 错误；本片未建立通用 BaseCallError 的 wire 映射。基础结果仍只含 `role_id`、原样 `reply`、`product_extensions: unavailable`；既有 Host 错误经现有 HTTP / IPC 错误链传递。主聊天的状态消费和实际接线见 §0.12 / §0.13；历史、幂等恢复、流式和媒体不由这个结果承诺。

### 0.12 主聊天可消费的临时最小状态

shared 的 [`useMinimalRoleChatStore`](../distros/shared/src/stores/minimalRoleChatStore.ts) 提供显式来源绑定、临时气泡和一次基础发送。它供发行版主聊天接线复用，**没有新增另一个会话产品**；ChatPro 现有选角区 / composer 的实际调用见 §0.13。`bindSource` 复制并冻结三个来源字段；这是临时输入绑定，不是文件校验、Host 角色激活或扩展发现。Host 仍在每次调用时按 §0.11 权威检查文件 / 资产与预算；角色文件可能在两次调用间改变，不宣称已锁住内容快照。

最小状态不构造 `RoleInfo`，也不接入旧 chatStore 的 DB / IDB / persist 链。正文和正常空模型输出原样成为 `ChatMessage`；气泡 / 回合标识用 `minimal-local-*` 明确表示客户端本地身份，不当作后端行 ID。已完成气泡现在同时作为 §0.16 的当前会话候选；这不表示持久历史或长期记忆。重新绑定清空会话；空白输入或无绑定先拒绝，失败移除在途用户气泡并保留原错误，无普通发送 / 恢复 / 重试。返回身份 / unavailable / 正文类型异常时拒绝消费，不能用假产品状态补齐响应。

取消、新发送和重绑定使旧调用失效，旧成功 / 失败不会覆盖新气泡、加载状态或最终事件。取消只停止客户端展示，非流式 IPC 与 Host 生成仍可能继续；本片没有服务端取消协议。提交 / 最终事件复用真实 hostEventBus，并显式 `skip_auto_tts: true`；既有语音提交消费者也尊重该标记，避免加载配置、角色语音档案和预热媒体资源，原未标记的语音行为保持。扩展仍表示 unavailable，不发送 fake emotion / relation / scene 或 stream 字段。

本共享状态的原证据是实际 Pinia / mitt、IPC 替身与语音消费者的内存验证；主流程控件 / 列表联动另见 §0.13。两层都不代表真实 webview、音频、文件读写或所有发行版验收。

### 0.13 ChatPro 两套主界面的基础接线

Fluent / Tool 的主选角区共用 [`MinimalRoleSourceControls`](../distros/shared/src/components/role/MinimalRoleSourceControls.vue)。填写转换器准备的**绝对资产根**与**相对定义引用**，绑定后继续用原主聊天输入框与消息列表；每次绑定由前端生成新的技术身份，不要求作者提供展示名 / 关系 / 蓝图，也不规定定义文件名。绑定只保存临时来源，第一次发送才由 Host 检查可读内容和所选模型。界面明确显示基础文本及产品扩展不可用；停止等待仅丢弃客户端气泡 / 最终事件，Host 可能继续生成。可返回原完整角色，不需要将基础结果伪装成丰富 RoleInfo。

[`useMinimalRoleSelection`](../distros/shared/src/composables/useMinimalRoleSelection.ts) 先校验来源字段，再取消旧客户端发送与当前成人队列；取消失败不切换上下文。在过渡期间主发送守卫拒绝新回合；返回 / 卸载 / 更晚选择使等待取消中的绑定失效。原 `currentRoleId` / RoleInfo 保留为可返回的完整角色上下文，`minimalRoleActive` 只表示前端主聊天选择，**不表示丰富 Host 后台服务已停机或最小角色已被丰富激活**。这不是后端角色生命周期迁移。

[`useMainShellChat`](../distros/chat-pro/src/composables/useMainShellChat.ts) 与 shared `useChatSend` 将最小发送 / 消费接到 §0.12 状态；已完成临时气泡都可见，列表历史分割为 0，下次请求仍只带当前正文。关系 / 场景 / 立绘 / 人格 / 成人 / 插件工具 / 语音 / 历史及角色设置不可用；模型管理保留为 Host 资源入口。丰富面板与工具停用，关联快照轮询、插件角色变更 / ASR、语音预热和设置 / 录音热键也检查当前范围。已经按住的录音只结束一次；旧角色包主题清除，返回时恢复，不将其属性展示成最小角色能力。

主流程联动证据来自真实 Pinia / mitt、实际来源表单 / composer / 消息列表及共用 hook，只有 IPC 和浏览器缺失的 ResizeObserver / matchMedia 等环境能力用内存替身。scope 回归另核实际轮询守卫、角色事件与热键消费者，真实按键注册器验证停用后释放按住状态并能再次启用；语音仍为既有内存消费者测试。两套壳通过类型与构建检查；这不算真实桌面 / 联机模型 / 音频、持久化恢复或所有发行版验收。足以落实参考发行版的基础主流程接线，不把未验媒体 / 平台扩成这一片必须穷尽的清单，父债继续 Partial。

### 0.14 六槽可替换的最小角色消费

共享运行库在既有消费者模块提供 [`MinimalRoleBaseBindings` / `MinimalRoleBaseConsumer`](../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs)。调用者显式绑定六个 Base 实现；构造时复用唯一最小定义校验，借用定义与能力，不构造完整 `Role`、读取资产或补产品默认值。所选 Prompt 必须消费尚未添加本角色人设的片段；共享消费者复用 §0.10 的既有人设准备。其它五槽接收调用者按对应 Base 形状提供的原请求，不向每个槽隐式塞人设、状态包、身份或权限。

六种方法仍是独立的 Base 调用：Memory 只接本次候选材料与查询，Emotion / Event 接本次材料与可选背景，Agent 接合法委托的任务与背景，LLM 接已经准备的输入。LLM 不自动调用 Prompt，Event 不自动调用 Emotion；各槽只调用绑定的那个能力一次，完整返回正常结果或原错误，不重试、修补或推断领域效果。共享消费者不提供固定六阶段、存储或权限机制；Host 仍负责材料来源 / 命名空间、资源授权、调用依赖、异步调度和结果应用。具体请求 / 结果语义以 [MODULE_MAP 的六槽契约](MODULE_MAP_AND_HANDOFF.md) §0.6 为准。

[外部 crate 回归](../kernel/crates/oclive_kernel_runtime/tests/minimal_role_six_slots.rs)使用同一份最小定义，分别绑定生产关键词 Memory / Emotion 和顺序不同的关键词检索 / 保留材料主体的内存 Emotion。两组都实际消费六槽，并让检索与分析材料进入最终 Prompt / LLM；调用关系不同，人设只准备一次。五项测试另核请求原值、独立调用、正常空值与五类完整失败、逻辑拒绝零调用、本地非 Send 的 Pending 借用；旧消费者八项回归保持。这是可复用六槽接入面的局部证据，不是第二个发行版实接或模型质量保证。

**可运行的原生装配案例**：既有 [`minimal_role_host`](../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs) 在原文本 / 自选 Prompt 用法之外，增加 Host 自己组织的六槽操作。本地文件与内存转换内容保留同一最小定义及资产；分别绑定 KeywordMemoryBase / QueryMemoryRetrieval，以及 KeywordEmotionBase / BuiltinUserEmotionAnalyzer 的 Base 入口。后两种 Emotion 类型复用同一分析 core，不宣称两个独立情绪算法。Prompt 选择 BuiltinPromptAssembler 的 Base 视图，Event 选择不依赖丰富人格的 LlmEventAnalyzer，Agent 显式承接其支持的 Unicode 标量值计数。它们的实际检索、分析、报告都进入最终模型输入，未填完整 Role 或产品状态。

这个案例的 Event 协议生成器与正文 LLM 是内存替身，**每条六槽路径 2 次假生成**（分析 1 + 正文 1），不是只调用一次模型；native 示例另保留原三次基础调用，整个演示共 7 次假生成。该 Host 选择无额外 Emotion 背景、将情绪报告作为 Event 背景，并委托纯计算任务；这些是有限操作的私有输入安排，不成为其它 Host 的固定次序 / 任务 / 格式。原六项加新三项案例测试验证实际装配、技术身份拒绝及 Event 格式 / Agent 任务 / Prompt 要求失败；后续失败不会抹掉已发生的分析，未声称回滚或零副作用。案例的旧窄 HostError 只展示原因 kind，共享消费者仍保留完整错误。

**当前止点**：生产参考 Host / ChatPro 目前仍通过 §0.9–0.13 的基础文本路径，当前会话 Memory 的生产消费见 §0.16；Emotion / Event / Agent 的产品材料来源 / 任务策略仍分别处理。参考 Rust Host 的真实正文模型可按 §0.15 显式绑定给消费者，不用四槽空值或“扩展不可用”冒充运行。保留小 Kernel、原六槽接口、旧丰富生命周期与可返回上下文；本片不把 D-CLI-BLUEPRINT-05 改为 Done，也不扩大为全部发行版 / 媒体 / 崩溃窗口调查。

### 0.15 复用参考 Host 已装配的正文模型

可信 Rust 集成方可以调用 [`OcliveKernel::text_generation_base()`](../kernel/crates/oclive_kernel_host/src/role_kernel.rs)，把借用的 `LlmBase` 交给共享文本 / 六槽消费者或自己的独立调用。构造不发请求；poll 时沿参考 Host 当前用户模型设置和已装配客户端的 `generate_with_opts`，options 保持 None。准备好的输入、正常空回复及正文原样保留，不自动添加人设。与原基础消息入口共用 canonical 正文 helper，不另建 Ollama 客户端，因此实际客户端已有的资源 / 授权包装和内部策略仍在；一次适配调用只调该客户端一次，**不保证客户端内部只发一次 provider 请求**。

Base 视图按 typed Host 错误投射：`HighRiskCapabilityNotGranted` / `RemoteServiceUnavailable` 为 Unavailable，其余 Failed，原诊断保留。此边界的 AppError 没有 typed 超时 / 取消来源，不从文本猜 TimedOut / Cancelled；旧基础入口仍直接保留原 Host 错误。没有适配层额外重试、预热、聊天 / 角色落库或全局六阶段；原用户设置同步及其环境更新保持，由 Host 承担，不归小 Kernel。借用能力需要调用方调度本地 future，drop 不证明 provider 已停止。

[参考 Host 公共 API 回归](../kernel/crates/oclive_kernel_host/tests/minimal_role_public_api.rs)实际使用生产 builder 和内存 LlmClient，把同一准备句柄与原生 Memory / Emotion / Event / Prompt / Agent 绑定给共享消费者，再消费 Host 的这个 LLM 视图；检索、分析和显式有限任务结果进入最终输入，人设一次。本测试显式选择 Event 分析和正文共两次假生成，不改变 ChatPro 的默认调用数，也不声称真实模型能满足 Event 协议或正文质量。其它回归核原输入 / 空输入、正常空回复、构造零调用、失败完整诊断与旧入口保留原 AppError；这个 LLM 绑定本身不替代其它槽的实际材料 / 任务接线。

### 0.16 当前最小会话材料的 Memory 消费

维护者选择当前临时会话，**不设计持久记忆身份，不读取旧完整角色记忆**。主聊天 store 只快照当前绑定已完成的 user/reply 对，先移除旧在途用户气泡，再在本轮 submit 事件前取值；失败、取消、晚结果和其它绑定不能进入候选。正常空回复仍是完成对。两套主 composer 沿同一个共享 store 发送，返回完整角色仍保留旧丰富上下文，但不将其拷给最小会话。

参考 Host 以新增 `MinimalRoleLocalConversationRequest` 承载可选 `conversation: [{ user_message, reply }]`，缺省为空且序列化空值省略；明确 null 或未知字段拒绝。旧 `MinimalRoleLocalMessageRequest` 保留原两个字段及 Rust struct literal 用法，旧本地入口仍可调用，通过 `From` 转成空候选的新封装；HTTP 与原 IPC 命令接收增量封装，旧 wire 载荷仍有效。它是调用者提供的引用文本，不是存储、身份、权限、收据或服务端落库证明，也不是最小角色作者的新字段。最多最近八个完整对话对、原正文合计 UTF-8 64 KiB；前端取预算内连续的最近后缀，保留原字节和发言人，不切断文字或跳过过大最新对拼接旧材料。Host 独立检查预算，超限先于资产读取 / 模型调用返回 INVALID_PARAMETER。当前用户消息与人设仍走原输入 / 资产规则，不被计作“已完成会话”。

Host 在 canonical 本地入口显式绑定已有 `QueryMemoryRetrieval` 的 Memory Base，以本轮用户原文查询候选。每个候选保留 prior user / assistant 的引用标签；选中整对原文进入共享 Prompt 人设 / 材料准备，然后使用 §0.15 的同一正文 helper。此检索是既有有限词项规则，不承诺语义召回、持久记忆或防注入；正常无命中直接沿原基础 Prompt，旧无字段载荷也保留原 Prompt 字节，不读取 rich MemoryRepository。正文仍一次客户端调用，不额外启用 Emotion / Event / Agent 或模型分析。

IPC / HTTP 沿 §0.11 的已鉴权薄转发，`product_extensions: unavailable` 仍说明丰富产品扩展，不代表已调用的 Memory Base 不可用。旧客户端可省略字段；新会话字段需配套的新 Host，旧 Host 若拒绝则保留错误，不能删除材料、换 rich 发送或重试来假装兼容。实际 HTTP 回归核配对文本、主体 / 否定保持、未命中、旧载荷、预算及零 rich 行；真实 store / 主列表 / IPC 对象回归核第二轮、取消失败、切换、快照与最近后缀。模型和 IPC 为内存替身，没有真实模型质量、桌面进程或持久恢复验收。参考生产路径目前接通 Memory / Prompt / LLM；其余槽继续按合法材料 / 任务分别处理，不把这个切片称为全六槽迁移。

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
