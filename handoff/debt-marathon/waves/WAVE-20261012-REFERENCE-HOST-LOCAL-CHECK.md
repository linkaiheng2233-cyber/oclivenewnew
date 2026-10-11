# 参考 Host 默认配置本地反馈 · 2026-10-12

**状态**：Locally verified，实际入口和适用检查通过；本片未合入或推送main，不宣称正式CI已验证新入口。
**基线**：`12bc7dc8243a2d3c3bf17f98306da1cf4cb8ca92`；独立分支 `codex/ci-reference-host-local-20261012-r0`。controller实施和语义自查 `independent=false`。
**范围**：[计划](../ROUND-02-PLAN.md#参考-host-默认配置本地检查入口2026-10-12)、[三层架构](../../MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries)和[检查说明](../../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#reference-host-local-check)分别拥有计划、边界和使用口径。

## 原因、范围与停止线

公共层已有窄反馈及完整检查；第二层开发者若只需参考Host默认配置，旧workspace入口的桌面资源前置过大。本片新增一个默认反馈命令，实际只运行现有Host Clippy/tests，不新建Host运行框架。会话材料绑定等改动可先走它，实际HTTP/持久化/产品影响仍按原门禁补验。

4组直接盘点到可行动即停：原rust含剩余workspace与桌面资源；frontend是产品/components；dual_core是参考Host可选条件；cross-host是组合，不能冒充公共义务。桌面manifest显式依赖Host的 `tauri-commands`，默认workspace条件与独立Host不同；这已足以反对未经对齐的机械迁移，不需要穷尽每个cfg或测试。此耦合暂保留，未新增需维护者决定的执行语义。

本片精确10路径：package仅新增一个命令，两个新脚本，CI设计与人类矩阵中英，PLAN/DCL只追加和本Wave。7份原件/13项不变控制面与manifest摘要保全于ignored `.cursor/plans/ci-reference-host-local-20261012-r0/`。不改Rust、manifest、锁、workflow/catalog/选择权限、既有入口/helper或旧报告。

第三层已有 `check:frontend`，原命令不变，本片只将它写到三层开发入口和人类矩阵；复用它的lint/typecheck/unit/Theater smoke/E2E构建，不再新建前端runner。前端反馈不含Cargo，不代替桌面/IPC、Linux预览或发布构建。`npm run build` 仍在实际需要该产物的任务中使用，不能把E2E构建冒充发布构建。

人类矩阵原前端生产 `npm run build` 要求保留，新增组合入口没有以E2E构建替换它。实施首个apply_patch批次曾因PLAN定位文字不相符被预检拒绝，Git确认零路径变化；重新读取实际尾部后写入有限计划并实施，没有覆盖原件或放宽验收。

## 验证及证据边界

固定入口用现有native helper，每步20分钟，缺Cargo/非零/超时停止。5项回归覆盖固定默认范围、首步失败、次步失败、真实缺工具和未知参数。R0缺入口native exit1保存于独立日志；红只证明入口尚未实施，不是产品失败。

| 本机证据 | 实际结果与含义 |
|---|---|
| `G1-node-regressions` | 新Host入口5＋旧公共入口10＋native helper10，**25/25**，包括真实native失败和CLI负控。 |
| `G2-reference-host-default` | native **exit0**，**61.86秒**；Clippy通过，Host lib **637 passed**；12个集成目标合计 **119 passed / 19 ignored**，全入口合计 **756 passed / 19 ignored / 0 failed**。未运行ignored业务场景。 |
| `G3-dimension5` | native exit0，**31项PASS**；原sample workspace lib按 `--ci` 模式跳过。Voice沿既有子进程Python覆盖，不改变用户环境。 |
| `G12-existing-frontend-entry` | 真实既有npm入口 native **exit0**，**84.52秒**；shared **54文件/280 passed**，ChatPro **24文件/92 passed**，Theater **11文件/53 passed**；lint/typecheck、两项Theater smoke及E2E构建完成。 |
| 文档/结构/保全 | 默认及本轮7文链接、镜像、登记、docs-only旧路径、债结构、编码；7份原件/13项不变摘要、历史前缀、10路径、package仅新增一个命令。 |

上述计数来自本轮日志，不沿用旧批次的634、70或243等历史数字。Host默认Cargo的13个目标均实际执行，doctest=false仍保持。前端日志含预期错误路径及构建提示，不把exit0写成零stderr；真实音频/模型或完整发行版未验证。

Host目前 `default=[]`，入口不启用tauri-commands/dual_core/test-faults；Clippy all-targets但非all-features，测试不筛选目标。manifest的Host doctest=false不改；默认测试成功不证明Host doctest、真实模型、真实网络、桌面IPC或发行版全部功能。原all-feature workspace/dual-core/组合job继续负责适用范围。

真实命令与结果在本机ignored证据和最终交付中披露；stdout/stderr/native exit独立保留，不把成功写成零stderr。未变Rust公共面，不额外workspace doctest；未影响集成链，不重复全量本地/远端矩阵。该切片仅本地冻结，正在受验main的正式结论单独绑定其SHA；K-CI-IMPACT-01不转Done。
