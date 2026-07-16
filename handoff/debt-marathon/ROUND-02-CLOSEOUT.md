# 第二轮马拉松收尾记录

> 本文是第二轮马拉松的收尾记录，不是新的技术债务 SSOT。债务状态仍以 [`../TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) 为准。  
> 计划：[`ROUND-02-PLAN.md`](./ROUND-02-PLAN.md) · Wave 0：[`waves/WAVE-20260716-ROUND-02-W0.md`](./waves/WAVE-20260716-ROUND-02-W0.md)

## 结论

- 分支：`debt/marathon-round-02`（Round-02 专用 worktree；**未合 main**）。
- Wave 0 发现 `#124/#125/#126` **均已 MERGED**；队列从过期 `pr-open` reconcile 为证据/blocked。
- 本轮 **Done**：`T-DOC-02` · `D-ROLEVER-01`（仅写合入后证据，未重复实现）。
- 本轮 **Partial / blocked（保持）**：`K-RESILIENCE-01` Minimal Partial（Full 无书/需 RFC）· `K-SUPPLY-05-Full` `blocked:needs-ecosystem` · `K-CROSS-01` · `K-DIST-01` · `V-MARKET-01` · `K-VOICE-07`。
- `npm run check:debt-marathon -- --assert-no-runnable` **PASS** → 无 runnable auto 实现 Stage。
- 禁止项遵守：未改姊妹仓 · 未合 main · 未把 human/skip 改 auto · 未 invent Full ResilienceLayer。

## 本轮交付清单

| 债 | 终态 | SHA / PR / CI | 测试或核实命令 |
|----|------|---------------|----------------|
| **T-DOC-02** | **Done** | merge `94b380ce` · [#124](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/124) · CI [29498022046](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29498022046) | `gh run view 29498022046 --json headSha,conclusion,url` |
| **D-ROLEVER-01** | **Done** | main `23e4e184` · 内容锚 `601f48cf` · [#125](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/125) · CI [29500322721](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29500322721) | `git diff --check` · 同上 `gh run view` |
| **K-RESILIENCE-01** Minimal | Partial · plan **closed** keep-open | 经 [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) | Full **另册**；无 RFC 不施工 |
| **K-SUPPLY-05-Full** | **blocked:needs-ecosystem** | [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) · ratchet 75 · 仍有 skip | `cargo deny check bans` |
| **K-CROSS-01** | **blocked:needs-cross-platform-smoke** | Minimal 文档已合 | 三平台实机 |
| **K-DIST-01** | **blocked:needs-signing-secrets** | Minimal gaps 已合 | 签名/updater 密钥 |
| **V-MARKET-01** | **blocked:needs-sibling-repo** | SCOPE 已合 | 姊妹仓市场实现 |
| Round-02 证据提交 | pushed | `23b73467` · `c8e5eb6d` · `2b42acb1` @ `debt/marathon-round-02` | `npm run check:debt-marathon` |

## 剩余主仓优先级（不扩大权限）

1. **P1** [`V-VSCODE-PERF-05`](../TECHNICAL_DEBT_INVENTORY.md) — 姊妹仓实机（human）
2. **K-RESILIENCE-01 Full** — 须 RFC + 新 long-plan 后才可 ready
3. **K-SUPPLY-05 Full** — 生态收敛后再解 `needs-ecosystem`
4. **K-VOICE-07** — RFC v2 锚点
5. **K-CROSS-01 / K-DIST-01 / V-MARKET-01** — 外部条件

## 后续联动接手坐标（只链既有 SSOT · 本轮未改姊妹仓）

| 延伸项目 | 接手坐标（主仓内） | 版本 / 协议要点 |
|----------|-------------------|-----------------|
| **oclive-launcher** | [`COMPATIBILITY.md`](../../creator-docs/COMPATIBILITY.md) 姊妹仓表 · launcher README（外链） | 注入 `OCLIVE_ROLES_DIR`；不替代主契约；发版对拍主程序 **0.5.0** |
| **oclive-pack-editor** | [`COMPATIBILITY.md`](../../creator-docs/COMPATIBILITY.md) · 债 **PE-TURN-01** / **PE-UID-01**（TECHNICAL_DEBT · human） | 编写器 **0.5.0** ↔ 主程序 **0.5.x** · `ui.json` / `HOST_RUNTIME_VERSION` |
| **oclive-vscode** | [`handoff/vscode/README.md`](../vscode/README.md) · [`VSCODE_DISTRIBUTION.md`](../vscode/VSCODE_DISTRIBUTION.md) · 债 **V-VSCODE-PERF-05** | 扩展 **0.4.1**；F5/`.vsix` 实机 · spawn/attach ≥0.4.0 推荐主程序 0.5.0 |
| **oclive-plugin-market** | [`PRODUCT_LINE_TASK_BUCKETS.md` § V-MARKET-01](../PRODUCT_LINE_TASK_BUCKETS.md) · 债 **V-MARKET-01** Partial | 主仓 CLI/`MarketView` 已有；Full UI = 姊妹仓 human |
| **主仓整体联动** | [`COMPATIBILITY.md`](../../creator-docs/COMPATIBILITY.md) · [`RELEASE_VERSIONING.md`](../../creator-docs/development/RELEASE_VERSIONING.md) · [`handoff/distros/README.md`](../distros/README.md) K-DIST gaps · [`AGENTS.md`](../../AGENTS.md) 发版表 | 桌面宿主 **0.5.0** · CI `ci.yml` 硬门禁 · 发版 `check:release` |

### 版本 · 协议 · 发布 · 测试矩阵（指针）

| 主题 | SSOT |
|------|------|
| 版本对拍 | [`COMPATIBILITY.md`](../../creator-docs/COMPATIBILITY.md) |
| 发版 SemVer | [`RELEASE_VERSIONING.md`](../../creator-docs/development/RELEASE_VERSIONING.md) |
| 插件/六槽契约 | [`PLUGIN_V1.md`](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 角色包迁移 | ROLE_PACK_SPEC §11（本轮 D-ROLEVER Done） |
| 分发缺口 | [`handoff/distros/README.md`](../distros/README.md) |
| 测试分层 | [`AI_VERIFICATION_PROTOCOL.md`](../AI_VERIFICATION_PROTOCOL.md) · OOCP / invoke / dimension5 |
| 马拉松队列 | [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) |

## 不变更的外部事项

- 不自动合并 `main`（Round-02 分支待人工审查后合）。
- 不把缺少签名密钥、姊妹仓权限或三平台实机的事项伪报为 Done。
- merge-main CI [`29510167890`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29510167890) 若仍跑 rust windows，不影响已用 PR-head 硬门禁证据收口的 D-ROLEVER；合入后可补 Verification 脚注。
