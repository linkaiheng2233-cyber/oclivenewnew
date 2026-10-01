# K-SUPPLY-05-Full

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · Cargo.lock→audit+KNOWN · 假 Full Done 禁止

| 字段 | 值 |
|------|-----|
| **债 ID** | K-SUPPLY-05-Full（Minimal 已 Done） |
| **台账** | Minimal Done；Full Partial，完成条件仍为零 `[bans.skip]` |
| **标题** | 消除 skip / 收敛 duplicate 至可无例外 deny |
| **尺寸** | L |
| **Minimal / Full** | **Full** |
| **Owner** | main-repo |
| **状态** | Blocked · needs-reconcile（历史依赖切片已合入；Full 未完成） |
| **更新** | 2026-10-01 |

**接手前提**：从当前 `deny.toml`、lock 与生态约束重新确认剩余 skip/依赖族，再决定获准的下一 Stage；不得把历史 PR 合入或 Stage 3 坐标当成 Full 可结案。依据见 [DCL-20260928-01](../DEBT_CHANGELOG.md#dcl-20260928-01--初始化审查与调度对账)。本轮不改依赖或运行收敛。

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-SUPPLY-05-Full",
  "runner": "auto",
  "planStatus": "blocked",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 3,
  "prerequisites": ["Reconcile the merged dependency slice against current deny skips and ecosystem constraints; approve a remaining family scope or supply complete zero-skip Full evidence before resuming"],
  "stages": [
    {"id": 0, "title": "Baseline duplicate families", "files": ["read-only"], "actions": ["Capture deny skips, duplicate tree and current ratchet"], "checks": [{"command": "cargo tree -d", "why": "Full scope is defined by current duplicate families"}, {"command": "cargo deny check bans", "why": "The initial deny state is the comparison baseline"}], "outputs": ["Prioritized duplicate-family baseline"], "rollback": "No writes"},
    {"id": 1, "title": "Converge one dependency family", "files": ["Cargo.toml", "Cargo.lock", "deny.toml"], "actions": ["Converge exactly one compatible family per dispatch"], "checks": [{"command": "cargo deny check bans", "why": "The family change must not require an undocumented skip"}, {"command": "cargo audit", "why": "Cargo.lock changes require supply-chain verification"}], "outputs": ["One reviewed dependency-family reduction"], "rollback": "Revert the family change when compatibility or audit regresses"},
    {"id": 2, "title": "Remove final skips", "files": ["deny.toml", "Cargo.lock"], "actions": ["Remove skip entries only after all families are compatible"], "checks": [{"command": "cargo deny check bans", "why": "Full closure requires bans with no skip exception"}, {"command": "node scripts/check-cargo-dedup-ratchet.mjs", "why": "Duplicate groups must not regress"}], "outputs": ["Empty bans.skip and green deny gate"], "rollback": "Restore the documented skip and keep Full open if the ecosystem cannot converge"},
    {"id": 3, "title": "Remote evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record audit, deny and target CI evidence"], "checks": [{"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "Full Done requires target remote CI success"}], "outputs": ["Done-eligible Full evidence or honest remaining skip"], "rollback": "Keep Full OPEN on any remaining skip or missing CI"}
  ]
}
-->

## 目标
- `deny.toml` multiple-versions=deny 且 **skip 列表为空**；有期限的紧急例外也只可记 Partial，不符合 Full Done
- `cargo deny check bans` PASS · dedup ratchet 不升
- TECHNICAL_DEBT 注明 Full Done 或仍 Partial+剩余理由

## 非目标
- 为消重破坏 Tauri/sqlx 功能 · 盲 pin 导致漏洞

## 分阶段
### Stage 0 · 导出当前 skip 与 cargo tree -d
### Stage 1 · 按族收敛（可多 PR）：每次只动一小族依赖 · audit
### Stage 2 · 删除 skip 条目 · deny 绿
### Stage 3 · 证据

## 停条件
生态不可消 → 停止 · Wave 写「仍须 skip: …」· **不准假 Full Done**。

## 2026-10-01 · reqwest 单族重对账（D1，实施未启动）

**基线 / 写集**：`053ebdadf990c9c278bfc601c1e9b1640258c962`，开场干净。限本计划、主台账、DEBT_CHANGELOG、既有 Actions Wave 的前轮远端证据，以及 `deny.toml` 的 reqwest 说明文字；不改版本、features、Cargo.lock、skip 成员或产品代码。本轮不是执行历史 Stage 3，机器计划与 QUEUE 仍 blocked:needs-reconcile；签名架构 K-SUPPLY-09 不在范围。

| 核对 | 现行证据 / 结论 |
|------|------------------|
| 基线 | `deny.toml` 有 38 条 skip；`cargo deny --offline check bans` exit 0。临时配置删除 tree 顶层未显示的 17 条后，deny 对这 17 族均报重复并 exit 2；不能从 tree 的显示范围推断 skip 已过期 |
| 直接客户端 | 根 Cargo.toml 的 reqwest 0.12、禁 default、启用 `rustls-tls`，由 CLI、Host、桌面继承；`cargo tree --locked --offline -e features -i reqwest@0.12.28 --depth 2` 确认当前 Windows 图使用 webpki 根与 ring |
| 另一个版本 | `cargo tree --locked --offline --target all -i reqwest@0.13.4 --depth 3` 指向 Tauri 2.11.5；其 Cargo.toml 将该依赖限定在 Android / 非 macOS Apple 目标。当前 Windows 反向树无 0.13 条目，不能把全目标重复写成当前桌面双 HTTP 栈 |
| 兼容止点 | reqwest 0.12.28 的 `rustls-tls` 指向 webpki roots；0.13.4 的 `rustls` 改用平台验证器并引入 aws-lc。仅改版本与 feature 名不是行为等价更新；影响云端模型、远程插件、下载等 HTTPS 调用，不只是本地恢复入口 |
| 后续 | 维护者于 2026-10-02 选择暂缓 reqwest，继续其他债务。保留该 skip；若未来重启，平台证书策略及跨平台 HTTPS 验证另定独立迁移切片。本轮未做升级编译或 HTTPS 实机验收 |

本机源码依据为 Cargo registry 的 reqwest 0.12.28/0.13.4 `Cargo.toml`、`src/async_impl/client.rs` 与 tauri 2.11.5 `Cargo.toml`；可携带的上游说明见 [reqwest 0.13 发布说明](https://seanmonstar.com/blog/reqwest-v013-rustls-default/)。结论足以分类后停止，不为减少一个 skip 自动改变安全语义。原始负控留在本机忽略目录 `.cursor/plans/debt-rust-toolchain-update-20261001-r0/`，不随 Git 携带。
