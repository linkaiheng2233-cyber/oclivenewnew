# RFC：OCLive Scaffold Package v1

> **状态（2026-08-01）**：Stage 2A 发现契约已冻结，Stage 2B 受限声明式生成契约已冻结；实现必须保持 CI 与脚手架相互独立。本文是 Scaffold Package、发现顺序、来源锁定、命令命名空间、生成事务和兼容策略的 SSOT。CI 影响规划仍以 [`SOMEDAY_TOOLCHAIN_CI.md`](../roadmap/SOMEDAY_TOOLCHAIN_CI.md) 为准。

## 1. 定位与硬边界

Scaffold Package 是面向开发者的本地**指令与生成声明包**，可携带说明、生成规则、默认值和带命名空间的命令声明。它帮助创建或初始化工程，但不证明产物正确，也不替代领域契约。

CI 是治理层。任何 Scaffold Package 均不得声明、覆盖或修改：

- `oclive ci` 官方命令；
- 主仓 workflow、验证器坐标、Runner、Secret、缓存、并发、超时或门禁强度；
- CI 影响传播算法或“跳过哪些 Job”的决策。

脚手架生成的文件仍由 CI 重新分析。Stage 2B 只执行本文定义的本地声明式文件物化，不执行第三方脚本或内置命令代理，也不开放市场、联网安装或组合运行时。

## 2. 现状审查与命令收口

2026-08-01 对 `oclive-cli` 和仓内生成路径的只读审查得到以下事实：

| 入口 | 当前职责 | Stage 2A 处理 |
|------|----------|---------------|
| `init` + 五个 `--template` | 生成内核/库工程 | 保留为稳定领域入口；官方 kernel scaffold 只引用它 |
| `plugin create` | 生成 directory / remote 插件 | 保留为稳定领域入口 |
| `pack create` | 生成基础角色包 | 保留为稳定领域入口 |
| `scripts/scaffold-ui-slot-plugin.mjs` | 生成 UI-slot 插件 | 先登记为官方生成器；本轮不搬迁或自动执行 |
| `template create` / `pack` | 反向打包旧 `.oclive-template.tar.gz` 工程归档 | 改称 legacy project archive；保持可调用但从默认帮助隐藏 |
| `init --template-url` | 下载并展开旧工程归档 | 保持兼容；不等同 Scaffold Package，也不接入新发现链 |
| `ci init/check/plan/explain` | 官方 CI 生成、检查和影响规划 | 保留官方专属；第三方 namespace 永远不能覆盖 |

审查时默认帮助直接展示 **25** 个顶层命令，其中 **10** 个实际要求 `--experimental`，导致稳定面与试验面混排；`template` 还与“内核配方”和新 Scaffold Package 共用“模板”一词。Stage 2A 增加 `scaffold` 后，默认帮助只展示稳定入口；现有试验命令保持可调用但隐藏，避免破坏脚本。

### 2.1 默认可见的官方命令

`init`、`dev`、`pack`、`doctor`、`plugin`、`registry`、`lint`、`profile`、`config`、`ci`、`scaffold`、`kernel`、`explain`、`migrate-app-data`、`completions`。

### 2.2 保留但隐藏

- 试验命令：`build`、`bench`、`blueprint`、`compose`、`debug`、`dashboard`、`learn`、`test`、`market`、`collab`；仍须 `--experimental`。
- 兼容入口：`template`；只处理旧工程归档，不代表 Scaffold Package。

隐藏不是删除。真正移除仍须遵循 breaking-change 流程并提供迁移期。

## 3. 文件与发现顺序

包清单文件固定为 `oclive.scaffold.json`。发现源为：

1. 项目级：`<project>/.oclive/scaffolds/*/oclive.scaffold.json`，随项目维护；
2. 用户级：`<OCLIVE_HOME>/scaffolds/*/oclive.scaffold.json`，由本机用户维护；
3. 官方内置：随 `oclive-cli` 编译发布，始终作为默认兜底。

默认优先级是 `project > user > official`。用户级 `<OCLIVE_HOME>/scaffold.config.json` 提供个人默认，项目级 `<project>/.oclive/scaffold.config.json` 覆盖同名设置；命令行仅可覆盖本次只读解析。配置可调整 `source_order`、为具体包固定来源或显式启停包。

高优先级同 ID 包覆盖低优先级包；同一来源内出现重复 ID、配置指定的来源不存在、清单损坏或越界路径时必须明确失败，不静默猜测。项目包不得逃逸项目根，用户包不得逃逸 `OCLIVE_HOME`；符号链接逃逸同样拒绝。

## 4. v1 清单最小职责

`oclive.scaffold.json` 仅承载以下字段组：

- `package`：反向域名 ID、独立 SemVer、显示名、说明与维护者；
- `compatibility`：支持的 `oclive-cli` 与 scaffold contract 版本范围；
- `command_namespace`：命令声明的唯一命名空间；
- `generators`：内置驱动或本地指令/生成规则入口的声明；Stage 2B 可执行的本地 `instruction` 必须同时固定指令文件 SHA-256；
- `commands`：名称、说明、入口和请求权限；Stage 2A 只展示，不执行；
- `defaults`：包级默认配置；
- `dependencies`、`extends`、`composition`：为以后组合预留，Stage 2A 保留并诊断，不解析或运行；
- `extensions`：第三方命名空间扩展外壳；未知 required 扩展拒绝，未知 optional 扩展保留并告警。

