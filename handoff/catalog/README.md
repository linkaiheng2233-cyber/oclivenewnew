# AI 文档导航试点

**SSOT 范围**：本试点命令、登记格式与维护方式；不定义项目架构、规范权威、技术债状态或验收规则。

**最后更新**：2026-10-20。

这是可选的本地导航工具，只覆盖 [records.json](records.json) 登记的八份文档。现有 [AI 阅读索引](../AI_READING_INDEX.md) 和 [文档状态判读](../README.md#documentation-status) 继续适用；缺卡、失配和未知均可以回原文，不表示没有实现或没有规范。

## 使用

在仓库根目录运行，要求 Node.js 和 Git，不需安装 npm 依赖或启动 Host：

```sh
node scripts/ai-docs.mjs query --task documentation
node scripts/ai-docs.mjs query --query 关系
node scripts/ai-docs.mjs boot --task kernel
node scripts/ai-docs.mjs check
```

输出包括原文路径、真实定位行、试点覆盖、受检模式与版本、失配和回退入口。Base、规则、状态由原文解释；导航成功不代表实现或运行验收成功。默认只列八个结果，截断时提示总数；可以缩小查询或用 `--limit 1..20`。任务标签见登记输入，标签不表示架构层级。`query` 与 `boot` 不打印原文全文，不执行其中的指令。

关键词只匹配登记身份、路径、用途、标签和入口标题，不搜索全仓原文。工作模式是逐项观察，不是跨并发编辑的原子快照；需要精确复现时使用提交模式。

固定提交复现使用完整 SHA；**记录与原文都从同一提交读取**。以下 `FULL_SHA` 是占位符，应替换为包含本试点的真实提交：

```sh
node scripts/ai-docs.mjs check --mode commit --rev FULL_SHA
node scripts/ai-docs.mjs query --task role --mode commit --rev FULL_SHA --json
```

`--root PATH` 可明确指定另一 Git checkout 根目录，默认使用脚本所属仓库。提交检查不借用该 checkout 的现行记录。`--json` 输出结构化结果，不自动写文件。退出码：`0` 表示本次选定导航／结构检查无已发现问题，缺少查询结果也会以 `0` 给出回退；`1` 表示选定记录需要修复或核对；`2` 表示参数、登记输入、读取或 Git 工具故障。三者都不表示整份内容认证。

## 记录怎样维护

唯一输入是 `records.json`。每份入选文档一条记录，可有多个章节入口；不会按文件大小拆分文档，也不复制完整规则或债务状态表。

- `id` 是试点内唯一身份；`path` 是仓库内 Markdown；`purpose`、`tags`、`entries` 只用于定位。精确标题路径包含其上级标题，不使用推断的 GitHub slug；显式锚点和标题必须在非代码、非注释区域唯一。
- `source_ref` 保存 Git blob OID 和对象格式。工作模式同时检查 HEAD／index／工作树；已暂存与未暂存分别报告。对受选 Markdown 的工作内容只归一 CRLF→LF 后与 index blob 比较，不丢弃空格、大小写或其它内容，也不修改换行或索引。
- 原文编辑后可以先查询；失配时显示旧版本关系。正常暂存原文后，可用 `git rev-parse :path/to/document.md` 读取索引的完整 blob OID，**经核对后**更新记录。工具不会自动暂存、刷新哈希或签署核对。与更新相关的原文和记录在同一正常分支提交，不需要主干专用基线。
- `summary` 可选，没有摘要也能导航。若保留摘要，其 `review` 必须绑定原文 OID 与 `reviewDigest(entry)` 返回的条目 SHA-256，并包含日期、核对主体／方式。主体可为 AI 或人工；controller 自查登记 `independent=false`。该方法只登记摘要与原文的一致性，不证明语义正确或运行验收。原文、定位、标签或摘要变化后必须重新判断，不能靠刷新摘要值自动续期。
- 一份文档若混有现行、草案和历史，分别定位，不给整篇打 `Done`。原文版本未变不代表代码未变；实现和运行证据仍看其绑定版本。替代关系可选，空值不证明现行。

初始样本仅在最小角色章节保留一条限定摘要，其余以导航为主。摘要过期／未核对时不展示它来支撑决策；原文仍可读取。核对日期仅是记录，不是到期自动判错或自动转绿的许可证。

## 检查覆盖与停止线

```sh
node --test scripts/ai-docs.test.mjs
node scripts/ai-docs.mjs check
```

动态夹具覆盖真实命令、缺卡回退、源／摘要变化、staged／unstaged、提交隔离、换行差异、定位失败／歧义、错误输入、目录链接和读取前后原件／索引不变。它们验证工具机制，不表示八份原文或全仓事实已通过审查。

该命令没有加入 Dimension 5、required CI、自动启动规则或常驻进程。本片达到“找对原文、更新可见、失败可回退”后停止，后续按实际使用反馈决定是否扩面。执行证据见 [本批 Wave](../debt-marathon/waves/WAVE-20261020-AI-DOCS-PILOT.md)；该报告不拥有当前规范或债务状态。
