# Kernel 契约基础工程脚手架（2026-10-11）

**状态：Locally verified。** 本片提供现有公共契约的开发工程，没有交付独立可启动的小 Kernel。controller 直接实施及语义自查，`independent=false`；正式远端 CI 未发起，main 未合入或推送。

本报告是交接快照，不另定义模块或脚手架规则。边界 SSOT 见 [MODULE_MAP](../../MODULE_MAP_AND_HANDOFF.md)，脚手架规则见 [Scaffold RFC](../../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)，使用入口见 [CLI 指南](../../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)。执行范围见 [本轮计划](../ROUND-02-PLAN.md#kernel-基础脚手架--现有契约开发工程2026-10-11)，变动见 [DCL-87](../DEBT_CHANGELOG.md#dcl-20261011-87--kernel-契约基础工程脚手架)。

## 1. 为什么做这一片

维护者希望先分开 Kernel 基础与变化的 Host，脚手架只准备开发工程，模块替换由作者自己实现。原五个模板仍是参考运行配方；即便原 `library-embed`，也会引用参考 Host/runtime。不能把它改名后当作无 Host 的开发基础。

用户进一步确认本片只用现有契约，不要求提取新的可执行小 Kernel 核心。因此新增显式 `kernel-base`，保持旧默认不变。例如作者只想编写一个 Memory Base，不再需要生成角色、数据库、HTTP 服务或其它槽的默认实现；仍可从原公共类型实现契约，没有新强制义务。

基线：`2b0390cdecaeed428f9ea1f753098f4aa69f3ebd`。独立分支：`codex/kernel-base-scaffold-20261011-r0`。工作树：`E:/OCLive/_worktrees/kernel-base-scaffold-20261011-r0`。相关磁盘原件及初始身份已保存在本机 ignored `.cursor/plans/kernel-base-scaffold-20261011-r0/baseline.json` 与 `originals/`；它们不随 Git 交付。

## 2. 使用与实际产物

在含本片的 OCLive checkout 根目录执行，输出选择尚未使用的目录：

```bash
cargo run -p oclive-cli -- init --non-interactive --template kernel-base --kernel-source . --project-name my-capabilities -o ../my-capabilities
cd ../my-capabilities
cargo test
```

初次生成精确五个文件：

```text
my-capabilities/
  Cargo.toml
  src/lib.rs
  tests/base_contracts.rs
  README.md
  .gitignore
```

`Cargo.toml` 为独立 Cargo workspace，直接依赖现有 contracts/types 的本地路径。`src/lib.rs` 是作者入口，不提供 SDK 重新导出层。测试内 Memory 实现只做本次材料的字面筛选，并保留 `Unsupported` 的完整类型化失败；`literal:` 是该例子的约定，不是 Base 协议。即时 future helper 只供该测试，不是生产异步执行器。

没有参考 Host/shared runtime/Tauri、默认模块、角色包、插件、模型配置、数据库、HTTP 服务或六槽顺序。contracts crate 自身仍含参考端口，其全部导出不能统称纯小 Kernel；本片没有做物理迁移。生成工程也不是已发布 SDK，移动源码或工程后应维护本地依赖路径。

`--dry-run --json` 共用实际五文件清单；`--check --json` 检查工具与源路径。两者不物化工程。Host/server/preset/backend/角色/插件/聊天存储/自定义 pipeline/Monolith/dual-core/weld 及 quick/smart/归档/已有项目等替代流程不适用，明确拒绝，不静默退回 stub 或参考 Host。成功生成也不写用户项目注册表。

## 3. 关联改动与兼容保留

- CLI：模板枚举和目录、参数早期校验、配置/交互/生成、检查/预览、完成提示与穷尽 match 接线；专用五文件模板和外部工程 E2E。
- 官方 scaffold：在原五个 profile 之后新增 `kernel-base`；复用既有 builtin init 路径，没有另一套包执行机制。
- 文档：既有中英 RFC 和 CLI 指南同步当前基础入口，历史审查正文保留；计划与 DCL 只追加；本 Wave 位于既有报告目录。
- 不改：types/contracts/runtime/Host 的公共或生产实现、依赖/锁、CI/门禁组合、现有 rich 接口、技术债状态、其它并行本地切片。

旧五模板的位置、默认角色/Monolith/运行行为与覆写逻辑保留；显式基础模板走独立分支。选单新增在末尾，避免改变原默认选中项。交互接线做源码核对，自动验证使用非交互 CLI；没有声称实际操作终端 TUI。

## 4. 实际验证及复现

以下 native exit 均来自独立保存的 stdout/stderr 与退出件；预期红单独保留，不包装成成功。CLI E2E 会嵌套 Cargo，因此串行执行。

| 检查 | 实际结果 | 本机证据前缀 |
|---|---|---|
| 旧 CLI 的新模板调用 | exit 101；CLI 编译成功，子进程 exit 2 拒绝未知 `kernel-base` | `T1-red` |
| 初次实现外部工程 | exit 0；当时2条 E2E，生成工程2条契约测试 | `T2-green-attempt1` |
| CLI 及旧模板/包回归 | exit 0；98单测 + 7旧模板 + 3基础工程 + 5包合同测试 | `T3-cli-regression` |
| 最后基础用例 | exit 0；3条 E2E，含注册表零写入、11组运行参数拒绝及检查/预览/失败路径 | `T4-foundation-final` |
| CLI Clippy / workspace fmt | exit 0；无 Clippy 诊断，fmt 无差量 | `G6-clippy-final` / `G7-fmt-final` |
| 分层 / module-compat / 插件索引 | exit 0；3 imports/1 FQ ref；10 slots/9 manifests/7 UI；6 plugins | `G3-layering` / `G4-module-compat` / `G5-plugins-index` |
| 显式 workspace doctest | exit 0；总计51 passed，0 failed | `D2-workspace-doctest` |
| 留存生成工程、Cargo/图/fmt | exit 0；2条契约测试；解析59个包，根只有contracts/types两项依赖，受拒组件均不在图中 | `A1`–`A4` |

最后命令：

```bash
cargo test --locked --offline -p oclive-cli --test e2e_kernel_base -- --test-threads=1 --nocapture
cargo clippy --locked --offline -p oclive-cli --all-targets -- -D warnings
cargo fmt --all -- --check
```

T3 使用 `cargo test --locked --offline -p oclive-cli --test e2e_kernel_base --test e2e_init_templates --test scaffold_contract --bin oclive-cli -- --test-threads=1`。不宣称它等于 CLI 全测试或全 workspace 普通测试。子工程的两条测试独立计数，不混入113条父级回归。

workspace doctest 明确执行 `cargo test --locked --offline --workspace --doc`。新工作树先执行既有 `node scripts/ensure-tauri-dist.mjs`（编译用途 stub）与 `node scripts/bundle-kernel-for-tauri.mjs --profile debug`（资源准备），均 exit 0；只准备编译资源，没有运行 Host、服务或真实前端。普通 `--lib` 不能代替这项 doctest。

离线生成工程验证复制本仓库锁文件作为本地版本种子，再由子 Cargo 解析独立 root；该复制是测试夹具，不是生成器的第六个文件。留存示例在 ignored `generated/kernel-base-example/`，初始五文件集合另存 `A1-initial-files.json`，构建后的 Cargo.lock 与初始生成物区分。开发者正常 `cargo test` 仍需要自己的 Rust 工具链和可取得的依赖，初始化不安装依赖。

文档链接通过（默认52份、显式7份）；docs-only 旧路径、镜像、登记和债结构均 exit 0，结构绿不等于债务状态结案。精确写集为27文件，20份原件 hash 相符，计划/DCL 的原历史字节前缀保持；27份 UTF-8/无 BOM/异常字符检查与 diff 通过。原始日志留在该 ignored 证据目录。stderr 中出现编译、Git LF/CRLF 和本地 Git fixture 信息不等于测试失败，不宣称零 stderr。

## 5. 有限语义自查与出口

G17 链为 CLI 参数 → 原模板选择/配置 → 专用生成 → 实际文件/公共契约 → 外部 Cargo 调用与失败 → 回归。没有跨入 Tauri/shared/发行版运行；它们不需本片改动。没有新 DTO/trait/re-export、资源权限、远程下载或执行策略；不引入生产 unwrap/expect、新调度器或并行六槽 parser。

自查未发现阻断项。五文件清单同时供生成与预览，源检查供建配置/预检/写入复用；不为此构造新框架或抽象接口。输出渲染在写文件前完成、非空目录拒绝，但没有声称 IO 故障下是事务生成；builtin init 继续沿用现有写入语义。原技术债状态、Full/签名/TLS/双核/多Agent等冻结保持。

达到新工程真实生成、公共契约调用成功、依赖/产物边界明确即停止，不扩查五个槽实现、SDK、Host 下载、运行内核入口或 CI 重新设计。保存本地提交后只读交接；精确提交身份在交付报告披露，不为回写 SHA 再造提交。后续合流和正式 CI 另按批次安排，不能借原基线绿灯替本片盖章。
