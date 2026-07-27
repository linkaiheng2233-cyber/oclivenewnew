# Chat Pro 角色目录

本目录保存 Chat Pro 随发行版提供的角色包。文件名为了兼容已有链接暂时保留为
`README_MANIFEST.md`，但当前角色包不再以 `manifest.json` 为主清单。

## 当前契约

- 主清单：`roles/<id>/pipeline.ocblueprint`（v2/v3 SSOT）
- 核心人设：`core_personality.txt`
- 可选运行策略：`config.json`
- 可选身份模板：`user_identities/index.json` + Markdown
- 可选场景：`scenes/<scene id>/`
- 可选立绘：`portrait_catalog.json` + 包内资产
- 可选语音：`voice_profile.json`

不要让新包同时包含 `pipeline.ocblueprint` 与 legacy
`manifest.json` / `settings.json`。完整格式见
[`ROLE_PACK_SPEC.md`](../../../creator-docs/role-pack/ROLE_PACK_SPEC.md)，创作步骤见
[`CREATOR_ROLE_PACK_CUSTOMIZATION.md`](../../../creator-docs/role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md)。

## 随发行版角色的定位

- `mumu/`：面向 Chat Pro 体验定制，包含更完整的场景、立绘、身份、连续性和语音能力；
- `deepseek/`：结构较轻、满足 Portable Core 的跨发行版参考角色；
- `gugu-gaga/`：咕咕嘎嘎小企鹅，以高频企鹅口头禅和七情绪映射形成完整角色表达；
- `phoebe-chubi/`：非官方民间 Q 版菲比啾比，以标点变化驱动同一口头禅的不同情绪；
- `doro/`：非官方社区 Doro/Dora，以高频自称 `doro` 和对用户的「人」称呼形成关系语言；
- `枫侵月/`：内容和语音侧保留自身设计，并与 DeepSeek 对齐 Portable Core 基线；
- `polish-dev/`：回复后处理等开发链路的调试包，不作为正式角色模板。
- `gentle-landlady/`：18+ 成年关系、房东权力边界与明确同意的测试角色；当前角色契约尚无年龄门槛字段，因此保持 `dev_only`。

这里的角色没有统一能力上限。角色只需为自己声明的文件和能力提供依据，并通过对应
校验；发行版专属扩展不应被误写成所有角色的强制要求。

## 校验

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<id>
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<id> --profile portable-core
```

第二条只适用于声明 Portable Core 的角色。它检查核心人设、启用的立绘目录以及七个
基础情绪资产，不代表语音、完整 UI 或所有发行版增强能力已经验收。

## 导入与兼容

`.ocpak` / `.zip` 推荐使用 `{角色 id}/...` 单一顶层目录。应用导入时优先识别
`pipeline.ocblueprint`；legacy 包仅作为兼容输入。旧包维护与迁移见
[`V1_TO_V2_MIGRATION.md`](../../../creator-docs/role-pack/V1_TO_V2_MIGRATION.md)。