第三方包不得使用 `com.oclive.*` 命名空间，也不得声明 `ci.*` 能力。V1 可声明的本地权限必须来自有限集合，例如项目读写、用户配置读取、环境读取、进程启动和网络访问；实际执行阶段仍需单独设计确认与沙箱，声明本身不授予权限。

## 5. 兼容与迁移

- Scaffold Package 版本与 OCLive 产品版本独立。
- 新版读取器须读取其支持范围内的旧版 v1 包；包自己的 SemVer 不改变清单 schema。
- 旧读取器遇到更高 `schema_version` 不做反向兼容猜测，必须拒绝并给出升级/迁移提示。
- `compatibility` 不满足时拒绝激活；不得以警告代替硬失败。
- v1 的 `dependencies`、`extends`、`composition` 只是稳定字段形状；不得据此宣称组合已实现。

## 6. 来源记录与锁文件

解析结果可写入 `<project>/.oclive/scaffold.lock.json`。锁文件必须确定性记录：

- 有效发现顺序和读取器版本；
- 每个选中包的 ID、版本、来源类型、相对 locator、维护者和清单 SHA-256；
- `official` 或 `untrusted_local` 信任分类；
- 请求权限与命令命名空间；
- 未执行的依赖/继承/组合声明及诊断。

锁文件不是信任授权，也不是 CI 通过证明。清单内容或来源变化会改变摘要；写锁必须显式请求并采用原子替换。

## 7. 信任提示

项目级和用户级包统一视为 `untrusted_local`。列出、检查或解析时必须显示来源、维护者、作用域、请求权限及以下事实：

- 第三方自行开发和维护，OCLive 不为其行为或兼容性背书；
- Stage 2B 仍不执行其命令；受限生成也必须逐次显式确认不可信来源；
- 包不能控制 CI、Runner、Secret 或门禁；
- 官方包是兜底，不会因第三方包存在而被删除。

## 8. Stage 2A CLI

`oclive scaffold` 首轮只提供：

- `list`：列出候选、选中来源、遮蔽关系和信任提示；
- `inspect <id>`：查看解析后的包、命令和权限；
- `validate <path>`：严格校验单个 v1 清单；
- `resolve`：生成确定性解析报告；仅 `--write-lock` 时写锁文件。

Stage 2A 不提供 `add/remove/update/install/run`，不联网，不执行第三方 entrypoint，不解析组合图，也不替换 `init` / `plugin create` / `pack create`。

## 9. Stage 2B 受限声明式生成

Stage 2B 新增 `oclive scaffold generate <package-id> <generator-id> --output <new-directory>`，但它不是通用命令执行器。生成链只接受本地 `instruction` 驱动，并遵守以下硬边界：

1. 清单中的 `instruction.path` 必须是包内相对路径，`instruction.sha256` 必须是该指令文档的 64 位小写 SHA-256；缺失摘要的旧 v1 包仍可发现，但不能生成，并提示把兼容范围提高到 `>=1.1,<2` 后补摘要。
2. 指令文档使用独立 `schema_version: 1`，只声明字符串变量和文件映射。每个源文件必须声明 SHA-256；`text` 模式只支持精确 `{{variable}}` 替换，`copy` 模式逐字节复制。没有条件、循环、表达式、include、shell、网络或生命周期 hook。
3. 变量优先级为 `--set key=value` > 清单 `defaults` 中的字符串值 > 指令变量默认值。未知变量、非字符串清单默认值、未提供的必填变量和未知占位符均硬失败；生成凭据只记录变量名，不记录值。
4. 包必须声明 `project.write`。项目级和用户级包还必须同时满足：当前 `.oclive/scaffold.lock.json` 与已解析包的 ID、版本、来源、locator 和清单摘要完全一致；本次命令显式传入 `--accept-untrusted`。该确认只授权本次受限文件写入，不授予包声明的进程、网络、环境或用户配置能力。
5. 包根、指令和所有源文件都必须规范化后仍位于已选包根内；符号链接逃逸、绝对路径、`..`、重复目标及文件/目录冲突全部拒绝。
6. 输出目录必须不存在，父目录必须已经存在。实现先完成全部读取、摘要验证、变量解析和内存渲染，再写入同一父目录内的临时树，最后用目录重命名一次落位；任何失败都不能留下半成品。Stage 2B 不提供 `--force`，不修改已有工程。
7. `--dry-run` 完成同等校验和渲染规划但不写盘。成功落位时自动写入 `.oclive/scaffold.provenance.json`，记录包来源、维护者、清单/指令摘要、生成器 ID、使用的变量名以及每个产物的路径与摘要，不记录时间戳或变量值。
8. 官方 `builtin` 驱动继续指向既有 `init` / `plugin create` / `pack create` 等领域入口；`scaffold generate` 只返回准确的委托提示，不重复实现官方生成逻辑。

Stage 2B 仍不提供 `add/remove/update/install/run`，不执行 `commands[].entry`，不解析 `dependencies` / `extends` / `composition`，不联网，也不允许 Scaffold Package 控制 CI。`scaffold generate` 的产物必须继续通过普通 CI/本地验证，生成凭据不是质量证明。
