# A.I.Live（OCLive）

**可组合的 AI 角色运行时：角色包负责表达角色，Host 负责把能力接起来。**

[English](README.en.md) · [五分钟了解项目](human-docs/START_HERE.md) · [用户手册](creator-docs/getting-started/USER_MANUAL.md) · [公开内测发布](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)
[![Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A.I.Live 让创作者编写人设、立绘、场景与身份模板，让用户在本地运行自己的角色；开发者也可以只使用所需的契约和模块，构建自己的 Host。

**项目仍在开发与内测。** 本仓公开安装包、默认分支源码和新前端候选分别维护，不能用源码截图推断旧安装包已经包含同样功能。近期 Chat Pro 优化见[候选 PR #225](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/225)；版本与验收边界查[项目状态](creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)，可下载文件以各条 Release 的附件为准。

## 选择你的入口

| 你想做什么 | 从这里开始 |
|---|---|
| 体验角色、导入角色包、开始聊天 | [用户手册](creator-docs/getting-started/USER_MANUAL.md) · [公开内测发布](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases) |
| 制作或转换自己的角色包 | [角色包编写器](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) · [创作者黄金路径](creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) |
| 使用小内核、实现模块或自己的 Host | [独立 OCLive Kernel](https://github.com/linkaiheng2233-cyber/oclive-kernel) · [源码 SDK / Memory 配方](examples/developer-sdk-memory/README.md) |
| 编写插件或扩展界面 | [插件作者学习路径](creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) |
| 参与本仓开发 | [人类开发者入口](human-docs/README.md) · [贡献说明](CONTRIBUTING.md) |

## 角色、能力与运行空间

- **角色包**保存人设与素材，可由编写器创作，也可按支持的格式导入、转换；内置的沐沐是示例角色。
- **本地优先**：参考运行时在本机保存对话与角色状态。可配置本地模型，也可使用所选远端后端；软件与外部模型／云服务的使用条件分别核对。
- **能力可替换**：记忆、Prompt、LLM 等能力通过契约接入，由 Host 选择实现与调用顺序。六类能力不是固定的六个步骤。
- **Chat Pro** 是本仓的桌面参考 Host；VS Code 等入口按各自支持范围复用角色与运行时能力。

![Chat Pro 候选前端的角色运行空间预览](creator-docs/assets/chat-pro-chat.jpg)

*候选前端预览；实际界面与行为以你使用的版本为准。*

## 给开发者：先选接入层

[独立内核](https://github.com/linkaiheng2233-cyber/oclive-kernel)提供六个 Rust Base 契约与默认开启、可关闭的角色一致性剖面，已发布[源码预览版](https://github.com/linkaiheng2233-cyber/oclive-kernel/releases/tag/v0.1.0-preview.1)。它适合从小接口开始构建自己的 Host。

本仓同时包含更完整的**参考运行时**：具体能力实现、状态应用、持久化、权限与插件接线、HTTP/Tauri 适配和发行版 UI。选择复用它的部分，不会把整套参考编排变成所有 Host 的义务。

职责与实现对照只在[模块与边界说明](handoff/MODULE_MAP_AND_HANDOFF.md)维护；接入操作见[内核集成路径](creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)，源码兼容依据见[兼容说明](creator-docs/COMPATIBILITY.md)。

## 从源码启动桌面参考应用

先按[开发者入口](human-docs/README.md)准备 Node.js 22、Rust 和平台桌面构建依赖，再执行：

```bash
git clone https://github.com/linkaiheng2233-cyber/oclivenewnew.git
cd oclivenewnew
npm ci
npm run tauri:dev
```

开始聊天前，需要准备可用的本地模型后端，或按[用户手册](creator-docs/getting-started/USER_MANUAL.md)配置远端后端。源码仓库与轻量安装包不等于已经附带可运行的模型。

## 相关项目

| 项目 | 当前用途 |
|---|---|
| [oclive-kernel](https://github.com/linkaiheng2233-cyber/oclive-kernel) | 独立 Rust Base 源码库与示例 Host |
| [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) | 角色包创作、转换、校验与导出 |
| [oclive-vscode](https://github.com/linkaiheng2233-cyber/oclive-vscode) | VS Code 角色聊天 Host；日记／来信等为可选插件 |
| [角色包目录](https://github.com/linkaiheng2233-cyber/awesome-oclive-roles) · [插件目录](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins) | 发现条目与来源，兼容范围以所选 Host 和发布者说明为准 |
| [实体角色宿主计划](https://github.com/linkaiheng2233-cyber/oclive-skippy-plan) | 设备事件与小屏角色反馈的 P0 硬件原型，尚非成品 |

旧启动器与暂停的社区站是独立历史入口，不是使用 Chat Pro 的必装组件。完整文档导航见 [DOCUMENTATION_INDEX](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。

## 反馈、贡献与许可

这是持续开发的爱好项目。欢迎通过 [Issue](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues)或 **ocliveproject@163.com** 提出建议；有意思的想法会考虑采纳，但不保证每一项都会实现。

代码贡献看 [CONTRIBUTING.md](CONTRIBUTING.md)，AI 协作者从 [AGENTS.md](AGENTS.md)按任务读取资料。安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

主仓采用 [Apache-2.0](LICENSE)，保留 [NOTICE](NOTICE)。模型、第三方插件与角色素材的许可分别以其来源声明为准，见[许可政策](creator-docs/LICENSE_POLICY.md)。


<!-- Preserve previously published section bookmarks. -->
<a id="ailive--可插拔的角色动脉织机"></a>
<a id="从哪里开始"></a>
<a id="这是什么"></a>
<a id="四个例子30-秒看懂能做什么"></a>
<a id="例子-1--创作者做一个可对话-oc"></a>
<a id="例子-2--开发者只换-llm不动人设"></a>
<a id="例子-3--集成方同一角色包多端复用"></a>
<a id="例子-4--模块作者只写新能力其余白送"></a>
<a id="和常见方案有什么不同"></a>
<a id="三发行版同一内核--不同-hostprofile"></a>
<a id="架构一句话版"></a>
<a id="三十秒跑通贡献者"></a>
<a id="生态姊妹仓"></a>
<a id="ai--agent-接手"></a>
<a id="支持--许可"></a>

旧版章节书签保留在此。[整理前的首页](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/fb361ff5b878c633ef95dc66d717105d5b5074c0/README.md)可用于查阅历史说明；现行入口以本页与所链文档为准。
