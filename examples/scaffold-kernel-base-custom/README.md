# 自己维护 Kernel 基础模板

这个例子使用现有本地 Scaffold Package；生成空白 contracts/types 工程。你可以编辑模板文件、默认值和文件映射，不需要修改 OCLive 的公共契约或内置生成器。六槽接口仍可使用，运行绑定由作者负责。

## 1. 复制到自己的项目作用域

下面命令在包含本例的OCLive源码根执行，使用PowerShell；先准备一个新的模板工作目录。基础 `init` 本身不需要Node，这里的摘要辅助脚本才需要Node。

```powershell
$repo = (Get-Location).Path
$workspace = Join-Path (Split-Path $repo) 'my-template-workspace'
New-Item -ItemType Directory -Path "$workspace/.oclive/scaffolds" -ErrorAction Stop
Copy-Item -LiteralPath "$repo/examples/scaffold-kernel-base-custom/package" -Destination "$workspace/.oclive/scaffolds/package" -Recurse
Copy-Item -LiteralPath "$repo/examples/scaffold-kernel-base-custom/refresh-digests.mjs" -Destination "$workspace/.oclive/scaffolds/refresh-digests.mjs"
$sourceForCargo = $repo.Replace('\', '/')
```

首次复制使用新目录，避免覆盖自己的原件。Linux/macOS可使用对应文件复制工具；保持相同的 `.oclive/scaffolds/package/` 布局及包内原字节。版本控制须保持包内LF换行，源摘要按实际字节核对。

下面步骤在同一个PowerShell会话继续，保留 `$repo`、`$workspace` 和 `$sourceForCargo`。CLI命令从 `$repo`（OCLive源码根）执行，Cargo检查从生成工程根执行。

## 2. 查看、锁定与生成

```powershell
cargo run -p oclive-cli -- scaffold validate "$workspace/.oclive/scaffolds/package/oclive.scaffold.json"
cargo run -p oclive-cli -- scaffold inspect dev.example.kernel-base --path $workspace
cargo run -p oclive-cli -- scaffold resolve --path $workspace --write-lock
cargo run -p oclive-cli -- scaffold generate dev.example.kernel-base project --path $workspace --output ../custom-memory --set project_name=custom-memory --set "kernel_source=$sourceForCargo" --set "module_goal=Memory experiment" --accept-untrusted --dry-run
```

检查来源、权限、说明和预览，再移除最后的 `--dry-run` 执行同一生成命令。输出目录必须不存在，父目录必须已经存在。之后：

```powershell
cd ../custom-memory
cargo check
cargo test
cargo fmt -- --check
Set-Location -LiteralPath $repo
```

应得到Cargo、源码入口、自己的README、忽略文件及生成来源记录。README的开发目标应为 `Memory experiment`；换成 `--set "module_goal=Event experiment"` 并使用新输出目录，文本也随之改变。模板不预装Memory或其它槽实现，初始 `cargo test` 执行零项测试，不能解释为功能通过。

第三方/自己维护的本地包仍按 `untrusted_local` 处理；锁定来源与本次 `--accept-untrusted` 都必需。该确认仅授权受限文件生成，不允许执行模板中的脚本或授予网络/进程权限。生成器不会自动执行本例的摘要辅助脚本。

## 3. 改哪里，怎样继续使用

| 要调整的内容 | 编辑位置 |
|---|---|
| 工程布局、自己的模块和测试文件 | `package/templates/`；新增文件时在instruction的files中声明 |
| 字符串参数及缺省值 | `package/instructions.json` 的variables、manifest的defaults |
| 包ID、说明、维护者和版本 | `package/oclive.scaffold.json`；自己的ID/namespace不要使用官方 `com.oclive.*` |
| 生成文件目标及text/copy模式 | `package/instructions.json` 的files |

例如在 `templates/README.md.tmpl` 加入作者说明，或在自己的模板加入Memory测试文件。编辑后显式执行：

```powershell
Set-Location -LiteralPath $repo
node "$workspace/.oclive/scaffolds/refresh-digests.mjs"
cargo run -p oclive-cli -- scaffold validate "$workspace/.oclive/scaffolds/package/oclive.scaffold.json"
cargo run -p oclive-cli -- scaffold inspect dev.example.kernel-base --path $workspace
cargo run -p oclive-cli -- scaffold resolve --path $workspace --write-lock
```

然后用新输出目录重新预览与生成。未更新源摘要会拒绝生成；更新摘要但未重新锁定来源也会拒绝。辅助脚本只更新本例的源/指令摘要，不负责schema校验、授权或生成；它不是通用打包器，也不提供跨两个元数据文件的原子事务。真实生成校验由现有CLI完成。

`text` 只支持精确 `{{variable}}` 替换，没有条件、循环、include或自动继承。`extends/composition` 当前只是保留声明，不能用来扩展内置模板。需要不同文件集时维护自己的文件映射或生成器；字符串变量不是通用六槽配置协议。

公共定义、运行开关和功能验收仍归原责任层。内置基础模板的 `--base-example none` 只不生成Memory教学内容，不撤销MemoryBase，也不产生运行时Noop。真实Memory实现和消费者接线见 [Memory 开工包](../../human-docs/modules/slots/memory.md)。生成协议及来源规则见 [Scaffold RFC](../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)，日常操作见 [CLI 指南](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start)。
