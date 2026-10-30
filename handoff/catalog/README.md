# AI 文档导航

**SSOT 范围**：本导航命令、登记格式与维护方式；不定义项目架构、规范权威、技术债状态或验收规则。

**最后更新**：2026-10-27。

这是可选的本地导航工具。普通查询按次读取 Git 跟踪的文档，同时保留 [records.json](records.json) 的精选入口与限定摘要。现有 [AI 阅读索引](../AI_READING_INDEX.md) 和 [文档状态判读](../README.md#documentation-status) 继续适用；缺卡、失配和未知均可以回原文，不表示没有实现或没有规范。

## 使用

在仓库根目录运行，要求 Node.js 和 Git，不需安装 npm 依赖或启动 Host：

```sh
node scripts/ai-docs.mjs query --task documentation
node scripts/ai-docs.mjs query --task plugin --query plugin_rpc_invoke
node scripts/ai-docs.mjs query --task cli --query --kernel-source
node scripts/ai-docs.mjs query --query MemoryBase
node scripts/ai-docs.mjs query --query MemoryBase --select entries
node scripts/ai-docs.mjs query --query registry --count
node scripts/ai-docs.mjs query --query registry --all
node scripts/ai-docs.mjs boot --task kernel
node scripts/ai-docs.mjs check
```

先用 `query` 找入口，再读原文对应章节与适用前提。默认 `--select documents`：每份文档只列一次（取首个命中入口），`--limit` 数的是**文档**；需要逐标题展开与正文短引文时用 `--select entries`（此时 `--limit` 数的是入口）。输出包括章节行、来源、覆盖范围、受检模式与版本、失配和回退入口。

**窗口与穷举**：默认只列 20 份文档，超过时提示 `Showing 20/N`。耦合核对（“哪些文档提到 X”）若命中数超过 20，先跑 `--count` 看规模，再用 `--all` 穷举：一行一文档、不带正文引文，实测 112 份约 4.5k token，比读一份大部头便宜一个数量级。`--all` 只用于文档单位，截断提示会给出穷举成本。文档模式下精选记录优先占**前 3 席**，其余排在即时导航之后（仍可用 `--all` 列出）；若同一路径存在 `*-en/` 镜像且其中文原文也在结果里，只保留原文。正文引文最多 240 个 Unicode 字符，被截断时以 `…` 结尾，不是摘要、完整上下文或经过审阅的结论。工具不执行文中指令。

### 什么时候值得用

- **文档量大、问题宽**：例如“哪些文档提到 module-compat”，`grep` 给的是无预算的数百行；这里给 20 份候选，并可用 `--count` / `--all` 看到规模与成本；
- **需要确认覆盖**：`--all` 的结果可与全文基线对比且可复现（见下方评测），适合耦合核对的第一遍；
- **上下文紧张**：20 份文档约 1.3k token，先定位再读原文，比逐份读省一个数量级；
- **已知 ID 或路径**：直接 `grep` / `read` 更快，本工具没有优势。

**边界**：它把读者带到文档，但文档句子常是粗粒度的——默认值、失败行为、顺序类结论必须回到源码核对。例：`reply_post_process` 的降级分层分布在 resolver 与 `post_llm`，只读文档会漏掉“解析失败回退 builtin”这一层。

### 质量评测

`node scripts/ai-docs.eval.mjs` 报告两组指标：8 个定位问题在 `limit 8/12/20` 下的召回，以及 3 个字面标记用 `--all` 穷举后与全文基线的完整性对比。用例来自 [`eval.cases.json`](eval.cases.json)（可用 `--cases PATH` 指向别处），所以同一套评测器可以换仓库复用。默认只打印；`--gate` 在 `locate@20` 不足 8/8 或耦合不完整时退出 1。**2026-10-20 的历史快照**为 5/8、6/8、8/8，耦合 3/3 完整；原文和排序变化后，当前数字以本轮命令为准，不把旧快照当成永久性能。它衡量本工具的定位与覆盖，不代表原文正确、实现符合或运行验收，也未接入必需 CI。

### 来源与范围

| 来源 | 读取范围与使用方法 | 可以得出的结论 |
|---|---|---|
| `discovered`：即时原文导航 | `query` 默认在 `AGENTS.md`、根 `README.md`、`handoff/`、`creator-docs/`、`human-docs/` 的已跟踪 Markdown 中匹配路径、标题和正文（该列表来自 `records.json` 的 `roots`，可用 `--roots a,b` 覆盖；没有 catalog 时默认整仓已跟踪 Markdown）。排除路径含 `archive` 的目录及 `handoff/debt-marathon/waves/`；不含未跟踪笔记、顶层 `*-en` 镜像树或其它工程文件（`creator-docs/architecture-en/` 这类嵌在已约定根目录内的子目录仍会被查询）。单份文档编码异常时记 `source_encoding` 并跳过该篇，不中断本次查询。 | 本次读取找到该位置；`read-time-only; not pre-reviewed`，不推断现行／草案／完成状态。 |
| `curated`：精选入口 | `records.json` 登记的路径、用途、标签和章节（默认依次尝试 `handoff/catalog/records.json` 与 `catalog/records.json`，可用 `--catalog PATH` 指定）。使用 `query --scope catalog` 限定查询；`boot`、`check` 默认只处理此层。 | 定位和版本是否符合登记；可选摘要另有明确核对范围，仍不等于内容或运行验收。 |

两类入口可以同时出现，输出保留来源。即时导航不生成数百份卡片、摘要或审阅记录；每次读取当前版本，无需为每个正文维护另一份哈希。精选入口优先展示，其后按路径与行号排序，不表示规范权威排序。

`--query` 是大小写不敏感的**字面匹配**，不是自然语言检索。即时导航的 `--task` 标签从路径词生成，例如 `cli`、`plugin`、`memory`、`catalog`；精选标签由记录维护。标签只是筛选条件，不能证明责任归属或状态；词形不同可能找不到，应去掉筛选或换关键词，不把零结果解释为能力不存在。引用代码块也可命中，不能把示例误当成生产接线。

工作模式是逐项观察，不是跨并发编辑的原子快照；需要精确复现时使用提交模式。JSON 中 `source_oid` 对即时导航表示当前 index／提交的 blob，对精选入口表示登记版本；`observed_oid` 表示实际 index／提交版本。工作文件有 `unstaged_change` 时，正文还包含未暂存修改，OID **不代表这些未暂存字节**。`staged_change` 与 `unstaged_change` 分开报告，输出不是内容认证。

固定提交复现使用完整 SHA；**记录与原文都从同一提交读取**。以下 `FULL_SHA` 是占位符，应替换为包含本试点的真实提交：

```sh
node scripts/ai-docs.mjs check --mode commit --rev FULL_SHA
node scripts/ai-docs.mjs query --task role --mode commit --rev FULL_SHA --json
```

`--root PATH` 可明确指定具有本工具登记输入的另一 Git checkout 根目录，默认使用脚本所属仓库。提交检查不借用该 checkout 的现行记录。`--json` 输出结构化结果，不自动写文件。退出码：`0` 表示本次所列导航／结构检查无已发现问题，缺少查询结果也会以 `0` 给出回退；`1` 表示所列记录需要修复或核对（包括工作修改）；`2` 表示参数、登记输入、读取或 Git 工具故障。三者都不表示整份内容认证。截断输出不能证明未展示项无问题。

`check` 只检查精选层的登记、定位及限定摘要，拒绝 `--scope documents`：即时读取没有预先核对基线，不能冒充全仓审查。`boot` 只列相关入口，不代替已有必读规则；本工具未安装或某项读取失败时，可以直接回到原有阅读索引。

**没有 catalog 时**：`query` 与 `boot` 仍可用——精选层为空，只做即时导航（默认扫描整仓已跟踪 Markdown）。`check` 需要精选登记，缺 catalog 会以 exit 2 给出回退提示。这是把它用到陌生仓库的最小前提。

## 记录怎样维护

精选层唯一登记输入是 `records.json`，初始保留八份文档；即时导航的输入则是上文范围内的 Git 跟踪原文。每份精选文档一条记录，可有多个章节入口；不按文件大小拆分文档，也不复制完整规则或债务状态表。普通原文新增、改名或编辑无需同步一份即时导航清单；新文档需正常暂存或提交后才进入发现范围。

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

动态夹具覆盖真实命令、缺卡回退、源／摘要变化、staged／unstaged、提交隔离、换行差异、定位失败／歧义、错误输入、目录链接和读取前后原件／索引不变；扩面另覆盖正文匹配、历史目录排除、未跟踪笔记排除、以 `--` 开头的查询词和批量对象缺失。它们验证工具机制，不表示原文或全仓事实已通过审查。

该命令没有加入 Dimension 5、required CI、自动启动规则或常驻进程。批量读取 Git 对象，不持久化搜索索引、缓存或锁文件。原文、Git index 与登记文件均只读。

当前范围达到“实际任务能找对原文、更新可见、失败可回退”后冻结，后续使用中按具体障碍改进，不开展全仓穷尽审查。历史实施分别见 [初始试点](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261020-AI-DOCS-PILOT.md) 与 [扩面和实际使用](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261020-AI-DOCS-EXPANSION.md)；报告不拥有当前规范或债务状态。
