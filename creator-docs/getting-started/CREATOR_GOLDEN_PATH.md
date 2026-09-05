# 创作者黄金路径：30 分钟做出第一个角色包

[English](../../creator-docs-en/getting-started/CREATOR_GOLDEN_PATH.md)

这条路径只解决一件事：做出一个能被当前 A.I.Live 参考宿主加载、校验和对话的组合角色包。格式真源是 [角色包规范](../role-pack/ROLE_PACK_SPEC.md)；这里不重复字段表，也不把该完整目录当作内核最小角色 contract。

## 你需要什么

- A.I.Live 运行时；
- [角色包编写器](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)，或本仓库的 `oclive-cli`；
- 至少一份人设正文；准备面向最小 contract 的内容时，还需至少一个视觉资产。场景、知识与预置记忆可以稍后补。

## 1. 建立可加载的参考宿主包（5 分钟）

在编写器中新建角色包，或者在仓库根执行：

```powershell
cargo run -p oclive-cli -- pack create -o .\work\my-role --flat --id my-role --name "My Role" --format-blueprint-v2
```

该命令当前生成参考宿主 v2 蓝图样例；不要再同时创建 legacy 的 `manifest.json` / `settings.json`。CLI 尚未提供独立的内核最小角色 contract 生成器。

## 2. 写角色，而不是写运行时（15 分钟）

只先处理这些内容：

| 内容 | 放在哪里 |
|------|----------|
| 角色是谁、如何说话、不可越过的边界 | `core_personality.txt` |
| 名称、作者、七维人格、关系默认值 | `pipeline.ocblueprint` 的 `meta` |
| 可选：七张基础情绪立绘 | `portrait_catalog.json` 与对应 PNG |
| 可选的前置记忆事件 | `memory_seed.json` |

`memory_seed.json` 是创作者提供的只读种子，不是用户运行后产生的长期记忆。可变人设和用户长期记忆由运行时管理，不要写回角色包。

七图集是推荐的可选跨发行版标准，仅选择 `portable-core` profile 时才强制七张。当前参考宿主基础校验允许没有立绘；[kernel minimal contract](../../handoff/ROLE_PACK_BOUNDARY.md) 则要求 persona prompt + 至少 1 个视觉资产，映射该产品包时仍需满足这一内容要求。共享逻辑校验已实现，实际加载与 CLI 接入仍待完成。

初次创作不需要修改 `slot_registry`、`groups`、远程插件、双核或 MCP；这些属于发行版和高级集成能力。

## 3. 校验并试聊（5 分钟）

```powershell
cargo run -p oclive-cli -- pack validate .\work\my-role
```

然后通过 A.I.Live 的角色包导入入口安装目录或压缩包，加载角色并至少试聊三轮：普通问候、情绪变化、一次人设边界问题。

如果要从源码运行，可让 `OCLIVE_ROLES_DIR` 指向包含角色目录的 roles 根，再启动桌面应用。

## 4. 发布（5 分钟）

```powershell
cargo run -p oclive-cli -- pack publish .\work\my-role -o .\work\my-role-0.1.0.oclivepack
```

发布前确认：包能通过校验、没有密钥或用户记忆、素材授权说明完整。版本与兼容性规则见 [角色包版本管理](../role-pack/PACK_VERSIONING.md) 和 [兼容性](../COMPATIBILITY.md)。

## 下一步按需阅读

- 想补场景、知识、身份或记忆：[创作者学习路径](../role-pack/CREATOR_LEARNING_PATH.md)
- 想理解编写器与运行时分工：[创作者工作流](CREATOR_WORKFLOW.md)
- 想做发行版或插件：[文档总入口](DOCUMENTATION_INDEX.md)
