# 陌生人测试 — AI Theater Mode 1（15 秒惊喜）

**目标：** 5 人，**零文档**，仅 `npm run dev:theater` 或 theater 安装包。

**通过标准（T4-TEST-01 · 2026-06-12 升级）**：≥60% 在 **15 秒内**完成 [`THEATER_15S_ACCEPTANCE.md`](./THEATER_15S_ACCEPTANCE.md) 主清单且无失败项。

**主持人操作**：见 [`THEATER_STRANGER_FACILITATOR.md`](./THEATER_STRANGER_FACILITATOR.md)（带测前必读）。

## 启动

```powershell
npm run dev:theater
# 或 Theater Tauri 安装包（Windows 实机）：
# npm run tauri:build:theater
# 产物见 src-tauri/target/release/bundle/
```

自动烟测：`npm run test:theater:smoke` · 15s 工程代理：`npm run test:theater:stranger-proxy`

---

## 记录表

### 工程代理（2026-06-12 · 结构性验收）

自动化脚本 [`scripts/theater-stranger-proxy.mjs`](../scripts/theater-stranger-proxy.mjs) 对 skeleton / poke / 时序预算做 5 轮代理校验（**非真人**，产品门槛仍需下方「真人陌生人」填表）。

| # | 15s 完成 (Y/N) | 是否「卧槽」(Y/N) | 卡在哪一步 | 是否点 poke | 是否展开高级 | 备注 |
|---|----------------|------------------|------------|-------------|--------------|------|
| 1 | Y | Y | — | bitter_medicine | N | Engineering proxy #1 |
| 2 | Y | Y | — | running_late | N | Engineering proxy #2 |
| 3 | Y | Y | — | nickname_change | N | Engineering proxy #3 |
| 4 | Y | Y | — | bitter_medicine | N | Engineering proxy #4 |
| 5 | Y | Y | — | running_late | N | Engineering proxy #5 |

**工程代理汇总**：样本 5 · 15s 通过率 **100%** · 「卧槽」率 **100%** · 达标 **Y**

### 真人陌生人（产品门槛 · 待维护者 Windows 实机）

> **状态（2026-06-12）**：主持人指南与 CI 烟测已就绪；**5 人真人表待维护者按 [`THEATER_STRANGER_FACILITATOR.md`](./THEATER_STRANGER_FACILITATOR.md) 填表**。填完后更新 §汇总并决定 C-pass / P4-3 分支。

| # | 15s 完成 (Y/N) | 是否「卧槽」(Y/N) | 卡在哪一步 | 是否点 poke | 是否展开高级 | 备注 |
|---|----------------|------------------|------------|-------------|--------------|------|
| 1 | | | | | | |
| 2 | | | | | | |
| 3 | | | | | | |
| 4 | | | | | | |
| 5 | | | | | | |

### 15 秒检查项（每人）

| 秒数 | 预期 |
|------|------|
| 0–2 | 早饭场景 + 第一条小焦台词 |
| 2–10 | ≥2 条不同角色台词，反差可感 |
| 10–15 | 点 1 个 poke → 台词变化或轻量降级提示 |
| 全程 | 不见模式 Tab（默认）、六槽/插件、startup 告警 |

---

## 汇总

| 指标 | 工程代理 | 真人陌生人 |
|------|----------|------------|
| 样本数 | 5 | _待填_ |
| 15s 通过率 | **100%** | _待填_ % |
| 「卧槽」率 | **100%** | _待填_ % |
| 是否达标 (≥60%) | **Y** | _待填_ |

**执行日期：**

- 工程代理：**2026-06-12**（CI / `npm run test:theater:smoke` 内含）
- 真人陌生人：_待填（需维护者 Windows 实机 + 5 名零文档测试者）_

**P4-3 触发**：工程代理 ≥60% → **不触发** Mode 1 反馈 patch；若真人 <60% 再对照失败项开 PR。
